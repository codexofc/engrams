---
name: schema-migration-process
description: Warehouse schema changes as numbered SQL files applied ON CLUSTER, additive any day, drops after 14 days deprecation, no raw DDL by day
type: reference
status: active
verified: 2026-02-11
---

## Files

`migrations/NNNN_<slug>.sql` in the warehouse repository, applied in order by `wh-migrate` (a 300-line script that records applied files in `admin.migrations`). Every statement uses `ON CLUSTER hf_wh` so both shards and all replicas get it; `wh-migrate` refuses a DDL without it.

A migration is paired with the model change that needs it, in the same merge request, and CI runs `marmot lint` against the migrated schema on a staging cluster (a single-node ClickHouse with the same DDL, data sampled at 1 %).

## What ships when

| Change | Rule |
|---|---|
| add a column, add a table, add a dictionary attribute | any day, during the day, no announcement |
| change a column type | add a new column, backfill it ([[backfill-runbook]]), switch the models, drop the old one later; never `MODIFY COLUMN` on a fact table, it rewrites every part |
| drop a column or a table | 14 days after it is marked `deprecated` in `models/registry.yaml`; the weekly digest lists deprecated objects and who queried them in the last 30 days (from `system.query_log`) |
| change a sorting key or partition key | new table, backfill, `EXCHANGE TABLES`; treated as a project, not a migration |
| change a TTL | migration, with a line in [[retention-rules]] |
| anything on `raw.*` | outside 07:00 to 20:00 CET, because the DDL takes a lock that pauses inserts for a few seconds per part and the morning peak turns that into a lag alert |

## Adding a column to a fact table

The common case, for example a new surcharge column on `core.bids` ([[bids-fact-model]]):

1. Migration: `ALTER TABLE core.bids ON CLUSTER hf_wh ADD COLUMN surcharge_customs_cents Int64 DEFAULT 0 AFTER surcharge_multi_stop_cents`. A `DEFAULT` makes the column exist on old parts without a rewrite.
2. Model: read the new field in `models/core/bids.sql`.
3. Backfill the partitions where the source has the data, or none if the field is new in the app too.
4. Registry: add the column with a description in `models/registry.yaml`; the analysts' catalogue page renders from it.

## Rollback

Migrations have no down file. A wrong additive migration is reverted by a new migration that drops what was added; a wrong drop is a restore from the 35-day backups into a side table and a `REPLACE PARTITION`. It happened once (a `scratch` table with two months of manual annotations dropped by the stale-table job, restored in an hour).

## Who

Any member of the data team merges a migration after one review. Migrations on `core.loads`, `core.bids` and `marts.invoice_mart` need a second reviewer from the team that consumes them (pricing, billing), because their definitions are contracts, see [[invoice-mart]].
