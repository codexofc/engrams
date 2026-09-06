---
name: vat-rules-by-country
description: VAT on commissions per shipper country, standard rates table, B2B place-of-supply rule, CH, GB, MC and XI special cases
type: reference
status: active
verified: 2026-06-20
---

Halden Freight bills two things: the commission on each matched load (our own service) and, for carriers on self-billing, the transport itself on their behalf (see [[self-billing-carriers]]). The VAT logic lives in `billing/vat/VatResolver.kt`, and every rule below is a test case in `VatResolverTest`.

## Place of supply

For B2B services between EU businesses the place of supply is where the customer is established (general rule, art. 44 of the VAT directive). So:

- Shipper established in the same country as the billing entity: local VAT at the standard rate.
- Shipper in another EU member state with a valid VAT number: reverse charge, 0 % on our invoice, mention `Autoliquidation` / `Reverse charge, art. 196`. Details in [[reverse-charge-intra-eu]].
- Shipper outside the EU: out of scope, 0 %, mention `Hors champ TVA` and the article reference for the entity country.

The VAT number validity comes from the VIES check stored on the shipper account (`shipper_accounts.vies_status`, values `valid`, `invalid`, `unavailable`). If `unavailable` for more than 72 hours at invoice time we bill with local VAT and issue a credit note later; that decision was taken after HF-1980, when VIES was down for four days in September 2025 and we had 600 invoices blocked.

## Standard rates used

| Entity country | Rate | Since |
|---|---|---|
| FR | 20 % | unchanged |
| DE | 19 % | unchanged |
| PL | 23 % | unchanged |
| NL | 21 % | entity opened 2026-04 |

Rates are configuration, not code: table `vat_rates` (`country`, `rate`, `valid_from`, `valid_to`). A rate change is a new row, never an update, so historical invoices re-render identically.

## Special cases

- Switzerland: not EU, so our commission is out of scope. But Swiss shippers regularly ask for the Swiss VAT number of Halden Freight, which we do not have. Answer prepared in the support macro `vat-ch`.
- United Kingdom since 2021: same as a non-EU country. GB VAT numbers are not in VIES; we validate the format only (`GB` + 9 or 12 digits) and set `vies_status = 'not_applicable'`.
- Monaco is treated as France for VAT (FR rate, FR VAT number format `FR` + 11 characters). The address country is `MC` but `VatResolver` maps it to `FR` before resolution. This bit us once when a Monaco shipper got a reverse-charge invoice; test `monacoIsFrance()` covers it.
- Northern Ireland (`XI` prefix) still counts as EU for goods but not for services, so for us it is GB. `XI` numbers are rejected at onboarding with the message `vat.xi_not_supported`.

## Where the rate is frozen

An invoice stores `vat_rate`, `vat_amount_cents` and `vat_rule` (`local`, `reverse_charge`, `out_of_scope`) at issue time. Re-rendering a PDF (see [[invoice-pdf-rendering]]) never re-resolves VAT. If a rule was wrong, the fix is a credit note plus a new invoice, never an update of the stored fields.

## Self-billing invoices and the carrier side

When Halden Freight issues an invoice on behalf of a carrier ([[self-billing-carriers]]), the supplier is the carrier, so `VatResolver` is called with `supplierCountry = carrier.country` and `customerCountry = shipper.country`. A Polish carrier delivering for a French shipper produces a reverse-charge invoice with the Polish mention, even though the document is rendered by our FR entity. The resolver takes an explicit `SupplyParties` argument since HF-2260; before that it read the entity country implicitly and produced 14 wrong self-billing invoices for PL carriers in one week.

The carrier's VAT status also matters: a Polish sole trader under the `zwolniony` exemption (turnover below 200 000 PLN) invoices without VAT and the mention is `zw.` with the legal basis (art. 113 ust. 1). We store `carrier_accounts.vat_regime` with values `standard`, `exempt_small_business`, `not_registered`, and `VatResolver` refuses to issue a self-billing invoice for `not_registered`.

## Rate change procedure

A rate change (the last real one for us was the Dutch entity opening, not a change of an existing rate) follows four steps, in this order:

1. Insert the new row in `vat_rates` with `valid_from` set to the legal date, at least 7 days ahead. The back office refuses a `valid_from` in the past.
2. `billing:vat:preview --country PL --date 2026-07-01` renders 20 sample invoices with the new rate and posts the PDFs to the billing channel for the accountant.
3. On the legal date, invoices resolve the new rate automatically by `issue_date`. Credit notes for invoices issued before keep the old rate, from the stored `vat_rate`.
4. The month-close export ([[billing-runbook-month-close]]) groups lines by rate, so a month with two rates for the same country produces two blocks, which the JPK and the German ledger export both accept.

## Tests that guard this

`VatResolverTest` has 64 cases in June 2026. The ones people forget exist:

- `monacoIsFrance()` and `northernIrelandIsGbForServices()`.
- `viesUnavailableBeyond72hBillsLocalVat()`, the HF-1980 rule.
- `issuedInvoiceKeepsItsVatRule()`, shared with [[reverse-charge-intra-eu]].
- `selfBillingUsesCarrierAsSupplier()` and `exemptSmallBusinessCarrierHasNoVat()`.
- `rateChangeMidMonthProducesTwoExportBlocks()`, which runs the export on a synthetic month.

A change to `VatResolver` without a new test case is refused in review; the accountant reads the test names in the release note, which is the only piece of code they read.

## When in doubt

The VAT rule of a specific invoice is answered by `billing:vat:explain --invoice HF-FR-2026-000123`, which prints the parties, the stored rule, the VIES reference and the rate row that applied. Support has it as a back-office button. If the explanation looks wrong, the fix is a credit note ([[credit-notes-flow]]), and the resolver gets a new test case with the invoice's parties; do not argue with the stored rule on an issued invoice.
