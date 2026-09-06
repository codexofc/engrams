---
name: e-invoicing-mandates-2026
description: E-invoicing status per entity: FR Factur-X and the pending platform choice, PL KSeF live since April, DE inbound, NL none
type: project
status: active
verified: 2026-08-14
---

## Where each entity stands

**France.** Reception of electronic invoices is mandatory for every company from 2026-09-01, emission for large and mid-size companies at the same date, for the rest from 2027-09-01. Halden Freight FR is under the mid-size threshold but most of our shippers are not, and they want structured invoices now. We chose the Factur-X profile `EN16931` embedded in the PDF/A-3 (which is why [[invoice-pdf-rendering]] moved to PDF/A-3 in February). The XML is produced by `FacturXWriter` from the same snapshot as the PDF; a snapshot field missing from the XML profile would be a bug, the test `snapshotFieldsCoveredByFacturX()` enumerates them.

Transmission must go through a certified platform (PDP, now called PA). We have not chosen one. Two candidates evaluated in June 2026, working names `pa-a` and `pa-b` in the evaluation sheet: `pa-a` has an API with webhooks for status (`deposited`, `rejected`, `received`), `pa-b` only offers SFTP drop with a daily status file. Decision expected by 2026-09-15, with the API one favoured unless the price gap (about 0.11 EUR per invoice against 0.04) is judged too large by finance. Ticket HF-2610.

**Poland.** KSeF (national system) mandatory for large taxpayers from 2026-02-01 and for everyone from 2026-04-01. Live since April: `KsefClient` sends the FA(2) XML at issue, receives the KSeF reference number, stored in `invoices.ksef_reference`. An invoice without a KSeF number is not legally issued in Poland, so the PDF is rendered only after the reference is back, and `InvoiceIssuedEvent` is delayed accordingly (median 3.2 s, p99 41 s during their afternoon peak). Failure after 3 retries puts the invoice in `ksef_pending` and an alert; there were 22 such cases in April, 4 in July.

The KSeF session token expires every 2 hours and the first implementation refreshed it lazily on a `401`, which produced a burst of failures at every expiry. Since HF-2552 the token is refreshed at 100 minutes by a scheduler.

**Germany.** Reception of e-invoices mandatory since 2025-01, emission phased until 2028. We receive XRechnung and ZUGFeRD from carriers who invoice us for non-self-billing loads; the inbound parser `EInvoiceInboundParser` handles both. Emission: our Factur-X output is ZUGFeRD-compatible, so DE shippers get the same file.

**Netherlands.** No mandate yet; Peppol is optional and one shipper asked for it. Not planned before the FR platform is chosen.

## What is deliberately not done

- No separate document for the structured invoice: one PDF/A-3 with embedded XML for every entity, and KSeF as an additional channel for PL. Two documents for one invoice would mean two sources of truth.
- Credit notes follow the same path; in Factur-X they are type code 381 and in KSeF a `KOR` invoice referencing the original. See [[credit-notes-flow]].
- Self-billing invoices in FR ([[self-billing-carriers]]) will go through the same platform on behalf of the carrier; the mandate wording was updated in June 2026 to authorise it.

## Factur-X details that cost time

- The `EN16931` profile requires a buyer reference (`BT-10`) when the buyer is a public entity. Two of our shippers are port authorities; their invoices carry the reference they gave us in `shipper_accounts.public_buyer_reference`, and the writer fails the invoice with `facturx.missing_buyer_reference` if it is empty for an account flagged `is_public_entity`.
- Line-level VAT category codes: `S` for standard, `AE` for reverse charge, `O` for out of scope, `E` for exempt. The mapping from our `vat_rule` is in `FacturXWriter.categoryCode()` and a test enumerates every `vat_rule` value to make sure none falls to a default.
- Payment means: SEPA direct debit is code `59` with the mandate reference in `BT-89`; transfer is `58` with the IBAN. An invoice with both (a mandate on file but the debit failed, so we ask for a transfer) uses `58` with a note, which two validators rejected until we removed the second payment means block entirely.
- The XML is validated against the schematron of the profile at issue; a failure blocks the PDF like a KSeF failure blocks the Polish one. 0 failures in production since the schematron was added in May 2026; 40 during the month of testing.

## KSeF operational notes

- Authentication uses a certificate of the PL entity, stored in the vault, renewed yearly (`ksef/pl/cert`). The expiry is monitored; the certificate expiring on a Sunday in March 2027 has a ticket already.
- The FA(2) schema demands the buyer's NIP for Polish buyers and the VAT number with country prefix for EU buyers; a buyer with neither (non-EU) gets the `BrakID` marker. `KsefClient` derives it from `vies_status` and the country.
- KSeF returns an `UPO` (official receipt) per session, not per invoice; we fetch it after the session closes and store it in `ksef_sessions.upo_xml`. The accountant needs the UPO for the month, not per invoice.
- Test environment: KSeF has a public demo environment whose data is periodically wiped, so the staging invoices disappear; the staging reconciliation of `ksef_reference` against KSeF is disabled for that reason.

## Decision log

- 2026-02-10: PDF/A-3 with embedded Factur-X for all entities, one document. Alternative rejected: separate XML file delivered by email.
- 2026-04-01: KSeF live, PDF rendered after the KSeF number. Alternative rejected: render the PDF first with a placeholder and stamp the number later, which would have produced two versions of a legal document.
- 2026-06-15: the mandate for self-billing in FR updated to cover the platform transmission.
- Pending, 2026-09-15: the FR platform choice, `pa-a` favoured. If `pa-b` is chosen for price, the SFTP drop needs a status reconciliation job that does not exist yet, estimated at three weeks of work, which is part of the price comparison.
