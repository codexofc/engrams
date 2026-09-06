---
name: incident-2025-10-ingest-rebalance-storm
description: Oct 2025: ingest-svc rebalanced 140 times in 2 h because one slow member exceeded max.poll.interval.ms, lag 52 min, fixed with static membership, HF-4405
type: project
status: active
verified: 2025-11-12
---

# Incident 2025-10-22: rebalance storm on ingest-svc

## Impact

From 09:10 to 11:05 UTC, `ingest-svc` (the warehouse ingestion consumer, six members, reading 40 topics) rebalanced 140 times and made almost no progress. Lag on `cdc.app.loads` and `domain.load.status-changed` reached 52 minutes. The warehouse's `core` tables were 52 minutes stale at the worst point; the morning dashboards of the dispatch team showed yesterday's numbers as "current", and two people asked in the channel whether the warehouse was down. No data was lost: the offsets were not committed for the batches that did not complete, and the exactly-once offset table on the warehouse side ([[exactly-once-vs-idempotent-consumers]]) meant the retried batches did not duplicate.

## What a rebalance storm is

A consumer group rebalances when a member joins or leaves. During a rebalance, no member of the group consumes. If a member is considered dead because it did not call `poll()` within `max.poll.interval.ms` (default 5 minutes, ours was 60 s at the time because "we want fast failure detection"), the broker evicts it, the group rebalances, the member comes back, the group rebalances again. If the member keeps being slow, this repeats forever, and every rebalance costs the whole group 5 to 15 seconds of doing nothing.

## Timeline (UTC)

- 09:05 the warehouse's 09:00 materialised view refresh (a heavy one, since removed) made inserts into `raw.load_status_history` take 40 to 50 s instead of 3 s.

- 09:10 the `ingest-svc` member holding the `cdc.app.load_status_history` partitions spent 45 s in the insert of a 50 000-row batch, did not poll for 60 s, was evicted. Rebalance number 1.

- 09:10 to 11:00 the member rejoined, got partitions (not necessarily the same ones, dynamic membership), started a batch, the batch was slow because every insert was slow, was evicted again. Other members occasionally got the slow partitions and were evicted in turn. 140 rebalances. Lag climbed on every topic the group reads, not only the slow one, because the whole group stops during each rebalance.

- 09:40 `ConsumerLagHigh` paged at the 900 s threshold (the alert was still in messages at the time and had been fired for 20 minutes at `warn`; the seconds-based alert of [[consumer-lag-alerting]] came out of this incident).

- 10:20 the on-call saw `torrent_consumer_rebalances_total` (the panel existed, the alert did not) and understood it was a storm, not a crash. Killed the view refresh on the warehouse. Inserts back to 3 s at 10:25.

- 10:25 to 11:05 the group stabilised by itself after two more rebalances and caught up at about 3× the produce rate. Lag back under 60 s at 11:05.

## Root cause

Three things.

1. `max.poll.interval.ms = 60000` was shorter than the worst case of one batch. The batch size (50 000 rows or 30 s) and the poll interval were chosen separately by different people. A batch that takes 45 s to insert under load is not a dead consumer, it is a slow one, and evicting it makes everything worse.

2. Dynamic membership: every eviction reassigned partitions across all members, so the slow partitions moved around and took the slowness with them, and every member's in-flight batch was thrown away at each rebalance.

3. The trigger, a slow warehouse insert, was outside the consumer, and the consumer had no back-pressure other than "take longer", which is what tripped the interval.

## Fixes (HF-4405)

- `max.poll.interval.ms = 600000` (10 minutes), `session.timeout.ms = 45000` with heartbeats every 3 s. A member that is alive but slow keeps its partitions; a member that is dead is detected in 45 s by the missing heartbeats, which is a separate thread. "Fast failure detection" was conflating two things.

- Static membership: `group.instance.id = ingest-svc-<ordinal>` from the StatefulSet ordinal. A member restarting within `session.timeout.ms` gets its partitions back without a rebalance. Rolling restarts of `ingest-svc` went from 6 rebalances to 0.

- Cooperative incremental assignment (`CooperativeStickyAssignor`) so that when a rebalance does happen, only the moved partitions stop.

- Batch insert timeout in `ingest-svc`: an insert taking more than 120 s is aborted and retried with half the batch, and `ingest.insert_duration_ms` above 30 s is a `warn` pointed at the warehouse team, not the streaming team.

- `torrent_consumer_rebalances_total` alert, 3 in 10 minutes, `warn`.

## Since

Rebalances of `ingest-svc` per month: 400 in September 2025 (mostly deploys), 2 in December, 0 to 3 since. The same four settings were applied to every critical consumer group in November 2025 and are the defaults in the client wrappers; a consumer using dynamic membership now needs to say why in `topics.yaml`. The broker side of the picture is in [[torrent-broker-overview]].

## What we did not do

Scale `ingest-svc` up. Six members were enough; more members would have meant more partitions moving at each rebalance. The fix was to stop rebalancing, not to have more things to rebalance.
