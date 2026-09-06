---
name: bounce-handling-and-suppression
description: Hard bounces suppress at once, soft after 5 in 30 days, complaints for good; suppressed_recipients is per channel and checked before every send
type: reference
status: active
verified: 2026-04-08
---

# Bounces, complaints and the suppression list

## Sources

- Courrix webhook events `bounced` (with `bounce_type: hard | soft`, `smtp_code`, `diagnostic`), `complained` (feedback loop from the big mailbox providers), `unsubscribed` (list-unsubscribe header click).

- Bipline webhook events `undelivered` with `reason: unknown_number | blocked`.

- The push side reports stale device tokens, handled by the mobile transport, not here.

Every event lands in `notification_provider_events` (append only, 90 days) and is applied to `notification_deliveries.status` by `ProviderEventApplier`. The suppression logic runs on top, in `SuppressionPolicy::apply(ProviderEvent $e)`.

## Rules

| Event | Effect | Duration |
|---|---|---|
| e-mail hard bounce (5xx, mailbox does not exist) | suppress address | until the address changes in `users` |
| e-mail soft bounce (4xx, mailbox full, greylist) | count in `soft_bounce_count`, suppress at 5 in 30 days | 30 days, then one probe |
| e-mail complaint | suppress address | for good, no automatic expiry |
| list-unsubscribe | set preference `all_email = false` except `critical` | until the user changes it |
| SMS `unknown_number` | count, suppress at 3 in 30 days | until the number changes |
| SMS `blocked` | suppress number | for good |

"Until the address changes" is implemented by keying the suppression on the normalised address, not on the user: a user who corrects a typo in their e-mail is a different key and is not suppressed.

## The table

`suppressed_recipients(channel, recipient_key, reason, source_event_id, created_at, expires_at NULL)`, unique on `(channel, recipient_key)`. 39 000 rows in April 2026, 34 000 e-mail, of which 61 % are hard bounces from carrier contact addresses that no longer exist (drivers who left, companies that closed). `notify-worker` checks the table before the rate limit; a hit sets the delivery to `suppressed` with `suppression_reason = 'suppressed_recipient'`.

The check is a single indexed lookup, 0.3 ms. There was a proposal to cache the list in Redis; not done, the lookup is not measurable on the send latency.

## Complaints

Complaint rate is the figure the mailbox providers judge us on. Threshold in our alerting: 0.08 % of delivered over 24 h pages the platform on-call; the providers' published tolerance is around 0.1 % and we want to know before they do. Normal is 0.01 to 0.02 %. The one time it went to 0.06 % was a shipper who forwarded all bid notifications to a shared inbox and someone there clicked "spam" on 40 of them in one afternoon. Suppressing that one address was the whole fix. See [[complaint-rate-and-feedback-loops]] for the provider registrations.

## Probing a soft-bounced address

After the 30 days, the next notification to a soft-suppressed address goes through once (`probe = true` on the delivery). If it bounces again the suppression is renewed for 90 days; if it is delivered the counter resets. This avoids permanently losing a recipient because their mailbox was full during their holiday.

## Manual operations

- `bin/console notifications:suppression show <address>`: why, since when, from which event.

- `bin/console notifications:suppression lift <address> --reason "support ticket 12345"`: removes the row and writes an `auth_events`-style audit row in `notification_audit`. Used about twice a month, always after support confirms with the person that the mailbox works.

- `bin/console notifications:suppression add <address> --reason ...`: used for the few addresses that ask by e-mail to receive nothing, and for the addresses of people who have died, which support handles with care and without a form.

Never lift a complaint suppression on request of the sender side (a shipper asking that their carrier receive notifications again). The recipient asked not to receive our mail, that is the end of it. This has been refused three times and the refusal is now written in the support handbook.

## What the list does not do

It does not cover marketing, which is a separate tool with its own list. The two lists are not synchronised on purpose: a complaint about a campaign should not stop an invoice, and an invoice bounce says nothing about marketing consent. The one exception is a hard bounce, exported weekly to the marketing tool as "do not send", since a mailbox that does not exist does not exist for anyone.
