---
name: reconciliation-nightly-job
description: Runbook for the 07:45 reconciliation job that matches Payla settlements and bank statements against payments, the three match tiers, and how to clear the unmatched queue
type: reference
status: active
verified: 2026-06-02
---

The job `ReconciliationJob` runs at 07:45 Europe/Paris, after Payla publishes the settlement CSV of the previous day (available from 07:30, see [[payla-integration-overview]]) and after the bank statements (CAMT.053) of the four entities have been fetched at 07:00.

## Inputs

- `payla_settlement_lines`: one row per settled charge, debit, payout or fee, keyed by `payla_reference`.
- `bank_statement_lines`: one row per transaction from CAMT.053, keyed by `entity_code, statement_date, entry_reference`.
- `payments`: our side, with `payla_reference` when the payment went through Payla, or `expected_transfer_reference` for wire transfers.

## Match tiers

1. Exact: `payla_reference` equal, amount equal. About 97.5 % of lines.
2. Reference in remittance text: for wire transfers, the invoice number (current or legacy format, see [[invoice-numbering-legacy]]) found in `remittance_info` by regex `HF-[A-Z]{2}-(CN-)?\d{4}-\d{6}|INV-\d{8}`, amount equal to the open amount of the invoice or to the sum of several open invoices of the same account (subset sum limited to 6 invoices). About 2 %.
3. Heuristic: same account IBAN as a previous matched transfer, amount equal to one open invoice. Marked `match_confidence = 'low'` and must be confirmed by finance in the back office. Under 0.5 %.

Whatever remains goes to `reconciliation_unmatched` with a reason (`no_reference`, `amount_mismatch`, `unknown_iban`, `duplicate_candidate`).

## Runbook: the unmatched queue

Open `/backoffice/reconciliation/unmatched`. Every morning finance expects fewer than 40 lines. Above 80, check first whether the Payla CSV was complete: the job logs `settlement lines: N` and Payla sometimes publishes a partial file and republishes at 09:30. Re-run with `billing:reconcile --date <yesterday> --source payla` after 09:30 in that case.

Amount mismatches of exactly the Payla fee (0.35 EUR per SEPA debit, 1.4 % + 0.25 EUR per card charge) mean the settlement line is net and our payment is gross. That should not happen since HF-2140 made the importer read `gross_amount`, but the CSV has two amount columns and Payla swapped them once (2025-11, see [[reconciliation-drift-2025-11]]).

Unknown IBAN with a plausible amount: usually a shipper paying from a new bank account. Match by hand and tick `remember iban`, which adds the IBAN to `shipper_bank_accounts` for tier 3 next time.

## Commands

- `billing:reconcile --date 2026-06-01` full run for a day.
- `billing:reconcile --date 2026-06-01 --dry-run` prints the tiers without writing.
- `billing:reconcile:unmatch --payment-id 8812345` undoes a wrong match and puts both lines back in the queue.

The job is idempotent per day; running it twice does not double-match because matches are keyed on the settlement line id.
