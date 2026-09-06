---
name: notifications-oncall-runbook
description: Runbook for NotifyBacklog, NotifyRecipientBurst, NotifyBounceRateHigh, NotifyWebhookSilent and NotifyLatencyHigh: first commands, pause, purge, IP warm-up
type: reference
status: active
verified: 2026-07-14
---

# Notifications on-call runbook

Read this with the Grafana board `Notifications / Pipeline` open. Every alert below names the panel to look at first. The general architecture is in [[notifications-pipeline-overview]].

## First three commands, whatever the alert

```
kubectl -n platform-prod get pods -l app=notify-worker
bin/console messenger:stats notifications
bin/console notifications:trace <any recent notification_id from the panel>
```

`messenger:stats` prints the depth of the `email`, `sms`, `push`, `retry`, `email_bulk` and `notifications_failed` queues. `trace` shows the whole life of one notification, including provider events; the shape of one is often enough to know which of the sections below applies.

## NotifyBacklog (page)

Queue depth above 20 000 for 15 minutes. Panel "Queue depth per queue".

1. Which queue. If only `email_bulk`: not an incident, that queue drains at 20 a second by design, a 100 000 import takes 80 minutes. Acknowledge and move on unless it is above 500 000.

2. If `email` or `sms`: is the provider answering? Panel "Provider latency and errors". A wall of 5xx or timeouts means a provider incident, go to "Provider down". A wall of 429 means we are the problem, go to "Producer in a loop".

3. If the providers are fine and the workers are running, the workers may be stuck on Redis (rate limiter) or on the database. `kubectl logs` of one worker; a `RedisException` or a lock wait is the usual find. Restart the workers one at a time, `kubectl rollout restart deployment/notify-worker`.

## Provider down

Do not scale the workers, do not purge. The queue is doing its job. Steps:

- Check the provider status page (`status.courrix.example`, `status.bipline.example`) and our own probe panel.

- If Bipline is down and the backlog holds OTPs: the 90 s e-mail fallback for OTP is automatic, nothing to do. If it lasts more than 30 minutes, purge OTP deliveries older than their TTL so that the recovery does not send stale codes: `bin/console notifications:purge-queue --event auth.otp --older-than 5m` (the TTL check does this on send anyway since HF-4133, the purge only saves the provider calls).

- If Courrix is down for more than 2 hours, pause the digests (`bin/console notifications:pause-event bid.digest`) so that the recovery sends today's bids, not this morning's. Resume with `resume-event`. See the June 2026 case in [[backlog-drain-2026-06-courrix-outage]] for what a long outage looks like and what the drain rate was.

- When the provider is back, let the queue drain by itself. Six workers do about 150 e-mails a second against Courrix; a backlog of 400 000 takes 45 minutes. Do not add workers unless the provider account's rate limit allows it (Courrix: 300 a second on our plan).

## Producer in a loop

Symptoms: 429 from a provider, `NotifyRecipientBurst` firing, one event type dominating the panel "Deliveries by event type", often one client version dominating "Deliveries by client version".

1. Identify the event type and, if a client, its version. `trace` on three deliveries from the burst.

2. Stop the producer at the source if you can (roll back the client release, pause the CronJob, disable the feature flag). If not, `bin/console notifications:pause-event <type>` stops the deliveries of that type at the worker, everything else continues.

3. Purge what is already stale: `notifications:purge-queue --event <type> --older-than <ttl>`.

4. Only then look at why the per-recipient rate limit ([[per-recipient-rate-limits]]) did not catch it. If the burst is spread across many recipients under the limit, it is a producer bug and the limit is not the tool.

## NotifyBounceRateHigh (page)

Bounce rate above 2 % over 1 hour. Panel "Bounces by type and event type".

- Mostly hard bounces on one event type from one account: a bulk import with bad addresses. The import throttle (HF-4212) should have limited it; check that the producer marked it `bulk`. Suppression handles the rest, nothing else to do.

- Soft bounces or deferrals across everything: reputation. Check `reputation-probe` output (`bin/console notifications:reputation`) and the provider's reputation page. If the dedicated IP is listed, follow "IP listed" below. Details of the 2026 case in [[incident-2026-04-dedicated-ip-blocklist]].

- Bounces concentrated on one recipient domain: their gateway. Panel "Delivery by recipient domain". Nothing to fix on our side, tell support so they can answer the tickets, and if it is a big customer, support contacts their IT with our sending IPs and DKIM selector.

## IP listed

1. Move `reputation_tier: low` and `normal` off the dedicated IP: `bin/console notifications:ip-pool --tier low,normal --pool shared`. High stays.

2. File the delisting with the blocklist; the form URLs and what to write are in the vault note `notifications/blocklists`. Say what changed. Do not argue.

3. Do not send warm-up traffic to force it. Wait.

4. After delisting, ramp `normal` back over 3 days, then `low` if ever (it lives on the shared pool by default since April 2026).

## NotifyWebhookSilent (page)

No Courrix webhook event for 10 minutes during 07:00 to 20:00 CET. Either our ingress or their sender. `curl -s -o /dev/null -w '%{http_code}' https://api.halden.example/webhooks/courrix` should give 405 (GET not allowed), anything else is us. If it is them, deliveries pile up in `sent`; the hourly `poll-status` and the `refetch` command ([[provider-webhook-verification-and-replay]]) recover the statuses afterwards, no user impact.

## NotifyLatencyHigh (warn)

p99 dispatch-to-sent above 60 s for 10 minutes. Usually the rendering: a template with a payload that fetches something (it should not, payloads are prepared by the producer). Check "Render time by event type". The second cause is the Redis rate limiter under a burst. Not an incident by itself, fix during the day.

## Warm-up plan for a new IP

Written after April 2026, applies if we ever need a second dedicated IP.

| Day | Daily volume on the new IP | Event groups allowed |
|---|---|---|
| 1 to 3 | 2 000, 4 000, 8 000 | `billing`, `account` |
| 4 to 6 | 16 000, 30 000, 50 000 | plus `awards`, `documents` |
| 7 to 10 | 80 000 to 160 000 | plus `assignments`, `tracking` |
| 11 onwards | full | plus `bids` |
| never | | `reputation_tier: low` |

`reputation-probe` runs every 6 hours regardless and pages during warm-up on any listing. Keep the previous IP or the shared pool as overflow for the whole period.

## What not to do

- Do not delete rows from `notification_deliveries` to "clear" anything. The queue is in RabbitMQ; the table is the record.

- Do not raise a per-recipient limit during an incident. The incident is the reason the limit exists.

- Do not lift a suppression because a shipper asks. See the suppression rules in [[bounce-handling-and-suppression]].
