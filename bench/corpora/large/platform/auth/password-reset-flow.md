---
name: password-reset-flow
description: Reset is a 32-byte token hashed in password_reset_tokens, 30 min, single use; the request endpoint answers 202 in constant time, confirm revokes all sessions
type: reference
status: active
verified: 2026-04-02
---

# Password reset

## Request

`POST /v1/auth/reset/request {"email": ...}`. Whatever the input, the response is `202 {}` and takes the same time (see [[incident-2026-03-reset-timing-enumeration]] for why "the same time" is now measured, not assumed). Behind it:

1. Normalise the address (lowercase, trim, one Unicode normalisation pass). Look up the user.

2. If the user exists and has a password (drivers do not, they get a 202 and nothing else, their reset is "log in with an OTP"), generate 32 random bytes, store `sha256(token)` in `password_reset_tokens(user_id, token_hash, created_at, expires_at, used_at, requested_ip)`, and dispatch `auth.password_reset` through the notifications pipeline with the URL `https://app.halden.example/reset?t=<base64url token>`.

3. If the user does not exist, run a dummy hash computation of the same cost and dispatch nothing.

Rate limit: 3 requests per normalised address per hour and 20 per source IP per hour, applied before the lookup, so that the limit does not reveal whether the address exists either (the same 202 is returned, the request is simply dropped and counted in `auth.reset_rate_limited`).

The token lives 30 minutes. The old system used 24 hours; the review of the credential stuffing wave asked why anyone would need a day to click a link, and nobody had an answer.

## Confirm

`POST /v1/auth/reset/confirm {"token": ..., "password": ...}`:

1. Hash the token, look it up, check `used_at IS NULL` and `expires_at > now()`. Any failure is the same `400 {"error": "invalid_or_expired"}`.

2. Validate the new password against the policy ([[password-policy-and-argon2]]), including the breach list.

3. In one transaction: update `credentials`, set `used_at`, mark every other token of the user used, revoke all sessions and all trusted devices ([[session-model-and-revocation]], [[device-trust-and-remember-me]]), write `password.reset_completed` to `auth_events`.

4. Dispatch `auth.password_changed` (critical, not switchable) to the account's e-mail, with the IP and approximate location of the confirmation, and a line "if this was not you, contact support" with the ticket link. This e-mail has generated 4 tickets in six months; all four were the user themselves forgetting they had done it.

The user is not logged in after a reset. They log in with the new password, and if MFA is enrolled the challenge applies. A reset does not touch MFA; a lost phone is the MFA reset procedure ([[mfa-recovery-codes]]), which is deliberately harder.

## Why the e-mail is the only channel

A reset link by SMS was proposed for dispatchers without e-mail access. Refused: the SMS number is a weaker identity than the e-mail we verified at signup, and the reset flow must not be weaker than the login it replaces. Dispatchers without e-mail access are a carrier IT problem and there are 40 of them.

## Numbers

- 2 100 reset requests a day on average, 1 300 completed. The 800 gap is mostly people who remember their password after asking, plus the 30-minute expiry (about 90 a day expire; the e-mail says 30 minutes in bold since February).

- 60 requests a day for addresses that do not exist, steady. Attackers testing the endpoint, or typos. The constant-time answer means they learn nothing either way.

- p50 of `/reset/request` 210 ms with or without a matching user, p99 480 ms. The dummy hash is what makes the two branches equal.

## Support side

Support cannot trigger a reset for a user; the user does it. Support can see in `auth:events --user <id>` that a request was made, from which IP, and whether it was confirmed. The one exception: an account whose e-mail address itself must change (a company domain migration, 3 or 4 a month), handled by the address change procedure with the same two-proof rule as the MFA reset, after which the user requests a reset normally.

## Testing

`PasswordResetTest` includes a timing test: 200 requests for an existing address and 200 for a random one, on the CI runner, and the test fails if the medians differ by more than 15 %. It is noisy on a loaded runner and has flaked twice; both times rerun green. We keep it because the alternative is not knowing.
