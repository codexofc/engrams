---
name: timezone-handling-utc-rule
description: Store instants as timestamptz in UTC, keep local dates as date plus an IANA zone column, never convert in SQL, and the pickup window bug that made this a rule
type: feedback
status: active
verified: 2026-01-15
---

# Time zones: the rule and why

**Instants are UTC `timestamptz`. Local dates are `date` plus a `tz` column. Conversion happens in PHP at the edge, never in SQL, never in the entity.**

## How to apply

- Anything that answers "when did it happen" (event, creation, acceptance, position fix) is an instant. Column `timestamptz`, PHP `DateTimeImmutable` in UTC, JSON as ISO 8601 with `Z`. `App\Time\Instant` is a thin wrapper that refuses to be constructed with a non-UTC zone.

- Anything that answers "on which day, as seen by a human at a place" is a local date. Pickup day, delivery day, invoice date. Column `date` plus `<name>_tz text` holding an IANA name (`Europe/Paris`), taken from the address's country and region via `App\Geo\TimezoneResolver`. Not from the user's browser.

- A pickup *window* is two instants (`pickup_window_start`, `pickup_window_end`, both `timestamptz`) computed from the local date, the local hours and the address timezone at the moment the load is created. We store the instants, not the local hours. If the user edits, we recompute.

- Display: the web front formats in the dispatcher's zone, the driver app in the device zone. The API never formats.

- `SET TIME ZONE` on the connection is forbidden. PgBouncer would make it random anyway, see [[postgres-connection-pool-pgbouncer]].

- The clock is injected, see [[api-tests-phpunit-conventions]].

## The bug that made this a rule (HF-1121, October 2025)

A Spanish shipper posted loads with pickup at "08:00" in Valencia. The dispatcher in Warsaw saw 08:00 too, because the first implementation stored `pickup_date` as `timestamp without time zone` and every client displayed it raw. The Polish carrier arrived at 08:00 Warsaw time, one hour early, three days in a row, and invoiced waiting time. Small money, but the shipper threatened to leave and support spent two days on it.

The migration from `pickup_date` to the window columns is the one that caused [[incident-2025-11-migration-lock-loads]], so this bug cost us twice.

## Things that still bite

- `DATE_TRUNC('day', created_at)` in a report groups by UTC day. Reports that need local days must `DATE_TRUNC('day', created_at AT TIME ZONE tz)`, and only reporting queries in `App\Reporting` are allowed to do that.

- The DST switch: a window from 01:30 to 02:30 on the last Sunday of March in Paris is 0 minutes or 120 minutes long depending on which way you compute it. `TimezoneResolver::window()` has a test for both switches, keep it.

- The invoice year boundary, see [[invoice-number-allocation-gapless]].
