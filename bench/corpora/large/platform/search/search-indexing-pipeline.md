---
name: search-indexing-pipeline
description: How a load change reaches haystack: indexer on the load-events topic reads the current row, bulk-indexes with external versions, lag, rejects, nightly check
type: reference
status: active
verified: 2026-07-24
---

# Indexing pipeline: from PostgreSQL to haystack

The search index is derived data. The pipeline's job is to make `loads-write` reflect PostgreSQL's `loads` within seconds, never to be a source of anything.

## Components

1. **Source**: the `load-events` Kafka topic, already produced by the API for the data platform (one message per `load_events` row, key = `load_id`, so a load's events are ordered within a partition). 12 partitions. Field changes that do not create a `load_events` row (a shipper editing the goods description) are captured since HF-3012 by a `load.updated` event emitted by the entity listener, so the topic sees every change that matters to the index.

2. **`haystack-indexer`**: a Go service, one Deployment with 4 replicas in a consumer group, so 3 partitions each. It does not trust the event payload; it reads the **current** row from PostgreSQL (`pg-ro.hf.internal`, the read replica, with a fallback to the primary if the replica lags more than 5 s) and builds the document from that. An event is a signal that something changed, not the data.

3. **Bulk writer**: documents are accumulated per replica and flushed every 2 s or at 500 documents, whichever first, with the `_bulk` API to `loads-write`. Each document is indexed with `version = loads.version` (the row's optimistic-locking counter) and `version_type = external_gte`, so an older document can never overwrite a newer one, whatever order the flushes land in. This is what lets four consumers run without coordination.

4. **Deletes**: a load leaving the searchable states (`DISPATCHED`, `CANCELLED`, `INVOICED`, or `DRAFT` after unpublish) is not deleted immediately; the document is updated with `searchable = false` and `terminated_at`, and a daily job deletes documents with `terminated_at` older than 7 days. The "recently dispatched" panel in the carrier app reads the non-searchable ones for 7 days. Loads archived in PostgreSQL after 180 days are long gone from the index.

## Lag

Lag is measured two ways:

- Kafka consumer lag per partition (`hf_haystack_indexer_lag_messages`), the standard one.

- End to end: the document carries `indexed_at`; the indexer also writes `load_id` and `updated_at` of the newest row it processed into a Redis key per partition, and an exporter computes `now() - updated_at` as `hf_haystack_index_delay_seconds`. This is what the support playbook and the shipper's "indexed X seconds ago" read.

Normal: under 3 s at p99. Alert `HaystackIndexLag` at 30 s for 2 minutes (notify), 120 s for 2 minutes (page). The [[search-incident-2026-03-index-lag]] note is the one time the page fired.

Throughput at peak: about 120 events per second in, 60 documents per second out after the indexer coalesces multiple events for the same load within a flush window (it keeps the latest per `load_id` in the batch).

## Rejected documents

A `_bulk` item can fail: mapping conflict (a field with an unexpected type), a document too large, a version conflict (fine, ignored, that is `external_gte` doing its job). Real rejections are written to the `haystack-indexer-rejects` Kafka topic with the error, counted in `hf_haystack_indexer_rejects_total`, and alert above 10 per hour. The typical cause is a mapping change deployed to the indexer before the index ([[search-index-mapping-loads]] describes the order). Support's `hfctl load reindex <load_id>` re-emits a synthetic event for one load; if it is rejected again the reject topic has the reason.

## Nightly comparison

At 02:30, `haystack-reconcile` compares the set of searchable loads in PostgreSQL (`status IN ('OPEN','BIDDING') AND visibility <> 'PARTNER_ONLY'`) with the documents where `searchable = true`, by `load_id` and `version`. Missing or stale documents are re-emitted; extra documents (in the index, not searchable in PostgreSQL) are updated. Result posted to `#search`. Since April 2026 it finds 0 to 4 discrepancies per night, typically loads whose event landed during an indexer deploy.

## Full reindex

Handled by [[search-reindex-runbook]]: a `haystack-indexer --mode=backfill` reads `loads` in `id` order in batches of 2 000 and writes to a new physical index while the live consumers keep writing to the current one. Twelve minutes for the whole table.

## Failure modes and what happens to search

- Kafka unavailable: the indexer stalls, lag grows, search keeps serving stale results. Bids on stale results get `load_not_open` from the API, which the app handles.

- PostgreSQL replica lagging: fallback to primary after 5 s of replica lag; a metric counts fallbacks.

- haystack unavailable: the indexer retries the bulk with backoff (1 s to 60 s) and does not commit offsets; nothing is lost, everything is late. The search API falls back to the simplified PostgreSQL query with a banner.

- Indexer crash loop: offsets are committed only after a successful bulk, so a restart replays at most one flush window.

## Owners

The search pair (two people in the platform team). The indexer's on-call is the platform on-call; the runbooks are in this project.

## Operational checks, in order

When the lag alert fires or support reports a load missing from the index, this is the sequence, each step under a minute:

1. `kubectl -n search get pods -l app=haystack-indexer`: four pods running, restarts at zero. A crash loop shows here first.

2. Grafana "haystack", row "indexer": consumer lag per partition. One partition lagging and eleven fine means one consumer stuck (a poison message; check the reject topic). All twelve lagging means the sink is slow (haystack CPU or PostgreSQL replica).

3. `hf_haystack_indexer_pg_fallback_total` rising: the replica is lagging and the indexer is on the primary; tell the database on-call, the indexer itself is fine.

4. `hf_haystack_indexer_rejects_total` rising: mapping problem, read one message from `haystack-indexer-rejects` for the error text.

5. For one load: `hfctl load get <id>` shows `indexed_at` and `index_version`; `GET loads-read/_doc/<load_id>` (through the port-forward) shows what the index holds. If `version` in the document is lower than `loads.version`, the event was lost or is late; `hfctl load reindex` fixes one, the nightly comparison fixes all.
