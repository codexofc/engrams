---
name: vat-rules-by-country
description: VAT applied on Halden Freight commissions per shipper country, standard rates table, place-of-supply rule for B2B transport services, and the CH and GB special cases
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
