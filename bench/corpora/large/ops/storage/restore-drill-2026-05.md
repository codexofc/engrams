---
name: restore-drill-2026-05
description: May 2026 drill rebuilt a working platform from backups alone in 5 h 10: appliance failover, vault, PITR, Velero, batch version restore, 6 findings, HF-4620
type: project
status: active
verified: 2026-06-04
---

# Restore drill, 2026-05-13

## Purpose

Quarterly drill, this one bigger than usual: the question was "if rack A is a hole in the ground on Monday morning, what do we have by Monday evening", with the scope of every line in [[backup-inventory-and-retention]] except the warehouse cold parts (31 TB, three days from offsite, planned for November). Six people, one day, the throwaway cluster `hf-drill` built by the production playbooks on the three spare nodes, and `stash-b` as the only object store (the drill pretends `stash-a` is gone).

## Sequence and timings

| Step | Started | Duration | Result |
|---|---|---|---|
| appliance failover: VIP to `stash-b`, promote replicated buckets to writable | 08:30 | 15 min | ok, one bucket (`hf-exports-prod`) had replication disabled, see finding 1 |
| vault restore from `hf-vault-snapshots` (latest hourly) to a fresh vault on `hf-drill` | 08:50 | 25 min | ok, secrets readable, unseal keys from the custody envelopes ([[backup-encryption-and-key-custody]]) |
| etcd not restored (fresh cluster), Velero restore of `platform-prod` objects from `objects-6h` | 09:20 | 20 min | ok, 1 400 objects, 3 warnings on reflector-copied secrets (known) |
| PostgreSQL PITR of the main cluster to 2026-05-13T06:00:00Z on `hf-drill` | 09:40 | 2 h 05 | ok, 5 h of WAL at 1.2 GB per 10 min plus base restore |
| Velero volume restore of RabbitMQ and Redis (Longhorn) | 09:45 | 30 min | ok, in parallel with PITR |

### Sequence, continued: documents, warehouse, registry

| Step | Started | Duration | Result |
|---|---|---|---|
| documents: batch version restore of 5 000 keys on `hf-documents-staging` after deleting them | 10:20 | 12 min | ok, from the runbook without the vendor doc |
| ClickHouse hot restore from the daily snapshot into a 1-node warehouse | 10:40 | 1 h 30 | ok for `core` and `mart`, `raw` restored partially by design |
| schema registry and CDC state from `hf-torrent-snapshots` | 11:50 | 10 min | ok |

### Sequence, end: applications and tear-down

| Step | Started | Duration | Result |
|---|---|---|---|
| API, auth-svc, dispatch tool pointed at the restored stores, smoke test | 12:00 | 40 min | ok after finding 3 |
| end-to-end: log in, publish a load, bid, assign, upload a document, invoice | 12:40 | 30 min | ok |
| tear down, VIP back, buckets back to replica mode | 13:20 | 20 min | ok |

Total to a working platform: 5 h 10 (08:30 to 13:40 with the smoke test). The 2025 drill on the same scope minus the appliance failover took 6 h 30; the difference is the PITR runbook rewritten after [[restore-2026-02-postgres-pitr-billing]] and the batch restore command learned in [[restore-2025-11-documents-prefix-deleted]].

## Findings

1. **`hf-exports-prod` was not replicated to `stash-b`.** Created in 2025 by hand before bucket creation went through the CI, replication never enabled. 90 GB of customer exports with 7-day retention, so the loss would have been small, but the inventory said "replicated" for every prod bucket and the inventory was wrong. Fixed the next day; the CI now refuses a `-prod` bucket without replication, and a nightly job compares the replication configuration of every bucket to the declared one.

2. **Promoting replicated buckets is a per-bucket click** in the appliance UI or one `stashctl` call per bucket. 14 buckets, 15 minutes, error-prone under stress. A script `storage-failover.sh` now does all of them from the bucket list in Git, with a dry run.

3. **`auth-svc` refused to start against the restored PostgreSQL** because its JWT signing key reference in the vault pointed at a key version that the hourly vault snapshot did not have yet (the key had been rotated at 07:00, the snapshot used was 06:00, the PITR target was 06:00 too, so the database and the vault were consistent with each other but the running `auth-svc` config had the newer reference from the Velero object restore taken at 06:15). Consistency between four stores restored to "about the same time" is not automatic. Fix: the drill and the runbook now pick one target timestamp and choose every snapshot as the latest *before* it, and `auth-svc` accepts the previous key version for 24 h after a rotation anyway (it did not; it does now).

4. **The WAL replay rate** was 1.2 GB per 10 minutes again. It is now the planning number in the runbook, with the base backup at 13:00 (since March) halving the worst case.

5. **ClickHouse `raw` tables**: the daily snapshot covers `core`, `mart`, `feat` and the last 7 days of `raw`; older `raw` is on cold storage which was out of scope. The restored warehouse could serve dashboards but could not rebuild a projection from more than 7 days of raw history. Accepted for a rack loss (the cold parts are on `stash-b` too, just not exercised); the November drill restores them.

6. **The reflector-copied TLS secrets** warning from the 2025 drill is still there. Harmless. Still noted.

## What we did not need

The offsite copy ([[offsite-weekly-copy-contract]]). Everything came from `stash-b`. That is the expected outcome for a rack loss; a datacentre loss is the scenario where the offsite copy matters and it is a different drill (a restore of one week-old bucket from the offsite service, done in 2025 for `hf-vault-snapshots` only, 4 hours including the courier).

## Costs

One day for six people, three spare nodes for a day, 2 TB of transient writes to `stash-b`. Nothing else. The drill found one silent gap (finding 1) that no monitoring had, which is the usual return on a drill: the backups you have are the ones you have restored.

## Next

November 2026: cold parts from offsite (three days, planned as a background job with checkpoints), and the datacentre-loss scenario for the vault and PostgreSQL from the offsite service. Capacity for the transient restore space is in [[storage-capacity-plan-2026]].
