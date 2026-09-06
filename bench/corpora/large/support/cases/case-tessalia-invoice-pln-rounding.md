---
name: case-tessalia-invoice-pln-rounding
description: January 2026, Tessalia Textiles disputed 1.37 PLN over 140 lines (per-line vs per-invoice VAT rounding); answered with a signed rounding statement
type: feedback
status: active
verified: 2026-03-05
---

# Case: Tessalia Textiles and the two grosz

Fictional Greek shipper with a Polish subsidiary, Business, invoiced in PLN. `invoice:dispute`, ticket 2026-01-13 from their accounts payable, escalated by them to the account manager on the 15th.

## What happened

Tessalia's ERP imports our invoices line by line and recomputes VAT per line, rounding each line to the grosz. We compute VAT on the invoice total and round once, as the billing rules for PLN say. On an invoice with 140 lines, the two methods differ by up to 0.02 PLN per line in either direction, and the totals differed by 1.37 PLN. Their ERP refused to match the invoice and blocked the payment of 212 000 PLN.

Their accountant wanted us to "fix the invoice". Their ERP vendor said our invoice was wrong. Our billing rules say per-invoice rounding is correct and is what the Polish tax authority accepts.

## What we did

- Support asked billing, billing confirmed: per-invoice rounding, no change. This is a documented rule, not a bug.

- Support wrote a one-page statement of our VAT rounding method, in English and Polish, signed by the finance lead, that Tessalia could attach to the invoice in their ERP as a justification for the 1.37 PLN difference. Their auditors accepted it. Payment released on the 22nd.

- The statement became a standard document, `invoice-rounding-statement`, available in three currencies. Four other customers have asked for it since.

- The account manager offered nothing else. A 1.37 PLN write-off would have been trivial to grant and would have set the precedent that our totals are negotiable.

## What we learned

- "Not a bug" must still get a deliverable. The rounding rule was right, and the customer needed a piece of paper, not an argument.

- Line-level VAT in the invoice PDF was requested by Tessalia so their ERP could match. Billing considered and declined: showing per-line VAT that does not sum to the total invites the exact dispute we had. The PDF now shows the rounding method in the footer instead.

- The triage's list of irritants has this as "accepted, no fix", with the statement as the answer. L1 no longer escalates rounding disputes under 1 PLN, EUR or CZK per invoice; they send the statement.

## Figures

Rounding disputes: 9 in Q4 2025, 11 in Q1 2026 (the statement did not reduce them, it shortened them), median time to close from 6 days to 1 day. Listed in [[case-lessons-recurring-themes-2026-h1]] as the model for "accepted, no fix" answers.
