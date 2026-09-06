---
name: webhook-replay-tool
description: Replaying webhook deliveries (single, 24 h range, self-service since HF-3116): same X-Halden-Delivery, fresh signature, 30-day limit, URL guard
type: project
status: active
verified: 2026-06-23
---

# Replaying webhook deliveries

Built in two steps: `hfctl webhooks replay` for support (HF-3060, January 2026) and the self-service replay in the integrator's web screen (HF-3116, April 2026). Same backend endpoint, `POST /v2/webhooks/deliveries/{id}/replay` and `POST /v2/webhooks/subscriptions/{id}/replay-range`.

## What a replay is

A replay creates a **new delivery row** in `sys_outbox` with the same `event_type`, the same `payload`, the same `schema_version`, and `replay_of = <original id>`. It is delivered through the normal path with the normal retry schedule ([[webhook-retry-schedule-v2]]). The headers:

- `X-Halden-Delivery`: the **original** delivery id, not the new row's id. This is the whole point: the integrator's dedup key is stable, so a replay of something they did process is dropped by them, and a replay of something they missed is processed once.

- `X-Halden-Replay: <new row id>` so they can tell it is a replay if they care.

- `X-Halden-Timestamp` and `X-Halden-Signature`: current time, fresh signature. A replay is a new request; it must pass the 5-minute skew check ([[webhook-signature-verification-guide]]).

The payload is what it was at the time of the event. A replay of `load.dispatched` from Tuesday says what the load looked like on Tuesday, even if it was delivered on Wednesday. Integrators who want current state fetch the load by id.

## Single replay

Any delivery in `DELIVERED`, `DEAD`, `FAILED` or `EXPIRED`, up to 30 days old (older rows are purged). From the web screen: a "Replay" button on the delivery row, for `org_admin`. From support: `hfctl webhooks replay <delivery_id> --apply`.

Replaying a `DELIVERED` row is allowed because "delivered" means their server said 2xx, not that they processed it (the Brenner case in the support collection is the reason we insist on this distinction). The web screen asks for confirmation on a `DELIVERED` replay with the text "Your endpoint acknowledged this delivery. Replay only if you know it was not processed."

## Range replay

`replay-range` takes `from`, `to` (max 24 h apart), optional `event_types[]`, and re-sends every delivery of the subscription in that window whatever its status, in creation order, with a 50 ms spacing so that 2 000 replays take 100 s and not 1 s. From the web screen since HF-3116 with the same limits; from support with `hfctl webhooks replay-range`. Support did 140 range replays between January and April 2026; integrators did 95 themselves between April and June, and support did 12. That was the goal.

## Guard: changed URL

A replay goes to the subscription's **current** URL. If the URL changed after the original delivery, the replay endpoint returns 409 `url_changed_since` unless `force=true`. The reason: an integrator who moved to a new vendor should not replay a month of events into the old vendor's endpoint by mistake, and the other way round. The web screen explains and offers the force.

## Guard: payload version

If the subscription's `payload_version` changed, the replay still uses the original delivery's `schema_version`. Their new code may not accept the old shape. The screen warns. We considered re-rendering the payload in the new version and decided against: the event's data is frozen, and re-rendering a v2 event as v3 would invent fields that did not exist at the time (see [[webhook-payload-versioning-v2-v3]]).

## Limits and abuse

Per subscription: 5 range replays per hour, 200 single replays per hour. Enough for recovery, not enough to use replay as a polling mechanism, which one integrator tried (a script replaying the last hour every 5 minutes, "to be safe"). Their subscription was paused by support with an explanation.

## Metrics

Replays are counted separately in the delivery metrics so that the success rate of first-time deliveries is not polluted by replays into known-broken endpoints.

## Implementation notes

The replay endpoint runs in the API, not in the relay: it inserts the new `sys_outbox` row in a short transaction and returns 202 with the row id. The relay picks it up through the normal `NOTIFY`, so a replay has the same first-attempt latency as any event (under 60 s at p99, [[webhook-delivery-metrics-and-slo]]). Range replays insert rows in a loop with `created_at` spaced by 50 ms, which is how the pacing is achieved without a scheduler: the relay processes rows in `created_at` order and never runs ahead of the clock for rows with `next_attempt_at` in the future, so setting `next_attempt_at = now() + i * 50 ms` on the i-th row is the whole throttling mechanism.

The `replay_of` column is indexed; the delivery viewer shows a "replayed 2 times" badge on the original and links both ways. Replays of replays point to the original, not to the intermediate row, so the chain stays flat.

Rate limits are enforced in Redis per subscription (`webhooks:replay:<sub_id>:range:<hour>`, `...:single:<hour>`), returning 429 with `Retry-After` at the top of the next hour.

## Audit

Every replay, from the screen or from `hfctl`, writes a `sys_audit_log` row with the actor (the org user, or the support login), the delivery ids, and the range. The security review reads the support ones quarterly. Customers can see their own replays in the delivery viewer with the actor's name, which settled one internal argument at a customer about who had "resent everything twice" (their own on-call, twice, on two different nights).

## Figures, April to June 2026

95 range replays by integrators, 12 by support; about 1 400 single replays by integrators, 60 by support. 7 `url_changed_since` refusals, 2 forced. 1 abuse (the 5-minute script) paused by support. Median size of an integrator range replay: 3 hours, 240 deliveries; the largest: 24 hours, 4 100 deliveries, taking 3 minutes 25 seconds to insert and about 6 minutes to deliver.
