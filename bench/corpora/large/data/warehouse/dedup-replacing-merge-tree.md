---
name: dedup-replacing-merge-tree
description: core facts use ReplicatedReplacingMergeTree with the CDC timestamp as version, reads need FINAL, OPTIMIZE only after backfills
type: reference
status: active
verified: 2026-03-10
---

## Why ReplacingMergeTree

`core.loads`, `core.bids`, `core.invoices`, `core.payments` receive several CDC rows per business row (creation, then every update). The model keeps the latest by `cdc_ts_ms`, but marmot writes whole partitions ([[marmot-model-runner]]) and a partition can be written while another process (the erasure mutation, a manual insert) touches it. `ReplacingMergeTree(cdc_ts_ms)` guarantees that whatever ends up in the parts, the merge keeps the row with the highest version per sorting key. It is a safety net for the model's own deduplication, not a replacement for it.

The sorting key must therefore contain the business key: `ORDER BY (shipper_id, posted_at, load_id)` for loads works because `load_id` is in it; two rows with the same `load_id` but a different `posted_at` would not be deduplicated. `posted_at` never changes in the app, so it is fine; `awarded_at` would not have been.

## Reading

Until the background merge runs, both versions are in the table. Reading without `FINAL` can double count for minutes to hours. Three ways to read correctly:

1. `SELECT ... FROM core.bids FINAL WHERE ...`, with a partition filter, costs about 30 % more.
2. The views `core.bids_current`, `core.loads_current`, which are `SELECT * FROM core.bids FINAL`. Preferred for analysts.
3. `GROUP BY bid_id` with `argMax(col, cdc_ts_ms)` for every column, the old way, still faster on very large scans but unreadable; only in models, and only two models still do it.

The incident of [[duplicate-bids-incident-2026-01]] came from a mart reading `core.bids` without `FINAL`. A `marmot lint` rule now flags any model reading a `ReplacingMergeTree` table without `FINAL` or `argMax`.

## OPTIMIZE

`OPTIMIZE TABLE ... PARTITION ... FINAL` forces the merge. It is run after a backfill ([[backfill-runbook]]) on the backfilled partitions, and never on a schedule: on a 40 GB partition it takes 3 to 8 minutes and doubles the disk usage of the partition during the merge. During the October 2025 incident ([[disk-full-incident-2025-10]]) that was exactly the wrong thing to do on a 97 % full disk, and it was done, and it made things worse for 10 minutes.

## Settings

- `do_not_merge_across_partitions_select_final = 1` at the profile level for analysts: `FINAL` then only merges within a partition, which is correct for us (a business row never changes partition) and much faster.
- `replicated_deduplication_window = 10000` on `raw` tables for the insert token of [[ingestion-kafka-to-clickhouse]]; `core` tables keep the default, they are written by `REPLACE PARTITION`, not by inserts.

## What is not ReplacingMergeTree

`marts.*` are plain `MergeTree`, rebuilt by partition, no versions. `core.carriers` is `MergeTree` too: it is a full rebuild with `EXCHANGE TABLES`, so there is nothing to deduplicate. `raw.*` are `MergeTree` with the insert token as the only deduplication; duplicates beyond the window stay and are handled by the models.
