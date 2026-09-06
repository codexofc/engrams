---
name: query-guidelines-analysts
description: How to query within the 20 GB quota: filter the partition column, marts before core, FINAL on ReplacingMergeTree, join sizes
type: feedback
status: active
verified: 2026-03-27
---

Written after the data team spent a week in February 2026 answering "my query exceeded the memory limit". The quotas are in [[clickhouse-cluster-layout]]; this note is how to live with them.

## Filter the partition column, always

Every fact table is partitioned by month of its business timestamp ([[partitioning-and-ttl]]). A `WHERE created_at >= '2026-01-01'` lets ClickHouse skip partitions; a `WHERE toDate(created_at) >= ...` does too (the optimiser handles `toDate`), but `WHERE formatDateTime(created_at, '%Y-%m') >= '2026-01'` does not and reads 400 days, mostly from cold storage. Rule: compare the raw column with a literal.

## Use the marts first

`marts.load_daily`, `marts.bids_enriched`, `marts.pricing_daily`, `marts.invoice_mart` answer 80 % of questions and are pre-joined. `core` is for when the question is new. `raw` is for the data team; if you need `raw`, ask, and the answer is usually a column to add to `core`.

## FINAL

`core.loads`, `core.bids`, `core.invoices` are `ReplacingMergeTree` ([[dedup-replacing-merge-tree]]). Reading them without `FINAL` can count a row twice for a few hours after a change. Use the `_current` views (`core.loads_current`), which add `FINAL`, or write `FINAL` yourself. `FINAL` costs about 30 % on a partition-filtered query and much more without a partition filter, which is another reason for the first rule.

Do not put `FINAL` on `marts` tables; they are plain `MergeTree` rebuilt by partition and it is a no-op that confuses the next reader.

## Joins

- Loads to bids: local to the shard (both sharded by `load_id`), cheap.
- Invoices to loads: different sharding keys, the join goes through the network and materialises the right side on every node. Use [[invoice-mart]], which pre-joins them.
- Carrier attributes at the time of the fact: `marts.bids_enriched`, or the `ASOF JOIN` pattern from [[carrier-dimension-scd]] with carriers on the right side.
- Put the smaller table on the right side of a `JOIN`. ClickHouse builds a hash table of the right side in memory; a 40 M row right side is the memory limit error every time.

## Settings that help

At the top of the query:

- `SET max_memory_usage = 20000000000` is the ceiling, not something to set. Instead:
- `SET max_bytes_before_external_group_by = 8000000000` to let a big `GROUP BY` spill to disk.
- `SET join_algorithm = 'grace_hash'` for a join that does not fit.
- `SET max_execution_time = 120` on exploratory queries so a mistake costs two minutes, not five.
- `SET use_query_cache = 1` on a dashboard query that runs every minute with the same text.

## Sampling

`SELECT ... FROM core.bids SAMPLE 0.1` works on the fact tables (the sampling key is the hash of the primary id) and is the right way to develop a query before running it in full. Numbers from a sample must be multiplied back and labelled as estimates.

## Before asking for help

Run `EXPLAIN indexes = 1 <your query>` and look for `Parts: N/M`: if N is close to M, the partition filter is not applied. Paste that output with the question.
