---
name: credit-notes-flow
description: Credit notes are the only correction of an issued invoice, partial credit allowed, finance approval above 5 000 EUR, never automatic
type: project
status: active
verified: 2026-04-17
---

An issued invoice is immutable. Any correction (wrong amount, wrong VAT rule, cancelled load, commercial gesture) goes through a credit note (avoir), then possibly a new invoice. Numbering of credit notes is described in [[invoice-numbering-sequence]] (separate `CN` sequence per entity and year).

## Data model

- `credit_notes`: `id`, `number`, `invoice_id` (mandatory, foreign key to the invoice being corrected), `reason` (`cancelled_load`, `pricing_error`, `vat_error`, `commercial_gesture`, `dispute_settlement`), `amount_cents`, `vat_amount_cents`, `status` (`draft`, `pending_approval`, `issued`), `approved_by`.
- `credit_note_lines`: one line per invoice line credited, with `quantity` and `unit_amount_cents`, so a partial credit is a line with a smaller quantity or amount than the original.
- The VAT on a credit note is computed from the original invoice's stored `vat_rate` and `vat_rule`, never re-resolved; a credit note for a VAT error therefore credits the full original and the corrected invoice is issued with the right rule (see [[vat-rules-by-country]]).

## Flow

1. Support or finance creates the credit note from the invoice page in the back office (`POST /internal/invoices/{id}/credit-notes`). The API refuses if the sum of issued credit notes plus this one exceeds the invoice total (`credit_note.exceeds_invoice`).
2. Under 5 000 EUR excluding VAT the credit note is issued immediately. Above, it goes to `pending_approval` and a finance lead approves in the queue `/backoffice/approvals`. Threshold in `BILLING_CREDIT_NOTE_APPROVAL_THRESHOLD_CENTS`, default 500000.
3. On issue: PDF rendered (same pipeline as [[invoice-pdf-rendering]]), email to the shipper's billing contact, open amount of the invoice reduced, dunning state re-evaluated.
4. If the invoice was already paid, the credit becomes a balance on the account (`shipper_accounts.credit_balance_cents`) and is deducted from the next invoice. A refund to the bank account is only done on request, through a Payla payout, and only for the full balance.

## Decisions

- No negative invoices. Some accountants asked for it; the German entity's advisor was firm that a credit note must be a distinct document type.
- Credit notes are never issued automatically by the system, even for a cancelled load. The cancellation creates a task for support; automating it produced 200 credit notes in one night in October 2025 when a bug cancelled loads in bulk (HF-2105), and reversing credit notes is painful.
- A credit note cannot be credited. If a credit note is wrong, issue a new invoice for the amount.

## Figures

March 2026: 1 104 credit notes for 28 900 invoices (3.8 %). Reasons: `cancelled_load` 61 %, `pricing_error` 22 %, `commercial_gesture` 9 %, `vat_error` 5 %, `dispute_settlement` 3 %. The `pricing_error` share is the one we are trying to reduce with the pricing team.
