---
name: legacy-swiftmailer-cron-sender
description: Until Nov 2025 e-mail left through a cron-driven SwiftMailer spool, SMS through a shell script, push through the mobile transport: three retry policies
type: reference
status: archived
superseded_by: [[notifications-pipeline-overview]]
verified: 2025-10-02
---

# The old senders (archived)

Kept for the archaeology of tickets before HF-4100. Everything here was replaced in November 2025, see [[notifications-pipeline-overview]].

## E-mail

A SwiftMailer file spool at `var/spool/` on the API pods (yes, on a pod, on a `local-path` volume), flushed by a CronJob `spool-send` every minute with `swiftmailer:spool:send --message-limit 500 --time-limit 50`. Transport: the datacentre SMTP relay, plain SMTP, no DKIM on our side. The relay added its own DKIM for its domain, which is why alignment failed once DMARC was enforced ([[incident-2025-11-dmarc-quarantine-spam]]).

Retry: the spool retried a failed message on every run, forever, until someone deleted the file. Two pods meant two spools, and a message spooled on a pod that was then rescheduled was lost with the volume. Nobody could say how many; the estimate from support tickets was "a few dozen a month".

Templates: Twig HTML in `templates/emails/`, with translations through the Symfony translator and 4 locales. The text version was generated from the HTML with `strip_tags`.

## SMS

A shell script, `send_sms.sh`, on the old dispatch back-office server, called by the PHP application with `exec()`, which posted to an SMS gateway with an account shared with the sales team's outreach tool. No record of what was sent except the gateway's monthly invoice. Retry: none; a failed `curl` was a warning in a log file that rotated weekly.

## Push

Already through the mobile transport (FCM and APNs), which is the one part that survived the rework mostly unchanged, now behind the `PushRelay` adapter.

## Why it was replaced

- No common record: "did this shipper receive the invoice" had three answers from three places, two of which were "we do not know".

- Three retry policies, one of which was infinite. The May 2026 duplicate invoice incident happened after the rework but on the same idea (retry without idempotency), which says how deep the habit ran.

- Provider coupling: the gateway account was shared with sales, and their outreach campaign in September 2025 exhausted the monthly quota on the 20th, so OTPs failed for the last ten days of the month for anyone who did not know to ask sales.

- The spool on a pod volume. Enough said.

## What was kept

The e-mail templates' text was migrated to the MJML structure of [[email-templates-and-locales]] by hand, with the product team re-reading every one. The 4 locales became 11 over the following six months. The `From` display names per flow were kept because customers recognised them.
