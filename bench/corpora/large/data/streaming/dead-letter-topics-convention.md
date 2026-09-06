---
name: dead-letter-topics-convention
description: One dead letter topic per consumer group named dlq.<group>, 90 days, the failed message copied verbatim with headers for source topic, partition, offset, error class and attempt, mandatory for critical consumers since Feb 2026, drained by a weekly review not by automation
type: reference
status: active
verified: 2026-04-30
---

# Dead letter topics

## Shape

`dlq.<consumer-group>`, one per group, not per source topic: a group reading 40 topics has one dead letter topic, and the source is in the headers. 6 partitions, 90 days, `delete`, owner is the group's owner in `topics.yaml` ([[topic-naming-and-ownership]]). The CI creates it automatically for every group with `critical: true`; other groups opt in.

The message written to the dead letter topic is the original message, key and value untouched (so it can be re-produced to the source topic as is, if that is ever the right thing), with headers added:

| Header | Content |
|---|---|
| `dlq-source-topic` | `domain.bid.placed` |
| `dlq-source-partition` | `17` |
| `dlq-source-offset` | `88412336` |
| `dlq-error-class` | `DeserializationError`, `SchemaValidationError`, `HandlerError`, `Panic` |
| `dlq-error-message` | first 1 000 bytes of the error |
| `dlq-attempt` | `1` (or more, if the consumer retries in place before giving up) |
| `dlq-consumer` | `pricing-projector@4.12.0` |
| `dlq-at` | RFC 3339 |

The client wrappers (PHP `TorrentConsumer`, TypeScript `@hf/torrent`, Rust `torrent_client`) do this by default: any exception or panic escaping the handler for a message writes the dead letter record, commits the offset, moves on, and counts `torrent.dlq_written{group, error_class}`. Opting out is a named argument that must carry a reason string, which the review reads.

## What goes there and what does not

Goes there: a message the consumer cannot process and will not be able to process by retrying (bad schema, business rule violated, a null where the handler needs a value). The February case ([[incident-2026-02-poison-message-bids]]) is the archetype.

Does not go there: a transient failure (database down, a timeout). The wrapper retries those in place with backoff (3 attempts, 1 s, 5 s, 30 s) and only dead-letters if the error class is still `Transient` after the third; at that point it is `warn`-alerted separately because 30 s of transient failure is probably an outage, not a message problem, and the partition stopping would be better than 5 000 messages in the dead letter topic. In practice `HandlerError` after transient retries is 2 % of dead letter writes.

Also does not go there: messages the consumer chooses to skip (an event type it does not handle, a tombstone). Skipping is normal and silent.

## Alerting

- Any write to a `dlq.*` topic: `warn` to the owning team, batched to one notification per 10 minutes.

- More than 100 writes in 10 minutes for one group: `page`. Something upstream is producing garbage in bulk, and the consumer skipping it is hiding a producer problem.

- Dead letter topic older than 7 days with unreviewed messages: weekly reminder in the owner's channel. "Reviewed" is a committed offset of the group `dlq-review-<group>`, which the review tool advances.

## Review

Weekly, by the owning team, with `torrent-dlq review --group pricing-projector`, our small CLI that lists the unreviewed messages with their headers, groups by error class and error message, and offers three actions per group of identical errors:

- `discard`: advance the review offset, write a one-line reason to `ops.dlq.decisions` (a topic, 400 days, the audit of what was thrown away and why).

- `reproduce`: send the original message back to the source topic, same key, same value, with a header `replayed-from-dlq: <offset>` so the consumer can tell (it usually does not care). Used when the consumer bug was fixed and the message is fine.

- `fix-and-reproduce`: same, after editing the value in an editor. Used twice ever, both times on a currency code typo in a manual settlement message.

Numbers since February 2026 across all groups: 1 900 dead letter messages, 1 720 discarded (mostly duplicates from one bad producer release in March, one review action), 178 reproduced, 2 fixed. Median time from write to review: 3 days.

## What we do not do

- No automatic re-drive. A message that failed once is re-tried by a person who understands why. Automatic re-drive turns a poison message into a periodic incident.

- No shared dead letter topic for everyone. It was the first design; the review became nobody's job.

- No expiry shorter than 90 days. 30 days of retention on the source topic plus 90 on the dead letter topic means a message can be reproduced after the source has forgotten it, which matters for a replay ([[replay-procedure-runbook]]) that runs after the fact.
