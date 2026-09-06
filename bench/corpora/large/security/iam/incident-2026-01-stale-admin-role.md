---
name: incident-2026-01-stale-admin-role
description: A contractor kept staff_admin 47 days after leaving because the role row and an API key outlived the SSO group; fixed by deriving roles from the live claim
type: project
status: active
verified: 2026-02-26
---

# Stale staff_admin after a contractor left (January 2026)

## What happened

A contractor who worked on the billing back-office from September to November 2025 had `staff_admin` through the Idento group `hf-staff-admin` (see [[sso-oidc-provider-setup]]). Their contract ended 2025-11-28. The group membership was removed the same day by their manager, which is the documented procedure. Two things were not removed:

1. The row in `staff_users` with `role_code = 'staff_admin'`. `StaffGroupToRoleMapper` wrote the role into `staff_users` at each login and only updated it at the next login. No login, no update. The row said `staff_admin` until someone looked.

2. A personal API key the contractor had created on a **test shipper organization** in production, with `apikey.*` and `load.*` scopes, used by a script that imported fixtures. It was still valid (no expiry at the time, see [[api-keys-legacy-plaintext]]).

Neither was used after 2025-11-28. `audit_events` shows the last `auth.login_succeeded` on 2025-11-27 and the last API key use on 2025-11-26. The row and the key were found on 2026-01-14 by the quarterly access review (compliance project), 47 days later.

## Why it matters even though nothing was used

The Idento account itself was disabled, so the SSO path was closed. But `staff_users` is what the back-office consults for permission checks after the OIDC token is validated, and the OIDC session cookie for the back-office had a 12 h absolute lifetime. Had the contractor left at 09:00 with an open session, they would have had `staff_admin` in the back-office until 21:00 with the group already gone. The API key path was fully open until the review.

## Fixes

- **Roles derived from the live claim, not from the stored row** (HF-2108, deployed 2026-01-22). `StaffGroupToRoleMapper` now recomputes the role from the `groups` claim of the *current* access token at every request. `staff_users.role_code` is kept as a display cache only and is refreshed on every request. Access token lifetime is 15 min, so a group removal takes effect within 15 min.

- **Session revocation on group change**: Idento's admin event webhook (`GROUP_MEMBERSHIP` events) now calls `POST /internal/staff/{subject}/revoke-sessions`, which invalidates the back-office session store entries for that subject. Same mechanism as [[session-revocation-on-role-change]] for customers.

- **API keys created by staff on any organization** now expire after 30 days instead of 365, and the creation UI shows a warning. Staff should not need long-lived keys on customer organizations; when they do, it is a service account (see [[service-accounts-convention]]).

- **Leaver checklist** gained a line: run `bin/console iam:leaver-report <email>` which lists open sessions, API keys created by the user, roles, and organization memberships, and prints the commands to revoke each. The report is attached to the offboarding ticket.

## Detection gap

Nothing alerted. The access review found it because it lists `staff_users` rows without a matching active Idento account, a check added in the Q4 2025 review after a smaller miss. The review cadence (quarterly) is the maximum acceptable delay for this class of problem, and the fixes above are meant to make the review a confirmation rather than the first line.

Ticket: HF-2106 (incident), HF-2108 (fix).
