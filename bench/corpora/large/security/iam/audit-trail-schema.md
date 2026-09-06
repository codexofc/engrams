---
name: audit-trail-schema
description: Every security event is a row in audit_events (actor, action, target, org, ip, request_id, details) written in the same transaction, 34 action codes
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

## Querying it well

Three shapes cover nearly every question the table gets asked.

Everything about one organization in a window (support, customer request):

```sql
SELECT occurred_at, actor_type, actor_id, action, target_type, target_id, details
FROM audit_events
WHERE organization_id = :org AND occurred_at >= :from AND occurred_at < :to
ORDER BY id;
```

Everything one actor did (offboarding, incident):

```sql
SELECT occurred_at, action, organization_id, target_type, target_id, ip
FROM audit_events
WHERE actor_id = :actor AND occurred_at >= now() - interval '30 days'
ORDER BY id;
```

Who touched one object (a disputed invoice, a load cancelled by staff):

```sql
SELECT occurred_at, actor_type, actor_id, acting_as_id, action, details
FROM audit_events
WHERE target_type = 'Invoice' AND target_id = :id
ORDER BY id;
```

All three hit an index and return in milliseconds on 13 months of data. Anything that starts with `WHERE details->>'...'` does not, and is a sign that the fact should be a column or a dedicated action code. We added `acting_as_id` as a column for exactly that reason after a month of `details->>'impersonated_user'` queries.

## Sizes and rates

- 2.1 million rows for a typical month in 2026, about 30 per second at the busiest hour, 190 MB per monthly partition with indexes.

- Top actions by volume: `auth.login_succeeded` (38 %), `authz.denied` even sampled (14 %), `apikey.*` usage is **not** audited per call (it would be 40 million rows a month); only lifecycle events are.

- `details` median size 140 bytes, maximum allowed 4 KB by the JSON schemas; a handler that tried to log a full load payload failed validation in staging and was fixed before release.

## Consistency with the entity change log

`audit_log` (the API project's per-field change log) and `audit_events` overlap for some actions: a role change appears in both, once as a field diff and once as `member.role_changed`. They are joined by `request_id` when someone needs both views. We considered making one the source of the other and decided against it: `audit_log` is generated from Doctrine lifecycle events and knows nothing about intent; `audit_events` is written by the service that knows why. Two tables, two purposes, one join key.
