---
name: access-review-quarterly
description: Each quarter system owners review generated lists of who has what across 11 systems and sign off in the ticket; unreviewed access is removed after 10 days
type: reference
status: active
verified: 2026-04-28
---

# Quarterly access review

The review exists to catch what offboarding, role changes and "temporary" grants miss. It found a stale `staff_admin` in Q1 2026 (the IAM project has the incident note) and has removed between 20 and 40 grants every quarter since it started in Q3 2025.

## Cadence and ownership

First two weeks of January, April, July, October. One YouTrack ticket per quarter (`Access review 2026-Q2`) with a checklist, one line per system, each line owned by the system owner. The security rota drives it, owners do the work.

## Systems in scope

| System | Source of the list | Owner |
|---|---|---|
| Staff roles (Idento groups) | `iam:review:staff` export | head of engineering |
| Back-office local accounts (break-glass) | same export | security |
| Vault policies and tokens | vault audit export | ops |
| Cloud provider IAM users and roles | provider CLI export | ops |
| Kubernetes RBAC bindings (humans) | `kubectl` export from both clusters | ops |
| Database roles with login | `pg_roles` on each cluster | ops |
| Vendor consoles (payments, KYC, email, telematics, support tool) | manual, screenshot of the user list | owner of the vendor relationship |
| Source forge groups and deploy tokens | forge API export | platform |
| Customer organizations with a staff member as `shipper_*` or `carrier_*` | `iam:review:staff-in-orgs` | support lead |
| Service accounts with no call in 60 days | `iam:review:service-accounts` | each owning team |
| Integrator grants with no call in 90 days | `iam:review:integrators` | integrations |

The exports are generated on day 1 by `bin/console compliance:access-review --quarter 2026-Q2`, written to `hf-compliance/access-reviews/2026-Q2/*.csv`, and attached to the ticket.

## What an owner does

For each line in their list: keep, change, or remove. "Keep" needs no comment when the person is on the current team roster. Anyone not on the roster (contractor, someone who moved teams, a shared account) needs a one-line justification. "Change" and "remove" become sub-tasks assigned to whoever can make the change, due within the review window.

Signing off is a comment "Reviewed, N kept, N changed, N removed" on the ticket. No comment by day 10 means the security rota removes every grant on that list that is not on the roster, then tells the owner. This happened once (Q4 2025, a vendor console); it was not popular and it has not been needed again.

## Rules of thumb that came out of the reviews

- Staff members who belong to a customer organization for testing must use organizations flagged `is_probe` or `is_internal`. A staff member inside a real customer organization is removed unless support explains why (typical valid reason: a customer asked for hands-on onboarding, time-boxed).

- A service account with no call in 60 days is disabled, not deleted, and deleted at the next review if nobody claimed it.

- Vendor consoles are the weakest link: no export API, no SSO for two of them, and the person who signed the contract is often the only admin. Two consoles were moved behind SSO in 2026 as a result.

## Evidence

The ticket, the CSVs and the sub-tasks are the evidence for the security questionnaire ([[vendor-security-questionnaire]]) and for the DPA audit clauses ([[dpa-carriers-template]]). Results of a specific quarter: [[access-review-2026-q1-results]].
