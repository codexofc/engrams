---
name: device-trust-and-remember-me
description: A trusted device is a cookie hf_dt holding a random 32-byte id matched against trusted_devices, valid 30 days, skips the MFA challenge but never the password, revoked on reset, at most 5 per user, no fingerprinting
type: reference
status: active
verified: 2026-05-14
---

# Device trust ("remember this device")

## What it is and is not

After a successful login with MFA, the client may ask to trust the device. `auth-svc` then sets a cookie `hf_dt` on `auth.halden.example` (HttpOnly, Secure, SameSite=Lax, 30 days) containing a random 32-byte identifier, and stores `sha256(id)` in `trusted_devices` with the user, a label derived from the user agent at that moment ("Firefox on Windows"), and the dates. On the next login, if the cookie matches a non-expired, non-revoked row of the same user, the MFA challenge is skipped. That is the whole feature.

It is not a session: the password is always required. It is not a fingerprint: we do not compute anything from the browser beyond the label, and the label is display only. A trusted device is a cookie, and a stolen cookie is a stolen trust, which is why the trust is bounded in time and scope.

## Rules

- 30 days from creation (`expires_at`), not sliding. A dispatcher who logs in every day gets the MFA challenge once a month per browser. The product team asked for sliding; refused because a trust that never expires while in use is a trust that never expires.

- At most 5 trusted devices per user. Creating a sixth revokes the oldest by `last_seen_at`. Median is 1.4 per user, the maximum 5 is reached by 900 users, mostly dispatch offices with shared workstations (see [[abuse-shared-dispatcher-accounts]] for what that means).

- Revoked on password reset, on password change, on MFA reset, on `revoke-all` sessions, and individually from the "connected devices" page (`DELETE /v1/me/devices/{id}`).

- Trust never applies to step-up operations: bank account change, MFA settings, SSO connection changes, and support impersonation always require a fresh TOTP code (`mfa_at` within 5 minutes on the session), device or not.

- Trust is per user. Two users on the same browser have two cookies (the cookie is set with the user id in the row, and a cookie matching a row of another user is ignored and logged as `device.mismatch`). This happens 400 times a month, always shared workstations.

## Storage

`trusted_devices(id, user_id, device_hash, label, first_seen_at, last_seen_at, expires_at, revoked_at, revoke_reason)`, index `(user_id, revoked_at)`. 96 000 active rows in May 2026 for 68 000 users with MFA. Purged 90 days after expiry or revocation.

## What the cookie looks like from the outside

Clearing browser cookies loses the trust, and the user gets the MFA challenge. Private browsing loses it every time. Browser profiles are separate trusts. Support has a canned answer for all three, and "why does it keep asking for the code" is the most frequent MFA ticket since the wall (see the support numbers in [[mfa-totp-rollout-dispatchers]]).

Corporate proxies that strip cookies on `auth.halden.example` were a real case: one carrier's proxy dropped `Set-Cookie` headers with `SameSite` attributes it did not understand. Their 60 dispatchers got the challenge at every login for two weeks until their IT updated the proxy. We did not remove `SameSite`.

## Decisions

- **No fingerprinting.** A browser fingerprint (canvas, fonts, screen) would let us skip the cookie and "recognise" the device. It also breaks on every browser update, is contested under privacy rules, and gives a false sense that the device is identified when it is only approximately guessed. The cookie is honest: it says "this browser was here and the user asked us to remember it".

- **No IP binding.** Dispatch offices are behind NAT; a home worker changes IP weekly. IP is logged in `auth_events` for investigation, not used for the decision. The one IP-based rule is elsewhere: a login from a country never seen for this user triggers the challenge even on a trusted device (`device.trust_overridden_geo` in the audit log, 2 100 times in May, 3 tickets, all travelling dispatchers who understood the message).

- **Trust is offered, not applied.** The checkbox is unticked by default. 74 % of users tick it. The 26 % who do not are mostly on shared workstations where they should not.

- **Drivers are out of scope.** The driver app binds to a `device_id` generated at install and stored server-side with the OTP login ([[driver-otp-login-server-side]]); that is a different mechanism with a different threat model (the phone is the factor).

## Metrics

| Metric | May 2026 |
|---|---|
| logins with MFA enrolled | 1 190 000 |
| of which skipped by trusted device | 1 030 000 (87 %) |
| trust created | 71 000 |
| trust revoked by user | 2 300 |
| trust revoked by reset or change | 9 800 |
| `device.mismatch` (other user's cookie) | 400 |
| `device.trust_overridden_geo` | 2 100 |

87 % of MFA-enrolled logins skipping the challenge is what made mandatory MFA acceptable. The 13 % that do get the challenge are new browsers, expired trusts, and the geo override.

## What we would change

If the driver app ever gets a dispatcher mode, push-based approval on a trusted phone would replace TOTP for most dispatchers and the cookie would become less central. Until then, the cookie is the simplest thing that holds. The review notes in [[auth-review-feedback-2026]] have the discussion.
