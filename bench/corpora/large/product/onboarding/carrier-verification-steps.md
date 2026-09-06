---
name: carrier-verification-steps
description: A carrier account goes through five verification steps (identity, company, transport licence, insurance, bank) tracked in carrier_verifications, can bid after step 3, gets paid after step 5, and the whole thing takes a median of 2.3 days
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
