---
name: costs-reviewer-preferences
description: How the rotating cost reviewer works: invoices typed by hand on day one, usage before invoice, requests not usage, no internal billing, one saving a month
type: user
status: active
verified: 2026-07-08
---

Habits of the three people (platform lead, data lead, one ops engineer) who rotate monthly as the cost reviewer, as applied in 2026.

- **Day one, by hand.** The first working day of the month, the reviewer types every variable invoice into `finops/costs.yaml` from the PDF and runs the reconciliation ([[vendor-invoices-reconciliation]]) the same morning. Typing by hand is the point: the person reads the invoice. An automated import would have paid the 9 400 expired SMS.

- **Usage before invoice.** The daily anomaly message ([[cost-anomaly-alerts]]) is read every morning by whoever is on the rota; the invoice should confirm what usage already said. A surprise on the invoice is a gap in the usage metrics, and gets a rule.

- **Requests, not usage, for compute.** A team's line moves when its requests move ([[cost-allocation-labels]]). "But we do not use it" is the argument for lowering the request, not for changing the formula.

- **No internal billing.** The per-service table is for decisions. The day it becomes a chargeback, teams will optimise the label instead of the cost. This was decided in 2025 and is re-decided every time someone from finance suggests it.

- **One saving a month, measured.** Each review names one thing done that month and its before-and-after figure, written in `finops/savings.md` with the ticket. Eight entries since October 2025, from 4 200 EUR a month (tiles) down to 90 (a licence nobody used). The list is the answer to "what does the finops hour produce".

- **Nothing under 200 EUR a month is worth an engineer's afternoon.** A change that saves 150 a month and takes half a day to do and half a day to verify pays back in eight months and displaces something better. The threshold is written down so that the argument does not recur.

- **Fixed lines are decided at purchase.** Once a server is bought its cost is sunk for four years; the review looks at fixed lines only when a purchase order or a renewal is on the table ([[reserved-capacity-decision-2026-01]] and the storage drawers). The monthly hour is for the variable lines.

- **Per load, not per user.** The figure quoted to the business is EUR per load per month (0.46 in H1 2026), because loads are what the business counts and users are a proxy.

- **The vendor's usage report is read before the vendor's invoice.** The Skyvale per-host report found the tiles. The Bipline per-country report found the expired billing. The invoice is the last document, not the first.

- **French or English**, whoever writes; yaml keys, labels and service names in English.

- **The review is one hour** ([[finops-monthly-review-feedback]]), with the table, the anomalies file, the savings file and the rightsizing report open, and ends with the next month's one thing named.
