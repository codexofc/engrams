---
name: dunning-schedule
description: The five-step dunning ladder (J+3 reminder to J+45 account suspension), which steps are automatic, and the 2 500 EUR threshold that routes an account to a human collector
type: reference
status: active
verified: 2026-05-27
---

Dunning (relance) is driven by `DunningScheduler`, which runs every morning at 06:30 Europe/Paris and evaluates every open invoice of every shipper account. The steps are configured in `dunning_steps` and the state per account is in `dunning_states`.

## The ladder

| Step | Trigger | Action | Automatic |
|---|---|---|---|
| 0 | invoice issued | nothing, payment terms apply (30 days by default) | yes |
| 1 | due date + 3 days | email `dunning.reminder_soft`, no fee | yes |
| 2 | due date + 10 days | email `dunning.reminder_firm`, adds late interest line on the next invoice, see [[late-payment-interest-fr]] | yes |
| 3 | due date + 20 days | email plus in-app banner, new load posting blocked for the account (`shipper_accounts.posting_blocked_reason = 'dunning'`) | yes |
| 4 | due date + 30 days | handed to a human collector, task created in the collections queue | no |
| 5 | due date + 45 days | account suspended, open bids cancelled, carriers notified | manual confirmation |

Every step is skipped if the total overdue amount of the account is under 50 EUR (rounding leftovers, small credit-note differences). This threshold was 0 until 2025-12 and we were sending firm reminders for 0.12 EUR.

## Human routing

Accounts with more than 2 500 EUR overdue go to a collector at step 2 already, not step 4. The collectors are two people in finance who work the queue in the back office (`/backoffice/collections`). The queue is sorted by overdue amount descending, then by oldest due date.

Accounts in `dunning_states.paused_until` are skipped entirely. A pause is set by support or finance, always with a reason (`dispute`, `payment_plan`, `known_delay`) and a date; a pause without a date is rejected by the API.

## What stops the ladder

- A payment that brings overdue under 50 EUR resets the account to step 0 and lifts the posting block within the next scheduler run. Shippers complained that the block lasted until the next morning, so since HF-2380 a settled payment event triggers an immediate re-evaluation of that account.
- A dispute opened on an invoice (`invoices.disputed = true`) freezes that invoice's contribution to the overdue amount until the dispute is closed.

## Emails

Templates live in `notifications/templates/dunning/*.mjml`, in FR, DE, PL, NL and EN. The tone of the firm reminder was reworked after the feedback in [[dunning-tone-feedback]]. Every email lists the invoices concerned with their numbers, amounts and a link to the PDF, and the IBAN of the entity for a transfer. Shippers on SEPA mandate get a different first line saying that the debit failed and why (`insufficient_funds` versus `mandate_revoked`), since a reminder that ignores the failed debit looks like a bug to them.
