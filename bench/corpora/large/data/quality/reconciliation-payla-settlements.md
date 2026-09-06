---
name: reconciliation-payla-settlements
description: Daily reconciliation between core.invoices paid amounts and Payla settlements from billing.payla.settlements, per currency, tolerance 100 cents or 5 unmatched settlements, page above, the four kinds of mismatch it finds and what each means
type: reference
status: active
verified: 2026-06-03
---

# Reconciling invoices against Payla settlements

## What is compared

Every morning at 06:30, `invoices_vs_payla_settlements` (`reconciliation`, `page`, owner `billing`, see [[dq-framework-overview]]) compares, for the previous day and per currency:

- **Our side**: `sum(amount_cents)` of `core.invoices` transitions to `paid` with `paid_at` on that day, plus refunds (`credited`) as negatives. From `core.invoice_status_history`, not from the current status, so that a `paid` then `credited` on the same day nets correctly.

- **Payla's side**: `sum(net_amount_cents)` of `raw.payla_settlements` (the warehouse copy of the `billing.payla.settlements` topic) with `settled_at` on that day, per currency, excluding Payla's own fees (a separate `fee_amount_cents` column) and chargebacks (a separate settlement type, reconciled by a second rule with a 30-day window).

Then, at the item level: every settlement's `reference` must match an `invoice_id` on our side with `paid_at` within 2 days, and every `paid` invoice must have a settlement within 3 days (Payla's contractual settlement delay is 1 business day; 3 covers a weekend).

## Tolerances

| Check | Warn | Page |
|---|---|---|
| daily sum difference per currency | 100 cents | 10 000 cents |
| settlements without a matching invoice | 1 | 5 |
| paid invoices without a settlement after 3 days | 3 | 20 |

100 cents of tolerance on the sum: rounding of FX on the rare settlement Payla converts before paying out. In practice the daily difference is 0 on EUR (99 % of the volume) and 0 to 40 cents on PLN.

## The four kinds of mismatch, from experience

1. **Timing.** An invoice marked `paid` at 23:50 CET, settled by Payla at 00:10 the next day UTC. The daily sums differ by that invoice on both days, the item-level match is fine. The rule's sum check runs on `paid_at` in UTC since December 2025 (it ran in CET before, and Payla reports UTC; a month of 2-invoice mismatches every night until someone read the timezone convention). If the sums differ but every item matches, the alert text says "timing only" and stays `warn`.

2. **Our projection is wrong.** An invoice paid in the billing system but `core.invoices` not updated, because the projector was behind or had a bug. Item-level: settlement without matching invoice. This is what the rule exists for. It fired on 2026-03-03 during the invoice projection replay on the streaming side (expected, silenced with the ticket) and on 2026-01-27 for real, when `billing-projector` had skipped 14 messages into its dead letter topic after a schema change it did not know; 14 settlements without invoices at 06:30, page, dead letter reviewed, messages reproduced, rule green by 08:00.

3. **Payla is wrong or late.** Invoices paid, no settlement after 3 days. Happened twice: once Payla held payouts for a merchant KYC re-verification (Verifid on our side had flagged the merchant; the two systems agreed, we had not connected them), once a Payla incident of 2 days in 2025. Both `warn`, escalated to billing, who knew.

4. **Duplicate settlements.** Payla sent the same settlement twice with two `settlement_id` values and one `reference` after a webhook retry storm on their side (2025-10). The item-level match found two settlements for one invoice; the sum was off by exactly that invoice. Our webhook handler now dedups on `reference + settled_at + amount` as well as on `settlement_id`. This is the mismatch that would have cost money, in our favour, and then in reputation when Payla noticed.

## Output

`dq.results.observed` holds the sum difference in cents; `sample_keys` holds up to 20 unmatched references. The alert links to a saved query in the warehouse UI that lists both sides for the day with the join, so that the billing on-call opens the list rather than writing it.

Monthly, the same query over the month is exported for finance's own reconciliation with Payla's statement; they do their reconciliation in their tool, and ours is the early warning. The two have agreed to the cent every month since December 2025.

## What is not reconciled here

- Payla fees against our expected fee schedule: a separate `warn` rule, `payla_fees_vs_schedule`, tolerance 2 %, has fired once (a fee change Payla announced by e-mail that nobody read; the rule was the notice).

- Carrier payouts (`billing.payout.requested` against Payla payout confirmations): same shape, separate rule, owner `billing`, 1-day window, 2 pages since 2025, both Payla-side delays.

- Anything about whether the invoice amount itself was right. That is the SEK story ([[missed-2026-04-sek-invoices-summed-as-eur]]): this rule compared SEK to SEK and was correct while the mart was wrong. A reconciliation checks that two systems agree; it says nothing about whether either is right.
