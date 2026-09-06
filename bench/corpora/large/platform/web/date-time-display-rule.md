---
name: date-time-display-rule
description: The front displays instants in the dispatcher's zone with an explicit zone suffix when it differs from the load's address zone, uses relative time only under 1 h, and never formats dates by hand
type: feedback
status: active
verified: 2026-01-08
---

# How to display dates and times

The API sends instants in UTC (ISO 8601 with `Z`) and local dates as `date` plus an IANA zone. What the front does with them is a source of support tickets when done wrong, so this is the rule.

## Display

- **Instants** (event times, created at, last position) are shown in the **viewer's zone** (from the profile, default browser zone), through `formatInstant(iso, { style })` in `src/utils/time.ts`, which wraps `Intl.DateTimeFormat`. Never `new Date(iso).toLocaleString()` in a component.

- **Pickup and delivery windows** are shown in the **address's zone** (the load carries `pickup_tz`), because that is when the dock is open. When the address zone differs from the viewer's zone, the suffix is mandatory: `08:00 (Europe/Madrid)`. This is the rule that came out of the Valencia bug on the API side, and a Warsaw dispatcher looking at a Spanish pickup now sees the difference.

- **Relative time** ("il y a 4 min") only for durations under 1 hour, and always with a `title` attribute carrying the absolute time. Beyond 1 hour, absolute. "il y a 3 jours" was judged useless by dispatchers who plan by date.

- **Dates without time** (invoice date, document expiry) use the `date` field and `Intl.DateTimeFormat` with `dateStyle: 'medium'`, no zone conversion at all. Converting a plain date through a `Date` object shifted invoice dates by a day for users west of Greenwich, which happened once with a Portuguese carrier.

## Input

`DateTimeField` (see [[forms-react-hook-form-zod]]) takes a local date, a local time and a zone, and produces an instant. The zone is the address zone, prefilled when the address is chosen, editable in an advanced section that nobody opens. The field never uses the browser zone for a load window.

## Formatting helpers

Everything in `src/utils/time.ts`, tested with three viewer zones (`Europe/Paris`, `Europe/Warsaw`, `Atlantic/Azores`) and the DST boundaries. `dayjs` was removed in HF-1280: it was used only for formatting and `Intl` does it, minus 12 KB in the bundle.

## Durations

Waiting times and transit durations are formatted as `2 h 15` by `formatDuration(ms)`, never as `2:15` (dispatchers read it as a clock time) and never with seconds.

## Check before merging

If a component renders a date, ask: which zone, and does the user need to know it? If the answer is "the browser's" for a load window, it is wrong. See also [[i18n-fr-en-strings]] for the locale used by `Intl`.
