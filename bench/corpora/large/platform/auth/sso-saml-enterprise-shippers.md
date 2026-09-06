---
name: sso-saml-enterprise-shippers
description: SAML 2.0 SP for enterprise shippers since Feb 2026 (HF-4350): 9 connections live, SP-initiated only, attribute map per connection, enforced mode, JIT off
type: project
status: active
verified: 2026-06-10
---

# SSO for enterprise shippers

## Scope

Large shippers (the ones with 50 or more users, 14 organisations in 2026) asked for login through their own identity provider. We built a SAML 2.0 service provider in `auth-svc` (HF-4350, live 2026-02-16). Nine connections are active in June 2026, covering 3 800 shipper users, 2 % of daily logins and 100 % of the "can we integrate with our IdP" sales questions.

OIDC was considered; every one of the 14 IdPs offered SAML, four did not offer OIDC without an extra licence. SAML it is, with the usual dread.

## Model

`sso_connections(id, org_id, idp_entity_id, idp_sso_url, idp_certificates JSONB, metadata_xml, attribute_map JSONB, enforced boolean, jit_provisioning boolean, created_at, created_by)`. One connection per organisation; an organisation with two IdPs (one merger case) has two rows and the login page asks the user which.

Our SP metadata is at `https://auth.halden.example/v1/auth/sso/metadata`, entity id `https://auth.halden.example/sp`, one signing certificate for AuthnRequests, rotated yearly with a two-month overlap during which both are published.

## Flow

SP-initiated only. `GET /v1/auth/sso/{connection}/start` builds a signed `AuthnRequest`, stores its id in Redis for 5 minutes (`sso:req:<id>`), redirects to the IdP. `POST /v1/auth/sso/{connection}/acs` receives the response, checks: signature against the connection's certificates, `InResponseTo` matches a stored request id (then deletes it), `Destination` is our ACS URL, `NotBefore`/`NotOnOrAfter` with 2 minutes of clock tolerance, audience is our entity id, one assertion, one subject. Then the attribute map turns the assertion into `email`, `given_name`, `family_name`, optionally `groups`.

The user is matched on e-mail within the organisation's verified domains (`org_domains`, which sales verifies by DNS TXT record at connection setup). A match creates a session like a password login would, with `mfa_at` set to the assertion's `AuthnInstant` if the IdP asserted an MFA context class, else the user gets our TOTP challenge (only 2 of 9 IdPs assert it, so 7 connections have "SSO then TOTP", which their admins accepted after a discussion about who owns the second factor).

IdP-initiated login (the IdP posts an unsolicited response) is refused with a 400 and a message explaining how to start from our side. Every IdP vendor guide recommends allowing it and every SAML security review recommends refusing it; we refused. One customer complained, then set up a bookmark app in their IdP portal that points to our `/start` URL, which is what everyone ends up doing.

## Enforced mode

`enforced = true` means every user of the organisation must log in through SSO: password login for those users answers the usual `401 invalid_credentials` (no hint), reset requests are dropped, and the sessions created before enforcement are revoked at the switch. 6 of the 9 connections are enforced. The org admin flips it from the admin page after a warning that lists users who have never logged in via SSO yet.

The break-glass case: the IdP is down, the shipper cannot log in. There is no bypass. This was discussed for a month. Their IdP being down also means their own e-mail and everything else is down, and a bypass that lets an admin disable SSO by password is exactly the account an attacker wants. What exists is support's ability, with the two-proof identity procedure of [[mfa-recovery-codes]], to set `enforced = false` for an organisation with a ticket reference, which has been used once (an IdP certificate expired on a Friday evening).

## JIT provisioning

Off by default. A SAML login for an e-mail unknown to us fails with "ask your administrator to invite you". Two connections have `jit_provisioning = true`: the user is created with the `shipper_user` role and no permissions on any site until an admin grants them. The `groups` attribute mapping to roles was requested by three customers and not built: each customer's group naming is different, the mapping UI would be a project, and the admins currently do it by hand for 20 to 60 users a year each.

## Operational notes

- IdP certificate expiry is the recurring failure. `sso:check-certs` runs nightly and opens a ticket 30 days before any `idp_certificates` entry expires, then e-mails the org admin at 14 and 7 days. Since it exists (April 2026) there has been no expiry outage.

- Clock skew: one IdP was 4 minutes off and every assertion failed `NotBefore`. We did not raise our tolerance beyond 2 minutes; they fixed their NTP.

- The attribute map is the setup pain. The nine live connections took between 1 hour and 3 weeks to set up, the 3-week one because the IdP sent the e-mail in `NameID` with format `unspecified` and in an attribute named after a URN nobody could find in the documentation. The setup page now shows the raw decoded assertion of the last failed attempt to the org admin, which cut the average setup to 2 days.

## Audit

Every SSO event is in `auth_events` with `type` starting `sso.` ([[auth-events-audit-log]]): `sso.start`, `sso.assertion_ok`, `sso.assertion_rejected` (with the reason), `sso.user_unknown`, `sso.enforced_changed`. Rejections in June 2026: 210, of which 180 `InResponseTo` unknown (users double-clicking or waiting more than 5 minutes on the IdP page), 25 signature failures during one customer's certificate rotation, 5 audience mismatches from a customer's test IdP pointed at prod.

## Setting up a connection

The checklist sales and the org admin go through, in order, with the average time each step took over the nine connections:

1. Verify the organisation's e-mail domains by DNS TXT record (`hf-sso-verify=<token>`), 1 day, mostly waiting for the customer's DNS team.

2. Exchange metadata: they import ours from the metadata URL, we import theirs (file upload on the admin page, or URL if they publish one). 1 hour.

3. Map attributes on the admin page, using a test login whose decoded assertion is shown. Median 2 hours, worst 3 weeks (the `NameID` format case).

4. Test with three users, including one who does not exist on our side (to see the "ask your administrator" message) and one admin.

5. Switch `enforced` when the admin says so, never on our initiative. Two connections have been live for months without enforcement because the customer wants password fallback for contractors, which is their call.

6. Confirm the certificate expiry date is in `idp_certificates` and that `sso:check-certs` sees it; the admin gets the 30-day reminder e-mail address confirmed.
