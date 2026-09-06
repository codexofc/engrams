---
name: payla-integration-overview
description: How Halden Freight uses the Payla payment provider, PaylaClient, the four API flows we call, sandbox vs live config, and the env vars PAYLA_API_BASE and PAYLA_WEBHOOK_SECRET_REF
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
