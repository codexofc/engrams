---
name: auth-events-audit-log
description: auth_events is the append-only audit table of auth-svc: 60 event types, stable naming, 2-year monthly partitions, 1.9 GB, read by auth:events
type: reference
status: active
verified: 2026-05-14
---

# The auth audit log

## Table

`auth.auth_events(id bigserial, occurred_at timestamptz, type text, user_id bigint NULL, session_id uuid NULL, actor_user_id bigint NULL, ip inet, user_agent_hash bytea, outcome text, details jsonb)`, partitioned by month on `occurred_at`, 24 partitions kept (2 years), the oldest dropped on the first of each month by `auth:events prune`. Indexes: `(user_id, occurred_at desc)`, `(type, occurred_at desc)`, `(ip, occurred_at desc)`. 1.9 GB, 210 M rows, 400 000 rows a day.

Append only: no `UPDATE`, no `DELETE` except the partition drop, enforced by a trigger that raises on both, and by the application role having only `INSERT` and `SELECT`. Writes are synchronous in the same transaction as the action they record (a login that cannot be logged fails); we measured the cost at 1.5 ms and kept it.

## Naming

`<subject>.<verb_or_state>`, lowercase, dot separated, past tense or state. Every type is declared in `AuthEventType` (a PHP enum) with a one-line description and the list of `details` keys it carries, and `auth:events types` prints them. Sixty types in May 2026, the families:

| Family | Examples | Daily volume |
|---|---|---|
| `login.*` | `login.success`, `login.failure`, `login.throttled`, `login.locked` | 130 000 |
| `otp.*` | `otp.requested`, `otp.verified`, `otp.failed`, `otp.exhausted` | 150 000 |
| `session.*` | `session.created`, `session.revoked`, `session.refresh`, `session.reuse_detected` | 90 000 |
| `mfa.*` | `mfa.enrolled`, `mfa.challenge_ok`, `mfa.challenge_failed`, `mfa.recovery_used`, `mfa.reset` | 20 000 |
| `device.*` | `device.trusted`, `device.revoked`, `device.mismatch`, `device.trust_overridden_geo` | 5 000 |
| `password.*` | `password.changed`, `password.reset_requested`, `password.reset_completed`, `password.breached_flagged` | 4 000 |
| `sso.*` | `sso.start`, `sso.assertion_ok`, `sso.assertion_rejected`, `sso.enforced_changed` | 2 500 |
| `lock.*` | `lock.applied`, `lock.lifted` | 300 |
| `impersonation.*` | `impersonation.started`, `impersonation.ended` | 130 |
| `admin.*` | `admin.user_created`, `admin.role_changed`, `admin.connection_changed` | 600 |

`outcome` is `ok`, `denied`, `error`. `login.failure` has `outcome = denied` and `details.reason` in `bad_password`, `unknown_user`, `locked`, `throttled`, `sso_enforced`. Note that `unknown_user` is recorded here even though the HTTP response does not distinguish it; the log is internal.

## Reading it

`bin/console auth:events --user <id> --last 7d` prints one line per event, oldest first, with the IP truncated to /24 and the user agent label. It is the first thing support and the on-call run. Variants: `--ip <addr>`, `--type login.failure --last 1h --group-by ip` (used during the [[incident-2025-12-credential-stuffing]] response to list the attacking IPs), `--session <uuid>`.

The Grafana board `Auth / Login` is fed from the same table through a read replica with 5-minute panels: failures by reason, distinct IPs failing, OTP request-to-verify ratio, MFA challenge outcomes, SSO rejections by connection.

## To the warehouse

A daily marmot job copies the previous day's rows into `analytics.auth_events` in the warehouse, without `ip` and `user_agent_hash`, with `user_id` kept (the warehouse has the user dimension anyway). Product uses it for login funnel and MFA adoption ([[mfa-totp-rollout-dispatchers]] numbers come from there); the security-shaped questions (which IP, which device) stay in PostgreSQL where the retention and access are tighter.

## Access

The `auth` schema is readable by the `auth-svc` role, the support back-office role (through the `auth:events` command's API equivalent, `GET /internal/auth/events?user_id=`, which applies the same IP truncation), and two named humans on the platform team with a `psql` role for incidents. Nobody else, and the warehouse copy is the way to get aggregates.

## What is not logged

- Passwords, obviously, and not their length or any derived value. The `details` of `login.failure` say `bad_password`, nothing about how close it was.

- OTP codes, even hashed.

- Full user agents (hash only, plus a coarse label computed at insert).

- Successful `GET` requests during an impersonation session, only the count at `impersonation.ended` ([[support-impersonation-audit]]).

## Retention decision

Two years was chosen against the 5 years the finance side keeps for billing audit: authentication events are not financial records, and the incident investigations we have done never went back more than 90 days. The lock and impersonation families are the exception; they are copied to `notification_audit`-style long-term tables (`auth.audit_long`) with 5-year retention because they are decisions by a person about a person.
