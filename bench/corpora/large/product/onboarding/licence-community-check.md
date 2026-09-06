---
name: licence-community-check
description: Community licence check by OCR and national registers (FR, PL, LT, RO), certified copy accepted, national-only licences restricted
type: project
status: active
verified: 2026-05-09
---

Step 3 of [[carrier-verification-steps]]. A carrier for hire and reward operating internationally in the EU must hold a Community licence (licence communautaire, Gemeinschaftslizenz, licencja wspólnotowa), issued by its national authority for up to 10 years, with certified true copies for each vehicle.

## What we accept

- The original licence, or a certified copy (they carry the same licence number and the copy number). Accepting the copy was the single most effective onboarding change of 2026, see [[support-findings-onboarding-2026-04]].
- A national licence only (French `licence de transport intérieur`, for instance) is accepted but the carrier is restricted to domestic loads: `carrier_accounts.licence_scope = 'national'`, and bids on cross-border loads are refused with `bid.licence_scope`.
- Vehicles under 2.5 tonnes did not need a licence until May 2022; since then vans between 2.5 and 3.5 t on international routes do. We ask the same document regardless of vehicle size and accept a `light_vehicle` licence scope for those.

## Automatic check

`LicenceExtractor` reads the licence number, the holder name, the issuing authority and the validity dates. Then, where a lookup exists:

- FR: the national register of transport companies, by SIREN, returns the licence status and the number of copies. Auto-pass if the number matches and the status is `valide`.
- PL: the road transport inspectorate register by NIP.
- LT and RO: public lookups by company code.
- DE, NL, BE, CZ, HU: no usable online lookup; OCR result plus name match goes to the [[manual-review-queue]] with the register link when the reviewer can check by hand (DE has a manual portal).

Auto-pass rates by country in May 2026: PL 71 %, FR 68 %, LT 65 %, RO 59 %, DE 0 %, others 0 %. Overall 62 %.

## Validity

Expiry stored in `carrier_verifications.expires_at`. The daily job marks the step `expired` the day after and blocks bidding. Renewals are frequent around the 5-year mark for licences issued in 2021 (many were issued then after a regulatory change), so the queue sees a renewal bump in 2026.

## Frauds seen

- The same licence number on two accounts with different company names: 9 cases in 2025, detected by a uniqueness check on `licence_number` across accounts added in HF-2145. The second account goes to fraud review.
- A licence of a closed company (SIRENE says closed, register says licence withdrawn): 4 cases. The registry lookup at step 2 ([[vies-vat-check-integration]]) catches the closure first.

More patterns in [[carrier-fraud-patterns]].
