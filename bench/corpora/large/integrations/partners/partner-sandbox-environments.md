---
name: partner-sandbox-environments
description: Cargolink and Fretzone sandboxes from staging: HFTEST prefix, nightly reset vs no quota enforcement, recorded-response suite, local stub toggles
type: reference
status: active
verified: 2026-05-30
---

# Partner sandboxes and how we test

Both partners provide a sandbox. Neither behaves exactly like production, and knowing the differences saves a day per integration change.

## Where things point

Environment variable per partner: `PARTNER_CARGOLINK_BASE_URL`, `PARTNER_FRETZONE_BASE_URL`. Staging points to the sandboxes, production to production, dev to a local stub (below). Credentials live in the secret store under `partners/<name>/sandbox/*` and `partners/<name>/prod/*`; the application reads the path matching `APP_ENV`. Sandbox credentials are shared with the whole team; production credentials are readable by the deploy role only.

Every listing we create in a sandbox carries the reference prefix `HFTEST-`. Cargolink filters that prefix from their sandbox carriers' view (they asked for it after our load tests appeared in their demo). Fretzone does not filter; their sandbox has no carriers anyway.

## Cargolink sandbox

- Reset nightly at 02:00 CET. Anything we created is gone in the morning, and `partner_load_refs` on staging then points to listings that do not exist, which the staging reconciliation "fixes" every morning by recreating them. Expected noise; the staging alert thresholds are 10 times production's.

- Rate limits enforced, same as production. Good, we found the pagination bug there.

- Callbacks: their sandbox calls our staging callback URL when we place a bid *as a sandbox carrier* through their sandbox UI. We have two sandbox carrier accounts for that. It is the only way to test inbound bids end to end and it is manual; the automated tests use recorded responses.

- Listing lifetime in the sandbox is 2 days, not 14, "to help test expiry". It did not help; it made us think expiry was frequent and fast to detect. Production's 14 days is what matters ([[partner-incident-2025-12-cargolink-mass-expiry]]).

- Their sandbox is one version ahead of production for about two weeks before each of their releases. Useful as an early warning when we remember to run the recorded-response suite against it weekly (a scheduled CI job does since March).

## Fretzone sandbox

- Persistent, no reset. Our staging has about 3 000 `HFTEST-` listings there; we clean them with `app:partners:fretzone:sandbox-clean` monthly.

- **Does not enforce** the 2-hour exclusivity, the daily quota, or the price rounding they introduced in April. Every one of their production-only behaviours has surprised us at least once. Their contact says the sandbox "focuses on the data format". The quota guard and the rounding are therefore tested with our stub, not their sandbox.

- Polls our staging feed every 10 minutes like production. Their sandbox poller uses a different source IP than production; our feed's IP check has both.

- Slow: 1 to 3 seconds per call. Production is faster, oddly.

## The recorded-response suite

`tests/Partners/Recorded/` holds request and response pairs recorded against each sandbox, replayed by a stub HTTP client in unit tests. Recording is `bin/console partners:record --partner cargolink --scenario listing-lifecycle`, which runs the scenario against the sandbox and writes the pairs. Recordings are reviewed like fixtures: a diff in a recorded response is a partner change to understand, not a file to regenerate blindly. The suite runs on every PR in under 20 s and weekly against the live sandboxes to detect drift.

## Local stub

`docker compose --profile partners up` starts `partner-stub`, a small server that implements both partners' endpoints from the recorded pairs plus configurable behaviours: `STUB_FRETZONE_ROUND_PRICES=1`, `STUB_CARGOLINK_QUOTA=100`, `STUB_LATENCY_MS=800`. This is where the April loop is reproduced as a regression test ([[partner-incident-2026-04-fretzone-price-drift]]). The stub is deliberately dumb; it does not try to be the partner, it replays and applies the toggles.

## What only production tells us

Real carrier behaviour (bids on expired listings), real quotas under real load, and partner-side changes they do not put in the sandbox first. Hence the nightly reconciliation report and the weekly drift job, and a habit: after any partner release announcement, run the recorded suite against production read-only endpoints (listing GETs) the same day.
