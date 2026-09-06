---
name: incident-2026-05-torrent-3-disk-full
description: May 2026: torrent-3 hit 100 % disk after driver.positions retention went 7 to 14 days without a per-broker check, broker down 12 min, 0 loss, HF-4450
type: project
status: active
verified: 2026-06-05
---

# Incident 2026-05-19: torrent-3 disk full

## Impact

From 06:48 to 07:00 UTC `torrent-3.hf.internal` stopped (the broker halts when its log directory is full, by design, rather than corrupting segments). 41 partitions of which it was the leader elected new leaders on other brokers within 15 s; producers with `acks=all` saw 3 to 8 s of retries during the elections, nothing failed ([[producer-acks-and-durability]] for why). With `min.insync.replicas = 2` and replication factor 3, every partition kept accepting writes on its two remaining replicas. So: 12 minutes of under-replication, 8 seconds of visible produce latency, no data loss, no consumer error beyond a rebalance for the groups whose coordinator was on `torrent-3`.

The CDC connector ([[cdc-tap-postgres-connector]]) buffered to its spool for the 12 minutes (2 GB) and drained in 4 minutes after.

## Timeline (UTC)

- 2026-05-18 15:20 a merge request changing `driver.positions` retention from 7 to 14 days is merged and applied by the CI. Reason in the MR: the ETA team wanted two weeks of raw positions for a backtest without going through the warehouse. Reviewed by one data team member who checked the topic, not the brokers.

- 15:20 to 06:40 the topic stops deleting segments older than 7 days. It produces 270 GB a day logical, 810 GB with replication, spread over 5 brokers. Each broker gains about 160 GB a day of `driver.positions` segments that would have been deleted.

- 2026-05-19 04:10 `torrent-3` disk at 75 %, `TorrentDiskHigh` fires (`page`). The platform on-call sees a broker at 75 % with the other four at 62 to 66 %, judges it "growing slowly, look in the morning" and acknowledges. In hindsight the panel showed a slope of 7 GB an hour, which the alert text did not mention.

- 06:48 100 %. `torrent-3` logs `No space left on device` and shuts down its log manager. Leader elections. `OfflinePartitions` stays 0, `UnderReplicatedPartitions` goes to 41, pages.

- 06:52 the on-call (same person, now awake) identifies the retention change from the `topics.yaml` history, reverts the MR, the CI applies 7 days at 06:55.

- 06:55 retention 7 days does nothing for a stopped broker: the log cleaner runs on running brokers. On `torrent-3`, 40 GB freed by hand by deleting the oldest `driver.positions` segments in its log directory (`ls -t | tail` on the partition directories, with the broker down this is safe because the broker rebuilds its index from the segments it finds, and the deleted data was replicated elsewhere).

- 07:00 `torrent-3` started, caught up its 41 partitions from the leaders in 6 minutes, `UnderReplicatedPartitions` back to 0 at 07:06.

- 07:10 the other four brokers at 71 to 74 %; the log cleaner with the reverted retention brings them to 62 to 65 % over the next 3 hours.

## Why torrent-3 and not the others

Rack-aware placement puts more `driver.positions` replicas on rack B (two brokers) than on rack A (three brokers) per broker: each rack holds at least one replica of every partition, so the two rack-B brokers carry a larger share of every topic than the three rack-A brokers. `torrent-3` and `torrent-4` had 60 % each of the topic's data footprint that the rack-A brokers had 45 % of. The 75 % alert was real on `torrent-3` a day before it would have been on `torrent-1`. This asymmetry was known ([[torrent-broker-overview]] mentions the rack-A-heavy layout) and not connected to the retention review.

## Root cause

A retention change was reviewed for its effect on the topic (270 GB a day × 7 more days = 1.9 TB more logical, "we have 3.6 TB free across the cluster") and not for its effect on the fullest broker. The cluster-level free space was right and irrelevant.

## Fixes (HF-4450)

- The CI's `topics.yaml` check computes, for any retention or partition change, the projected per-broker disk after the change from the current segment sizes per broker (`torrent-log-dirs --describe`) and the topic's daily rate, and refuses a change that puts any broker above 70 %. It printed 96 % for `torrent-3` when run against the reverted MR as a test.

- `TorrentDiskHigh` text includes the slope over the last 6 hours and the projected time to 100 %. The 04:10 alert would have said "2 h 40 to full".

- `TorrentDiskCritical` at 85 %, `page`, with the runbook step "delete oldest segments of the largest topic on this broker with the broker stopped, then start it" written down with the exact commands.

- The ETA team got their two weeks of positions from the warehouse, where `raw.driver_positions` has 90 days. The reason they asked torrent was that the warehouse query took 20 minutes; that was fixed by a projection on their side. The topic stays at 7 days ([[retention-and-compaction-policy]]).

- A rebalance of partition replicas to even out the rack asymmetry was discussed and not done: the asymmetry is the price of rack awareness with 3 and 2 brokers, and the fix is a sixth broker in rack B in the next hardware cycle, which is now written in the capacity plan.

## Numbers

| | Value |
|---|---|
| broker stopped | 12 min |
| partitions under-replicated | 41 for 18 min |
| partitions offline | 0 |
| produce latency spike | 3 to 8 s for 15 s |
| messages lost | 0 |
| CDC spool used | 2 GB |
| disk freed by hand | 40 GB |
| segment deletion after revert, cluster-wide | 1.6 TB over 3 h |
