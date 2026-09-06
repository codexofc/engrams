---
name: ingestion-kafka-to-clickhouse
description: ingest-svc reads Kafka topics into raw in 50 000-row batches with exactly-once via an offset table, 99 % landed within 90 s
type: reference
status: active
verified: 2026-04-24
---

## Sources

| Topic | Content | Rows/day | Format |
|---|---|---|---|
| `product.events.v2` | product analytics events with the standard envelope | 28 M | JSON |
| `cdc.app.loads`, `cdc.app.bids`, `cdc.app.invoices`, `cdc.app.carriers`, ... (22 topics) | change data capture from the application databases, one message per row change with `op` (`c`, `u`, `d`), `before`, `after`, `ts_ms` | 41 M total | JSON |
| `billing.payla.settlements` | settlement lines republished by billing after import | 60 k | JSON |
| `ml.predictions.eta`, `ml.predictions.price` | model outputs, for evaluation | 3 M | JSON |

## Consumer

`ingest-svc` (Rust) replaced the batch loader described in [[ingestion-legacy-batch]] in September 2025. One consumer group per topic family, one instance per Kafka partition (topics have 12 partitions), running on the two `ingest-1` and `ingest-2` hosts.

Batching: an insert into `raw.<topic>` every 50 000 rows or 5 seconds, whichever first. Inserts go to a local table on one shard chosen by the topic's sharding key, using `insert_deduplication_token` set to `<topic>-<partition>-<first_offset>-<last_offset>`.

## Exactly once

The offset is committed to `raw._offsets` (`topic`, `partition`, `offset`, `inserted_at`) in the same insert batch as the data, through a two-table insert that ClickHouse treats atomically within one block per table (it is not a transaction; what saves us is the deduplication token). On restart the consumer reads `raw._offsets` and seeks, and any re-inserted batch is dropped by the token. We validated it by killing the consumer 200 times in staging during a replay: 0 duplicates, 0 gaps.

The token only deduplicates within the replicated table's window (`replicated_deduplication_window`, set to 10 000 blocks per table, about 14 hours at peak). A replay older than that window would duplicate, which is why replays go through the backfill procedure ([[backfill-runbook]]) and not through a plain offset reset.

## Latency

`raw` freshness: 99 % of rows visible within 90 s of the Kafka timestamp, median 6 s. The 5-second batch window dominates. `core` and `marts` freshness depends on the model schedule; see [[freshness-slo]].

## Schema handling

`raw` tables have typed columns for the envelope (`event`, `occurred_at`, `actor_id`, ...) and a `payload String` column with the full JSON. A new property in an event needs no ingestion change; models extract it with `JSONExtract`. A change to the envelope does need a `raw` schema change through [[schema-migration-process]].

CDC rows with `op = 'd'` are kept as rows with `after` null; the `core` models turn them into `is_deleted` flags. Nothing is physically deleted in `raw` except by TTL and by the erasure pipeline ([[gdpr-erasure-pipeline]]).

## Monitoring

- `ingest.lag_seconds` per topic and partition, alert above 300 s.
- `ingest.batch_rows` and `ingest.insert_duration_ms`, an insert above 20 s means a merge storm on the shard.
- The daily row count reconciliation between Kafka (by offset arithmetic) and `raw` (by count) runs at 05:00 and posts the difference, which must be 0. It was not 0 on 2026-01-14, see [[duplicate-bids-incident-2026-01]].

## Sharding at ingestion

`ingest-svc` writes to local tables, not through a `Distributed` table, to keep the insert on one node and the deduplication token meaningful (a `Distributed` insert splits a block across shards and the token would be checked per shard against different data). The shard for a row is `cityHash64(<sharding key>) % 2`, with the key per topic in `topics.yaml`: `load_id` for loads, bids, quotes and load events; `carrier_id` for carrier and driver topics; `shipper_id` for invoices and payments. A batch of 50 000 rows is therefore split into two inserts, one per shard, each with its own token suffixed by the shard number.

Rows whose sharding key is null (about 0.01 %, malformed events from old app versions) go to shard 1 with `_shard_reason = 'null_key'` and are excluded by the models.

## Poison messages

A message that fails to parse (invalid JSON, missing envelope fields) is written to `raw._dead_letters` with the topic, partition, offset, the raw bytes and the error, and the consumer continues. 2 000 to 5 000 a day, almost all from a test harness of the mobile team that sends malformed events to staging topics that are mirrored to production by mistake; a ticket exists. A message that is valid but violates a type declared in the events registry (a string in a numeric property) is ingested with the property nulled and counted in `ingest.type_violations` per event name, which is the collector-side check the ML team asked for after their drift incident.

## Consumer configuration

- `fetch.max.bytes` 50 MB, `max.poll.records` 20 000, `session.timeout.ms` 45 000. The session timeout was 10 s and caused rebalances during long inserts; 45 s with a heartbeat thread is stable.
- `auto.offset.reset` is `error`, never `earliest` or `latest`: a consumer group without a committed offset in `raw._offsets` refuses to start. Bootstrapping a new topic is a deliberate command, `ingest-svc bootstrap --topic <t> --from <earliest|timestamp>`.
- One consumer group per topic family, named `wh-ingest-<family>-v2`; the `-v2` suffix dates from the consumer rewrite, and renaming the group is how a full replay into a side table is started ([[backfill-runbook]] describes the swap).

## Runbook: consumer stuck

Symptoms: `ingest.lag_seconds` rising on one partition only, `ingest.batch_rows` at zero for it.

1. `ingest-svc status` shows per partition the last offset committed and the last poll time.
2. If the last poll is recent and the offset does not move, the insert is failing: look for `Not enough space` or `Too many parts` in the log ([[partitioning-and-ttl]] for the parts case).
3. If the last poll is old, the instance is stuck on a poison message that escaped the dead-letter path; `ingest-svc skip --topic <t> --partition <p> --offset <o>` records the skip in `raw._skipped` and moves on. Used twice in 2026.
4. Never reset the group offset by hand in Kafka; the deduplication token will not save a replay older than the window.
