---
name: session-revocation-on-role-change
description: Removing a role or disabling a user rotates users.security_stamp, invalidating refresh tokens and the permission cache; worst case is the 15 min access token
type: project
status: active
verified: 2026-02-12
---

# Making a role change take effect (HF-2101)

## The problem

Permissions are cached 60 s per `(user, organization)` in Redis (see [[permission-check-voter-symfony]]) and access tokens are JWTs valid 15 min. A `shipper_admin` who removes a dispatcher's role expected the dispatcher to lose access "now". In practice the dispatcher kept working for up to 15 min, and refresh tokens kept minting new access tokens after that because nothing tied them to the role.

This became urgent after [[incident-2026-01-stale-admin-role]] on the staff side, and after a shipper reported that an employee they had removed on Friday afternoon still accepted a bid on Friday evening from the mobile app, whose refresh token was valid 30 days.

## The mechanism

One column: `users.security_stamp`, a random 16 byte value. It is embedded in every access token (`claim sst`) and stored alongside every refresh token row (`refresh_tokens.security_stamp`).

`SecurityStampRotator::rotate(User $user, string $reason)` writes a new stamp and:

1. deletes `perm:{user}:*` from Redis, so the next permission check reloads;

2. leaves the refresh token rows in place but they no longer match, so the next refresh fails with 401 `session_revoked` and the client goes back to login;

3. records `auth.session_revoked` in the audit trail with `details.reason` (see [[audit-trail-schema]]).

Access tokens already issued remain valid until their `exp`, at most 15 min. `JwtAuthenticator` does NOT check the stamp against the database on each request, on purpose: that would be one query per request and would turn the JWT into a session cookie. Fifteen minutes is the accepted worst case and it is written in the customer-facing security documentation.

## What triggers a rotation

- `member.role_changed` when a role is removed or replaced (not when one is added).

- `member.removed`.

- `users.disabled_at` set, including through SCIM (see [[scim-provisioning-shipper-tenants]]).

- Password change and MFA reset.

- The user clicking "log out everywhere".

- Staff `staff_admin` clicking "revoke sessions" on a user in the back-office, typically during an incident.

Adding a role does not rotate: it would log out the user for no reason and the 60 s cache handles it.

## The mobile app

Drivers get a 401 `session_revoked` on their next sync and are sent to the PIN screen with the message "Votre accès a été modifié, reconnectez-vous". Pending offline uploads (PODs, positions) are kept on the device and sent after re-login if the user still has the permission, dropped otherwise after 7 days. The mobile team insisted on the 7 day grace period so that a driver who was mistakenly removed and re-added does not lose their proof of delivery photos.

## Numbers

In January 2026, 1 140 rotations, 88 % from role removals. Median time from admin click to the user's refresh failing: 4 min 30 s (most access tokens are more than half spent when the change happens). Support tickets about "removed user still active": zero since deploy, against 5 in Q4 2025.
