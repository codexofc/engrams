---
name: incident-2025-11-dmarc-quarantine-spam
description: Nov 2025: DMARC p=quarantine while the dispatch tool sent from an unaligned relay, 38 % of dispatch e-mails in spam for 6 days, HF-4102
type: project
status: active
verified: 2025-12-15
---

# Incident 2025-11-18: dispatch e-mails in spam after DMARC quarantine

## What happened

On 2025-11-12 the DMARC record of `halden.example` was changed from `p=none` to `p=quarantine` (ticket HF-4098, part of the reputation work before the pipeline rework). The change was correct for everything sent through Courrix from `mail.halden.example`, which was SPF and DKIM aligned.

It was not correct for the dispatch tool's own sender. The old dispatch back-office still sent assignment confirmations directly through the datacentre's SMTP relay, from `dispatch@halden.example`, with no DKIM and an SPF record that did not include the relay. With `p=none` those messages had been delivered for years on reputation alone. With `p=quarantine`, receivers started honouring our own instruction and put them in spam.

Nobody noticed for six days because the e-mails were technically delivered (no bounces), and the dispatch team's own mailboxes are on our domain where the relay is whitelisted.

## Detection

2025-11-18, a carrier in Poland called support: three assignment confirmations in spam. Support found two more tickets from the same week. The platform on-call ran the DMARC aggregate reports (`rua` mailbox, parsed by the `dmarc-report` script into `ops.dmarc_reports`) and saw 38 % of messages with `From: halden.example` failing alignment since the 12th, all from one source IP: the relay.

## Fix

- 2025-11-18 16:40: the dispatch back-office `MAILER_DSN` pointed at Courrix (`courrix+api://...` through the vault), envelope sender changed to `dispatch@mail.halden.example`. Aligned within the hour. This was the first sender migrated to what became [[notifications-pipeline-overview]].

- 2025-11-19: `From: dispatch@halden.example` kept as the display address, but the DKIM signature and the SPF domain are `mail.halden.example`. DMARC alignment in relaxed mode accepts the organisational domain match.

- 2025-11-25: `dmarc-report` gained an alert: any source not in the allowed list sending more than 50 messages a day with our domain pages the platform on-call. The allowed list is Courrix's IP ranges and nothing else.

- 2026-02-03: `p=reject`, after two months of reports showing 99.7 % alignment, the remaining 0.3 % being forwarders.

## Impact

About 21 000 assignment e-mails over 6 days. 38 % is the aggregate-report figure; the real share in spam folders is unknown because a receiver that quarantines does not tell us what the user saw. Support counted 14 tickets. Two loads were picked up late because the carrier did not see the confirmation; the dispatch tool's push notification existed but neither driver had the app installed yet.

## Root cause

The DMARC change was reviewed as a DNS change (correct record, correct syntax), not as a change of policy applied to every sender of the domain. The inventory of senders did not exist. Now it does: `config/notifications/senders.yaml` lists every address that may appear in a `From` header and the path it uses, and the [[template-review-checklist]] asks for it on any new event type.

## What we did not do

We did not roll back to `p=none`. Rolling back would have hidden the problem again and the goal of the change was exactly to find unaligned senders before `p=reject`. Six days of spam for one flow was the price of learning where the senders were, and we said so in the review.
