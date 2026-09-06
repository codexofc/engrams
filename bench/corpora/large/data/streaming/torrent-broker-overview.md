---
name: torrent-broker-overview
description: torrent is the Kafka-protocol event bus: 5 bare-metal nodes torrent-1..5.hf.internal on two racks, 3.4 in 2026, 210 topics, 18 000 msg/s peak, 4.1 TB retained
type: reference
status: active
verified: 2026-07-20
---

# torrent, the event bus

## What it is

`torrent` is the broker that carries every domain event, every CDC stream and every metric batch between the API, the dispatch tool, the mobile backend, the warehouse ingestion and the ML services. It speaks the Kafka wire protocol, so every client library and every tool that works with Kafka works with it, and everyone in the company says "Kafka" when they mean torrent; the notes in this project say torrent for the broker and Kafka for the protocol. The predecessor, RabbitMQ used as a domain event bus, is in [[legacy-rabbitmq-domain-events]]; RabbitMQ still exists for Symfony Messenger transports (work queues), which is a different job.

## Cluster

| Node | Rack | Role | Disk |
|---|---|---|---|
| `torrent-1.hf.internal` | A | broker + controller | 2 × 3.84 TB NVMe |
| `torrent-2.hf.internal` | A | broker + controller | 2 × 3.84 TB NVMe |
| `torrent-3.hf.internal` | B | broker + controller | 2 × 3.84 TB NVMe |
| `torrent-4.hf.internal` | B | broker | 2 × 3.84 TB NVMe |
| `torrent-5.hf.internal` | A | broker | 2 × 3.84 TB NVMe |

Bare metal, not on the Kubernetes cluster: the broker wants its disks and its network to itself, and a broker pod being rescheduled during a node drain is a rebalance nobody asked for ([[incident-2025-10-ingest-rebalance-storm]] is what one of those looks like from the consumer side). Version 3.4 since the [[torrent-upgrade-2026-07]] work, controller quorum on nodes 1 to 3 (no external coordination service since 3.x).

Replication factor 3 for every topic, `min.insync.replicas = 2`, rack-aware replica placement so that a rack failure leaves every partition with at least one replica. The rack-A-heavy layout (3 of 5) means losing rack A leaves two brokers with the full topic set; it fits, at 70 % disk, and it is the reason we watch disk ([[incident-2026-05-torrent-3-disk-full]]).

Bootstrap address for clients: `torrent.hf.internal:9093` (TLS, mutual, certificates from the internal CA, one per client application, CN is the application name and is what the ACLs match).

## Numbers (July 2026)

- 210 topics, 1 640 partitions, 4.1 TB retained across the cluster after replication (1.4 TB logical).

- 18 000 messages a second at the morning peak (07:00 to 09:00 CET), 6 000 average, 1.2 GB a minute in at peak.

- 61 consumer groups, of which 9 are "critical" in the alerting sense ([[consumer-lag-alerting]]).

- 34 producing applications, each with its own certificate and ACL.

## Topic families

Detailed in [[core-topics-catalog]] and [[topic-naming-and-ownership]]. The short version:

- `cdc.app.*`: change data capture from the API's PostgreSQL through [[cdc-tap-postgres-connector]], one topic per table, keyed by primary key, compacted.

- `domain.*`: business events published by the applications themselves (`domain.load.assigned`, `domain.bid.placed`, ...), keyed by aggregate id, 30 days.

- `billing.*`: Payla settlements, invoice lifecycle, 400 days for audit.

- `driver.*`: positions and state reports from the driver app, high volume, 7 days.

- `ml.*`: predictions and feature updates, 30 days.

- `ops.*`: internal, metrics batches, audit copies.

Every message has the envelope of [[event-envelope-format]] and a schema registered in the registry ([[schema-registry-compatibility-rules]]).

## Who owns what

The data team owns the cluster, the registry, the CDC connector, the retention policy and the replay tooling. The platform on-call is paged for broker-level alerts (a broker down, disk, under-replicated partitions) because they are the ones awake; the data team is paged for lag on critical groups during business hours and for anything about the registry. Topic ownership is per topic, declared in `topics.yaml`, and the owning team is paged for the lag of its own consumers. The team's habits are in [[streaming-team-preferences]].

## Operational commands worth knowing

```
torrent-topics --bootstrap-server torrent.hf.internal:9093 --command-config /etc/torrent/admin.properties --describe --under-replicated-partitions
torrent-consumer-groups ... --describe --group ingest-svc
torrent-consumer-groups ... --reset-offsets --group <g> --topic <t> --to-datetime 2026-03-02T00:00:00.000 --execute
```

The last one is the heart of [[replay-procedure-runbook]] and is never run without the checklist there. The admin properties file with the admin certificate lives on `ops-tools` only.

## What torrent is not used for

- Work queues with per-message acknowledgement and retry delays: that is RabbitMQ through Messenger (notifications, document processing). A partition is not a queue and a consumer group is not a worker pool with retries.

- Request-response between services: HTTP.

- Large payloads: the documents themselves go to the object store, the event carries the reference. `message.max.bytes` is 1 MB and a producer that needs more has a design problem.

## Disk and retention in one paragraph

Retention is per topic ([[retention-and-compaction-policy]]). The 4.1 TB retained sits at 55 % of the 7.7 TB usable per node after RAID-1 across the 5 nodes, and the alert is at 75 %. The growth is 40 GB a week, dominated by `driver.positions` (7 days but 9 000 messages a second at peak) and `cdc.app.loads` (compacted, but the tombstones for 3 M loads take their time). At the current growth the 75 % line is in 2028, which is after the next hardware cycle.
