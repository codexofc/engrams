---
name: torrent-upgrade-2026-07
description: torrent 3.1 to 3.4 rolling upgrade on 2026-07-06 and 07 (HF-4460), one broker at a time, protocol bumped a week later, one old TypeScript client broke
type: project
status: active
verified: 2026-07-20
---

# torrent upgrade 3.1 to 3.4

## Why

3.1 (in place since 2025-02) was going out of the vendor's support window in September 2026. 3.4 brought a fix for a controller failover bug we had hit once (a 4-minute produce stall in early 2025, the one that killed the transaction experiment described in [[exactly-once-vs-idempotent-consumers]]), tiered storage (not enabled, see [[retention-and-compaction-policy]]), and a saner consumer group rebalance protocol that the client libraries will need a year to adopt.

## Preparation

- Read the release notes of 3.2, 3.3 and 3.4 in full, with the breaking changes list in the ticket (HF-4460). Two mattered: the deprecation of a metric name we alerted on (`ReplicaManager.UnderReplicatedPartitions` moved namespace), and the removal of a legacy request version that very old clients use.

- Client inventory from the broker's connection metrics (`torrent.clients{software_name, software_version}`): 34 producers and 61 consumer groups on 6 client libraries. One outlier: `search-indexer`'s TypeScript client at a 2022 version, still speaking the request version being removed.

- Staging cluster (3 brokers on VMs, same topics at 1 % of volume) upgraded on 2026-06-22. `search-indexer`'s client failed as predicted (`UNSUPPORTED_VERSION` on `Fetch`). Updated the library in the search team's repo, deployed to staging 06-25, fine.

- Alert rules updated for the renamed metric with both names active during the transition.

- The rack-B asymmetry ([[incident-2026-05-torrent-3-disk-full]]) meant `torrent-3` and `torrent-4` would each take longer to catch up; planned them for the second day.

## Procedure

Rolling, one broker at a time, inter-broker protocol version left at 3.1 until every broker ran 3.4 (so that a rollback to 3.1 stayed possible for a week).

For each broker:

1. `torrent-topics --describe --under-replicated-partitions` empty, all 9 critical groups at zero lag.

2. Controlled shutdown (`systemctl stop torrent`, which triggers leader migration off the broker before it stops: 20 s). Producers see 1 to 2 s of retries.

3. Package upgrade, config diff reviewed (three new defaults accepted, one overridden: `group.consumer.heartbeat.interval.ms`, left at our value).

4. Start, watch the broker catch up its replicas (`torrent_replica_lag_max` per broker), wait for zero under-replicated partitions.

5. Wait 30 minutes more. Move to the next broker.

Catch-up times: `torrent-1` 22 min, `torrent-5` 24 min, `torrent-2` 26 min (day 1, 2026-07-06, 09:00 to 13:00 CET). `torrent-3` 41 min, `torrent-4` 38 min (day 2, 07-07, 09:00 to 12:00). Deliberately during business hours: the data team was at their desks and the traffic was representative; a 4 am upgrade finds problems at 9 am anyway.

Inter-broker protocol bumped to 3.4 on 2026-07-14 after a week without a reason to roll back. After that, no rollback without a full rebuild, which is fine.

## What broke

- `search-indexer` in production had not deployed the library update the search team had tested in staging: their deploy was scheduled for the following week. When `torrent-1` (its coordinator at that moment) came back on 3.4, the old client got `UNSUPPORTED_VERSION` and crash-looped. Lag on `cdc.app.loads` for `search-indexer` (not critical, `warn` at 600 s) reached 50 minutes before the search team deployed the update. The search index was 50 minutes stale, nobody outside the team noticed. Lesson written in the ticket: the staging check is worth nothing if production is not on the same version, and the upgrade checklist now includes "every client version seen in production in the last 7 days is one that passed staging".

- The renamed metric: the old alert rule silently evaluated to "no data" on upgraded brokers for two hours before someone looked at the dashboard. The dual-name rule was correct for the new name and the old rule was not removed; the dashboard panel used the old name only. Cosmetic, fixed the same day.

Nothing else. No consumer lag on critical groups above 20 s during the two days, no dead letter writes, no CDC spool usage above 200 MB.

## What we have now

- The controller failover fix, which we hope never to notice.

- `torrent-log-dirs --describe` with per-topic breakdown, which is what the CI's per-broker disk projection now reads ([[torrent-broker-overview]] for the cluster layout it protects).

- The new group protocol available server-side; the Rust and PHP wrappers will move to it when their libraries do, and the [[incident-2025-10-ingest-rebalance-storm]] shape becomes much less likely with it (the broker computes the assignment, members do not stop the world).

- Tiered storage, tested, off.

## Numbers

| | Value |
|---|---|
| brokers | 5 |
| total time, both days | 7 h including waits |
| produce latency spikes | 5, each 1 to 2 s |
| critical group max lag | 18 s |
| non-critical incidents | 1 (`search-indexer`, 50 min) |
| rollback used | no |
