---
name: kyc-provider-verifid
description: Identity checks by the Verifid hosted flow, webhook result in 2 minutes for 94 %, 0.9 EUR per check, verdict stored, never the images
type: reference
status: active
verified: 2026-02-27
---

Step 1 of [[carrier-verification-steps]] (identity of the person opening the account) is delegated to Verifid, a KYC provider. We chose a hosted flow: the carrier is redirected to Verifid's page, takes a photo of the identity document and a selfie, and comes back.

## Integration

- `VerifidClient.createSession(carrierId, locale)` returns a session URL, valid 30 minutes. We support FR, DE, PL, RO, LT, EN locales; Verifid handles 40 document types across the EU plus Ukrainian and Moldovan passports, which matter for drivers but not for account owners (the owner must be an EU-registered company representative anyway).
- The result arrives on `POST /internal/webhooks/verifid` with an HMAC signature (`X-Verifid-Signature`, SHA-256 over the body). Result `approved`, `declined` (with reasons: `document_expired`, `face_mismatch`, `document_tampered`, `unsupported_document`), or `review` when their automation is unsure and a human on their side takes over (up to 24 hours, 3 % of sessions).
- 94 % of results arrive within 2 minutes of the carrier finishing; the app polls `GET /api/v1/onboarding/identity/status` every 5 seconds for 3 minutes, then tells the carrier to come back later.

## What we store

`carrier_verifications.evidence` for step 1 contains: `verifid_session_id`, the verdict, the reasons, the document type and issuing country, the first name and last name as read, and the date of birth's year only. We never receive the document images or the selfie; the Verifid contract keeps them on their side for 30 days then deletes them. This was a deliberate choice with the data protection officer: fewer copies of identity documents.

The name read on the document is compared with the account contact name; a mismatch does not fail the step but flags it in the [[manual-review-queue]] with `identity_name_mismatch`, because spouses and accountants sign carriers up for them, and the actual representative is checked at step 2 against the registry.

## Cost and volume

0.9 EUR per completed session, 0.3 EUR per abandoned one. About 2 000 sessions a month, 1 850 EUR. The abandon rate is 11 %, mostly on the selfie step on old Android phones.

## Failure handling

- Verifid down: the step stays `pending`, the carrier can continue to step 2 (registry checks do not depend on identity), and `RetryPendingVerifid` re-creates a session and emails the link after 2 hours. Happened twice in 2025, 40 minutes and 3 hours.
- A `declined` for `document_expired` lets the carrier retry immediately with another document. `document_tampered` blocks the account and creates a fraud review; see [[carrier-fraud-patterns]].
