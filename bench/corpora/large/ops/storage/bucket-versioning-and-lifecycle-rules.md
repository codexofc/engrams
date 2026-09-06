---
name: bucket-versioning-and-lifecycle-rules
description: Versioning on every prod bucket holding primary data, noncurrent versions 90 days on documents and 30 elsewhere, lifecycle rules per bucket in Git
type: reference
status: active
verified: 2026-04-15
---

# Versioning and lifecycle

## Versioning

Enabled on every `-prod` bucket whose contents are not rebuildable: documents, PostgreSQL backups, vault snapshots, ML artifacts, torrent snapshots, ops-misc. Not enabled on caches (`hf-tiles-prod`, `hf-exports-prod`), on Tempo, or on `hf-warehouse-cold-prod` (ClickHouse writes immutable parts with unique names and deletes them under its own TTL; versioning there would keep every dropped part for 30 days, about 10 TB of nothing, measured in staging).

A delete on a versioned bucket is a delete marker; the object stays as a noncurrent version until the lifecycle rule expires it. This is what made [[restore-2025-11-documents-prefix-deleted]] an afternoon rather than a disaster. An overwrite keeps the previous version too, which covers the other accident: a `PUT` of the wrong content on an existing key.

## Lifecycle rules

Declared in `halden-infra/storage/lifecycle/<bucket>.yaml`, applied by the CI with `stashctl put-bucket-lifecycle`, and compared nightly to what the appliance reports (drift alert, `warn`). One rule set per bucket:

### Rules per bucket, April 2026

| Bucket | Noncurrent expiry | Current expiry | Multipart abort | Notes |
|---|---|---|---|---|
| `hf-documents-prod` | 90 days | none | 2 days | the app purges by `retain_until`, the bucket never expires a current object |
| `hf-pg-backups-prod` | 30 days | 40 days on `wal/`, 40 days on `base/` | 2 days | the operator deletes at 35 days; the rule is the safety net 5 days later |
| `hf-velero-prod` | none (not versioned) | none | 2 days | Velero manages its own TTL per schedule |
| `hf-vault-snapshots` | 30 days | 95 days | 1 day | |
| `hf-torrent-snapshots` | 30 days | 35 days | 1 day | |
| `hf-ml-artifacts-prod` | 30 days | none | 2 days | artifacts are addressed by hash; pruning is a manual job with the registry |

### Rules per bucket, continued: unversioned buckets

| Bucket | Noncurrent expiry | Current expiry | Multipart abort | Notes |
|---|---|---|---|---|
| `hf-tiles-prod` | not versioned | 30 days since last access is unsupported, so 60 days since creation | 1 day | the tile server re-fetches |
| `hf-exports-prod` | not versioned | 7 days | 1 day | |
| `hf-tempo` | not versioned | none, Tempo compacts and deletes | 1 day | |
| `hf-warehouse-cold-prod` | not versioned | none, ClickHouse TTL | 2 days | |
| `hf-ops-misc` | 30 days | none, quarterly review | 2 days | |

## Why documents never expire by lifecycle

A document's retention depends on its type and on legal holds ([[retention-by-data-class]]), which the bucket does not know. A lifecycle rule that expired current objects at 10 years would be almost right and wrong for the holds and for the short-lived identity scans. So the rule expires nothing, and `documents:purge` deletes explicitly with the one key that can. The appliance's compliance-mode object lock on held documents is the second lock.

## Multipart abort

Incomplete multipart uploads (a client that started a 50 MB upload and went away) are invisible in listings and count against capacity. Before the abort rule existed, `hf-documents-prod` carried 140 GB of them, found during the 2025 capacity review. Two days is long enough for any legitimate upload (the presigned PUT is valid 15 minutes) and short enough to matter.

## What the CI checks

- Every `-prod` bucket has a lifecycle file; a bucket in the appliance without one fails the nightly drift check.

- Versioned buckets have a noncurrent expiry (an unbounded version history is a capacity leak, and it happened on `hf-ops-misc` in 2025: 400 GB of noncurrent versions of one person's repeated uploads of the same 2 GB tarball).

- Replication to `stash-b` is enabled on every `-prod` bucket ([[restore-drill-2026-05]] finding 1).

- No rule expires current objects on `hf-documents-prod`. This one is a hard failure with a message that names the November incident.

## Changing a rule

Merge request, one approval from the bucket's owner and one from storage. A shortening of any expiry requires a comment explaining what will be deleted and when the first deletion will happen, and the CI adds a 7-day delay before applying a shortening, during which the change is visible in the drift report as "pending". The one time this delay mattered, it was a rule shortening `hf-pg-backups-prod` current expiry to 20 days by mistake (the author meant noncurrent), noticed on day 3.
