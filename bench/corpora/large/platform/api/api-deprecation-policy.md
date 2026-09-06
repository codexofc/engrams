---
name: api-deprecation-policy
description: A deprecated endpoint or field is announced with Deprecation and Sunset headers, stays 6 months minimum, and is removed only when its 30-day call count is under 100 per day from fewer than 3 organisations
type: reference
status: active
verified: 2025-11-20
---

# Deprecation policy for the public API

Agreed with product and the integrations team in HF-1095 (October 2025), after the v1 pagination change annoyed two carriers who found out by reading the changelog.

## Announcing

- The response of a deprecated endpoint carries `Deprecation: true` and `Sunset: <HTTP date>` at least 6 months in the future, plus `Link: <https://docs.halden.example/changelog/...>; rel="deprecation"`.
- A deprecated field inside a response is not signalled by header (too noisy). It is marked `deprecated: true` in the OpenAPI document and listed in the changelog. It keeps being returned until the sunset date.
- The changelog page and an e-mail to every organisation that called the endpoint in the last 30 days. The list comes from the `api_route_calls_by_org` metric aggregated by the data platform, not from the API itself.

## Removing

An endpoint is removed when all of this is true:

- Sunset date passed.
- Fewer than 100 calls per day over the last 30 days.
- Those calls come from fewer than 3 distinct organisations, each contacted at least twice.

If the conditions are not met at sunset, the sunset is extended by 3 months and the header updated. We did this once for `GET /v1/loads` (the Belgian integrator again, see [[rate-limiting-per-carrier]]).

After removal the route returns 410 Gone with the error envelope and `type: .../errors/endpoint-removed`, for another 3 months, then 404.

## Current list (as of the verified date)

| Endpoint / field | Deprecated since | Sunset | Status |
|---|---|---|---|
| `GET /v1/loads` offset pagination | 2025-10-15 | 2026-06-30 | active, 12 orgs calling |
| `POST /v1/bids` | 2025-10-15 | 2026-06-30 | active |
| `loads.pickup_date` in v1 and v2 responses | 2025-11-06 | 2026-05-06 | field kept, copied from `pickup_window_start` |
| `GET /v2/carriers/{id}/score` (moved into carrier resource) | 2025-11-20 | 2026-05-20 | 2 orgs calling |

Historical notes: [[api-pagination-offset-convention]]. Versioning mechanics: [[api-versioning-header]].

## What we do not promise

Internal endpoints (`/internal/*`, used by the web front and the driver app) are not covered. Those change with the clients, and the clients are ours. The driver app has a minimum supported version enforced by `GET /internal/mobile/min-version`, which is the mobile team's tool for forcing updates.
