---
name: audit-trail-schema
description: Every security-relevant event is a row in audit_events (actor, action, target, org, ip, request_id, jsonb details) written through AuditRecorder in the same transaction as the change, 34 action codes, 13 month retention
type: reference
status: active
verified: 2026-05-20
---

# Audit trail

`audit_events` is the table you query when someone asks "who did that, when, from where". It is distinct from `audit_log` in the API project, which is the generic entity change log (every field change on every entity, partitioned monthly). `audit_events` is smaller, curated, security-oriented, and has a stable vocabulary that compliance and support can read without knowing the data model.

## Columns

| column | type | notes |
|---|---|---|
| `id` | uuid v7 | time-ordered, so `ORDER BY id` is chronological |
| `occurred_at` | timestamptz | server clock at commit |
| `actor_type` | enum | `user`, `staff`, `service`, `apikey`, `system` |
| `actor_id` | uuid | user id, staff id, service account id, api key id, null for `system` |
| `acting_as_id` | uuid | set during impersonation, the customer user being impersonated |
| `organization_id` | uuid | nullable for staff-only actions |
| `action` | text | one of the codes below, `<resource>.<verb_past>` |
| `target_type` | text | entity class short name |
| `target_id` | uuid | |
| `ip` | inet | from the trusted proxy header, see the API note on proxies |
| `user_agent` | text | truncated to 256 |
| `request_id` | text | the `X-Request-Id` of the HTTP request, joins with logs |
| `details` | jsonb | action-specific, never contains a secret or a full personal record |

Indexes: `(organization_id, occurred_at desc)`, `(actor_id, occurred_at desc)`, `(target_type, target_id)`, `(action, occurred_at desc)`. Partitioned by month like `audit_log`, 13 partitions kept.

## Action codes

Thirty-four in May 2026. The ones people search for most:

- `auth.login_succeeded`, `auth.login_failed`, `auth.logout`, `auth.password_changed`, `auth.mfa_enrolled`, `auth.session_revoked`

- `authz.denied` (sampled, see [[permission-check-voter-symfony]])

- `member.invited`, `member.role_changed`, `member.removed`

- `apikey.created`, `apikey.rotated`, `apikey.disabled`, `apikey.deleted`, `apikey.revealed`, `apikey.bruteforce_suspected`

- `staff.impersonation_started`, `staff.impersonation_ended` (see [[impersonation-support-mode]])

- `export.personal_data`, `export.invoices`, `export.loads`

- `load.cancelled_by_staff`, `invoice.voided`, `payout.held`

- `breakglass.used` (see [[break-glass-admin-procedure]])

Adding a code means adding it to the `AuditAction` enum, to this list, and to the compliance data map. A test fails if the enum and the documented list in `docs/audit-actions.md` differ.

## Writing

`AuditRecorder::record(AuditAction $action, ?object $target, array $details = [])` is called from the service that performs the change, inside the same Doctrine transaction. If the change rolls back, so does the audit row. We considered the outbox pattern and rejected it for this table: an audit row that says "role changed" without the role having changed is worse than no row.

The actor comes from `Security::getToken()`, so services called from a Messenger worker must set the token explicitly with `ActorContext::asSystem()` or `ActorContext::asService($account)`, otherwise `AuditRecorder` throws `MissingActorException`. This bit two people in 2025 who then added `ActorContext` calls at the top of their handlers. It is a feature.

`details` is validated against a per-action JSON schema in `config/audit/*.schema.json`. The schemas forbid keys named `password`, `secret`, `token`, `key` and any string matching the `hfk_` pattern, which is how we caught a handler that logged a freshly created API key in 2026-01 before it reached production.

## Reading

- Back-office: `/backoffice/organizations/{id}/audit` with filters on action and date, 90 days by default.

- Customers: `GET /v2/audit-events` for their organization, `shipper_admin` and `carrier_admin` only, last 13 months. Added in HF-2160 because two enterprise shippers asked for it in their security questionnaire.

- Analysts: replicated to the warehouse nightly with `ip` and `user_agent` dropped.

Export and retention specifics are in [[audit-trail-retention-and-export]].
