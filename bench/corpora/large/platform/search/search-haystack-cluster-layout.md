---
name: search-haystack-cluster-layout
description: The haystack cluster: 3 masters, 6 data nodes over two zones, loads index 6 primaries plus 1 replica behind aliases, snapshots, upgrades, staging
type: reference
status: active
verified: 2026-07-24
---

# haystack: cluster layout

`haystack` is the OpenSearch-compatible cluster behind the carrier load search since March 2026 ([[search-why-haystack-not-postgres]]). It runs on Kubernetes in the platform cluster, with its own node pool. Nothing but the search API and the indexer talks to it.

## Nodes

### Roles and sizing

| Role | Count | Zone spread | CPU / RAM / heap | Disk |
|---|---|---|---|---|
| master (dedicated) | 3 | a, a, b | 2 vCPU / 4 GB / 2 GB | 20 GB |
| data | 6 | 3 in a, 3 in b | 4 vCPU / 16 GB / 8 GB | 200 GB SSD |
| coordinating | 0 | | | |

No dedicated coordinating nodes: the data nodes coordinate, and at our query volume (about 40 queries per second at peak) that is fine. The three masters are the only nodes that hold cluster state votes; a data node loss changes nothing for the masters.

Heap is half the RAM, capped well under 32 GB as recommended; 8 GB is more than we need (the whole index fits in the OS page cache). `indices.memory.index_buffer_size` at the default 10 %.

Endpoint: `https://haystack.hf.internal:9200`, TLS with the internal CA, basic auth with two users: `search-api` (read on `loads-read`) and `haystack-indexer` (write on `loads-write`, index management on `loads-*`). Credentials come from the secret store, rotated quarterly. No human user; engineers use `kubectl port-forward` and their SSO-issued short-lived credentials through the `haystack-admin` role for debugging.

## Indices

One logical index, `loads`, physically `loads-v<N>` with two aliases:

- `loads-read`: what the search API queries.

- `loads-write`: what the indexer writes to.

During a reindex ([[search-reindex-runbook]]) both aliases can point to different physical indices, or `loads-write` can point to two at once (the indexer dual-writes). Current physical index in July 2026: `loads-v7` ([[search-index-mapping-loads]] describes the mapping).

Shards: **6 primaries, 1 replica** each, so 12 shards over 6 data nodes, 2 per node. The index holds 30 000 to 60 000 active documents plus terminated loads kept 7 days for the "recently dispatched" view, about 90 000 documents and 1.2 GB primary size. Six primaries is more than the data needs (one would do), chosen so that a query fans out across all six data nodes and uses their CPU; the [[search-incident-2026-05-shard-hotspot]] note explains what happened when routing made that fan-out uneven.

Refresh interval: 5 s (default 1 s was pointless churn; the indexer batches every 2 s anyway). `translog.durability: request`.

## Allocation and resilience

`cluster.routing.allocation.awareness.attributes: zone`, so a primary and its replica are never in the same zone. Losing zone b leaves every shard available in zone a with yellow status. Losing one data node: yellow, replicas re-allocate to the remaining nodes in the same zone within minutes, back to green.

`cluster.routing.allocation.disk.watermark.low/high/flood_stage` at 75 / 85 / 95 %. At 1.2 GB on 200 GB disks this is theoretical, but the snapshot repository once filled a disk during a misconfiguration test and the watermarks did their job.

## Snapshots

Hourly snapshots to an object storage bucket (`haystack-snapshots`, retained 7 days) through the snapshot lifecycle policy. Restoring is not the recovery path we plan for: the index is derived from PostgreSQL and a full reindex takes 12 minutes ([[search-reindex-runbook]]). Snapshots exist for the case where the reindex source is itself the problem (a bad migration on `loads`) and we want yesterday's index back quickly while sorting it out. Tested quarterly.

## Monitoring

Prometheus exporter on each node; the Grafana dashboard "haystack" shows cluster health, per-node heap and GC, indexing rate, search rate and latency percentiles, refresh time, and the indexer lag ([[search-indexing-pipeline]]). Alerts: `HaystackRed` (page), `HaystackYellowLong` (yellow over 30 minutes, notify), `HaystackHeapHigh` (over 85 % for 10 minutes), `HaystackDiskHigh`, and the search latency alerts from [[search-query-latency-slo]].

## Upgrades

One minor version at a time, rolling, one data node at a time with shard allocation disabled during the restart (`cluster.routing.allocation.enable: primaries`), masters last. Tested on the staging cluster (1 master, 2 data, same version) a week before. Never during the 6:00 to 10:00 window when carriers search most. Two upgrades done so far, each about 40 minutes, no downtime.

## Staging

`haystack.staging.hf.internal`, 1 master, 2 data nodes, same mapping, indexed from staging's PostgreSQL. It has about 2 000 documents, which is too few to see performance problems and enough to see mapping and relevance problems. Performance changes are tested with a copy of the production index restored from a snapshot onto a temporary 6-node cluster, twice a year or before a big change.

## Capacity and headroom

Measured in July 2026 at the morning peak: 40 queries per second, data node CPU 35 to 45 %, heap 40 % after GC, search thread pool queue empty, refresh under 50 ms. The load test on the restored production copy pushed 250 queries per second before p99 crossed 300 ms, with CPU at 85 %; that is six times the current peak. We expect to add data nodes (not shards) when peak passes 120 queries per second, which at current growth is 2028. Adding a node is a Helm value and a rebalancing hour; adding shards is a reindex.

Document growth is not a concern: the index holds only active loads plus seven days of terminated ones, so it grows with the marketplace's daily volume, not with history. Doubling the business doubles the index to 2.5 GB.

## Cost

Nine small nodes and 1.2 TB of provisioned SSD, about the price of one large PostgreSQL replica, which is what the carrier search was consuming before in CPU on the primary at peak. The snapshot bucket is negligible. Nobody has asked to shrink it; the three dedicated masters look wasteful at this size and are kept because a split brain at 6:00 on a Monday is the one thing we do not want to learn about.
