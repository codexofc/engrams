---
name: incident-2026-04-etcd-disk-full
description: April 2026, etcd on hf-cp-02 filled its 480 GB NVMe with WAL and snapshot files after auto-compaction silently stopped, the cluster went read-only for 22 minutes, fixed with defrag, compaction settings, and a dedicated etcd disk alert
type: project
status: active
verified: 2026-04-30
---

# Incident 2026-04-14: etcd disk full

## Impact

From 03:12 to 03:34 UTC the API server refused writes (`etcdserver: mvcc: database space exceeded`). Running pods kept running, so the application stayed up for users, but nothing could be scheduled, no secret rotation, no HPA scaling, and the 03:15 CronJobs (partition maintenance, document purge) did not start. ArgoCD showed every application as `Unknown`. No user-visible outage. It was 22 minutes from a 2-week-old cause that had been visible on a dashboard nobody looked at.

## Timeline (UTC)

- 03:12 `KubeAPIErrorsHigh` fires (warn, not page). `EtcdNoLeader` does not fire, there was a leader.

- 03:18 The ops on-call is paged by `ArgoCDAppsUnknown` (page, because 40 apps at once). Sees `mvcc: database space exceeded` in the API server logs.

- 03:20 `etcdctl endpoint status` shows `DB SIZE 8.1 GB` on all three members and `hf-cp-02` at 100 % disk. The other two at 60 %.

- 03:22 `etcdctl alarm list` shows `NOSPACE`. `etcdctl compact <rev>` then `etcdctl defrag` on each member, one at a time. Defrag on `hf-cp-02` takes 4 minutes.

- 03:31 `etcdctl alarm disarm`. Writes resume at 03:34.

- 03:40 CronJobs re-triggered by hand. ArgoCD back to `Synced` by 03:50.

## Root cause

Two things stacked.

1. RKE2 configures etcd auto-compaction (`--auto-compaction-retention=...`) through its own config. A manual `etcd-arg` override added during the [[rke2-upgrade-1-31-to-1-32]] work on 2026-03-30 to tune `--quota-backend-bytes` up to 8 GB **replaced** the whole `etcd-arg` list instead of adding to it, and dropped the auto-compaction setting. Nobody noticed because the cluster kept working. Revisions accumulated for two weeks. The database grew to the 8 GB quota and hit `NOSPACE`.

2. `hf-cp-02` was the member whose NVMe also hosted the RKE2 snapshot directory (`/var/lib/rancher/rke2/server/db/snapshots`), with snapshots every 6 hours and a retention of 20. 20 × 8 GB snapshots did not fit with the live database and the WAL. The other two members kept 5 snapshots because the retention setting had been rolled out with a per-node override in 2025 and `cp-02` had been rebuilt since without it.

So: a config override that clobbered a list, and a node that drifted from the others.

## Fixes

- `etcd-arg` in the RKE2 config is now managed as a complete list in `halden-infra/clusters/hf-main/rke2-config.yaml` and rendered onto the nodes by the node provisioning playbook, with `auto-compaction-mode: periodic`, `auto-compaction-retention: 1h`, `quota-backend-bytes: 8589934592`. The playbook diffs the running config against the desired one every night and reports drift.

- Snapshot retention set to 5 on all control-plane nodes, snapshots moved to the object store (`rke2 etcd-snapshot` S3 config) with local retention of 2.

- New alerts, both `page`: `EtcdDbSizeHigh` (`etcd_mvcc_db_total_size_in_bytes > 0.8 × quota`) and `EtcdDiskUsageHigh` (node filesystem of the etcd mount over 75 %). The generic node disk alert existed but at 90 %, and it was `warn`.

- The etcd dashboard now has "DB size vs quota" as its first panel, and the on-call handover checklist has a line for it.

- `etcdctl defrag` scheduled weekly on Sunday 04:00, one member at a time, from a CronJob on `ops-tools` that uses the etcd client certificates from the control-plane nodes' secret sync. Defrag on a healthy member takes 20 s.

## What we checked afterwards

- No data loss: etcd's `NOSPACE` alarm blocks writes before corruption, that is its purpose.

- The Velero backups (see [[backup-velero-schedule]]) from the two days before were restorable, tested on 2026-04-16 on a throwaway cluster.

- Whether the `PostSync` health checks of ArgoCD would have paged earlier: no, they run only on sync, and nothing synced at 3 in the morning.

## Lesson

An override that replaces a list is not the same as one that adds to it, and RKE2's config file does the former. Every `*-arg` key in the RKE2 config is now written out in full in Git, with a comment above each line saying what the default was. See [[rke2-cluster-layout]] for where etcd lives.

## The numbers, and how to read them today

What the members looked like at 03:20, from `etcdctl endpoint status --write-out=table`:

| Member | DB size | In use | Revisions kept | Disk |
|---|---|---|---|---|
| `hf-cp-01` | 8.1 GB | 7.9 GB | about 14 M | 61 % |
| `hf-cp-02` | 8.1 GB | 7.9 GB | about 14 M | 100 % |
| `hf-cp-03` | 8.1 GB | 7.9 GB | about 14 M | 58 % |

After compaction to the current revision and defrag: 310 MB in use per member. The cluster's real working set is a few hundred megabytes; the 8 GB was two weeks of history for about 12 000 objects that change often (Endpoints, Leases, the HPA status updates every 15 s, ArgoCD's application status). Fourteen million revisions in two weeks is about 12 per second, which matches the write rate on the API server metrics.

`etcd_debugging_mvcc_db_compaction_keys_total` stayed flat from 2026-03-30 on the dashboard, which was the visible signature of the missing auto-compaction, in plain sight, for two weeks. It is now the second panel on the etcd dashboard, and the alert `EtcdCompactionStalled` (`increase(etcd_debugging_mvcc_db_compaction_keys_total[2h]) == 0`, `warn`) exists so that the shape of this incident is recognised before the size alert.

To check today, from `ops-tools` with the etcd client certs:

```
etcdctl --endpoints=https://10.20.0.11:2379,https://10.20.0.12:2379,https://10.20.0.13:2379 endpoint status --write-out=table
etcdctl alarm list
```

`DB SIZE` under 1 GB and no alarm is normal. Between 1 and 4 GB, check the compaction panel. Above 6 GB (75 % of the quota), the `EtcdDbSizeHigh` alert has already fired and the on-call runs the compact and defrag steps from the etcd runbook, one member at a time, defrag last on the leader.

The weekly defrag CronJob reports the before and after sizes to the ops channel every Sunday morning. A typical line: `hf-cp-01 412MB -> 298MB`. If the before size climbs week over week, something is generating revisions faster than compaction removes them, and the usual suspect is a controller updating a status field in a tight loop, which is what [[incident-2026-06-argocd-sync-loop]] would have looked like from etcd's side had it lasted longer.
