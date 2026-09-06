---
name: sso-oidc-provider-setup
description: Staff SSO runs on the self-hosted Idento OIDC provider at sso.hf.internal, one client per app, groups claim mapped to roles, PKCE mandatory since HF-2104
type: reference
status: active
verified: 2026-05-12
---

# SSO: how staff authentication is wired

Everything internal (back-office, Grafana, ArgoCD, the data notebooks, the support console) authenticates against **Idento**, the OIDC provider we run ourselves on `ops-tools`, reachable at `https://sso.hf.internal`. Customer-facing logins (shippers, carriers, drivers) do NOT go through Idento, they use the platform's own password and JWT stack. The two worlds are separate on purpose: a compromise of a staff account must never be a customer session, and the reverse.

## Clients

One OIDC client per application, never a shared one. Naming: `hf-<app>-<env>`.

- `hf-backoffice-prod`, `hf-backoffice-staging`: authorization code flow with PKCE, redirect URIs pinned to `https://backoffice.halden.example/auth/callback` and the staging equivalent.

- `hf-grafana-prod`, `hf-argocd-prod`: same flow, groups claim consumed by the tool's own mapping.

- `hf-support-console-prod`: shorter session (see below) because this one can impersonate customers, see [[impersonation-support-mode]].

PKCE became mandatory for every client in HF-2104 (2026-02) after the pen-test noted that the back-office client still accepted a plain code exchange. Implicit flow and password grant are disabled at the realm level, and `hf-*` clients cannot request `offline_access`.

## Claims and role mapping

Idento emits a `groups` claim. Group names are the source of truth for staff roles and are listed in [[roles-permissions-model]]. The back-office maps them in `StaffGroupToRoleMapper`, configured by `IAM_GROUP_ROLE_MAP` (a JSON object, env var, not a database table, because we want the mapping reviewed in a merge request). Unknown groups are ignored and logged at `warning` with the group name, never denied silently, because two people spent an afternoon in 2025 wondering why a new group did nothing.

The `sub` claim is stable and stored in `staff_users.idp_subject`. Email is displayed, not used as the key: two people changed their email in 2025 and their accounts were not affected.

## Session lengths

| Context | Access token | SSO session | Re-auth for sensitive action |
|---|---|---|---|
| back-office | 15 min | 8 h idle, 12 h absolute | yes, 5 min window |
| support console | 15 min | 4 h absolute | yes, 5 min window |
| Grafana, ArgoCD | tool default | 8 h idle | n/a |

The "sensitive action" re-authentication uses `max_age=300` on a fresh authorization request. Actions in that list: role changes, API key reveal, impersonation start, export of personal data.

## Local development

Developers run Idento from the compose file in `halden-dev-tools` with a fixed realm export (`idento-realm-dev.json`), test users `dev.admin@halden.example` and `dev.viewer@halden.example` with passwords printed by `make idento-users`. Do not add real people to that export.

## Operations

- Realm config is exported nightly to the `halden-infra` repository by a CronJob (`idento-realm-export`) so that a change made by hand in the admin console shows up as a diff the next morning.

- Idento is behind the same alerting as the platform; if it is down, staff cannot log in but customer traffic continues.

- Break-glass access when Idento is down: [[break-glass-admin-procedure]].

The SAML connector that preceded this setup is described in [[sso-saml-legacy-connector]], archived.
