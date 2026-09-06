---
name: deprecation-policy-api
description: Public API deprecation: 12 months notice with Deprecation and Sunset headers, usage per key, v1 loads endpoints retire 2026-11-30
type: project
status: active
verified: 2026-06-25
---

## Scope

The public API (`/api/v1/...`, `/api/v2/...`) is used by about 180 shippers' TMS integrations and 40 carriers' fleet tools, through API keys. Breaking a field breaks a warehouse somewhere at 03:00, so deprecations are slow and loud.

## Policy

- Notice period: 12 months from the announcement to the removal, for any breaking change (removed endpoint, removed field, changed type, changed default). Additive changes need no notice.

- Announcement: release note for the internal and shipper audiences ([[release-notes-rules]]), an email to the technical contact of every API key that called the deprecated endpoint in the last 90 days, and the two HTTP headers on every response of the deprecated endpoint: `Deprecation: true` and `Sunset: <RFC 1123 date>`, plus `Link: <docs url>; rel="deprecation"`.

- Usage tracking: `api_usage_daily` (`api_key_id`, `endpoint`, `date`, `calls`) is what the reminders are based on. At 6, 3 and 1 months before the sunset, every key still calling gets an email. In the last month, the endpoint returns a `Warning` header and a 10 % sampled `299` warning in the body envelope.

- After the sunset date: `410 Gone` with a JSON body pointing to the replacement, for 6 more months, then the route is removed.

- No deprecation is announced without the replacement being available and documented at the same time.

## Current deprecations

| Endpoint | Announced | Sunset | Replacement | Keys still calling (June 2026) |
|---|---|---|---|---|
| `GET /api/v1/loads` (list) | 2025-11-30 | 2026-11-30 | `GET /api/v2/loads` with cursor pagination | 43 |
| `POST /api/v1/loads` | 2025-11-30 | 2026-11-30 | `POST /api/v2/loads` (structured address, `max_price` object with currency) | 51 |
| `price` integer field in EUR on v1 bids | 2025-11-30 | 2026-11-30 | `amount` object `{cents, currency}` | same keys |

The v1 to v2 change on loads is the multi-currency one: v1 assumed EUR everywhere, which stopped being true with PLN and CZK invoices. Keys still on v1 in June 2026: 51 of 180. The 3-month reminder goes out 2026-08-30.

## What we learned from the v0 retirement in 2024

The v0 API was shut down with 3 months notice and 12 shippers lost their integration for a week. Two of them churned. The 12-month period comes from that, and so does usage tracking per key: in 2024 we did not know who was calling what and emailed everyone, which nobody read.

## Sandbox

The sandbox environment (`api.sandbox.` host, separate keys) returns `410` for deprecated endpoints 3 months before production does, so that integrators who test find out first. Announced in the same email.
