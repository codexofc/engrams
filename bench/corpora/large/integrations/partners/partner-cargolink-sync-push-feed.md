---
name: partner-cargolink-sync-push-feed
description: Event-driven Cargolink sync since HF-3085: Messenger handler per load event, advisory lock per load, normalise before diff, 14-day re-list
type: project
status: active
verified: 2026-06-12
---

# Cargolink sync, event-driven (HF-3085)

Replaced [[partner-cargolink-sync-polling-v1]] on 2026-03-09. The idea: react to what changed instead of comparing everything every two minutes.

## Flow

1. Every completed load transition and every relevant field change writes a `load_events` row (already the case) and, for loads of organisations with Cargolink enabled, dispatches a `PartnerSyncRequested(load_id, partner='cargolink')` message on the `partners` Messenger transport (RabbitMQ), through the transactional outbox middleware so it is sent after commit.

2. `CargolinkSyncHandler` loads the current state of the load and the current `partner_load_refs` row, computes the desired listing, and calls one of: `create`, `update`, `delete`, `relist`. It writes the result (`external_id`, `sync_state`, `last_pushed_at`, `last_error`) back into `partner_load_refs` ([[partner-dedup-external-refs-table]]).

3. Failures go through Messenger retries (3 attempts, 10 s, 1 min, 10 min), then the message is dropped and the row stays `PUSH_FAILED` with `last_error`. The nightly comparison catches it if it is still wrong the next day.

Median delay from load change to Cargolink acknowledging the update: 4 seconds. p99: 40 seconds (their API's own latency spikes).

## Per-load serialisation

Two changes on the same load within seconds (price then date) must not race. The handler acquires `pg_advisory_xact_lock(hashtext('cargolink:' || load_id))` for the duration of the push. Messages for the same load are processed by one worker at a time; different loads run in parallel across the 4 consumer processes. Simpler than a partitioned queue, and the contention is nil in practice.

## Normalise, then diff

The v1 worker's phantom updates came from comparing our raw fields with their normalised ones. The handler now runs our fields through the same normalisation Cargolink applies (documented in [[partner-load-field-mapping-rules]]: ASCII folding of street names, postcode formats, rounding of prices to 5 EUR) and compares the normalised form with what we last pushed (`pushed_payload_hash` column). No difference, no call. Updates fell from about 60 000 per day (v1, mostly phantom) to about 4 000.

## Re-list at 14 days

Cargolink listings expire after 14 days. A scheduled command `app:partners:cargolink:relist` runs hourly, finds `partner_load_refs` rows whose `external_created_at` is older than 13 days 12 hours with the load still `OPEN` or `BIDDING`, and calls `relist`: create a new listing with `previous_listing_id` set (a Cargolink field that carries the bids' visibility over), store the new `external_id` and keep the old one in `previous_external_ids[]`. Bids received on the old listing id are still matched thanks to that array. This is the fix for [[partner-incident-2025-12-cargolink-mass-expiry]].

## Deletes

A load that leaves `OPEN`/`BIDDING` (dispatched, cancelled, expired) triggers `delete` on Cargolink. If the dispatch went to a Cargolink carrier, the listing is instead marked `awarded` with their carrier id so their side shows the outcome; that is a `status` call, not a delete.

## Nightly safety net

The v1 full comparison still runs at 03:00 as `app:partners:cargolink:reconcile --fix`, described in [[partner-reconciliation-nightly-job]]. Since the event-driven sync went live it has found between 0 and 6 discrepancies per night, all traced to dropped messages after three retries during Cargolink outages.

## Metrics

`hf_partner_sync_total{partner,action,result}`, `hf_partner_sync_delay_seconds` (load change to acknowledgement), `hf_partner_refs{partner,sync_state}` gauge. Alert `PartnerSyncBacklog` when the `partners` queue depth stays over 500 for 10 minutes.

## What we would do differently

Start with events. The v1 comparison seemed safer ("it converges") and it did converge, slowly, at the cost of phantom updates that annoyed the partner. The comparison is the right safety net and the wrong primary mechanism.
