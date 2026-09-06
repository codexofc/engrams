---
name: playbook-shipper-sso-login-failure
description: Shipper SSO login failure: domain mapping, identity provider change, sso-reset, break-glass admin, and the 7-day e-mail fallback (HF-3180)
type: project
status: active
verified: 2026-08-05
---

# Shipper user cannot log in (SSO)

Category `driver:login` does not apply, this is web users; use `other:sso` until the triage adds a category (three weeks running as of August). Shipper organisations on Business and Enterprise can use SSO (OIDC). When their identity provider changes, or a user's e-mail changes, the link between our user and their identity breaks.

## How the link works

A user row has `sso_subject` (the `sub` claim from the provider) and `email`. On login we match `sub` first, then fall back to e-mail if the org's SSO config has `email_fallback = true` and the e-mail domain is in the org's `sso_domains`. A first login with a new `sub` and a known e-mail re-links silently when fallback is on; when it is off, the user gets "No account matches your identity".

## Checks

1. `hfctl org get <org_id>`, section `sso`: `provider`, `issuer`, `sso_domains[]`, `email_fallback`, `enforced`.

2. `hfctl org users <org_id>`: find the user, read `sso_subject`, `last_login_at`, `login_method`.

3. The user's e-mail domain is not in `sso_domains`: they log in with a password, not SSO, and the SSO button will always refuse them. Macro `sso-domain-not-mapped`, the org admin adds the domain or the user uses the password form.

4. The org changed identity provider (`issuer` changed recently, `hfctl org history <org_id> --field sso` shows when): every user's `sub` is now wrong. With `email_fallback = true`, users re-link on first login, no ticket. With it off, each user fails. The org admin can turn fallback on for 7 days from their SSO settings (HF-3180, July 2026), which re-links everyone as they log in. Macro `sso-provider-changed`.

5. One user only, `sub` mismatch (they were recreated in the provider): `hfctl user sso-reset <user_id> --apply` (L2) clears `sso_subject`; next login re-links via e-mail regardless of the fallback flag, once. The user must confirm from their known e-mail in the ticket.

6. `enforced = true` and the org's provider is down: nobody can log in. The org's designated break-glass admin (`hfctl org get` shows `breakglass_user`) has a password that works even when SSO is enforced. Tell them who it is (a name they already know), not the password, we do not have it. Macro `sso-breakglass`.

## Do not

We never disable `enforced` for an org, that is a security setting the org chose. We never change a user's e-mail. We do not create users in SSO orgs; provisioning is theirs.

## Escalate

L2 for `sso-reset`. Backend if several orgs on the same provider fail at once (a provider-side change, seen once with a certificate rotation in 2026-02).

## Why this is a project note

HF-3180 was opened after the Nordwerk Stahl case (their IdP migration locked out 60 dispatchers for a morning). The 7-day fallback window is the fix; the remaining work is to show the admin which users are still unlinked, scheduled but not done.
