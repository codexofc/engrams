---
name: partition-count-decisions
description: Partition counts come from an allowed list (6, 12, 24, 48, 96), sized for the peak rate divided by 300 msg/s per partition with room for 3 years, driver.positions went 24 to 48 to 96, why counts never go down and what an increase does to key ordering
type: project
status: active
verified: 2026-05-22
---

# Partition counts

## The rule

A topic's partition count is one of 6, 12, 24, 48, 96, declared in `topics.yaml` ([[topic-naming-and-ownership]]). The number is chosen from the expected peak rate three years out, divided by what one partition can sustain for its slowest critical consumer, rounded up to the list. The list exists so that consumer group sizes divide evenly (a group of 4, 6, 8 or 12 members on 24 partitions gets equal shares) and so that nobody argues about 30 versus 32.

"What one partition can sustain" is not the broker's limit (a partition can take 10 000 messages a second with room) but the consumer's: the slowest critical consumer on our topics is a projector doing one database write per message, at about 300 messages a second per partition when it is well written. So a topic expected at 1 200 messages a second at peak in 2029 gets 6 partitions minimum for throughput, and then more for headroom and for the number of consumer members we want to be able to run: 24.

## Why counts never go down

The protocol does not allow reducing partitions, and the workaround (create a new topic, dual-write, migrate consumers) is a project. So counts are chosen with room, and a count that turns out too high costs a little broker overhead (open files, replication threads, about 1 MB of memory per partition per broker) and nothing else. 1 640 partitions on 5 brokers is fine; the broker documentation gets nervous at 4 000 per broker. We could triple.

## Why increasing is not free either

Increasing the count changes the key-to-partition mapping for every key: a message keyed `load:99120033` that went to partition 17 out of 24 goes to some other partition out of 48. Consequences:

- Ordering per key is broken across the change: a consumer may see the message before the change (partition 17) after the one after the change (partition 41), if it is behind on 17 and current on 41. For a projector doing `UPSERT` on the last state, this is a brief inconsistency; for anything replaying a sequence, it is a bug. So increases happen when consumers are at zero lag, on a quiet Sunday, with the producers paused for the 30 s the operation takes.

- Compacted topics keep the old messages on their old partitions forever (until a newer message with the same key lands on the new partition and the old one is... never compacted away, because compaction is per partition). A compacted `cdc.*` topic after an increase carries a duplicate of every key that was updated since, one per partition it has lived on. A consumer reading from the beginning sees both and must take the later by timestamp or version. The [[cdc-tap-postgres-connector]] notes have the consumer rule for this (use `after.updated_at`, never partition order).

So an increase needs `partitions_changed: <date>` in the yaml, a consumer-by-consumer check, and the data team's approval.

## History

### `driver.positions`

| Date | From | To | Reason |
|---|---|---|---|
| 2024-06 | 12 | 24 | 3 000 msg/s peak, `eta-projector` lagging at 2 members |
| 2025-05 | 24 | 48 | 6 000 msg/s, 8 members wanted |
| 2026-01 | 48 | 96 | 9 000 msg/s, and the 2029 projection is 20 000 |

The 2026 increase was done at 04:00 on a Sunday with the driver app's position uploads buffered client-side for 2 minutes (the app already buffers when offline; we told the backend to answer 503 for 90 s). No ordering issue: positions are appended with their own timestamp and every consumer sorts by it. The consumer group `eta-projector` at 12 members takes 8 partitions each.

### `cdc.app.loads` and `cdc.app.bids`

Created at 48 in 2024 with the rule above applied to a 2027 projection, never changed. The compaction issue above is the reason we sized these once and high.

### The `domain.*` topics

Created at 24 in 2024, except `domain.load.status-changed` at 48 (it follows the volume of the status history) and `domain.load.cancelled` at 12 (8 messages a second at peak). None changed. The 24-partition topics run at 5 to 15 % of their throughput budget, which is the intended shape.

### `billing.*`

12 or 24. The settlements topic could be 1 partition on volume (5 messages a second) and is 12 for uniformity with the consumer group sizes; a 1-partition topic is also a single point where a stuck consumer stops everything, and 12 makes the poison-message case ([[incident-2026-02-poison-message-bids]]) 1/12 of the topic rather than all of it.

## What we do not do

- Key-less round-robin topics to avoid the mapping problem: every topic has a key, because every consumer eventually wants per-key ordering.

- Custom partitioners: a producer that wants to control the mapping wants a different topic.

- Reducing by migration: not yet needed. If `driver.app-events` (24 partitions, 800 messages a second, could be 6) ever bothers anyone, it would be the candidate, and it does not bother anyone.
