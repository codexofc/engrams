---
name: object-store-buckets-and-layout
description: Two Stashbox S3 appliances, stash-a (rack A) and stash-b (rack B), 60 TB usable each, async replication a to b, endpoint s3.hf.internal, 14 buckets with one owner each, 46 TB used in June 2026, documents 9 TB, warehouse cold 31 TB, backups 6 TB
type: reference
status: active
verified: 2026-06-30
---

# The object store

## Hardware and topology

Two Stashbox appliances (the vendor's S3-compatible storage units, 24 × 8 TB drives each, erasure coded 16+8, 60 TB usable per appliance): `stash-a.hf.internal` in rack A, `stash-b.hf.internal` in rack B. Clients talk to `s3.hf.internal`, a virtual IP on the datacentre firewall pair that points at `stash-a`; `stash-b` receives everything by the appliances' own asynchronous bucket replication ([[incident-2026-03-appliance-replication-lag]] for what the "asynchronous" costs) and is the read target for backups verification and the failover target if `stash-a` dies. Failover is manual: move the VIP, promote the replicated buckets to writable, which has been rehearsed in the [[restore-drill-2026-05]] and takes 15 minutes.

The predecessor was an NFS share for documents ([[legacy-nfs-document-share]]), decommissioned in 2024.

## Buckets

One bucket per data class and environment, one owning team, versioning and lifecycle per bucket ([[bucket-versioning-and-lifecycle-rules]]). Names are `hf-<class>-<env>`.

### Bucket inventory, June 2026

| Bucket | Owner | Size | Objects | Notes |
|---|---|---|---|---|
| `hf-documents-prod` | platform | 9.1 TB | 41 M | PODs, CMRs, carrier documents, presigned access from clients |
| `hf-documents-staging`, `-dev` | platform | 400 GB | | staging is a 1 % sample copied nightly |
| `hf-warehouse-cold-prod` | data | 31 TB | 2.1 M | ClickHouse cold parts, 90 days and older |
| `hf-pg-backups-prod` | platform | 4.8 TB | 900 k | WAL archive plus base backups, 35 days |
| `hf-velero-prod` | ops | 1.1 TB | | cluster object and volume backups |
| `hf-torrent-snapshots` | data | 60 GB | | connector state, registry dumps, not the topics |
| `hf-tempo` | ops | 300 GB | | traces, 7 days |
| `hf-tiles-prod` | platform | 180 GB | 12 M | map tiles for the dispatch tool, cache, rebuildable |
| `hf-etcd-snapshots` | ops | 20 GB | | RKE2 snapshots, 30 days |
| `hf-vault-snapshots` | ops | 2 GB | | hourly, 90 days |
| `hf-ml-artifacts-prod` | ml | 210 GB | | model binaries and training sets by registry hash |
| `hf-exports-prod` | product | 90 GB | | customer CSV exports, 7 days |
| `hf-ops-misc` | ops | 40 GB | | everything that does not deserve a bucket, reviewed quarterly |

Total 46.4 TB on `stash-a`, 77 % of usable. The capacity plan ([[storage-capacity-plan-2026]]) is about the 31 TB line.

## Access

Every application has its own access key pair in the vault, scoped by bucket policy to its buckets and to the operations it needs (`documents-api` can `PutObject` and `GetObject` on `hf-documents-prod`, not `DeleteObject`; deletion is lifecycle only). Human access is through `ops-tools` with a read-only key for everything and a per-incident write key issued by the storage on-call and revoked after. Keys are 90-day rotated by a job that writes the new key to the vault and deletes the old one 7 days later; the applications read the key at start and on a `SIGHUP`, and the rotation job restarts nothing.

The appliances' admin interface is on the management VLAN only, two named admins, MFA on the appliance.

## What the object store is trusted for, and what not

Trusted: durability inside one appliance (erasure coding survives 8 drive failures), and a second copy in the other rack with a lag of seconds to minutes. Not trusted alone: anything that needs to survive the datacentre. The weekly offsite copy ([[offsite-weekly-copy-contract]]) is the third copy, and the backup inventory ([[backup-inventory-and-retention]]) says which buckets are in it.

## Monitoring

- Appliance health (drives, controllers, temperature) by the vendor's exporter, `page` on a failed drive (the spare kicks in, but the second failure is a page too).

- Used capacity per bucket, daily, on the storage dashboard; `warn` at 75 % of usable on either appliance, `page` at 85 %. We have been in `warn` since April 2026, see the capacity plan.

- Replication lag between appliances, `warn` at 15 min, `page` at 2 h.

- Request errors per bucket from the appliance access log shipped to the logging stack, `warn` on 5xx above 0.1 %.

## Operations that are never done by hand

- Deleting objects in `hf-documents-prod` or `hf-pg-backups-prod`. Lifecycle rules and legal hold flags do it, with the retention classes of [[retention-by-data-class]].

- Changing a bucket policy without a merge request in `halden-infra/storage/policies/`. The appliance applies what the CI pushes.

- Creating a bucket. Same path. The quarterly review of `hf-ops-misc` exists because the alternative is people asking for a bucket "just for this".
