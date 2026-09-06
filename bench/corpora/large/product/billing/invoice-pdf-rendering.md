---
name: invoice-pdf-rendering
description: Invoice PDFs are rendered once at issue by the render-svc worker from a frozen JSON snapshot, stored in the invoices bucket under entity/year/number.pdf, and re-rendered only from the snapshot
type: project
status: active
verified: 2026-03-30
---

The PDF of an invoice or credit note is produced by the `render-svc` worker from a JSON snapshot taken at issue time (`invoices.render_snapshot`), not from live data. The snapshot contains every string that appears on the document: addresses, VAT numbers, lines, rates, legal mentions, bank details of the entity. If the shipper changes its address tomorrow, yesterday's invoice must still show yesterday's address.

## Pipeline

1. `InvoiceIssuedEvent` is published on the internal bus with the invoice id.
2. `render-svc` consumes it, loads the snapshot, picks the template `invoice-<entity>-<lang>.html` (Thymeleaf-like templates in `render-svc/templates/`), renders HTML then PDF through the headless renderer, and stores the file in the object store bucket `invoices` at `<entity>/<year>/<number>.pdf`, e.g. `FR/2026/HF-FR-2026-000123.pdf`.
3. The object key is written back to `invoices.pdf_key` and the email to the shipper is sent only after that. A shipper never receives an email with a broken link; the email job waits up to 10 minutes for `pdf_key`, then alerts.

Rendering takes 350 ms median, 1.2 s p99, most of it in font loading. The worker keeps the fonts in memory since HF-2201 (before that, 2.1 s median).

## Re-rendering

`billing:invoice:rerender --number HF-FR-2026-000123` regenerates the PDF from the same snapshot. Used when a template bug is fixed (wrong logo, missing mention). The number, amounts and VAT never change because they come from the snapshot; see [[vat-rules-by-country]] for why VAT is frozen. A re-render overwrites the object at the same key and bumps `invoices.pdf_version`.

If the snapshot itself is wrong (it happened once, HF-2260, a missing reverse-charge mention because the snapshot builder ignored `vat_rule`), the fix is a credit note and a new invoice, as described in [[credit-notes-flow]]. We do not patch snapshots.

## Template rules

- One template per entity and language, no conditionals on entity inside a template. It made templates longer but the legal mentions differ enough that a shared template was unreadable.
- Amounts formatted by locale: `1 234,56 €` in FR, `1.234,56 €` in DE, `1 234,56 zł` in PL. Formatting lives in `MoneyFormatter`, the templates never format numbers themselves.
- The PDF is PDF/A-3 since 2026-02, which the e-invoicing work requires (see [[e-invoicing-mandates-2026]]). PDF/A-3 forbids transparent PNGs; the logo is now an SVG.
- Page 2 onwards repeats the invoice number and page count in the footer. A German shipper's accounting department rejected invoices without it.

## Storage

Object store with versioning, retention lock of 10 years per entity (legal retention for FR and DE). Deleting an invoice PDF is not possible through the API; a GDPR request on a shipper contact only anonymises the contact name in the database, the PDF keeps the company name, which is the legal requirement.
