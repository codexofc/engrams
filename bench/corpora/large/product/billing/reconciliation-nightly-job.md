---
name: reconciliation-nightly-job
description: Runbook for the 07:45 reconciliation job: Payla settlements and bank statements, three match tiers, clearing the unmatched queue
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

## Multi-currency lines

Since the PLN and CZK invoices ([[currency-rounding-pln-czk]]), a settlement line and a payment may be in different currencies: a CZK invoice debited in EUR through Payla conversion. Tier 1 compares in the settlement currency using the Payla-reported `converted_amount` and `fx_rate` columns, with a tolerance of 0.5 % to absorb the difference between the Payla rate at submission and at settlement. The FX difference is written to `payments.fx_difference_cents` and booked by finance monthly; over the first half of 2026 it netted to a loss of 1 840 EUR on 2.1 M EUR of CZK collections.

## Bank statement quirks per entity

- FR: CAMT.053 from the bank arrives at 06:40, one file per account. The remittance text is truncated to 140 characters by the bank, which cuts a list of more than 6 invoice numbers; tier 2 therefore only tries subsets up to 6.
- DE: the statement carries the `EndToEndId` of SEPA debits, which is our `payla_reference`, so DE debits match on tier 1 even without the Payla report. Useful when the Payla CSV is late.
- PL: the bank sends MT940, not CAMT; `Mt940Importer` converts it. Polish remittance texts often carry the invoice number without dashes (`HFPL2026000123`); the regex has a dash-optional variant for PL only.
- NL: no quirks so far, three months of history.

## What finance does with a duplicate candidate

`duplicate_candidate` means two settlement lines match the same payment (a Payla replay of a settlement line after a correction, 5 to 10 a month). The back office shows both with the Payla `line_id` and the analyst keeps the later one; the earlier is marked `superseded`. The importer cannot decide alone because in 2 cases of 60 the earlier line was the right one (Payla corrected a fee, then reverted).

## Metrics

- `billing.reconciliation.matched_ratio` per tier, daily. Alert if tier 1 drops below 95 % (it means the Payla file is incomplete or the format changed, see [[reconciliation-drift-2025-11]]).
- `billing.reconciliation.unmatched_open` with the age distribution; the monthly close blocks above 0.5 % of the month's collections.
- Runtime: 4 minutes for a normal day (35 000 lines), 22 minutes on the first business day of the month.

## Who watches it

The billing on-duty engineer checks the 08:15 summary message (`matched by tier`, `unmatched`, `runtime`) every morning; finance opens the queue at 09:00. If the summary is missing, the job did not run, and `billing:reconcile --date <yesterday>` by hand is the first action, before any investigation: finance needs the queue more than we need the root cause at 08:20. The root cause is found after, and it has been the Payla file twice, the bank SFTP once, and a deploy at 07:40 once (deploys are now refused between 07:30 and 08:30 by the deploy tool).
