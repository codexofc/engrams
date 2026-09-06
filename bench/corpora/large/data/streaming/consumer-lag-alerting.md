---
name: consumer-lag-alerting
description: Lag is alerted in seconds behind the log head, not messages, per group with thresholds in topics.yaml; 9 critical groups page, 60 s warn, 300 s page
type: project
status: active
verified: 2026-06-18
---

# Consumer lag alerting

## Seconds, not messages

A lag of 10 000 messages on `driver.positions` is one second of traffic; on `billing.payla.settlements` it is two months. Message counts are the number the tools give you, and they are meaningless without the topic's rate. The exporter (`torrent-lag-exporter`, our own 400 lines of Rust, one instance on `ops-tools`) computes for every group and partition the timestamp of the last committed offset's message and the timestamp of the head, and exposes `torrent_consumer_lag_seconds{group, topic, partition}`. The alerts are on the maximum over partitions.

Exceptions: `billing.payla.settlements` and `billing.payout.requested` have a threshold in messages (10), because a settlement arriving once an hour would show as 3 600 s of lag between messages with the time-based rule. The exporter handles both with a `lag_mode` per topic in `topics.yaml`.

## Thresholds

Declared per topic and consumer group in `topics.yaml`:

```
consumers_expected:
  - group: notify-fanout
    critical: true
    lag_warn: 60s
    lag_page: 300s
  - group: ingest-svc
    critical: true
    lag_warn: 120s
    lag_page: 900s
  - group: search-indexer
    critical: false
    lag_warn: 600s
```

Defaults when omitted: `warn` 60 s, `page` 300 s for `critical: true`, `warn` only at 600 s otherwise. `ingest-svc` gets a longer page threshold because it batches (50 000 rows or 30 seconds, whichever first) and a 2-minute lag is its normal shape at low traffic.

The nine critical groups in June 2026: `notify-fanout`, `ingest-svc`, `eta-projector`, `tracking-projector`, `pricing-projector`, `billing-projector`, `billing-svc`, `driver-sync`, `matching-svc`. Each pages its owning team during business hours (08:00 to 20:00 CET) and the platform on-call outside, with the runbook link of the group.

## What the alert says

`ConsumerLagHigh{group="eta-projector", topic="cdc.app.loads"}`: "eta-projector is 340 s behind on cdc.app.loads (threshold 300 s), 3 of 48 partitions, consumers: 4 members, last rebalance 12 min ago". The rebalance line is there because half the lag incidents are rebalances ([[incident-2025-10-ingest-rebalance-storm]]), and "3 of 48 partitions" says whether it is one stuck partition (a poison message, see [[incident-2026-02-poison-message-bids]]) or the whole group being slow.

## Other signals

- `torrent_consumer_group_members{group}` dropping to 0 pages immediately for critical groups, regardless of lag: a group with no members has no lag metric worth trusting.

- `torrent_consumer_rebalances_total{group}` increasing more than 3 times in 10 minutes: `warn`, "rebalance storm", with the runbook step "check for a member that keeps joining and leaving".

- `torrent_partition_lag_stuck{group, topic, partition}`: a partition whose committed offset has not moved in 5 minutes while the head moved. `page` for critical groups. This is the poison message alarm.

- Broker side: `UnderReplicatedPartitions > 0` for 5 minutes pages the platform on-call; `OfflinePartitions > 0` pages immediately. Disk at 75 % pages ([[incident-2026-05-torrent-3-disk-full]]).

## Dashboards

`Streaming / Lag` in Grafana: one row per critical group, lag seconds over 24 h, member count, rebalances, and a "catch-up rate" panel (messages per second consumed minus produced) that says how long the recovery will take when a group is behind. `Streaming / Topics` has the produce rate, size and retention margin per topic from [[core-topics-catalog]].

## Tuning history

- The original thresholds were in messages (10 000 warn, 100 000 page) for every group. They fired daily on `driver.positions` consumers and never on billing. Replaced by seconds in November 2025.

- `ingest-svc` paged at 300 s every night during the 03:00 partition maintenance of the warehouse, when inserts slow down for 5 minutes. Raised to 900 s rather than silencing the window: a real 15-minute lag on ingestion is worth a page, a 5-minute one is not.

- `search-indexer` was critical until the search team said a 10-minute-old index is fine for their use; downgraded, and their team owns the number.

## What the lag does not tell you

That the consumer is doing the right thing. A consumer committing offsets and dropping messages has zero lag. The data quality checks on the warehouse side (row counts against CDC counts, per hour) are the check for that, and the [[replay-procedure-runbook]] is what happens when they find something.
