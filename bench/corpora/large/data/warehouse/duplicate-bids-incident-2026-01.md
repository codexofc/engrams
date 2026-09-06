---
name: duplicate-bids-incident-2026-01
description: Incident of 2026-01-14: a Kafka rebalance replayed 19 h of bids past the dedup window, 2.1 M duplicates, marts wrong for a morning
type: project
status: active
verified: 2026-02-03
---

## Timeline

- 2026-01-13 22:10 UTC: the platform team upgrades the Kafka brokers one by one. During the upgrade of the second broker, the consumer group for `cdc.app.bids` is rebalanced twice within a minute.

- 22:11: one `ingest-svc` instance, after the rebalance, gets partitions 4 and 9 but reads its start offset from a stale in-memory cache instead of `raw._offsets` (a code path that only ran when the rebalance happened before the first commit of the process, which had restarted 40 s earlier for the upgrade).

- 22:11 to 23:05: partitions 4 and 9 are re-read from the offset of 03:00 UTC the previous day, about 19 hours, 2.1 M rows. The deduplication token ([[ingestion-kafka-to-clickhouse]]) covers about 14 hours of blocks per table at that rate, so the oldest 5 hours of the replay are inserted a second time. Later rows are rejected by the token as intended.

- 23:10: the 10-minute `core.bids` model ([[bids-fact-model]]) recomputes the touched partitions. `core.bids` uses `ReplacingMergeTree` on `bid_id`, so the duplicates collapse there after merge, but the hourly `marts.pricing_daily` ran at 23:05 on the unmerged data without `FINAL` and counted 31 % more bids for 2026-01-12 and 2026-01-13.

- 2026-01-14 05:00: the daily Kafka-versus-raw row count reconciliation reports a difference of 2 104 331 rows on `raw.cdc_app_bids` and posts it. Nobody is awake; the alert was a message, not a page.

- 08:30: an analyst on the pricing team sees bids per load at 8.1 instead of 6.2 on the dashboard and asks. Diagnosis in 40 minutes from the reconciliation message.

- 09:40: duplicates removed from `raw` by `ALTER TABLE raw.cdc_app_bids DELETE WHERE ...` on the affected offsets range with a `_partition` and `_offset` filter (we keep both as columns for exactly this), `core.bids` and `marts.pricing_daily` backfilled for the two days ([[backfill-runbook]]). Dashboards correct at 10:20.

No decision was taken on the wrong figures; the pricing weekly review is on Tuesday and the incident was Wednesday morning.

## Root causes

1. The consumer's stale-cache path on rebalance. Fixed in `ingest-svc` 1.9.2 (HF-2431): the start offset is always read from `raw._offsets` on assignment, the in-memory cache is only used to skip a read when it is newer.
2. `marts.pricing_daily` read `core.bids` without `FINAL` on a `ReplacingMergeTree`. The general rule and its cost are in [[dedup-replacing-merge-tree]]; the model was one of four that did not follow it, all fixed the same week.
3. The reconciliation alert was informational. Now it pages the data on-call if the difference is above 10 000 rows on any table.

## What we did not change

The deduplication window stays at 10 000 blocks. Making it cover a full replay would need about 100 000 blocks per table and costs Keeper memory; the correct fix was the consumer, and any replay beyond the window goes through the backfill procedure, which handles duplicates by recomputing partitions from a deduplicated `raw` read.

## How the cleanup was done, step by step

Kept here because the next replay incident will need the same steps.

1. Identify the replayed range per partition from `ingest-svc` logs: `partition 4: replayed offsets 88 120 331 to 89 402 110`, `partition 9: 88 004 002 to 89 311 950`.
2. Count what the token had already rejected versus what was inserted twice: `SELECT _partition, count(), uniqExact(_offset) FROM raw.cdc_app_bids WHERE _partition IN (4, 9) AND _offset BETWEEN ... GROUP BY _partition`. The difference between `count()` and `uniqExact(_offset)` was the duplicate count, 2 104 331.
3. Delete the second copies. `ALTER TABLE ... DELETE` cannot express "keep one of two identical rows", so the duplicates were removed by rewriting the two partitions: `INSERT INTO raw.cdc_app_bids_fix SELECT DISTINCT ON (_partition, _offset) ...` into a side table with the same schema, then `REPLACE PARTITION` for `202601` and `202512` from the side table. 12 minutes on shard 1 (the affected partitions were on shard 1 only, since both Kafka partitions hashed there, which was a coincidence).
4. `marmot backfill --model core.bids --from 2025-12 --to 2026-01 --descendants`, 6 minutes.
5. `OPTIMIZE TABLE core.bids PARTITION 202601 FINAL` and the same for `202512`, then the marmot tests on both partitions.
6. Compare `marts.pricing_daily` bid counts for 2026-01-12 and 2026-01-13 with the counts from the application database (the pricing team ran the same count on `bid-svc`'s replica): equal to the row.

## Timeline of the fix in ingest-svc

The stale-cache code path: on partition assignment, the consumer looked up `self.last_committed[partition]` and used it if present, falling back to `raw._offsets` only when the map had no entry. After a restart, the map was empty, which was correct; but the process had *not* restarted for the second rebalance, so the map held the offsets from the *first* assignment, taken 40 s earlier from a different partition set, and one entry had been populated from a debug snapshot with a stale value. The fix (`ingest-svc` 1.9.2) reads `raw._offsets` on every assignment and only uses the in-memory value when it is greater.

A regression test in `ingest-svc` simulates two rebalances within a second with a stale map and asserts that the seek goes to the table's offset. The test was written before the fix and failed, which is how we know it tests the bug.

## Communication

The incident review was written the same day in the data channel and linked from the pricing channel, with the two-day figures marked as corrected on the dashboard by a note ("bid counts for 12 and 13 January were recomputed on 14 January 10:20"). The pricing weekly review of the following Tuesday used the corrected figures; nobody had made a decision on the wrong ones, but a report sent to the carrier sales team on the 14th at 08:00 had the wrong bids-per-load figure and was re-sent.
