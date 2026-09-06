---
name: invoice-numbering-legacy
description: Old global INV- counter shared by all entities, retired by HF-2211 in November 2025 for breaking gap-free rules
type: project
status: archived
superseded_by: [[invoice-numbering-sequence]]
verified: 2025-10-02
---

Until November 2025 every invoice took its number from a single Postgres sequence `invoice_number_seq`, rendered as `INV-<8 digits>`. One counter for the French, German and Polish entities together.

Two problems made it untenable:

1. A Postgres sequence is not transactional. Every rollback (a failed Payla charge during checkout, a validation error on the shipper address) burned a number. We measured 3.1 % of gaps over October 2025, 412 gaps for 13 290 invoices. The French auditors flagged it during the FY2024 review and asked for a written explanation for each gap, which nobody could produce.
2. A shared counter across legal entities means the German entity's numbering has holes that correspond to French invoices. The GoBD checklist from our German accountant requires a continuous sequence per entity.

The replacement is described in [[invoice-numbering-sequence]]. The migration HF-2211 kept the old value in `invoices.legacy_number`. Payla payment references created before 2025-11-17 still carry the `INV-` form, and the reconciliation job matches on either column (see [[reconciliation-nightly-job]]).

Do not reuse `invoice_number_seq`; it still exists in the schema only because dropping it blocked on a stale grant, ticket HF-2290 tracks the drop.
