---
name: webhook-ordering-and-sequence-numbers
description: Webhooks are at-least-once and unordered; v3 carries a per-load sequence so integrators drop stale updates, with gaps, replays and load.snapshot
type: reference
status: active
verified: 2026-05-27
---

# Ordering: what we guarantee and what we do not

Short version: we do not guarantee order. Two events for the same load can arrive in the wrong order, and the same event can arrive twice. The integrator guide says this on the first page since HF-3102; before that it was in a footnote and people found out the hard way.

## Why order is not guaranteed

Deliveries are rows in `sys_outbox` picked in batches of 100 with `FOR UPDATE SKIP LOCKED`, sent concurrently (16 in flight per relay pod, two pods since June 2026). A `load.dispatched` and a `load.in_transit` created two seconds apart may be picked in the same batch and the second one may get its 2xx first. Add retries: if `load.dispatched` fails once and `load.in_transit` succeeds, the integrator sees `in_transit` then, 30 seconds later, `dispatched`.

Guaranteeing order would mean one in-flight delivery per subscription, serialised, with a slow customer endpoint blocking everything behind it. We measured it in staging: a 2-second endpoint turned a burst of 500 events into 17 minutes of backlog. Not acceptable for everyone because of one slow customer.

## The sequence number (v3)

Every `load.*` event in payload v3 ([[webhook-payload-versioning-v2-v3]]) carries `sequence`, a positive integer that increases with every event on that load. It is the `load_events` row's position for that load, assigned in the same transaction as the outbox row, so it reflects the order things actually happened, not the order they were sent.

Integrator rule: keep the highest `sequence` seen per `load_id`; drop any event with a lower or equal one. That gives a correct final state under reordering and under duplicates. It does not give them every intermediate state in order; if they need a full ordered history, they fetch `GET /v2/loads/{id}/events`, which is ordered.

`sequence` is per load, not global. Two loads' sequences say nothing about each other.

## Gaps

Gaps are normal. A load may have events that no subscription is interested in (a bid placed, a document uploaded) and each takes a sequence number. An integrator subscribed to `load.dispatched` and `load.delivered` may see 3 then 9. A gap is not a lost event. The guide has a paragraph on this because the first v3 integrator reported "missing events" that were gaps.

## Replays

A replay ([[webhook-replay-tool]]) carries the original `sequence`. The drop-if-not-higher rule handles it: if they processed it, it is dropped; if they missed it and a later event was processed, it is also dropped, correctly, because their state is already newer.

## Backwards transitions

Support corrections can move a load backwards (`DELIVERED` to `IN_TRANSIT`). The `sequence` still increases, so the integrator applies it and their state goes backwards too, which is right. `meta.reason` says why. An integrator who models the load as a monotonic state machine on their side must not.

## v2 subscribers

No `sequence` in v2 and we will not add it (v2 adds fields, but this one changes how you must process, and half of v2 integrators would ignore it and believe they were safe). The v2 advice: on any event, fetch the load and use the API's state as truth. It costs one GET per event and it is what most v2 integrators already did after their first reordering surprise.

## `load.snapshot`

Since HF-3109 an integrator can request, from the web screen or `POST /v2/webhooks/subscriptions/{id}/snapshot?load_id=`, a `load.snapshot` event with the full current state of a load and its current `sequence`. It is the reset button when their state is suspected wrong. Limited to 100 per hour per subscription.
