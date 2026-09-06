---
name: currency-rounding-pln-czk
description: Bids are in EUR, PLN or CZK, VAT is computed per invoice not per line, rounding is half-even on the total, and the NBP and CNB daily rates are frozen at issue for the local-currency VAT amount
type: project
status: active
verified: 2026-06-11
---

Since the Polish entity opened, invoices exist in PLN, and since March 2026 Czech shippers can be billed in CZK by the PL entity. Two things went wrong early and are now rules.

## VAT rounding per invoice, not per line

First implementation computed VAT per line and summed it. With 23 % VAT and PLN amounts to the grosz, an invoice of 40 lines could differ by up to 0.20 PLN from the VAT computed on the total, and the Polish accountant refused it: the JPK_VAT file requires VAT computed on the net total per rate. Same for France in practice, the tax administration tolerates both but auditors ask why.

Rule since HF-2330: `InvoiceTotals` computes `net_total` as the sum of lines, then `vat_total = round_half_even(net_total * rate)`, then `gross_total = net_total + vat_total`. Lines still display a VAT amount for information, but the header is authoritative. The test `fortyLinesOfOddGrosz()` locks it.

Half-even and not half-up: half-up biased the totals by about 0.005 per invoice in the same direction, which over 30 000 invoices a month is a measurable 150 EUR drift in the VAT return. Half-even is what the German advisor recommended and what the PL and FR advisors accepted.

## Local-currency VAT amount

An invoice in EUR issued by the PL entity to a Polish shipper must show the VAT amount in PLN too (art. 106e ust. 11 of the Polish VAT act), converted at the National Bank of Poland rate of the last business day before issue. Same for CZK invoices with the Czech National Bank rate.

Rates are fetched at 16:30 by `FxRateFetcher` into `fx_rates` (`date`, `base`, `quote`, `rate`, `source`). At issue, `invoices.vat_local_currency`, `invoices.vat_local_amount_cents` and `invoices.fx_rate` are frozen. A missing rate for the day (bank holiday not in our calendar) falls back to the previous available day and logs a warning, which matches the legal rule.

The first month we used the rate of the issue day itself instead of the previous business day, which is wrong by one day and produced a 0.4 % difference on 200 invoices. Corrected in HF-2341 with credit notes and re-issue; the shippers did not care but the JPK file had to be resubmitted.

## Display

CZK has no sub-unit in practice (haléř no longer circulates) but the legal amounts still carry two decimals. We keep two decimals in storage and in the PDF; rounding to the crown happens only on the SEPA debit amount, since Payla debits in EUR anyway after conversion at the Payla rate, which is a different rate from the CNB one. The difference between the two rates is booked as FX gain or loss by finance, not adjusted on the invoice.

Payla conversion margin is 0.6 % on CZK and 0.45 % on PLN. Shippers with PLN bank accounts are debited in PLN through the PL merchant account, which has no conversion; only CZK goes through conversion until Payla opens a CZK merchant account for us, promised for Q3 2026.
