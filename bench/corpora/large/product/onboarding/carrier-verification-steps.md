---
name: carrier-verification-steps
description: Five carrier verification steps (identity, company, licence, insurance, bank), bidding after step 3, payouts after step 5
type: reference
status: active
verified: 2026-06-04
---

## Steps

Each step is a row in `carrier_verifications` (`carrier_id`, `step`, `status`, `checked_at`, `checked_by`, `evidence jsonb`). Status is `pending`, `auto_passed`, `manual_passed`, `rejected`, `expired`.

| Step | What | How | Blocks |
|---|---|---|---|
| 1 `identity` | the person creating the account | identity document plus liveness through the KYC provider, see [[kyc-provider-verifid]] | nothing, but steps 2 to 5 wait for it |
| 2 `company` | legal existence, VAT number, the person's right to represent | registry lookup by country, VIES, see [[vies-vat-check-integration]] | posting a truck profile |
| 3 `licence` | Community licence for hire and reward, valid and matching the company | see [[licence-community-check]] | bidding |
| 4 `insurance` | goods-in-transit insurance (CMR), certificate on file, amount above 100 000 EUR or the load value | see [[insurance-certificate-parsing]] | being awarded a load above 20 000 EUR |
| 5 `bank` | IBAN in the company's name | Payla account verification, penny transfer with a reference | receiving payouts |

A carrier can bid after step 3 and be awarded after step 4 if the load is under 20 000 EUR of declared value. Payouts are held until step 5 (`carrier_accounts.payout_hold = true`), which is why some carriers deliver their first load before giving an IBAN and then ask support where the money is. The email sequence covers this at day 2 ([[onboarding-email-sequence]]).

## Automatic versus manual

Steps 1, 2 and 5 pass automatically in 91 %, 84 % and 97 % of cases. Steps 3 and 4 are automatic in 62 % and 48 %: the licence registers are inconsistent across countries and insurance certificates are free-form PDFs. Everything that does not auto-pass lands in the [[manual-review-queue]].

Document rules (formats, expiry, what is refused) are in [[document-check-rules]].

## Expiry

Licence and insurance have expiry dates. `ExpireCarrierVerifications` runs daily, sets `expired` on the day after expiry, which blocks bidding (step 3) or high-value awards (step 4). Carriers are reminded at 30, 14 and 3 days before. 22 % still let the licence expire, and the block is what makes them upload the renewal (median 1.5 days after block).

## Figures (May 2026)

- 1 940 carrier sign-ups, 1 210 reached step 3 (62 %), 980 reached step 5 (51 %).
- Median time from sign-up to step 3: 2.3 days; p90 9 days. Almost all of the p90 is waiting for the carrier to upload something, not our review time (median review time in the queue: 4 hours).
- Funnel details and evolution in [[activation-funnel-q1-2026]].

## Order and dependencies in code

`VerificationOrchestrator` runs after every document upload, every webhook (Verifid, Payla) and every registry retry. It evaluates the five steps in order but does not require them in order: step 2 can be `auto_passed` while step 1 is `pending`. The only hard dependencies are enforced at the capability level, not at the step level:

- `can_bid = step3 in (auto_passed, manual_passed) AND step1 != rejected`
- `can_be_awarded(load) = can_bid AND (load.declared_value <= 20000 EUR OR step4 passed AND insurance_cover >= declared_value)`
- `can_receive_payouts = step5 passed AND NOT payout_hold`

These three booleans are materialised on `carrier_accounts` by the orchestrator and read everywhere else; no other service re-derives them from the steps. A change in the rules is one place.

## Re-verification triggers

A passed step can go back to `pending` when:

- the underlying document expires (steps 3 and 4, daily job),
- the IBAN changes (step 5, with the 5-day payout hold described in [[carrier-fraud-patterns]]),
- the company's VAT number changes or VIES turns `invalid` at a billing re-check (step 2; billing shares the cache with us and posts `CompanyVatInvalidated`),
- the carrier changes its legal name (step 2 and, by name mismatch, steps 3 and 4).

A step back to `pending` after having been passed keeps the capabilities for 7 days (`grace_until`) so that a carrier with a load in progress is not blocked mid-trip, except for a VIES `invalid`, which blocks bidding immediately because billing cannot issue self-billing invoices for it.

## What the carrier sees

The onboarding home shows the five steps as a checklist with the state and, for `pending`, whether the ball is in their court ("send your licence") or ours ("we are checking, no action needed"). This distinction was added in November 2025 after support measured that 30 % of "where is my verification" tickets were carriers waiting for something we were waiting for them to do.

Estimated times shown: "usually under 4 hours" for a manual review during business hours, "next business day" otherwise, computed from the queue's current median ([[manual-review-queue]]) rounded up.

## Figures by step, May 2026, for the record

| Step | Auto-pass | Manual pass | Rejected | Median time in queue |
|---|---|---|---|---|
| 1 identity | 91 % | 5 % | 4 % | 3 h |
| 2 company | 84 % | 13 % | 3 % | 4 h |
| 3 licence | 62 % | 31 % | 7 % | 4 h |
| 4 insurance | 48 % | 44 % | 8 % | 5 h |
| 5 bank | 97 % | 2 % | 1 % | n/a (penny transfer, 1 business day) |
