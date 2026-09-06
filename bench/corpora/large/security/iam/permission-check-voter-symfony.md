---
name: permission-check-voter-symfony
description: A single PermissionVoter resolves the organization context, loads a 60 s cached permission set per (user, org), denies by default and logs denials
type: reference
status: active
verified: 2026-04-08
---

# How a permission check actually runs

Controllers and services call `$this->denyAccessUnlessGranted('load.cancel', $load)` or `#[IsGranted('load.cancel', subject: 'load')]`. The string is a `Permission` enum value (see [[roles-permissions-model]]). One voter handles all of them.

## `PermissionVoter`

`App\Security\Voter\PermissionVoter` supports any attribute that `Permission::tryFrom($attribute)` accepts. For anything else it abstains, so Symfony's built-in attributes (`IS_AUTHENTICATED_FULLY`) still work.

The vote does three things:

1. Resolve the **organization context**. For a subject implementing `OrganizationOwned`, the org is the subject's. For a `null` subject (creation, listing), the org is the one in the request header `X-Organization-Id`, validated by `OrganizationContextResolver` against the caller's memberships. No header and more than one membership: 400 `organization_required`. This is where most integration bugs came from in 2025, so the error message spells it out.

2. Load the **permission set** for `(user_id, organization_id)` through `PermissionSetLoader`, which is cached in the request scope and in Redis for 60 s under `perm:{user}:{org}`. The 60 s is why a role change is not instant, see [[session-revocation-on-role-change]] for how we force it.

3. Answer `ACCESS_GRANTED` if the permission is in the set, `ACCESS_DENIED` otherwise. Never `ABSTAIN` for a known permission: with the `unanimous` strategy an abstain from every voter would be a grant, and we do not want that behaviour to depend on strategy configuration.

Staff users bypass organization resolution: their permission set comes from `StaffGroupToRoleMapper` and their subject organization is "any". `staff_support_l1` can therefore read every organization, which is the intended model.

## API keys and service accounts

An API key request has no user. `ApiKeyAuthenticator` builds an `ApiKeyPrincipal` whose permission set is the key's own scopes, intersected with the permissions of the role of the user who created the key at the time of the call (not at creation). So if the creator loses `invoice.read`, keys they created stop reading invoices at the next cache expiry. Lifecycle details in [[api-keys-lifecycle]]. Service accounts work the same way with a fixed role, see [[service-accounts-convention]].

## Denials

Every denial is logged at `info` on the `security.authz` channel with `permission`, `user_id`, `organization_id`, `subject_class`, `subject_id`, `route`. It is not an error: the UI hides what you cannot do, so a denial is either a stale UI or someone probing. Denials per user per hour above 50 raise a Loki alert to the security rota (see the `security/common` family), which in 8 months fired twice, both times a misconfigured integrator script.

The denial is also appended to the audit trail as `authz.denied` (see [[audit-trail-schema]]), sampled at 1 in 10 for the same `(user, permission)` to keep the volume reasonable.

## Tests

`PermissionMatrixTest` is a data provider that walks every `(role, permission)` pair from a fixture copy of `role_permissions` and asserts the voter result. 312 cases in April 2026, 0.4 s. Adding a permission without updating the fixture fails the build, which is exactly the point.

## Things we tried and dropped

- One voter class per resource (`LoadVoter`, `BidVoter`). Twelve near-identical classes, and two of them had drifted to check role names. Merged into the single voter in HF-2033.

- Caching the permission set in the JWT. Tokens got large (a `shipper_admin` has 41 permissions) and a role change needed a new token. The Redis cache is a better trade.
