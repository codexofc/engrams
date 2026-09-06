---
name: exactly-once-vs-idempotent-consumers
description: At-least-once plus idempotent handlers beats broker transactions for every consumer except ingest-svc (offset in the sink); the review questions for a consumer
type: feedback
status: active
verified: 2026-04-30
---

# At-least-once with idempotent handlers, and when not

## The position

torrent, like any Kafka-protocol broker, delivers at-least-once by default: a consumer that processes a message and dies before committing the offset will see the message again. The protocol also offers transactions ("exactly-once semantics"), where produce and offset commit are atomic. We use transactions in one place and idempotent handlers everywhere else, and after two years this is a position, not an accident.

## Why idempotent handlers

A projector that does `INSERT ... ON CONFLICT (aggregate_id) DO UPDATE` with a version or timestamp guard can process the same message twice, or out of order within a key, and land on the same state. The cost is a well-chosen key and a guard column. The benefit is that every operational tool works: reset offsets to replay ([[replay-procedure-runbook]]), reproduce from the dead letter topic ([[dead-letter-topics-convention]]), restart a consumer mid-batch, increase partitions ([[partition-count-decisions]]). None of these are safe for a consumer that relies on seeing each message once.

Transactions protect the produce-consume-produce chain inside the broker. They do not protect the database write, the HTTP call to Payla, or the SMS. Every consumer we have writes somewhere else. So "exactly-once" in the broker's sense would have bought us exactly-once delivery to the handler and nothing about what the handler does, at the cost of slower produces, a coordinator to babysit, and consumers that must be `read_committed` and therefore see messages later.

## The one exception: `ingest-svc`

`ingest-svc` writes 50 000-row batches into the warehouse and must not duplicate rows, because the warehouse tables are append-only and a duplicate is not overwritten. Its pattern is "offset in the sink": the batch insert and the offset update go into the warehouse in one atomic operation (the insert with a deduplication token derived from topic, partition and first offset of the batch, and the offset table updated in the same statement group), and on restart the consumer reads its position from the warehouse, not from the broker. The broker-side committed offset is informational, for the lag exporter. This is exactly-once in the only place it can be: at the sink. It does not use broker transactions.

## What went wrong before we settled

- 2024: `tracking-projector` used broker transactions because a tutorial did. Its consumers had to be `read_committed`, which added 200 ms of latency to every position update on the dispatch map, and during a controller failover in early 2025 the transaction coordinator was unavailable for 4 minutes and every produce failed. Rewritten as an idempotent `UPSERT` keyed on `(load_id, reported_at)` in March 2025. Nobody missed the transactions.

- 2025: `notify-fanout` was "idempotent" by relying on the notifications pipeline's idempotency key, which is scoped to 180 days and to the same business event id. A replay after 180 days, or a replay of a topic where the business event id is regenerated (the outbox relay does that on some paths), would resend. It is not idempotent at the consumer; it delegates. We wrote that down in the replay checklist rather than fixing it, because fixing it means the fan-out remembering every notification it produced, which the pipeline already does. The right answer was the `DRY_RUN` flag and the checklist.

- 2026: the poison message ([[incident-2026-02-poison-message-bids]]) had nothing to do with delivery semantics, but the fix (dead letter by default) only works because skipping a message and committing past it is safe for an idempotent consumer that will get a corrected message later.

## Review questions for a new consumer

Asked in the MR that adds the group to `topics.yaml`.

1. What is the handler's idempotency key, and where is the guard against out-of-order within a key? Show the `UPSERT` or the equivalent.

2. If this consumer is restarted at an offset one hour back, what happens? "Nothing visible" is the right answer. "It resends X" means a `DRY_RUN` or `REPLAY_MODE` flag is required before merge.

3. Does it call anything external? Then the replay mode must stop at that boundary and log instead.

4. Does it produce to another topic? Then the produced message's key and the downstream consumers' idempotency are part of this review.

5. Does it need the sequence of changes or the last state? Sequence means reading the history topic, not the compacted CDC topic.

6. Static membership, cooperative assignment, dead letter topic: on by default in the wrappers, and the MR says so or explains why not.

## What we would still like

A test harness that takes a consumer, feeds it a topic slice twice with a restart in between, and diffs the sink. `torrent-replay-extract` plus a throwaway database gets us most of the way and it is done by hand for critical consumers before their first deploy. Automating it is a ticket (HF-4440) that has been "next quarter" for two quarters.
