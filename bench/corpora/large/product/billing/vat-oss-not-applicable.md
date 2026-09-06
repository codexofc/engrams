---
name: vat-oss-not-applicable
description: Why the EU One-Stop-Shop does not apply to Halden Freight (B2B only) and the two-line answer to give every new accountant
type: feedback
status: active
verified: 2025-11-25
---

Every new accountant, and about one auditor in two, asks whether Halden Freight should register for the OSS (One-Stop-Shop, guichet unique TVA). The answer is no, and it is worth having the reasoning written down because we have re-derived it four times.

## Why not

The OSS covers B2C supplies: distance sales of goods to consumers and services supplied to non-taxable persons in other member states. Halden Freight has no consumer customers. Shippers are companies with a VAT number (mandatory at onboarding, checked against VIES, see [[reverse-charge-intra-eu]]), and carriers are companies too. Every cross-border supply we make is B2B, so the place of supply is the customer's country and the customer self-assesses the VAT. Nothing to declare through OSS.

The confusion comes from the word "platform". Since 2021 marketplaces are deemed suppliers for certain B2C goods sales, and accountants pattern-match "freight exchange" to "marketplace". We are an intermediary for services between businesses; the deemed-supplier rules for platforms do not reach us.

## The two-line answer

"Halden Freight only has business customers with a valid VAT number. All cross-border supplies fall under the B2B general rule (customer's place of establishment, reverse charge), so OSS is not applicable." Then point to the `VatResolverTest` cases and to the memo from the FR advisor dated 2025-03, stored in the finance shared drive under `tva/oss-memo-2025-03.pdf`.

## When to revisit

- If we ever let a sole trader without a VAT number sign up as a shipper (some countries have franchise regimes below a turnover threshold). Today onboarding rejects it with `vat.number_required`.
- If we start selling anything to consumers, for example a paid tracking page for the final recipient of a load.
- If the deemed-supplier rules are extended to services, which the ViDA package discussed for 2028 for passenger transport and accommodation, not freight. Worth re-reading when ViDA is final.

Nothing here changes [[vat-rules-by-country]]; it only explains why that note has no OSS section.
