---
name: payla-outage-2026-02
description: Payla outage of 2026-02-17, 3 112 debits missed the cut-off, and the circuit breaker plus three-pass debit batch added after
type: project
status: active
verified: 2026-03-03
---

## Timeline

- 2026-02-17 11:40 CET: `PaylaClient` starts returning `503` on every endpoint. Alert `payla_error_rate > 20 %` fires at 11:46.

- 11:50: Payla status page acknowledges a database failover problem on their side, no ETA.

- 12:10: card checkouts are failing with the generic message `payment.unavailable`. We flip the flag `billing.card_checkout_enabled` off so shippers see a proper message (`payment.temporarily_unavailable`) and can post loads on invoice terms instead. Flags convention in the product common notes.

- 15:00: the SEPA debit batch runs as scheduled, every `POST /v2/debits` fails, the batch marks 3 112 debits `submission_failed` and stops. Nobody notices immediately because the batch job's own alert only fires when the job crashes, not when every item fails.

- 16:05: Payla recovers. Our webhook backlog drains in about 12 minutes (Payla replays the events it could not deliver, which produced the duplicate storm mentioned in [[payla-webhook-idempotency]]).

- 16:20: we discover the 3 112 failed debits. The Payla cut-off for same-day submission is 16:00, so they can only go out the next day, which delays settlement by one business day for 2 480 shippers.

- 17:30: manual re-submission via `billing:payla:resubmit-debits --date 2026-02-17`, all accepted. Settled 2026-02-20 instead of 2026-02-19.

Financial impact: about 1.9 M EUR of collections delayed by one day, no loss. Support received 61 tickets, mostly shippers asking why the debit was not on their statement.

## Root causes

1. No circuit breaker: the batch hammered a dead API 3 112 times with 8 s timeouts, so the batch itself took 45 minutes to fail.
2. The batch alert was on job failure, not on item failure rate.
3. No automatic retry before cut-off.

## Changes (HF-2455, HF-2456, HF-2461)

- `PaylaClient` now has a circuit breaker: opens after 10 consecutive `5xx` or timeouts, half-open probe every 30 s. While open, calls fail fast with `PaylaUnavailableException`.
- The debit batch is no longer a single pass. It runs at 13:00, 14:30 and 15:40, each pass picking up debits still `pending_submission`. Metric `billing.debit_batch.unsubmitted_after_last_pass` alerts if above 0 at 15:55.
- New alert `billing.debit_batch.item_failure_rate > 5 %` on any pass.
- The status page of Payla is polled every minute by `PaylaStatusProbe`; a declared incident on their side posts to the billing channel automatically. Not strictly needed but it saved 20 minutes on 2026-04-08 when they had a shorter blip.

## What we decided not to do

A second payment provider as a fallback was discussed and rejected. Mandates are held by Payla, so a fallback provider could not debit anything anyway, and the card volume is too small to justify a second integration. Revisit if card volume passes 15 % of collections.
