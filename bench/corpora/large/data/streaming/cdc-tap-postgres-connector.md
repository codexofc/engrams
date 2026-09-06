---
name: cdc-tap-postgres-connector
description: cdc-tap reads the API's PostgreSQL logical replication slot hf_cdc and produces one compacted topic per table under cdc.app.*, with before and after images, column filters, a heartbeat table against slot bloat, 2 instances active-passive, 60 tables, 4 000 changes/s peak
type: reference
status: active
verified: 2026-06-12
---

# cdc-tap, the PostgreSQL to torrent connector

## What it is

`cdc-tap` is the change data capture connector, a Rust service (5 000 lines, ours) that consumes the logical replication stream of the API's PostgreSQL primary through the `pgoutput` plugin and produces one message per row change to `cdc.app.<table>` on torrent ([[torrent-broker-overview]]). It replaced a general-purpose connector framework in 2024 that needed a JVM, a connector cluster, and a week per upgrade; the replacement did what we use in a fraction of the code and we can read all of it.

Two instances on `ops-tools`, active-passive with a lease in PostgreSQL (`cdc.leader`, 10 s TTL). The passive one is warm and takes over in under 30 s; a slot can have one reader, so there is no active-active.

## The replication slot

`hf_cdc`, a logical slot on the primary, publication `hf_cdc_pub` listing the 60 tables in `cdc-tap.yaml`. A slot holds WAL until the reader confirms it; a stopped reader means the primary's disk fills with WAL. This is the single most dangerous property of the setup, and three things guard it:

- `pg_replication_slots.confirmed_flush_lsn` lag in bytes, exported, `warn` at 5 GB, `page` at 20 GB (the primary has 400 GB free; 20 GB is about 40 minutes of peak WAL).

- A heartbeat: `cdc-tap` writes a row to `cdc.heartbeat` every 10 s, included in the publication, so that the slot always has something to confirm even when no business table changes (a quiet weekend night used to leave the slot idle and the WAL of unrelated databases on the same cluster piling up behind it).

- If the active instance cannot produce to torrent for 15 minutes (broker down), it keeps reading and confirming the slot and buffers to a local disk spool (up to 50 GB), rather than holding the slot. Losing the spool would lose changes; holding the slot would take the primary down. We chose the spool, and the spool is on RAID-1 and has never been used past 2 GB (the [[incident-2026-05-torrent-3-disk-full]] stop lasted 12 minutes).

## The message

Key: the primary key as JSON (`{"id": 99120033}`, or the composite for the four tables that have one). Value, per [[event-envelope-format]]:

```
{
  "op": "u",                      // c, u, d, r (r = snapshot read)
  "table": "loads",
  "lsn": "5F2/8A3B1C40",
  "tx_id": 88123441,
  "committed_at": "2026-06-12T07:14:03.221+00:00",
  "before": { ...full row or null... },
  "after":  { ...full row or null... }
}
```

`REPLICA IDENTITY FULL` on all 60 tables so that `before` is the full row, not only the key; the WAL cost was measured at +18 % and accepted because consumers that compute deltas (the status history projector, the billing reconciliation) need `before`. A delete has `after: null` and is followed by a tombstone (null value) on the same key so that compaction removes the row eventually.

Column filters: `cdc.app.users` excludes `phone`, `email`, `password`-anything and adds `email_hash`. Three other tables exclude a free-text notes column. The filters are in `cdc-tap.yaml` per table and the CI checks that every column of every published table is either listed as included or excluded, so that a new column is a conscious decision, not a default leak. Two columns have been caught by that check since 2025 (`carriers.iban` in a migration, `users.mfa_secret_ref` during the auth work).

## Compaction and consumers

`cdc.app.*` topics are compacted with `min.cleanable.dirty.ratio = 0.3` and `delete.retention.ms` of 7 days for tombstones ([[retention-and-compaction-policy]]). A consumer reading from the beginning gets the latest state of every live row plus whatever intermediate states compaction has not yet removed. The rule for consumers: take the message with the greatest `lsn` for a key, never assume partition order across a partition count change ([[partition-count-decisions]]). The client wrappers have a `CdcLatestState` helper that does this for the initial load.

## Snapshots

A new table added to the publication needs its existing rows in the topic. `cdc-tap snapshot --table <t>` reads the table in a repeatable-read transaction pinned to the slot's LSN (`pg_export_snapshot`), produces `op: r` messages for every row, then resumes the stream from that LSN. `loads` (3.2 M rows) snapshots in 9 minutes. Done 6 times in 2025 and 2026 for new tables; the ordering guarantee (snapshot rows before any stream change of the same row) held every time, which is what the LSN pinning is for.

## Schema changes

A migration that adds a column is picked up automatically (new field in `after`, schema registry version bumped by the CI from the table's information schema, `BACKWARD` compatible by construction, see [[schema-registry-compatibility-rules]]). A migration that drops or renames a column fails the CI's `cdc-schema-check` unless the column is marked deprecated in the topic schema first. This has blocked four migrations at the MR stage, which is the point.

## Numbers (June 2026)

| Metric | Value |
|---|---|
| tables published | 60 |
| changes per second, peak | 4 000 |
| end-to-end latency commit to topic, p50 | 180 ms |
| p99 | 900 ms (checkpoint pauses on the primary) |
| slot lag, normal | under 50 MB |
| failovers active to passive since 2025 | 7, all under 30 s, 4 planned |
| longest slot lag | 11 GB, during the May 2026 broker stop |

## Runbook lines

- Slot lag growing, `cdc-tap` healthy: torrent is slow or down, check the broker first, check the spool size second.

- Slot lag growing, `cdc-tap` unhealthy on both instances: restart the passive first, check it takes the lease. If neither takes it, the lease row may be held by a dead instance whose clock was ahead; `UPDATE cdc.leader SET expires_at = now()`.

- `cdc-tap` healthy, topic silent: check the publication includes the table (`\dRp+ hf_cdc_pub`), then check `REPLICA IDENTITY` (a table recreated by a migration loses it, the CI checks migrations for `CREATE TABLE` of a published name and has caught one).

- Never drop the slot to "fix" lag. Dropping the slot loses every change since `confirmed_flush_lsn`; the recovery is a snapshot of all 60 tables, 3 hours, and consumers see a burst of `op: r`. It has been done once, in 2024, and this line exists because of that.
