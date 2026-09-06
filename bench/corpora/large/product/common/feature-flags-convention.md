---
name: feature-flags-convention
description: Feature flags in flag-svc, named area.snake_case with owner, kind and expiry, default off, stable hash rollouts, SDKs and audit
type: reference
status: active
verified: 2026-03-05
---

## Where flags live

Table `flags` in the config database, served by `flag-svc` over HTTP with a 30 s client cache in every service (`FlagClient.isEnabled(name, context)`). No flags in environment variables, no flags in code constants. The back office page `/backoffice/flags` is the only writer, every change is logged in `flag_changes` with who and why.

## Naming

`<area>.<snake_case_name>`: `billing.card_checkout_enabled`, `pricing.bid_form_anchor`, `onboarding.lane_preview_home`. The area is the owning team's product area. A flag name is never reused after deletion; a second attempt at an experiment gets a `_v2` suffix.

## Mandatory fields

- `owner`: a team, not a person.
- `expires_at`: at most 90 days ahead at creation. A flag past its expiry shows in red on the flags page and in the weekly digest to the owning team. It can be extended, with a reason, at most twice. The reasoning is in [[feature-flag-cleanup-lesson]].
- `kind`: `release` (on/off toggle for a shipped feature, removed after full rollout), `experiment` (percentage split with variants), `ops` (kill switch, kept indefinitely, the only kind without expiry, such as `billing.card_checkout_enabled`).
- `description`: one line, what turns on.

## Evaluation

- Boolean flags: `on`, `off`, or a list of unit ids (carrier ids, shipper ids, entity codes) for targeted enablement.
- Percentage flags: `murmur3(unit_id + flag_name) % 100 < percentage`. Stable for a unit over time and independent between flags (two 50 % flags do not select the same half). The unit is given by the caller (`context.unitId`), and the caller decides which id it is; pricing's experiment guidelines cover why it must be the unit that receives the treatment.
- Default is off when the flag is unknown or `flag-svc` is unreachable, except `ops` flags, which default to their last known value from the local cache. A kill switch that defaults to off when the flag service is down would kill the feature at the wrong time.

## Analytics

Every evaluation that lands in a variant is not tracked (too much volume), but every event tracked by the client carries the active experiment variants in the `experiments` property, so analysis can segment. See the events convention in [[analytics-events-naming]].

## Lifecycle

1. Create with owner, kind, expiry, description. Off.
2. Enable for internal accounts (list) on staging, then production.
3. Percentage rollout or experiment split.
4. 100 %.
5. Remove the code paths, delete the flag. A ticket for step 5 is created at step 1 by the flags page, linked to the flag.

Flags in code after the flag is deleted evaluate to off and log `flag.unknown` at warn; the log is scraped weekly and produces cleanup tickets.

## Client SDKs and evaluation context

Three SDKs wrap `flag-svc`: Kotlin (`FlagClient`), Python (`flagclient`), and TypeScript for the web apps (`@hf/flags`). The mobile driver app evaluates flags through the API gateway, which resolves them server-side and returns the active variants in the session bootstrap response; the app never calls `flag-svc` directly, so a flag change reaches drivers at their next app start or after 30 minutes, whichever first.

The evaluation context is a small struct: `unitId`, `unitKind` (`carrier`, `shipper`, `load`, `entity`, `staff`), `entityCode`, `appVersion`, `country`. Targeting rules can match on any of them; a rule is a list of `(field, operator, value)` with `in`, `not_in`, `gte` (for `appVersion` only). Rules are evaluated before the percentage: a flag can be "on for entity NL, 20 % elsewhere".

## Staging and production

Flags are per environment; the flags page shows both side by side and a "copy to production" button that requires typing the flag name. Staging flags default to on for `release` kind so that features are visible in staging without a click, which was decided after three weeks of "it works in staging" that meant "the flag was on in staging".

## Audit and alerting

- `flag_changes` keeps every change with the actor, the previous and new values, the reason (mandatory, minimum 10 characters), and the ticket key if the reason contains one.
- A change to an `ops` flag posts to the incident channel automatically, whoever made it.
- A percentage change of more than 30 points in one step on an `experiment` flag asks for confirmation; experiments ramp by 10 or 25 points.
- `flag.unknown` warnings are counted per flag name and per service; the weekly digest lists them with the last commit that touched the call site.

## Numbers

March 2026: 52 flags (9 `ops`, 28 `release`, 15 `experiment`), 1 400 changes in the year, 6 800 evaluations per second across services at peak, `flag-svc` p99 1.2 ms from the local cache. The cache refresh is a long-poll to `flag-svc` with a 30 s timeout, so a change is visible in under a second on average, 30 s worst case.

## Relation to release notes and specs

A `release` flag at 100 % is what makes a feature announceable ([[release-notes-rules]]); the release note tool refuses a ticket whose flag is under 100 % for the audience. A spec names its flag, unit and expiry before the flag exists, which is the checklist of the product managers' preferences. The cleanup ticket created at step 1 is assigned to the owning team and is what the digest chases; the state of the world before this discipline is in [[feature-flag-cleanup-lesson]].
