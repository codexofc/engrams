---
name: carrier-fraud-patterns
description: Four carrier fraud patterns since 2025, the signals that caught each, FraudScorer weights and thresholds, review process
type: project
status: active
verified: 2026-07-21
---

Fraud on the carrier side is rare (about 0.4 % of sign-ups end in a fraud verdict) but expensive: a stolen load of electronics costs more than a year of Verifid fees. This note records the patterns and what caught them.

## 1. Usurpation d'un transporteur réel

Someone registers with the VAT number, licence number and insurance certificate of a real, well-rated carrier (all of which appear on that carrier's own invoices and CMR documents), with a fresh email address and phone. Everything automatic passes because the documents are genuine.

Caught by: the identity step. The person's document does not match anyone in the company's registry filing (the legal representative's name from SIRENE or KRS). Since HF-2200 a mismatch between the Verifid name ([[kyc-provider-verifid]]) and the registry representative sends the file to the queue with `identity_name_mismatch`, and the reviewer calls the company's registered phone number, not the one on the account. 7 cases in 2025, 3 in the first half of 2026.

## 2. Société coquille avec licence achetée

A recently created company (under 6 months), a licence transferred from a dissolved company, an insurance certificate from a broker who does not check much. Bids aggressively low on high-value loads.

Caught by: the combination of company age under 6 months, licence issued to a different company name than the current one (the register shows the transfer), and declared value of the first bids above 30 000 EUR. `FraudScorer` computes a score from these signals at each bid; above 0.7 the award is held for review before the shipper can accept. 5 cases in 2025, all detected before award since the scorer went live in November.

## 3. Changement d'IBAN après activation

The account is legitimate and active for months, then the IBAN changes to a personal account in another country, a week before a large payout. Either a compromised account or an insider.

Caught by: an IBAN change re-triggers step 5 (penny transfer with reference) and holds payouts for 5 business days, with an email to the previous contact address and an SMS to the previous phone. 2 cases in 2025, both stopped. The 5-day hold annoys legitimate carriers who changed bank (about 30 a month) and we keep it anyway.

## 4. Vol de marchandise

A real carrier, real driver, picks up a load of high value and disappears. Not an onboarding failure as such, but onboarding data is the first thing investigators ask for.

Signals we now surface to dispatch: first load above 50 000 EUR for a carrier with fewer than 5 completed loads, pickup in the last hour of the window, a driver phone number that is not in the carrier's declared list. Dispatch gets a warning and calls the carrier's registered number to confirm. 1 case in 2025 (electronics, 180 000 EUR, insured), 0 so far in 2026.

## Rules that exist because of this

- Licence number unique across accounts ([[licence-community-check]]).
- Insurance cover amount enforced against declared load value ([[insurance-certificate-parsing]]).
- IBAN change equals new step 5 plus 5-day hold.
- `FraudScorer` threshold 0.7 for held awards; 0.9 for automatic freeze with review.
- Fraud verdicts are never communicated to the account; the account sees `under_review` and support has a script that does not confirm or deny.

Data on fraud cases lives in `fraud_cases` with restricted access; the figures above are counts only.

## FraudScorer, what it actually computes

The score is a weighted sum in [0, 1] of signals, each a boolean or a bounded ratio, recomputed on every bid and on every account change. Weights as of July 2026:

| Signal | Weight |
|---|---|
| company age under 6 months | 0.15 |
| licence transferred from another company in the last 12 months | 0.20 |
| identity name differs from registry representative | 0.20 |
| IBAN country differs from company country | 0.10 |
| first bids' declared value over 30 000 EUR | 0.15 |
| bid under 70 % of suggestion on a high-value load | 0.10 |
| phone number country differs from company country | 0.05 |
| account created from a network seen on a previous fraud case | 0.05 |

Thresholds: 0.7 holds the award for review, 0.9 freezes the account. The weights were set by hand from the 2025 cases and reviewed once; a learned model was discussed and rejected for now, 20 cases are not a training set. The scorer logs every evaluation above 0.5 to `fraud_scores` with the signal breakdown, and the reviewer sees the breakdown, not the number.

False positive rate at 0.7: 4 % of new carriers with a high-value first bid get held for a review that takes a median of 2 hours during business hours. They are told "additional verification for high-value loads", which is true.

## Review process

Fraud reviews are done by the onboarding lead and one person from finance, never by the regular reviewers, in `/backoffice/fraud`. A case has: the score breakdown, the documents, the registry extracts, the call log, and a decision (`cleared`, `restricted`, `terminated`). `restricted` keeps the account but caps the declared value of loads it can be awarded at 10 000 EUR for 6 months.

Terminated accounts have their licence number, VAT number and IBAN hash added to `fraud_blocklist`, checked at sign-up; a new sign-up hitting the blocklist is created as `under_review` silently.

## Coordination with shippers and insurers

For pattern 4, the shipper's cargo insurer is the one who files the complaint; we provide the onboarding file, the position trail and the driver phone number through the data protection officer, on a written request. The template and the retention of the trail (5 years for loads flagged in a fraud case, overriding the 24-month pseudonymisation of the warehouse) were agreed in January 2026.

We do not share fraud information with other freight exchanges. It was proposed at an industry meeting and refused by legal: no basis to share personal data of a suspected, not convicted, person.
