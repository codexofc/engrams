---
name: notifications-team-preferences
description: How the two platform engineers who own notifications like to work, one path for every channel, yaml over code for policy, no new channel without a provider contract, incidents reviewed within a week
type: user
status: active
verified: 2026-07-14
---

Preferences of the two people who maintain the notifications pipeline, as applied in 2026.

- **One path.** Every message to a human goes through `NotificationDispatcher`. A team that wants to "just send an e-mail from this command" gets a new event type instead. The three-sender era ([[legacy-swiftmailer-cron-sender]]) is the reason, and the answer to "it is only for this once" is no.

- **Policy in the yaml, mechanism in the code.** Channels, fallbacks, TTLs, batching windows, reputation tiers, critical flags all live in `config/notifications/event_types.yaml`. The worker interprets, it does not decide. A behaviour that needs a code change to alter per event type is a design smell and gets moved to the yaml.

- **A new channel is a contract first.** Before any adapter is written, there is a signed provider contract, a status page we can probe, a webhook or a polling API for final status, and a per-message idempotency reference on their side. Bipline met the four; two other SMS providers did not on the last point and were not chosen.

- **Cost is an engineering metric.** The monthly SMS and e-mail invoices are reconciled against `notification_deliveries` by us, not by finance, because a mismatch is a bug in our pipeline before it is a billing dispute. See the monthly review in [[sms-country-routing-and-unit-costs]].

- **The recipient's choice wins.** Suppressions and unsubscribes are never lifted because a sender asks. Critical events are three, and the list is defended.

- **Templates are code.** Reviewed, linted, deployed. No template editing in a provider's web console. The [[template-review-checklist]] is applied even for a one-word change.

- **Incidents reviewed within a week**, written in the format of the four incident notes of this project, with a table of numbers and a "what we did not do" section. The review is with the producing team present, since most notification incidents are producer bugs seen from the pipeline.

- **French or English**, whoever writes. Event type names, yaml keys, class names in English.

- **No marketing in the pipeline.** Ever. The reputation of the transactional domain is not shared.
