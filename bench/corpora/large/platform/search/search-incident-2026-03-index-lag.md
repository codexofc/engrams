---
name: search-incident-2026-03-index-lag
description: March 2026, 14 minutes of index lag from 3 000 bid.placed events a minute; fixed by coalescing per load and partial updates for bid_count
type: project
status: active
verified: 2026-04-03
---

# Incident 2026-03-24: fourteen minutes of index lag (HF-3115)

## Timeline (UTC)

- 06:40 A large shipper's integration publishes 1 200 loads for the week in 4 minutes (a Monday import through the API). Normal, seen before.

- 06:45 Carriers' bidding integrations react. Three carriers run bots that bid on every load matching their lanes within seconds of publication. About 3 000 `bid.placed` events per minute on the `load-events` topic for 12 minutes.

- 06:47 `HaystackIndexLag` notify (30 s). 06:51 page (120 s). Lag climbs to 14 minutes by 07:20.

- 06:55 On-call sees the indexer consumers at 100 % CPU and the read replica at 80 %, all from `SELECT ... FROM loads WHERE id = $1` with joins, 60 per second per consumer.

- 07:05 Indexer scaled from 4 to 8 replicas. Kafka has 12 partitions, so 8 consumers get 1 or 2 each; throughput up about 60 %. Lag stops growing, starts to fall.

- 07:50 Lag under 10 s. Scaled back to 4 at 09:00 with no regression.

## Effects

For about an hour, carriers searching saw loads that already had many bids shown with `bid_count = 0`, and loads dispatched in the meantime still shown. Bids on those got `load_not_open`. Support had 9 tickets tagged `load:search`. No data wrong in PostgreSQL, everything wrong in the index for an hour.

## Root cause

The indexer treated every event the same: read the full load row with joins, build the full document, index it. A `bid.placed` event changes exactly one indexed field (`bid_count`), yet cost the same as a `load.published`. Three bots bidding on 1 200 loads produced 3 600 events, each a full read and a full document; with duplicates for the same load in the same second, the indexer built the same document several times per flush and sent them all (the `external_gte` version made them harmless but not free).

The design had assumed events were dominated by state transitions, a few per load. Bids are not a few per load.

## Fixes

- **Coalescing per load within the flush window.** The indexer keeps a map `load_id -> latest event` for the current batch; a load touched five times in 2 s is read once and indexed once. Shipped 2026-03-26. Under the same burst replayed on staging, documents out went from 3 600 to 1 200 per minute.

- **Partial updates for bid events.** `bid.placed` and `bid.withdrawn` no longer rebuild the document; the indexer sends an `update` with a script incrementing or decrementing `bid_count` and setting `version` from the event, no PostgreSQL read at all. Shipped 2026-04-01. This was debated: partial updates bypass "read the current row" and could drift; the nightly comparison ([[search-indexing-pipeline]]) checks `bid_count` too since then and has found no drift.

- **Autoscaling** on consumer lag: 4 to 8 replicas when lag exceeds 20 s for 1 minute, back to 4 after 10 minutes under 5 s. Fired twice since, both Monday mornings, both resolved in under 3 minutes.

- The read replica query got a covering index for the indexer's exact column set (`idx_loads_indexer`), taking the read from 6 ms to 1.5 ms.

## What we learned

- Event volume follows the noisiest actor, not the average one. Three bots defined our peak.

- "Read the current row" is the right principle for correctness and the wrong one for every event type. Partial updates are fine when the field is additive and the nightly comparison checks it.

- The 120 s page threshold was right; it gave the on-call 15 minutes before the lag was visible to many carriers. The 30 s notify was noise that morning and useful on quieter days; kept.

## Figures

Peak lag 14 min 20 s. Duration over 30 s: 63 minutes. 9 tickets. Since the fixes, the highest lag observed is 41 s (a haystack rolling upgrade in June), see [[search-query-latency-slo]] for the monthly figures.
