---
name: vies-vat-check-integration
description: Company verification calls VIES through ViesClient with a 4 s timeout and a 72 h cache, falls back to national registries for FR (SIRENE), PL (KRS and CEIDG) and RO, and treats VIES unavailable as pending rather than rejected
type: project
status: active
verified: 2026-03-11
---

Step 2 of [[carrier-verification-steps]] checks that the company exists and that the VAT number belongs to it. `ViesClient` is shared with billing (they use it at invoice time, same cache in `shipper_accounts` and `carrier_accounts`).

## Flow

1. The carrier enters a VAT number. Format validated client-side per country (`FR` + 11, `DE` + 9 digits, `PL` + 10 digits, `RO` + 2 to 10 digits, `LT` + 9 or 12).
2. `ViesClient.check(vatNumber)` with a 4 s timeout. VIES returns `valid` with the registered name and address for most countries; DE and ES return `valid` without a name (their administrations do not disclose it), which forces a registry lookup.
3. Name comparison between the VIES name and the account name, same normalised Levenshtein as documents (threshold 0.35). Match: `auto_passed`. Mismatch: manual queue with both names shown.
4. VIES down (`MS_UNAVAILABLE`, `SERVICE_UNAVAILABLE`, timeout): the step stays `pending`, the check is retried every 30 minutes for 72 hours by `RetryPendingVies`, and the carrier is told "we are checking your company, no action needed". Before HF-2180 (October 2025) an unavailable VIES was a rejection, which produced 400 support tickets during the four-day VIES outage of September 2025.

## National registries

When VIES gives no name, or when the carrier has no VAT number yet (a new company waiting for it, common in PL), we look up the national registry:

- FR: SIRENE open API by SIREN (first 9 digits after the VAT key). Gives the legal name, the activity code (must be 49.41Z road freight, or 52.29A/B freight forwarding, else manual review) and the status (closed companies are refused).
- PL: KRS for companies, CEIDG for sole traders, by NIP. Sole traders are 61 % of Polish carrier sign-ups, so CEIDG matters more than KRS here.
- RO: the tax agency's public lookup by CUI.
- DE: no online register with a usable API; the carrier uploads a Handelsregister extract of less than 3 months and it goes to the manual queue. This is why DE converts worst in [[activation-funnel-q1-2026]].

Registry results are stored in `carrier_verifications.evidence` with the raw response, so a reviewer sees what the automatic check saw.

## Rate limits

VIES has no documented limit but throttles above roughly 1 request per second per client IP. We queue checks with a 1.2 s spacing, which is enough for onboarding volume (about 300 checks a day) and billing's re-checks (about 900 a day, mostly at the month close).

## Known gap

A VAT number valid in VIES but belonging to a company in a different activity (a bakery registering as a carrier) passes step 2 automatically when the names match. It is caught at step 3, because the licence is issued to a transport company. Not worth a fix on its own; see [[carrier-fraud-patterns]] for the cases it actually let through.
