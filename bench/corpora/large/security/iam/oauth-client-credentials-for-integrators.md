---
name: oauth-client-credentials-for-integrators
description: TMS vendors get an OAuth client (client_credentials, 10 min tokens) plus per-customer consent grants in integrator_grants instead of collecting API keys
type: project
status: active
verified: 2026-07-08
---

# Integrator clients (HF-2170)

## Why API keys were the wrong tool

A TMS vendor that serves 200 carriers was, until mid 2026, asking each carrier for a Halden Freight API key and storing 200 keys in their database. We had no way to see that those 200 calls came from one vendor, the vendor's support could not rotate a key without the carrier, and if the vendor leaked their database we had 200 incidents. Three vendors were in this situation with a total of 540 customer keys.

## The model

An **integrator** is an organization of audience `integrator` (new in [[roles-permissions-model]], no user roles, no loads of its own). It has one or more **OAuth clients**: `oauth_clients (id, integrator_id, client_id, client_secret_hash, allowed_scopes, redirect_uris)`. Secrets are hashed like API keys (see [[api-key-hashing-and-prefix]]), prefix `hfc_`.

The integrator obtains a token with `client_credentials` at `POST /oauth/token`. Token lifetime is **10 minutes**, JWT, `aud = api.halden.example`, `sub = client_id`, and it carries no customer context by itself.

To act for a customer, the integrator sends the token plus `X-Organization-Id: <customer org>`. `IntegratorGrantResolver` then checks `integrator_grants (integrator_id, organization_id, scopes, granted_by, granted_at, revoked_at)`. No grant, or revoked, or scope missing: 403 `integrator_not_authorized`.

## Consent

The grant is created by the **customer**, not by us and not by the integrator. Two paths:

1. Authorization code flow: the vendor's UI sends the customer's admin to `https://app.halden.example/oauth/authorize?client_id=...&scope=load.read bid.create`, the admin logs in, sees "Vendor X asks to read your loads and place bids", accepts. We redirect with a code, the vendor exchanges it and we create the grant. No refresh token is issued; the grant is what persists, the client_credentials token is what authenticates.

2. Manual: the customer admin goes to `/settings/integrations`, picks the vendor from a list of approved integrators and ticks scopes. Used by customers whose vendor has not implemented the redirect.

Revocation: the customer admin clicks "disconnect", `revoked_at` is set, the vendor's next call for that organization gets 403, and a webhook `integrator.grant_revoked` tells the vendor if they registered one.

## Audit and visibility

Calls made through an integrator grant are audited with `actor_type = 'integrator'`, `actor_id = client_id`, `organization_id` of the customer, and `details.grant_id` (see [[audit-trail-schema]]). The customer's audit view shows "via Vendor X" on each row. Two customers told us in the first month that this was the first time they could see what their TMS did on their behalf.

## Migration

The three vendors migrated between May and July 2026. The old customer keys they held were disabled by the customers themselves, prompted by a banner "This key is used by Vendor X, who now connects directly; you can disable it". 498 of 540 disabled by July, the rest expire by rule (see [[api-keys-lifecycle]]).

## Rate limits

Per integrator, not per customer: 3 000 requests per minute, with a per-organization sub-limit of 600 so that one customer's batch import does not starve the others behind the same client.

## Not done

- Refresh tokens for integrators. `client_credentials` every 10 minutes is cheap and removes a stealable long-lived token.

- Letting integrators create organizations. Onboarding is a customer action with KYC, see the onboarding project.
