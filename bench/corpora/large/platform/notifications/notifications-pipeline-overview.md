---
name: notifications-pipeline-overview
description: One pipeline for e-mail, SMS and push since HF-4100 (Nov 2025): notify-worker on the Messenger transport, Courrix and Bipline, 1.9 M deliveries a week
type: reference
status: active
verified: 2026-07-14
---

# The notifications pipeline

Everything that leaves Halden Freight towards a human (e-mail, SMS, push) goes through one path since the HF-4100 rework of November 2025. Before that there were three senders with three retry policies, see [[legacy-swiftmailer-cron-sender]] for what was replaced and why.

## Path of a notification

1. A domain service calls `NotificationDispatcher::dispatch(NotificationRequest $r)`. The request names an event type (`bid.received`, `load.assigned`, `invoice.issued`, `auth.otp`, about 60 types), a recipient (`user_id` or a raw address for the few anonymous cases), a locale and a payload for the template.

2. `NotificationDispatcher` writes one row in `notifications` (the intent, immutable) and one row per channel in `notification_deliveries` (the attempt, mutable). The channel list comes from the recipient's preferences, the event type's defaults, and the channel policy of the event (an OTP is SMS only, a digest is e-mail only). The idempotency key introduced after the May 2026 duplicate invoice incident lives on `notification_deliveries.idempotency_key`, unique.

3. The row ids are published on the Messenger transport `notifications` (RabbitMQ, 4 queues: `email`, `sms`, `push`, `retry`).

4. `notify-worker` (the `messenger:consume notifications` process, 6 replicas in `platform-prod`) renders the template, applies the per-recipient rate limit, calls the provider adapter, and updates `notification_deliveries.status` (`queued`, `sent`, `delivered`, `bounced`, `failed`, `suppressed`).

5. Provider webhooks (`POST /webhooks/courrix`, `POST /webhooks/bipline`) update the delivery status asynchronously. Push receipts come from the mobile transport, described on the mobile side.

## Providers

| Channel | Provider | Adapter class | Env |
|---|---|---|---|
| e-mail | Courrix (`api.courrix.example`) | `CourrixMailer` | `NOTIFY_EMAIL_PROVIDER=courrix`, `COURRIX_API_BASE` |
| SMS | Bipline (`sms.bipline.example`) | `BiplineSmsClient` | `NOTIFY_SMS_PROVIDER=bipline`, `BIPLINE_API_BASE` |
| push | FCM and APNs through the mobile transport | `PushRelay` | none, reuses the mobile config |

Credentials are in the vault under `notifications/courrix` and `notifications/bipline`, projected as Secrets, never in env files. The dev compose file points both adapters at `notify-sink.hf.internal`, a catcher that logs and answers 202.

## Volumes (week of 2026-07-06)

| Channel | Requests | Sent | Suppressed | Failed after retries |
|---|---|---|---|---|
| e-mail | 1 410 000 | 1 362 000 | 41 000 | 7 000 |
| SMS | 288 000 | 284 000 | 2 100 | 1 900 |
| push | 2 050 000 | 1 980 000 | 0 | 70 000 (stale tokens) |

E-mail is 90 % transactional (bids, assignments, invoices) and 10 % digests ([[shipper-bid-digest-batching]]). SMS is 70 % OTP ([[incident-2026-01-otp-sms-retry-loop]] explains why that share is watched), 30 % driver assignments as fallback ([[push-first-sms-fallback-decision]]).

## Retry policy

One policy for all channels, in `config/packages/messenger.yaml`: 3 retries, delays 30 s, 5 min, 30 min, multiplier 1. After the third failure the delivery goes to `failed` and to the `retry` queue's dead-letter (`notifications_failed`), which the on-call drains by hand or lets expire after 7 days. The old mailer retried forever with a 1-minute delay, which is how a 2-hour provider timeout produced 5 copies of an invoice.

Retries are per delivery, not per notification: an SMS failing does not resend the e-mail.

## Where to look

- Grafana `Notifications / Pipeline`: queue depth per Messenger queue, deliveries per status per minute, provider latency p50/p99, bounce rate 1 h.

- Table `notification_deliveries`, indexed on `(recipient_id, created_at)` and `(status, updated_at)`. 45 M rows, purged at 180 days by the nightly `notifications:purge` command.

- `bin/console notifications:trace <notification_id>` prints the intent, every delivery, every provider event and the rendered subject. This is the first command of the [[notifications-oncall-runbook]].

## What is deliberately not in the pipeline

- Marketing e-mail. Sales uses a separate tool with its own sending domain (`news.halden.example`), so a campaign complaint cannot hurt transactional reputation. This was a condition of the [[incident-2026-04-dedicated-ip-blocklist]] review.

- In-app notifications. The bell in the dispatch tool reads `notifications` directly through the API; there is no delivery row for it because there is nothing to deliver.

## Rendering

Templates are described in [[email-templates-and-locales]]. The renderer is `TemplateRenderer::render(string $eventType, string $channel, string $locale, array $payload)`, it returns a `RenderedMessage` with `subject`, `text`, `html` (e-mail), `text` (SMS, max 3 segments enforced), or `title` and `body` (push). A missing template for a locale falls back to `en`, and a missing template for the event type is a hard failure at deploy time, caught by `bin/console notifications:lint-templates` in the CI.

## Sending domains and DNS

E-mail is sent from `notify@mail.halden.example` (transactional) with the `From` display name set per event type (`Halden Freight Dispatch`, `Halden Freight Billing`). SPF, DKIM (selector `crx1`, 2048 bits, rotated yearly by Courrix) and DMARC (`p=reject` since 2026-02, after the [[incident-2025-11-dmarc-quarantine-spam]] cleanup) are on `mail.halden.example`. Bounces go to `bounces@mail.halden.example`, which Courrix handles and reports through the webhook, see [[bounce-handling-and-suppression]].

## Numbers that define normal

- Median time from `dispatch()` to `sent`: 1.8 s for e-mail, 2.4 s for SMS, 0.9 s for push.

- 99th percentile: 14 s for e-mail. Above 60 s for 10 minutes the `NotifyLatencyHigh` alert fires (`warn`).

- Queue depth on `email` above 20 000 for 15 minutes pages (`NotifyBacklog`). It reached 410 000 once, see [[backlog-drain-2026-06-courrix-outage]].

- Bounce rate above 2 % over 1 h pages; the normal figure is 0.6 %.

## Ownership

The platform team owns the pipeline, the adapters and the runbook. Event types and their templates are owned by the product team that emits them, listed in `config/notifications/event_types.yaml` with an `owner` field; the CI refuses an event type without an owner. Preferences of the two people who maintain the pipeline are in [[notifications-team-preferences]].
