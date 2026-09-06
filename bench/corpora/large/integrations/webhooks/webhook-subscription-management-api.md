---
name: webhook-subscription-management-api
description: The /v2/webhooks subscription endpoints (create, patch, rotate secret with 24 h overlap, pause, test), validation codes and the 10-per-org cap
type: reference
status: active
verified: 2026-05-19
---

# Webhook subscriptions: the management API

Everything an integrator can do with subscriptions, as exposed since HF-3100 (February 2026) when we consolidated the four older endpoints. The delivery mechanism itself (outbox, relay, signature) is described in the platform API notes; this note is about the management surface.

## Endpoints

```
POST   /v2/webhooks/subscriptions
GET    /v2/webhooks/subscriptions
GET    /v2/webhooks/subscriptions/{id}
PATCH  /v2/webhooks/subscriptions/{id}
DELETE /v2/webhooks/subscriptions/{id}
POST   /v2/webhooks/subscriptions/{id}/rotate-secret
POST   /v2/webhooks/subscriptions/{id}/pause
POST   /v2/webhooks/subscriptions/{id}/resume
POST   /v2/webhooks/subscriptions/{id}/test
GET    /v2/webhooks/subscriptions/{id}/deliveries?status=&since=&cursor=
```

All require an API key with the `webhooks:manage` scope, or a web session with the `org_admin` role. Deliveries listing needs `webhooks:read`.

## Create

Body: `url` (https only, validated per [[webhook-endpoint-url-validation-ssrf]]), `events[]` (from the published list, at least one), `payload_version` (2 or 3, see [[webhook-payload-versioning-v2-v3]]), optional `description` (120 chars), optional `filters` (`load.visibility`, `load.pickup_country`, since HF-3108). Response 201 with the subscription and `secret` **once**. The secret is never returned again; losing it means rotating.

Validation errors come back as 422 in the standard envelope with `field` and `code`: `url_not_https`, `url_private_address`, `url_unreachable_hint` (a warning field, not a refusal: we do a HEAD at creation and tell you if it failed, but create anyway because some integrators create before deploying), `events_unknown`, `events_empty`, `payload_version_unsupported`.

Cap: 10 active subscriptions per organisation. The 11th is a 409 `subscription_limit`. Nobody has asked for more; one shipper has 7.

## Patch

Any of `url`, `events`, `payload_version`, `description`, `filters`. Changing `url` triggers the same validation and resets the failure counters. Changing `payload_version` applies to deliveries created after the change; pending deliveries keep the version they were created with.

## Rotate secret

Returns a new secret once. The previous secret stays valid for **24 hours**, during which deliveries carry two signatures (`X-Halden-Signature: sha256=<new>,sha256=<old>`). Integrators verify against either. After 24 h the old one is dropped. The overlap is what lets them deploy without a gap; before HF-3100 rotation was immediate and every rotation caused a few minutes of rejected deliveries on the customer side.

## Pause and resume

`pause` stops delivery attempts; events keep accumulating in the outbox as `PENDING` for up to 7 days, then are marked `EXPIRED` (not `DEAD`, they were never attempted). `resume` starts delivering the backlog in order of creation. Useful for planned maintenance on the customer side; three integrators use it every weekend. A paused subscription does not count failures, so it never trips the auto-disable ([[webhook-auto-disable-and-dead-letters]]).

## Delete

Soft delete: the row gets `deleted_at`, deliveries stay readable for 30 days, then the retention purge removes everything. A deleted subscription's URL can be reused by a new one immediately.

## Test

Sends a `ping` event with a fixed payload through the normal outbox path, so it exercises retries and signature like a real event. Response is 202 with the `delivery_id`; poll the deliveries endpoint to see the result. Support uses it through `hfctl webhooks test`.

## Web screen

Settings, Integrations, Webhooks. Same operations, plus a delivery viewer with the request body, response status, response time and `last_error`. The screen calls the same endpoints with the web session; there is no hidden capability.

## Not offered

No per-event URL (one URL per subscription; make several). No basic auth or custom headers on the request (signature is the authentication, and the [[webhook-mtls-request-declined]] note explains why we did not go further). No IP allowlist on our side, because the caller is us.
