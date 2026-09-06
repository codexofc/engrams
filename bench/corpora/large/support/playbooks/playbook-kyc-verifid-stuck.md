---
name: playbook-kyc-verifid-stuck
description: Carrier KYC stuck at Verifid: read kyc_state and case, the three rejection reasons (name mismatch, expired document, missing UBO), 15-min cache
type: reference
status: active
verified: 2026-04-30
---

# KYC pending or rejected at Verifid

Category `account:kyc`. A carrier cannot bid, or cannot be paid, because the organisation's KYC is not `VERIFIED`. Verifid does the verification; we store a copy of the state and show it.

## Checks

1. `hfctl org get <org_id>`: `kyc_state`, `kyc_updated_at`, `kyc_case_id`. States: `NOT_STARTED`, `PENDING`, `VERIFIED`, `REJECTED`, `EXPIRED`.

2. `hfctl kyc case <kyc_case_id>`: what Verifid says, as of our last poll. `verifid_status`, `checks[]` with each check's result, `rejection_reasons[]`, `documents_requested[]`. Our copy is refreshed by the KYC worker every 15 minutes and on Verifid's callback, so a customer who just uploaded something may be ahead of us. `hfctl kyc refresh <kyc_case_id> --apply` forces a poll (L1 may run it, it is read-only on Verifid's side).

## By state

**`NOT_STARTED`.** The org never began. Macro `kyc-start` with the path in the web app (Settings, Company, Verification). Nothing to do for us.

**`PENDING` for less than 2 business days.** Normal. Verifid's median is 6 hours, p95 about 30 hours in 2026. Macro `kyc-pending-wait`.

**`PENDING` for more than 2 business days.** Look at `documents_requested[]`. If it is not empty, Verifid is waiting for the customer, and the customer did not see the e-mail. Macro `kyc-documents-requested` with the list. If it is empty and the case is really stuck, escalate to L2 who has the Verifid dashboard.

**`REJECTED`.** `rejection_reasons[]`, the three we see:

- `name_mismatch`: the company name entered in Halden differs from the register extract (typically a trade name versus the legal name, or a missing legal form suffix). The customer edits the legal name in Settings and restarts. Macro `kyc-name-mismatch`.

- `document_expired`: an ID document of the representative is expired. New document, restart.

- `ubo_missing`: beneficial owners not declared for a company with more than 25 % held by another company. The customer must add them. This one generates the most back-and-forth, the macro has a diagram.

A rejected case is restarted by the customer with the button, it creates a new `kyc_case_id`. We do not restart on their behalf.

**`EXPIRED`.** Verification older than 24 months, Verifid requires a refresh. The customer gets e-mails 30 and 7 days before. The org keeps bidding for 30 days after expiry, then loses it. Macro `kyc-expired-renew`.

## Effects while not verified

Bidding refused (`carrier_not_verified` on `POST /v2/bids`), payouts held ([[playbook-carrier-payout-missing]]), drivers disabled with `disabled_reason = org_kyc` ([[playbook-driver-cannot-log-in]]). Explain all three at once, otherwise three tickets.

## Do not

We never look at the documents themselves, we do not have access and do not want it. We never mark an org verified by hand, there is no command for it, and the backend will refuse a ticket asking for it.

## Escalate

L2 for a `PENDING` case with no requested documents after 2 business days, or a rejection reason not listed here.
