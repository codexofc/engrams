---
name: push-first-sms-fallback-decision
description: Since Dec 2025 driver assignment notifications go push first, SMS only if the push is not acknowledged within 3 min, cut assignment SMS by 64 % with no measurable change in pickup lateness
type: project
status: active
verified: 2026-03-02
---

# Push first, SMS as fallback for driver assignments

## Before

Until November 2025 a `load.assigned` notification to a driver produced a push and an SMS at the same time, because "drivers do not look at the app". That belief dated from 2023 when app adoption was 40 %. In November 2025 it was 86 % of active drivers, and the SMS half of every assignment cost about 4 100 EUR a month for messages that were, for most drivers, a duplicate.

## The change (HF-4115)

`load.assigned`, `load.reassigned` and `load.time_window_changed` for drivers are now `channels: [push]` with `fallback: {channel: sms, after: 180s, unless: acknowledged}` in `event_types.yaml`. The fallback is implemented by `FallbackScheduler`: when the push delivery is `sent`, a delayed Messenger message (`ScheduleFallback`, delay 180 s) is published; when it fires, the scheduler checks whether the notification has been acknowledged and, if not, creates the SMS delivery.

"Acknowledged" means one of: the push receipt says `opened`, the driver app fetched the load (`GET /v1/driver/loads/{id}` with the load id, logged in `notification_acks`), or the driver changed the load status. A push merely delivered to the phone is not an acknowledgement, the phone might be in a glovebox.

Drivers without a registered device token, or whose token was pruned, go straight to SMS, no delay.

## Measured

Two weeks before (2025-11-17 to 11-30) against two weeks after full rollout (2026-01-12 to 01-25), excluding the Christmas weeks:

| Metric | Before | After |
|---|---|---|
| assignment SMS per week | 31 400 | 11 300 |
| share of assignments acknowledged within 3 min | 71 % (push and SMS) | 68 % (push only) |
| share acknowledged within 30 min | 94 % | 93 % |
| pickup lateness p50 on assignments made less than 12 h before pickup | 18 min | 17 min |
| driver-side support tickets mentioning "not notified" | 6 | 9 |

The 3 % drop in 3-minute acknowledgements is the population that used to read the SMS first; they now get the SMS 3 minutes later. Lateness did not move. The 9 tickets were looked at one by one: 5 were drivers whose app had notifications disabled at the OS level (now detected at app start and shown as a banner), 3 were pruned tokens after a phone change (straight-to-SMS path, worked, the ticket was about the delay), 1 was a real bug where the push was `sent` but the fallback message was never scheduled because the Messenger delay queue had been misconfigured on one of six workers. Fixed in HF-4140, and the delay queue now has its own depth panel.

## Why 3 minutes

We tried 1 minute in staging with the dispatch team's own phones: too short, a driver at a loading dock does not touch the phone for a minute. 10 minutes was the dispatch team's fear threshold ("if it's a same-day reassignment I want them to know now"). 3 minutes was the compromise, and the data above says nothing was lost. It is a per-event-type setting, `load.cancelled` uses 60 s.

## What we kept as SMS

- OTP, obviously, see [[sms-country-routing-and-unit-costs]] for the country rules.

- `load.cancelled` when the pickup is within 24 h: push and SMS at once. Rare, 300 a month, the cost of a missed one is a truck at an empty dock.

- Anything to a driver with `users.prefers_sms = true`, a flag support can set for drivers who ask. 340 drivers have it, mostly older drivers with company phones that block app notifications.

## Cost

About 20 000 SMS a month saved, 1 100 EUR at the mixed rate. Less than hoped because the German and Belgian drivers, the expensive ones, have the lowest app adoption (74 %) and hit the fallback more. The next step, if anyone wants it, is a nudge in the driver app when a fallback SMS was needed ("enable notifications to avoid delays"), not touched yet.

## Related

The rate limits ([[per-recipient-rate-limits]]) apply to the fallback SMS like any other, which is fine: a driver getting more than 5 assignment SMS an hour is a dispatch problem, not a notification one.
