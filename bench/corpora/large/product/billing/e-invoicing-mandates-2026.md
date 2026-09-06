---
name: e-invoicing-mandates-2026
description: Status of the electronic invoicing obligations per entity (FR Factur-X via a certified platform from September 2026, PL KSeF, DE inbound-only since 2025), what is built and the open decision on the FR platform
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
