---
name: legacy-rabbitmq-domain-events
description: Before 2024 domain events went through RabbitMQ topic exchanges with one queue per consumer, no replay, no retention, no schema, and a fan-out that lost messages whenever a consumer's queue was absent, replaced by torrent
type: reference
status: archived
superseded_by: [[torrent-broker-overview]]
verified: 2025-09-15
---

# Domain events on RabbitMQ (archived)

How events moved between services before torrent, kept for the tickets of 2022 to 2024. The current bus is described in [[torrent-broker-overview]]. RabbitMQ itself is still in use for Symfony Messenger work queues, which is the job it is good at.

## The setup

One topic exchange `domain.events`, routing keys like `load.assigned`, `bid.placed`. Each consuming service declared its own durable queue bound to the routing keys it cared about (`search-indexer.events`, `notify.events`, ...). Messages were JSON without a registered schema; the producer's serializer was the schema.

## What was wrong with it

- **No replay.** A message consumed and acknowledged was gone. A projector with a bug had no way to rebuild except from the database, with a script per projector. Three such scripts existed, none of them agreed with the projector they were meant to reproduce.

- **Lost fan-out.** A queue that did not exist at publish time (a new consumer, or one whose queue had been deleted during a cleanup) received nothing published before it was declared. A new projection started empty and stayed empty for history. In 2023 the pricing projection missed 4 days of bids because its queue had been deleted by mistake and recreated by the deploy; nobody knew until the numbers looked odd.

- **No ordering across a consumer's instances.** Two instances of a service on the same queue got interleaved messages for the same load. Every projector had a "last write wins by timestamp" workaround, some of them wrong.

- **Retention by accident.** A consumer that stopped left its queue growing; RabbitMQ's memory alarm then blocked all publishers, including the ones whose consumers were fine. Twice in 2023, once for 40 minutes, the API could not publish and therefore (because publishing was inline in the request) could not accept bids.

- **No schema, no contract.** A field renamed in the producer broke three consumers in three ways, discovered one at a time over a week.

## What the move to torrent changed

Retention and replay (30 days on `domain.*`, offsets that can be reset), per-key ordering through partitions, a registry with compatibility rules, an outbox so that publishing never blocks a business transaction, and consumers that come and go without losing history. It also changed the mental model, which took longer than the code: a partition is not a queue, a consumer group is not a worker pool, and "acknowledge" became "commit an offset" with everything that implies for idempotency.

The migration ran from 2024-02 to 2024-09, one consumer at a time, with dual publishing (exchange and topic) for the whole period and a comparison job that counted messages on both sides per hour. The exchange was deleted 2024-10-01. The three rebuild scripts were deleted with it.
