---
name: partitioning-and-ttl
description: Fact tables partitioned by month, TTL moves parts to cold storage at 90 days, under 300 parts per partition, mutation rules
type: reference
status: active
verified: 2026-04-03
---

## Partitioning

`PARTITION BY toYYYYMM(<business timestamp>)` on every fact table: `posted_at` for loads, `created_at` for bids, `issued_at` for invoices, `occurred_at` for events in `raw`. Never by day: daily partitions on a 400-day `raw` table gave 400 partitions times a few dozen parts each and the merge scheduler could not keep up (October 2025, see [[disk-full-incident-2025-10]], where too many parts was a contributing factor).

The business timestamp and not `received_at`, so that a late-arriving event lands in the partition of the month it belongs to and the model recomputation described in [[late-arriving-events]] finds it there.

## Storage tiers

Storage policy `tiered`: `hot` volume (local NVMe) and `cold` volume (object storage bucket `wh-cold`). TTL rules on every fact table:

```
TTL <ts> + INTERVAL 90 DAY TO VOLUME 'cold',
    <ts> + INTERVAL <retention> DELETE
```

with the retention per table from [[retention-rules]]. Moves happen at merge time or by the background TTL task (`merge_with_ttl_timeout` 4 hours). Queries over cold parts are 5 to 15 times slower on first read and cached locally afterwards (`cold` cache of 500 GB per node). The analyst guidelines say to always filter on the partition column for that reason ([[query-guidelines-analysts]]).

## Parts

- Target: fewer than 300 active parts per partition on any table. `system.parts` is scraped every 5 minutes and `wh.parts_per_partition_max` alerts above 250.
- The main cause of part explosion is small inserts. Ingestion batches at 50 000 rows or 5 s ([[ingestion-kafka-to-clickhouse]]); analysts inserting into `scratch` row by row from a notebook have hit the `too many parts` error twice, which is by design (`parts_to_throw_insert` at 300).
- `OPTIMIZE TABLE ... PARTITION ... FINAL` is not run on a schedule. It is run by hand after a backfill ([[backfill-runbook]]) on the backfilled partitions, because a `REPLACE PARTITION` leaves the partition unmerged.

## Dropping partitions

`ALTER TABLE ... DROP PARTITION` is the only fast delete. `ALTER TABLE ... DELETE` (a mutation) rewrites the affected parts and is used only for surgical removals like the erasure pipeline ([[gdpr-erasure-pipeline]]) and incident cleanup. A mutation on a 400-day `raw` table takes 2 to 6 hours and must be run on one shard at a time with `mutations_sync = 0` and watched through `system.mutations`.

## Checking a table

```
SELECT partition, count() AS parts, sum(rows) AS rows,
       formatReadableSize(sum(bytes_on_disk)) AS size, any(disk_name) AS disk
FROM system.parts
WHERE database = 'core' AND table = 'bids' AND active
GROUP BY partition ORDER BY partition DESC
```

This is the first query to run when a model is slow: a partition with 800 parts or a current month sitting on `cold` (it happened once after a clock problem set `created_at` in the past) explains most of it.
