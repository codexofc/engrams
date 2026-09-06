---
name: invoice-mart
description: marts.invoice_mart, the finance table: one row per invoice line with load and payment, hourly, blocking test against billing totals
type: project
status: active
verified: 2026-06-12
---

## What it is

Finance and the billing product team read one table: `marts.invoice_mart`. One row per invoice line (`invoice_id`, `line_no`), with the invoice header (number, entity, issue date, due date, VAT rule and rate, status, currency, FX rate), the load it relates to (`load_id`, lane, distance, awarded amount), the carrier and shipper at the time of the invoice (through the carrier dimension of [[carrier-dimension-scd]] and its shipper counterpart), the payment status (`paid_at`, `payment_method`, `days_to_pay`, `dunning_step_reached`) and the credit notes applied (`credited_cents`).

Sharded by `shipper_id` like `core.invoices`, with the load side pre-joined so that nobody has to do the cross-shard join described in [[query-guidelines-analysts]].

## Build

Hourly marmot model ([[marmot-model-runner]]) at H+05, incremental by month of `issued_at`, from `core.invoices`, `core.invoice_lines`, `core.credit_notes`, `core.payments`, `core.loads`, `core.carriers`, `core.shippers`. 3 minutes per partition. The current month and the previous one are recomputed every run (payments arrive for weeks after issue); older months on the weekly pass ([[late-arriving-events]]).

## The blocking test

`marmot test --model marts.invoice_mart` includes `totals_match_billing`: for every closed month and entity, `sum(net_cents)`, `sum(vat_cents)` and `sum(gross_cents)` must equal the figures in `raw.billing_month_close` (published by billing after each month close, one row per entity and month). A difference of even one cent fails the test and, unlike other models, the partition is **not** replaced: finance would rather have yesterday's correct figures than today's wrong ones. The alert goes to both the data and billing channels.

It failed three times since it exists (November 2025): twice because a credit note issued in month M+1 for an invoice of month M was attributed to M by the mart (fixed: credit notes are attributed to their own issue month, which is the accounting rule), once because a KSeF-pending Polish invoice was counted by billing but not yet in the CDC (fixed by treating `ksef_pending` as not issued on both sides).

## Definitions that are contracts

- `days_to_pay`: `paid_at - issue_date` in days, in the entity timezone ([[timezone-convention]]), null while unpaid. Finance's DSO is computed from this column and nowhere else.
- `dunning_step_reached`: the highest step the invoice contributed to, from the dunning state history, 0 if paid on time.
- `is_self_billing`: the invoice was issued on behalf of a carrier.
- `open_cents`: `gross_cents - paid_cents - credited_cents`, the amount still due.

Changing any of these needs a billing reviewer ([[schema-migration-process]]).

## Access

Role `finance` reads `marts.invoice_mart` and nothing under `core`. The controllers' spreadsheet tool connects with `readonly_app` and a saved query per report. The row-level filter by entity that the German advisor asked for is done with a row policy (`CREATE ROW POLICY ... USING entity_code = 'DE' TO finance_de`), the only row policy in the warehouse.
