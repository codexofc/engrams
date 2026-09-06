---
name: backup-velero-schedule
description: Velero backs up cluster objects of hf-main every 6 h to the object store (retention 14 d) and takes a daily full including Longhorn volumes at 02:30 (retention 30 d), PostgreSQL is backed up by its operator separately, restore drill quarterly
type: reference
status: active
verified: 2026-04-17
---

# Backups with Velero

Velero runs on `ops-tools` and targets `hf-main` through the cluster registration, with the object store bucket `hf-velero-prod` as backend and the Longhorn CSI snapshotter for volume snapshots.

## Schedules

| Schedule | What | When | Retention |
|---|---|---|---|
| `objects-6h` | All namespaces, objects only (`snapshotVolumes: false`) | every 6 h at :15 | 14 days |
| `full-daily` | All namespaces except `platform-staging`, objects plus Longhorn volume snapshots | 02:30 daily | 30 days |
| `etcd-snapshot` (not Velero, RKE2 native) | etcd snapshot to the object store | every 6 h | 5 local, 30 days remote, see [[incident-2026-04-etcd-disk-full]] |

`platform-staging` is excluded from the full backup: it is rebuilt from Git and from the prod restore every night anyway.

## What Velero does not cover

- **PostgreSQL**. The operator's own WAL archiving and base backups, see [[postgres-operator-cloudnative]]. Velero would snapshot the local NVMe volume inconsistently. The `local-path` volumes are annotated `backup.velero.io/backup-volumes-excludes`.

- **The object store itself** (documents, tiles, backups). It has its own replication to a second appliance in the other rack, and a weekly offline copy handled by the datacentre provider under contract. This is the one place we rely on someone else.

- **Secrets' source of truth**: the vault on `ops-tools` is backed up hourly by its own snapshot mechanism to the same bucket. Velero backs up the projected Secrets on `hf-main` as objects, which is a convenience, not the source.

## Restore

`velero restore create --from-backup full-daily-20260416023000 --include-namespaces platform-prod` restores the namespace's objects and volumes. Tested cases:

- Whole cluster rebuild (rehearsal, June 2025): Velero restored `ops` and `kube-system` extras, then ArgoCD re-synced everything from Git. 40 minutes. The Velero part was mostly Longhorn volumes for RabbitMQ and Redis, which we could have discarded.

- Single namespace from a bad prune (the `platform-prod-web` Application once pruned a ConfigMap it should not have): `--include-resources configmaps --selector app=halden-web`, 30 seconds.

- The etcd snapshot restore is a different procedure (`rke2 server --cluster-reset --cluster-reset-restore-path`), rehearsed on the throwaway cluster before the [[rke2-upgrade-1-31-to-1-32]] upgrade.

## Drill

Quarterly, on the throwaway cluster built by the same playbooks as production: restore the latest `full-daily`, restore the latest PostgreSQL base backup with PITR to the same timestamp, point a staging-like API at it, run the smoke test. Last drill 2026-04-16 (also used to check the etcd incident's backups), 1 h 45 total, one finding: the `reflector`-copied TLS Secrets were restored before the source and the reflector overwrote them with a stale copy, harmless but noted.

## Monitoring

`velero_backup_last_successful_timestamp` older than 7 h for `objects-6h` or 26 h for `full-daily` pages. `velero_backup_failure_total` increase pages. The bucket usage is on the storage dashboard (currently 1.1 TB for Velero, 9 TB for PostgreSQL WAL and base backups).

## Things learned

- Longhorn CSI snapshots of a 400 GB volume (the registry mirror, see [[registry-harbor-mirror]]) take 25 minutes and were making the daily backup overlap the 03:00 CronJobs. The registry volume is now excluded, the mirror is rebuildable.

- Velero's default `--ttl` was 30 days on both schedules until someone noticed the `objects-6h` backups were 4× 30 × 40 MB for nothing. Retention is now per schedule as in the table.
