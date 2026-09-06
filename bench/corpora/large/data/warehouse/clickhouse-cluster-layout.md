---
name: clickhouse-cluster-layout
description: The ClickHouse warehouse: 2 shards times 2 replicas, 3 Keeper nodes, databases raw, core, marts, scratch, roles, quotas, backups
type: reference
status: active
verified: 2026-05-08
---

## Topology

- Cluster name `hf_wh`: 2 shards, 2 replicas each. Hosts `ch-w-1a`, `ch-w-1b` (shard 1), `ch-w-2a`, `ch-w-2b` (shard 2). Each host: 32 vCPU, 128 GB RAM, 4 TB NVMe for hot data, plus an object storage tier for cold parts (see [[partitioning-and-ttl]]).
- Keeper: 3 nodes `ch-k-1` to `ch-k-3`, dedicated, 4 vCPU each. Keeper on the data nodes was the cause of the October 2025 incident ([[disk-full-incident-2025-10]]), do not go back to that.
- ClickHouse 25.3 LTS since April 2026. Upgrades follow the LTS line only, one replica at a time, shard 2 first (it carries less of the marts traffic).

## Databases

| Database | Content | Who writes |
|---|---|---|
| `raw` | events and change-data-capture rows as received, one table per source topic, kept 400 days | ingestion only, see [[ingestion-kafka-to-clickhouse]] |
| `core` | cleaned, deduplicated, typed tables: `core.loads`, `core.bids`, `core.invoices`, `core.carriers`, dimensions | marmot models, see [[marmot-model-runner]] |
| `marts` | analyst-facing aggregates and wide tables: `marts.load_daily`, `marts.pricing_experiments`, `marts.invoice_mart` | marmot models |
| `scratch` | analysts' own tables, dropped after 30 days without a query | anyone with the analyst role |

## Sharding

Sharding key is `cityHash64(load_id)` for anything keyed by load, `cityHash64(carrier_id)` for carrier-centric tables. Joins between loads and bids are therefore local to a shard. Invoices are sharded by `shipper_id`, which means an invoice-to-load join crosses shards; the invoice mart pre-joins them for that reason ([[invoice-mart]]).

`Distributed` tables exist only in `marts` and are what analysts query. `raw` and `core` are queried through the `Distributed` wrappers `core.loads_all` etc., but the models write to local tables on each shard through the ingestion path.

## Engines

`ReplicatedMergeTree` for facts, `ReplicatedReplacingMergeTree` where the source can resend rows ([[dedup-replacing-merge-tree]]), `ReplicatedAggregatingMergeTree` for the two materialized-view aggregates that survived ([[materialized-views-pitfalls]]). Dictionaries for geo and calendar lookups ([[geo-dictionaries]]).

## Access

- Roles: `ingest` (insert into raw), `modeler` (all on core and marts), `analyst` (select on core and marts, all on scratch), `readonly_app` (select on a handful of marts tables used by the product dashboards).
- Quotas: analysts are limited to 20 GB of memory per query and 300 s; `readonly_app` to 2 GB and 10 s. The quota errors are the second most frequent support question of the data team, the query guidelines note ([[query-guidelines-analysts]]) exists for that.

## Capacity (May 2026)

- 11.4 TB compressed on hot storage across the cluster (2.85 TB per node), 31 TB on the cold tier.
- Ingestion 4 200 rows per second average, 18 000 at the morning peak.
- 6 100 analyst queries per day, median 0.4 s, p95 9 s.

## Network and ports

Native protocol on 9000 between nodes and for `ingest-svc`, HTTP on 8123 for the analysts' SQL client and the dashboards, Keeper on 9181. The cluster is reachable only from the office network, the CI, and the two application subnets that run `ingest-svc` and the dashboard backend; there is no public endpoint. TLS on the HTTP port with an internal certificate; the native port is plain inside the private network, which the platform team reviewed and accepted in 2025.

Load balancing for analysts: the SQL client connects to `wh.internal` which round-robins over the four data nodes; a query on a `Distributed` table from any node fans out to one replica per shard, chosen by the `nearest_hostname` policy so that shard 1 queries prefer `ch-w-1a` from `ch-w-1a`. Dashboards use `ch-w-2a` and `ch-w-2b` explicitly to keep the marts traffic off shard 1, which carries the bigger `raw` inserts.

## Replication details

Replication is asynchronous through Keeper: an insert on `ch-w-1a` is visible on `ch-w-1b` after fetch, typically under 2 s, up to a minute during a merge storm. Models that write on one replica and read on another in the same run (it happened with a temporary table in `scratch`) see stale data; the rule is `insert_quorum = 2` for `core` and `marts` writes, so a write returns only when both replicas have the part. The cost is about 15 % on write latency and it removed a class of "the mart is missing yesterday's partition on one replica" tickets.

`system.replication_queue` is scraped every minute; an entry older than 10 minutes alerts, which catches a replica that lost its Keeper session.

## Backups

Nightly at 01:00, incremental, to the object storage bucket `wh-backup`, using the native backup command on one replica per shard (`ch-w-1b` and `ch-w-2b`, the ones that do not serve dashboards). 35 days kept. A full restore was rehearsed in December 2025 on a scratch cluster: 4 h 10 for `core` and `marts`, `raw` not restored because it is replayable from Kafka within its 400-day window... which is not true beyond the Kafka retention of 14 days, so `raw` is in the backup too since January 2026, and the restore rehearsal is due again.

## Upgrade procedure

1. Read the release notes of the LTS target for changed defaults; the 24.8 to 25.3 upgrade changed the default of `allow_experimental_analyzer`, which broke two models with ambiguous aliases, found in staging.
2. Upgrade the staging single node, run `marmot run --full-refresh` on a 1 % sample, run all tests.
3. Production: `ch-w-2b`, wait a day, `ch-w-2a`, wait a day, then shard 1 the same way. Keeper nodes last, one at a time, on a separate week.
4. Never during a month close or on a Friday ([[warehouse-team-preferences]]).

The whole thing takes two weeks of calendar time for a few hours of work, and that is fine.
