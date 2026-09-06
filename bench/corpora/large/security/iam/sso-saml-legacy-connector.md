---
name: sso-saml-legacy-connector
description: Until October 2025 the back-office used a SAML connector keyed on email with no group claim, roles set by hand in staff_users and 24 h sessions
type: reference
status: archived
superseded_by: [[sso-oidc-provider-setup]]
verified: 2025-10-05
---

# Legacy SAML connector (removed in HF-2041)

Kept for archaeology: some `staff_users` rows and old audit entries carry `auth_mode = 'saml'`. Current setup is in [[sso-oidc-provider-setup]].

## What it was

- SAML 2.0 SP in the back-office using a PHP SAML toolkit, IdP was the same Idento instance in SAML mode. Metadata exchanged by hand, the SP certificate was self-signed and valid 3 years, renewed once (2024).

- `NameID` was the email address. When someone's email changed (marriage, domain migration in 2024), a new `staff_users` row appeared and the old one kept its roles. Two such orphan rows were found in the Q3 2025 access review.

- No group or role assertion. Roles were set by hand in `staff_users.role_code` by an admin, so onboarding meant "log in once, then ask an admin to promote you", and offboarding meant remembering to demote. The stale-role class of problem later described in [[incident-2026-01-stale-admin-role]] was structural here, not accidental.

- Session: SAML assertion consumed once, then a 24 h PHP session cookie. No idle timeout.

- No re-authentication for sensitive actions; the concept did not exist.

## Why it was replaced

- The pen-test of October 2025 flagged the 24 h session, the absence of MFA enforcement at the SP level, and an XML signature wrapping check that relied on a library version two years old.

- Grafana and ArgoCD were already on OIDC against the same realm; running both protocols meant two sets of client configs and two sets of rules.

- Group claims through OIDC gave us role mapping in configuration instead of in a table maintained by hand.

## Migration (HF-2041, 2025-10-14)

Single switch-over during a Tuesday lunch break. `staff_users.idp_subject` was backfilled by matching email against Idento's user list (82 of 84 matched; the 2 orphans were deleted). The SAML SP endpoint `/saml/acs` returned 410 for 30 days with a message, then was removed. No one used it after day 2.
