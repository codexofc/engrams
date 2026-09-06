---
name: break-glass-admin-procedure
description: When Idento is down, two sealed local accounts (vault password plus hardware key in the safe) give staff_admin for 4 h; every use pages and needs a review
type: reference
status: active
verified: 2026-04-02
---

# Break-glass access

All staff access goes through Idento (see [[sso-oidc-provider-setup]]). If Idento is down, or a staff_admin needs to act on the back-office while the identity provider is in an unknown state, there are exactly two local accounts that bypass it.

## The accounts

- `breakglass-1@halden.example` and `breakglass-2@halden.example`, local accounts in the back-office (`staff_users.auth_mode = 'local'`), the only two rows with that mode. A test asserts that count.

- Each has a 40 character password stored in the ops vault under `iam/breakglass/<n>/password`, readable by the two named holders, and a hardware security key kept in a sealed envelope in the office safe. Both factors are required: the local login form demands WebAuthn.

- Role: `staff_admin`, but with `staff.impersonate` removed. Break-glass is for fixing the platform, not for looking at customers.

- Session: 4 h absolute, no refresh.

## When to use

- Idento is unreachable for more than 15 minutes and a production action needs the back-office (voiding an invoice, holding a payout, disabling a compromised account).

- A staff_admin account is suspected compromised and the person who would normally act is that account.

Not for: convenience when a personal MFA device is forgotten. That is an MFA reset (see [[mfa-rollout-backoffice]]), slower on purpose.

## Procedure

1. Announce in the incident channel: "Using break-glass-N, reason: ...". If there is no incident yet, opening one is the first step (the incidents project has the process).

2. Retrieve the password from the vault (the vault has its own auth, independent of Idento) and the key from the safe. Two people, one for each, when two people are available. One person alone is accepted at 03:00 and reviewed afterwards.

3. Log in at `https://backoffice.halden.example/breakglass`. The route is not linked anywhere and is rate-limited to 3 attempts per hour per account.

4. Do the thing. Every action in the session is audited as usual with `actor_type = 'staff'`, plus the session start emits `breakglass.used` (see [[audit-trail-schema]]) which pages the security rota and emails the head of engineering.

5. Log out. Rotate the password (`bin/console iam:breakglass:rotate N`, writes a new value to the vault). Re-seal the key with a dated signature on the envelope.

6. Within 2 working days: a short review in the incident post-mortem or a dedicated note, listing what was done and why the normal path was not available. The review is the price of the access.

## Testing

Quarterly, one holder performs a full dry run on staging (same accounts exist there with their own secrets) and on production logs in and out without doing anything else. The `breakglass.used` alert must fire; if it does not, the drill is an incident. Last run: 2026-03-12, alert fired in 40 s.

## History

Used for real twice: 2025-10-27 (Idento certificate expiry, 25 minutes) and 2026-02-03 (Idento database failover took longer than expected, 50 minutes). Both reviews concluded the procedure worked and both added a line to this note: the rate limit on the route, and the "two people when available" wording.
