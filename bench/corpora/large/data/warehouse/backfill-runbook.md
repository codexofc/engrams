---
name: backfill-runbook
description: Runbook for recomputing partitions with marmot backfill, descendants flag, OPTIMIZE after, and the checks before announcing it done
type: reference
status: active
verified: 2026-03-19
---

A backfill recomputes one or more monthly partitions of one or more models from their sources. It is the answer to a bug in a model, a bad source (duplicates, wrong FX rate), or a new column that must exist in history.

## Before

1. Write down which partitions and why in a ticket. Note the row counts before: `SELECT count() FROM core.bids WHERE toYYYYMM(created_at) = 202511`.
2. Check the sources are clean for the range. If `raw` has duplicates ([[duplicate-bids-incident-2026-01]]), clean `raw` first; a backfill reads `raw` as is.
3. Check nobody else is backfilling: `SELECT * FROM marmot._runs WHERE kind = 'backfill' AND finished_at IS NULL`.
4. Announce in the data channel with the expected duration. A month of `core.bids` takes 40 s; `core.loads` 25 s; `marts.invoice_mart` 3 min; the full 400 days of everything about 6 hours.

## Run

```
marmot backfill --model core.bids --from 2025-11 --to 2026-01 --descendants
```

- `--descendants` recomputes every model downstream in the DAG for the same partitions. Almost always what you want; without it `marts` keeps the old figures and someone asks why the dashboard and the table disagree.
- Order is handled by marmot: sources first. If `raw` itself must be re-read from Kafka (rare), that is not a backfill, it is a replay, done by the ingestion owners with an offset reset on a dedicated consumer group writing into a `raw.<topic>_replay` table, then swapped by partition.
- Backfills run with `max_threads = 8` and a memory cap of 40 GB, so that the 10-minute runs keep going alongside. A backfill of more than 3 months is run at night.
- A partition is written by `REPLACE PARTITION` from a temporary table, so readers never see a half-written partition.

## After

1. `OPTIMIZE TABLE core.bids PARTITION 202511 FINAL` on each backfilled partition of `ReplacingMergeTree` tables, otherwise the partition sits with duplicates until the background merge, which can take hours ([[dedup-replacing-merge-tree]]). One partition at a time.
2. `marmot test --model core.bids --partition 202511`.
3. Compare the row count with the one noted before. A difference must be explained in the ticket (deduplication removed N rows, the fix added the M rows that were missing).
4. Tell the data channel it is done, with the partitions and the ticket.

## Example: the CZK FX correction of 2026-03-18

Bids in CZK before 2026-03 had `amount_eur_cents` computed with a placeholder rate ([[bids-fact-model]]). Fix: load the historical CNB rates into `dict.fx_rates`, then `marmot backfill --model core.bids --from 2025-09 --to 2026-02 --descendants`. 6 partitions, 5 minutes for `core.bids`, 22 minutes for the descendants (the pricing marts recompute per-lane medians). Row counts unchanged, 3 140 rows with a changed `amount_eur_cents`, checked with a `scratch` copy taken before.

## What a backfill does not fix

A wrong `raw` row stays wrong. A model bug in the *current* code must be fixed and deployed first, otherwise the backfill recomputes the bug. And a backfill of `core` does not touch the ML feature snapshots, which the ML team materialises separately; tell them.
