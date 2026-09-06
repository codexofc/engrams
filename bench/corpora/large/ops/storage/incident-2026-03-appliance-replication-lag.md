---
name: incident-2026-03-appliance-replication-lag
description: March 2026, replication from stash-a to stash-b fell 9 hours behind over a weekend after a ClickHouse cold-part rewrite pushed 6 TB through the replication queue, the queue processes buckets in a single FIFO, fixed with per-bucket priority and a bandwidth reservation, no data lost, HF-4615
type: project
status: active
verified: 2026-04-10
---

# Incident 2026-03-14 to 03-15: replication lag between the appliances

## What happened

On Saturday 2026-03-14 at 02:00 the warehouse team ran a planned rewrite of `raw` cold parts to change the compression codec on the `payload` columns (LZ4 to ZSTD(3), the measure listed in their cost note). ClickHouse rewrites parts by writing new ones and dropping old ones: 6 TB of new objects into `hf-warehouse-cold-prod` over 10 hours, plus 9 TB of deletes.

The Stashbox replication from `stash-a` to `stash-b` ([[object-store-buckets-and-layout]]) is one queue for all buckets, in order of write. 6 TB at the replication link's effective 200 MB/s is 8.5 hours of queue. Every object written to any other bucket during that time waited behind the warehouse parts: documents uploaded by carriers on Saturday, the PostgreSQL WAL segments (one every few minutes), the hourly vault snapshots.

`ReplicationLagHigh` (`warn` at 15 min) fired at 02:40. `page` at 2 h fired at 04:20 and was acknowledged by the platform on-call as "the warehouse job, expected". The lag peaked at 9 h 10 at 11:30 and drained by 21:00 Saturday.

## Why it mattered

For 9 hours, `stash-b` did not have the latest WAL segments. Had `stash-a` failed at 11:00, a PITR from `stash-b` could have reached Saturday 02:00 at best: 9 hours of transactions recoverable only from the live PostgreSQL primary and its streaming replica, which are in the same rack A as `stash-a`. The "second copy in the other rack" promise of [[backup-inventory-and-retention]] was 9 hours stale for the most important line of the table, and the on-call had accepted it as expected.

Documents: 4 100 uploaded on Saturday were on one appliance only for up to 9 hours. Same reasoning.

Nothing failed, so nothing was lost. The incident is the exposure, not an outcome.

## Root cause

- One replication queue, FIFO across buckets. A bulk write to a low-value bucket delays the replication of high-value small objects. The appliance supports per-bucket replication priorities and a bandwidth reservation; neither was configured because the default had never been a problem at 40 GB a week of cold growth.

- The rewrite job was planned by the warehouse team and reviewed for warehouse impact (CPU, query latency, disk on the appliance), not for replication. The storage team was not asked; the job was in the calendar, which nobody on storage reads on Fridays.

- The `page` was acknowledged as expected without checking what was behind the warehouse parts in the queue. The alert text said "lag 2 h 05" and nothing about which buckets were waiting.

## Fixes (HF-4615)

- **Priorities**: `hf-pg-backups-prod`, `hf-vault-snapshots`, `hf-documents-prod` at priority 1; `hf-velero-prod`, `hf-torrent-snapshots`, `hf-ml-artifacts-prod` at 2; everything else at 3. The appliance schedules by priority then FIFO, so a WAL segment written during a bulk load replicates within its normal seconds.

- **Bandwidth reservation**: 50 MB/s reserved for priority 1 regardless of queue depth. Tested by re-running 1 TB of the rewrite in staging with WAL writes going: WAL lag stayed under 30 s.

- **Alert text** now lists the three oldest waiting objects' buckets and the lag per priority class. `ReplicationLagHigh{priority="1"}` pages at 15 minutes, not 2 hours; priority 3 pages at 6 hours and warns at 2.

- **Bulk writes to the object store above 1 TB are announced** in the storage channel and get a line in `halden-infra/storage/CHANGES.md`, whoever runs them. The warehouse team's runbook for part rewrites has the line. The next rewrite (April, 2 TB) was announced, throttled by ClickHouse's `max_replicated_sends_network_bandwidth` equivalent for cold writes to 100 MB/s, and lagged priority 3 by 40 minutes with priority 1 untouched.

- A weekly report of the replication lag distribution per priority class, because a 5-minute lag on WAL every Saturday would be a pattern worth seeing.

## Numbers

| | Value |
|---|---|
| objects written by the rewrite | 2.1 M, 6 TB |
| peak replication lag | 9 h 10 |
| time priority-1 buckets were behind by more than 15 min | 17 h (02:15 to 19:20) |
| WAL segments delayed | about 3 000 |
| documents delayed | 4 100 |
| data lost | 0 |
| replication throughput observed | 200 MB/s, link is 10 Gb/s, appliance-bound |

## What we did not do

Make replication synchronous. The appliance can, at the cost of every `PUT` waiting for `stash-b`, which doubles the write latency for documents (presigned uploads from phones on 4G would time out more) and couples the two appliances so that a `stash-b` maintenance stops writes. Asynchronous with priorities keeps the WAL within seconds and the documents within a minute, and the [[restore-drill-2026-05]] measured exactly that during its failover step.
