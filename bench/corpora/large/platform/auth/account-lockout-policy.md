---
name: account-lockout-policy
description: Lockout is distinct from throttling: triggered by signals (stale breached password, repeated reuse, admin, inactivity) never by failed attempts; unlock rules
type: reference
status: active
verified: 2026-04-09
---

# Account lockout

## Lockout is not throttling

The rate limiter ([[login-rate-limiting-rules]]) slows attempts and forgets after an hour. A lockout is a state on the account (`users.locked_at`, `users.lock_reason`) that refuses login regardless of the credentials until something explicit happens. The two are kept apart on purpose: if failed attempts locked the account, anyone could lock any account by trying ten wrong passwords, and during the December 2025 wave that would have locked 226 000 users out of their own accounts.

So failed attempts never lock. What locks:

| Reason | Trigger | Automatic unlock |
|---|---|---|
| `breached_password_stale` | `credentials.breached_at` older than 7 days and no change | on password reset |
| `refresh_reuse_repeated` | 3 refresh token reuse detections in 24 h on the same user | none, support |
| `admin` | org admin locks a user from the admin page | org admin unlocks |
| `support` | support locks after a report | support with ticket |
| `inactive` | no login for 90 days (drivers: 180) | on next successful OTP or reset, self-service |
| `offboarded` | carrier or shipper removes the user | none, the account is closed |

`inactive` is technically a lock but behaves like a soft state: the user who comes back proves ownership through the reset (password) or OTP (driver) and is unlocked in the same step, with an `auth.security_alert` e-mail saying the account was reactivated.

## What a locked account sees

`POST /v1/auth/login` with correct credentials on a locked account answers `423 {"error": "account_locked", "reason": "<public reason>", "next": "<what to do>"}`. Wrong credentials on a locked account answer the normal `401`, so that the lock state is not disclosed to someone who does not have the password. The `423` is only shown to whoever knows the password, which is the user or the person we want to stop, and in both cases telling them the account is locked is fine.

Public reasons are short: "your password was found in a data leak, reset it to continue", "your account was deactivated by your administrator", "contact support". `refresh_reuse_repeated` says "contact support" and nothing more.

## Unlocking

- By the user: password reset ([[password-reset-flow]]) unlocks `breached_password_stale` and `inactive`. OTP verification unlocks `inactive` for drivers.

- By the org admin: `admin` locks only. The admin page shows who locked and when.

- By support: `bin/console auth:unlock --user <id> --ticket <n>` after the two-proof identity check used for MFA resets ([[mfa-recovery-codes]]). `refresh_reuse_repeated` additionally revokes every session and trusted device on unlock, because the reason it happened is that someone else had the refresh token.

Every lock and unlock is an `auth_events` row with the actor ([[auth-events-audit-log]]).

## Numbers (March 2026)

| Reason | Locks in the month |
|---|---|
| `breached_password_stale` | 700 |
| `refresh_reuse_repeated` | 4 |
| `admin` | 310 |
| `support` | 6 |
| `inactive` | 5 100 |
| `offboarded` | 2 900 |

The four `refresh_reuse_repeated` were: two dispatch tool installs on the same shared workstation with two browsers fighting over a session (see [[abuse-shared-dispatcher-accounts]]), one driver whose phone had a broken clock and an old app that replayed refreshes, one unexplained and treated as a compromise (sessions revoked, password reset forced, no further sign).

## Things that were decided against

- Locking after N failures with an unlock e-mail. This is the textbook rule and it is a denial-of-service tool handed to anyone with a list of e-mails. The security alert at the 10th failed attempt on an account gives the user the information without giving the attacker the lever.

- Locking on "impossible travel" (login from two far countries within an hour). Dispatchers use VPNs whose exit country changes; drivers cross borders for a living. The geo signal triggers an MFA challenge on trusted devices ([[device-trust-and-remember-me]]), it does not lock.

- Automatic unlock of `admin` locks after a delay. The admin locked the person for a reason.
