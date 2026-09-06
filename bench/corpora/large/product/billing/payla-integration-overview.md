---
name: payla-integration-overview
description: How we use the Payla payment provider: PaylaClient, the four API flows, sandbox quirks, env vars and key rotation
type: reference
status: active
verified: 2026-05-11
---

Payla is our only payment provider since March 2025. We use it for card payments by shippers (rare, about 4 % of volume), SEPA direct debit (the bulk, see [[sepa-direct-debit-mandates]]) and payouts to carriers on self-billing.

## Code layout

Everything sits in `billing/payments/payla/`:

- `PaylaClient` wraps the HTTP API. One instance per legal entity because Payla accounts are per entity (`payla_accounts` table, one merchant id each).
- `PaylaWebhookController` receives events at `POST /internal/webhooks/payla/{entityCode}`.
- `PaylaEventProcessor` turns events into domain commands, with the idempotency rules of [[payla-webhook-idempotency]].
- `PaylaReconciliationSource` pulls the daily settlement report for [[reconciliation-nightly-job]].

## The four flows

1. `POST /v2/charges`: card charge, called synchronously at checkout. Returns `charge_id`, status `pending` or `succeeded`. We never trust the synchronous status, the webhook is the source of truth.
2. `POST /v2/debits`: SEPA debit against a mandate reference. Submitted in a batch at 15:00 Europe/Paris (Payla cut-off is 16:00), settles in 2 business days.
3. `POST /v2/payouts`: carrier payout, batch at 11:00 on business days, amount in cents, `reference` = our payout batch number `PO-2026-0412`.
4. `GET /v2/settlements/{date}`: settlement report, one CSV per merchant per day, available from 07:30 the next morning.

Amounts are always integer cents with an ISO 4217 currency. Payla rejects `amount: 12.50`, it wants `amount: 1250, currency: "EUR"`.

## Configuration

- `PAYLA_API_BASE`: `https://api.sandbox.payla.example` in staging, `https://api.payla.example` in production.
- `PAYLA_MERCHANT_ID_FR`, `PAYLA_MERCHANT_ID_DE`, `PAYLA_MERCHANT_ID_PL`, `PAYLA_MERCHANT_ID_NL`.
- `PAYLA_WEBHOOK_SECRET_REF`: the name of the secret in the vault, never the value. The webhook signature is HMAC-SHA256 over the raw body, header `X-Payla-Signature`, with a timestamp in `X-Payla-Timestamp` that we reject if older than 5 minutes.
- `PAYLA_TIMEOUT_MS`, default 8000. Payouts take longer than charges, the batch job overrides to 20000.

Sandbox quirk: the sandbox never sends `debit.failed` events unless the IBAN ends in `00`. Our fixtures use `FR7630006000011234567890189` for success and `FR7630006000011234567890100` for failure.

## Rate limits and errors

Payla allows 50 requests per second per merchant. The payout batch used to hit `429` on the first of the month; since HF-2318 the batch is throttled at 20 rps with a token bucket in `PaylaClient`. Error bodies carry `error.code` (`insufficient_funds`, `mandate_revoked`, `invalid_iban`) and we map them in `PaylaErrorMapper` to our `PaymentFailureReason` enum. Unknown codes go to `PaymentFailureReason.UNKNOWN` and raise an alert, they have meant an API change twice.

The February 2026 outage and what we changed afterwards is in [[payla-outage-2026-02]].

## Sandbox behaviour worth knowing

The sandbox is not a faithful copy of production, and every new engineer loses a day on one of these:

- Settlement reports in the sandbox are generated on demand by `POST /v2/sandbox/settlements/generate`, not at 07:30. The staging reconciliation job calls it at 07:20 for the previous day.
- Webhooks from the sandbox are delivered from a different IP range than production; the staging firewall rule is separate and was the cause of the "staging never receives `debit.settled`" ticket in April 2026.
- Sandbox mandates are auto-registered; the `mandate_not_registered` failure of production (12 % of first debits) cannot be reproduced there. Use the IBAN ending in `77` for it, which the sandbox maps to that error since Payla added it at our request in May 2026.
- The sandbox rate limit is 5 rps, not 50, so the payout batch test uses `PAYLA_PAYOUT_RPS=2`.

## Payout details

Payouts are batched by `PayoutBatchJob` at 11:00 on business days. One `POST /v2/payouts` per carrier with the sum of validated loads since the last batch, `reference = PO-<year>-<sequence>` plus the carrier id in `metadata.carrier_id`. Payla returns `payout_id` synchronously and `payout.settled` or `payout.failed` by webhook, typically the next business day.

Failure codes seen on payouts in the first half of 2026: `invalid_iban` (48 cases, all carriers who typed the IBAN wrong at step 5 and passed the penny transfer because the penny went to a valid but different account of theirs), `account_closed` (11), `compliance_hold` (3, Payla's own screening, resolved by documents within 5 days). A failed payout re-enters the next batch after the carrier fixes the IBAN, and the early-payout discount of 2.5 % is not charged twice.

Payout fees: 0.20 EUR per SEPA payout, 1.1 % for a payout in PLN from the PL merchant, nothing for the same-currency case. The fees appear as separate lines in the settlement report with `type = fee` and are booked by finance from [[reconciliation-nightly-job]]'s import, not from the payout job.

## Key rotation

The API keys per merchant live in the vault under `payla/<entity>/api_key` and are rotated every 90 days by the platform team's rotation job, which calls `POST /v2/keys/rotate` with the old key, stores the new one, and only then revokes the old one after a 10-minute overlap. `PaylaClient` reads the key at every request from the vault client's cache (TTL 60 s), so a rotation needs no restart. The webhook secret is rotated by hand once a year, because the overlap must be handled on our side: `PaylaWebhookController` accepts both the current and the previous secret for 24 hours after `PAYLA_WEBHOOK_SECRET_PREVIOUS_REF` is set.

## Contacts and escalation

Payla support is a ticket portal with a 4-hour first response on the contract we have since January 2026; the incident line (phone, in the vault under `payla/support`) is for outages only and was used once, in February. The technical account manager reviews our integration every quarter and is the person who accepted the sandbox IBAN `77` request and the 30-day notice on file formats. Anything about fees goes through finance, not through engineering, even when the question is technical.
