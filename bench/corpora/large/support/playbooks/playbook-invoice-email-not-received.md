---
name: playbook-invoice-email-not-received
description: Invoice e-mail not received: recipient on the org, send log, bounce suppression list (hfctl invoice suppression-check), then resend
type: reference
status: active
verified: 2026-05-20
---

# Invoice e-mail not received

Category `invoice:delivery`. The invoice exists and is correct, the customer says it never arrived. Usually a wrong recipient or a bounce that put the address on the suppression list.

## Checks

1. `hfctl invoice get <invoice_id>`: field `sent_to` (list) and `sent_at`. If `sent_at` is empty the invoice was finalized but the e-mail job did not run. That is a backend problem, escalate L2 with the id, do not resend.

2. Compare `sent_to` with what the customer expects. The recipients come from the org's billing contacts (`hfctl org get`, section `billing_contacts`), not from the user who opened the ticket. Half the tickets end here: the invoice went to the accounts payable address the customer set up two years ago. Macro `invoice-recipient-explain`, the customer updates the contacts in their settings.

3. `hfctl invoice suppression-check <email>` for each recipient. A hit shows the reason and date:

- `hard_bounce`: the mailbox does not exist. We stop sending to it after one hard bounce. The customer fixes the address, then `hfctl invoice suppression-remove <email> --apply` and resend.

- `complaint`: someone at the customer marked us as spam. Removal needs the customer's written confirmation in the ticket, then the same command.

- `soft_bounce_x5`: mailbox full or greylisted five times in a row. Remove and resend, and tell them.

4. Not suppressed, correct recipient, `sent_at` present: it arrived somewhere. Two large consumer ISPs put our invoice e-mails in spam for a few weeks in spring 2026 until ops fixed the DMARC alignment on the invoicing sub-domain (`invoices.halden.example`). Since May it is rare. Macro `invoice-check-spam`, and the customer can always download the PDF from the web app, the macro says where.

5. Resend: `hfctl invoice resend <invoice_id> --to <email> --apply`. The resend is logged on the invoice and the PDF is the same file, same number. Do not resend to a new address without the customer confirming it in writing, an invoice is financial data.

## Bulk

A customer who received none of this month's invoices: check the suppression list first, it is almost always that. If not, check Grafana for the invoice e-mail job on the day of the run (dashboard "Billing jobs"). One bad run in December 2025 skipped 212 invoices, the finance runbook covers the recovery, support only communicates.

## Do not

Do not change billing contacts for the customer. Do not attach a PDF to a Deskline reply, the invoice must go through the invoicing channel so that the send is logged.

## Related

Amount disputes are [[playbook-invoice-dispute-amount]].
