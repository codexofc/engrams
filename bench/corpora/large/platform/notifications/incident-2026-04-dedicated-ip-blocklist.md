---
name: incident-2026-04-dedicated-ip-blocklist
description: April 2026, our new dedicated Courrix IP landed on a public blocklist on day 6 of warm-up because the warm-up plan ramped on total volume while 40 % of it was carrier invites, 2 days at 30 % deferrals, HF-4210
type: project
status: active
verified: 2026-05-12
---

# Incident 2026-04-09: dedicated IP blocklisted during warm-up

## Background

Until April 2026 we sent through Courrix's shared IP pool. Shared pools are fine until a neighbour misbehaves, and in March a neighbour did: two days of deferrals from one big mailbox provider that had nothing to do with us. The platform team asked Courrix for a dedicated IP (HF-4205), which they provide with a warm-up schedule: the IP starts with no reputation, and volume must grow slowly so that receivers learn it is a legitimate sender.

Courrix's plan was 2 000 messages on day 1, doubling daily, until full volume (about 200 000 a day) around day 8, with the rest overflowing to the shared pool in the meantime. Their routing does the split; we only see `sending_ip` in the webhook events.

## What happened

Day 6 (2026-04-09), 64 000 messages on the dedicated IP. At 14:20 UTC the nightly `reputation-probe` (run by hand because someone was curious) showed the IP on one of the three public blocklists we watch. At 15:00 the big mailbox providers started deferring (`421 4.7.0 try again later`) about 30 % of the messages sent from that IP. Courrix's automatic overflow to the shared pool does not apply to deferrals, so those messages retried on the same IP for up to 48 h before Courrix gave up.

## Root cause

The warm-up plan ramped total volume, and we let Courrix pick which messages went to the new IP: they took them in order of arrival. The morning batch of day 6 was dominated by `carrier.invite`, because a large carrier onboarding had just imported 26 000 driver and contact addresses and the onboarding flow invited them all within an hour. Invites are the event type with our worst complaint rate (0.3 %, see [[complaint-rate-and-feedback-loops]]) and the most unknown addresses (11 % hard bounces on that import).

So a brand new IP sent 26 000 messages to strangers, 2 800 of which bounced hard and about 80 of which were reported as spam, in one morning. The blocklist's threshold for a fresh IP is low, and it did what it exists to do.

## Timeline of the recovery

- 2026-04-09 16:00: `carrier.invite` moved off the dedicated IP by setting `courrix_ip_pool: shared` on the event type (a routing hint Courrix accepts per message via the `X-Courrix-Pool` header). Same for `invoice.overdue`, the second worst.

- 16:30: delisting request filed with the blocklist, with the explanation. Their form asks what changed; we said what changed.

- 2026-04-10 09:00: delisted. Deferrals dropped to 5 % by noon, to the normal 0.4 % by the evening of the 11th.

- 2026-04-11: the invites import flow got a throttle: 500 invites an hour per account, and the first batch of any account's import goes to a list of 50 addresses for a day before the rest is released. HF-4212.

- Warm-up restarted from day 4 volume, with only `billing`, `awards` and `documents` event groups (best reputation, known recipients) on the dedicated IP for the first 10 days. Full volume reached 2026-04-24.

## Impact

About 60 000 e-mails delayed between 4 and 30 hours over two days, mostly bid notifications and assignment confirmations (already covered by push, see [[push-first-sms-fallback-decision]]). 11 support tickets. No invoice affected: `billing` was still overflowing to the shared pool at that point in the warm-up.

## What changed for good

- `event_types.yaml` has a `reputation_tier: high | normal | low` per event type. `low` (invites, overdue reminders, re-engagement) never goes to the dedicated IP; Courrix routes it to the shared pool. High is billing and account events. The dedicated IP therefore carries only mail to people who expect it.

- Warm-up, if we ever need another IP, is done per tier in that order, and the plan is written in the runbook ([[notifications-oncall-runbook]]) rather than taken from the provider's default.

- `reputation-probe` runs every 6 hours instead of nightly and opens a ticket on any listing. On a fresh IP during warm-up it pages.

- Bulk imports that produce notifications are a category the pipeline now knows: `NotificationRequest::bulk(true)` marks them, and bulk deliveries have their own queue (`email_bulk`) drained at a fixed rate of 20 a second, so a 26 000-address import takes 22 minutes and cannot spike anything.

## The numbers on the dedicated IP since

| Period | Messages / day on dedicated IP | Deferral rate | Complaint rate |
|---|---|---|---|
| warm-up 2 (04-12 to 04-24) | 8 000 to 160 000 | 0.5 % | 0.008 % |
| May 2026 | 190 000 | 0.4 % | 0.012 % |
| June 2026 | 205 000 | 0.3 % | 0.011 % |

The shared pool in the same months: 1.1 % deferrals. So the dedicated IP is doing what we bought it for, one week of grief included.
