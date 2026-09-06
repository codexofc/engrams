---
name: auth-service-overview
description: auth-svc owns login, sessions, MFA, reset, device trust and SSO for 380 000 users since HF-4300 (Oct 2025): endpoints under /v1/auth, tables in schema auth
type: reference
status: active
verified: 2026-06-30
---

# auth-svc

## Why a separate service

Until October 2025 authentication lived in the API monolith: the Symfony security component, a `User` entity with a `password` column, and a session in Redis. Three things pushed the extraction (HF-4300): the credential stuffing wave of late 2025 needed rate limiting and device logic that did not fit a controller, the SSO project for enterprise shippers needed SAML handling nobody wanted inside the API, and the driver app's OTP login had its own code path in the mobile backend with a second copy of the lockout rules.

`auth-svc` is a Symfony application too (same team, same tooling), 14 000 lines, deployed as its own Deployment in `platform-prod` (3 replicas), reachable at `auth.halden.example` for browsers and the mobile app, and at `auth-svc.platform-prod.svc` for the API and the dispatch tool. It issues the JWTs the API validates; the token format and refresh rotation did not change in the extraction.

## What it owns

| Concern | Where | Note |
|---|---|---|
| password login | `POST /v1/auth/login` | [[password-policy-and-argon2]] |
| OTP login (drivers) | `POST /v1/auth/otp/request`, `/otp/verify` | [[driver-otp-login-server-side]] |
| sessions | `sessions` table and Redis revocation set | [[session-model-and-revocation]] |
| MFA | `POST /v1/auth/mfa/challenge`, `/mfa/verify`, `/v1/me/mfa/*` | [[mfa-totp-rollout-dispatchers]], [[mfa-recovery-codes]] |
| password reset | `POST /v1/auth/reset/request`, `/reset/confirm` | [[password-reset-flow]] |
| device trust | cookie `hf_dt`, table `trusted_devices` | [[device-trust-and-remember-me]] |
| SSO | `/v1/auth/sso/{connection}/...` | [[sso-saml-enterprise-shippers]] |
| lockout and rate limits | `LoginThrottle`, `LockoutPolicy` | [[login-rate-limiting-rules]], [[account-lockout-policy]] |
| support impersonation | `POST /v1/auth/impersonate` | [[support-impersonation-audit]] |
| audit log | `auth_events` | [[auth-events-audit-log]] |

What it does not own: authorization. Roles and organisation membership are in the API's `memberships` table; `auth-svc` puts the role claims in the token from a read replica but does not decide them.

## Tables

- `credentials(user_id PK, password_hash, algo, params JSONB, changed_at, breached_at NULL)`: one row per user with a password. Drivers without a password have no row.

- `sessions(id, user_id, created_at, last_seen_at, expires_at, ip, user_agent_hash, device_id NULL, mfa_at NULL, revoked_at NULL, revoke_reason)`.

- `mfa_methods(id, user_id, type, secret_ref, confirmed_at, last_used_at)`: `type` is `totp` or `recovery`. The TOTP secret is in the vault under `secret_ref`, not in the table.

- `trusted_devices(id, user_id, device_hash, label, first_seen_at, last_seen_at, expires_at, revoked_at)`.

- `sso_connections(id, org_id, idp_entity_id, metadata_xml, attribute_map JSONB, enforced boolean, created_at)`.

- `auth_events(id, occurred_at, type, user_id NULL, session_id NULL, ip, outcome, details JSONB)`: append only, 2 years.

- `otp_codes(id, phone_hash, code_hash, expires_at, attempts, used_at)`.

All in the `auth` schema of the main PostgreSQL cluster, 2.1 GB, of which `auth_events` is 1.9 GB.

## Populations

| Population | Users | Login method | MFA |
|---|---|---|---|
| dispatchers (carrier side) | 41 000 | password | TOTP required since 2026-03 |
| shipper users | 62 000 | password or SSO | TOTP optional, required for admins |
| drivers | 268 000 | SMS OTP plus device binding | none, the device is the factor |
| support and internal | 90 | password plus TOTP, internal IdP | required |
| API clients (machine) | 1 400 | client credentials, in the API not here | none |

## Numbers (June 2026)

- 96 000 logins a day, 71 % OTP (drivers), 27 % password, 2 % SSO.

- p50 latency of `POST /v1/auth/login` 180 ms, of which 140 ms is Argon2id. p99 420 ms.

- 1.1 M active sessions, 30-day driver sessions dominating.

- Failed logins: 4 % of password attempts on a normal day, with peaks to 80 % of attempts during stuffing waves ([[incident-2025-12-credential-stuffing]]).

## Environment

`AUTH_JWT_SIGNING_KEY_REF` (vault path, ES256 key pair, rotation quarterly with a two-key JWKS at `/.well-known/jwks.json`), `AUTH_SESSION_TTL_DRIVER=30d`, `AUTH_SESSION_TTL_WEB=12h`, `AUTH_ARGON2_MEMORY_KIB=65536`, `AUTH_OTP_PROVIDER=notifications` (it dispatches through the notifications pipeline, no direct SMS), `AUTH_TRUST_COOKIE_TTL=30d`, `AUTH_BREACH_LIST_PATH` (the local hashed breach list, see the password note).

## Where to start when something is wrong

`bin/console auth:events --user <id> --last 24h` prints the audit trail of one user in order, which answers 90 % of support questions ("why was I logged out", "why did it ask for a code"). The other 10 % is the session table and the Redis revocation set, described in the session note. The team's review notes for the first six months are in [[auth-review-feedback-2026]].
