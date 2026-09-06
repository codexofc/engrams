---
name: telematics-integration-test-harness
description: hf-telematics-harness replays recorded or synthetic Trakko, Geolyx and app traffic from YAML scenarios, with a fixture per incident and a 2 400 pos/s load run
type: reference
status: active
verified: 2026-04-24
---

# Telematics test harness

You cannot develop a telematics pipeline against 2 900 real trucks, and the providers' sandboxes give you five vehicles driving in circles. `hf-telematics-harness` (a directory in the gateway repository, run with `make harness SCENARIO=...`) fakes the providers and the app, and replays scenarios against a gateway started with `HARNESS_MODE=1`.

## How it works

- **Fake Trakko**: an HTTP server implementing `/v3/fleets/{fleet}/positions` and `/history` and `/events`, serving from a scenario file, with the sliding window, the 60 s cursor expiry and the 120 req/min limit reproduced. Every quirk in [[trakko-api-contract-quirks]] has a toggle in the scenario (`device_clock_offset_s`, `ignition_case: mixed`, `odometer_unit: km`).

- **Fake Geolyx**: a client that POSTs batches to the gateway's webhook endpoint with a valid (or deliberately invalid) HMAC, with configurable batch size, rate and retry behaviour ([[geolyx-push-webhook-format]]).

- **Fake app**: a client that uploads position batches the way the mobile app does, including the "re-upload after lost ack" case.

- **Platform stub**: answers `GET /internal/assignments/windows` from the scenario, so window filtering ([[telematics-consent-and-masking]]) can be tested without the platform.

- **Assertions**: after the scenario, the harness reads `position_events`, the stage counters and the Kafka topics from the local stack and compares to `expect:` in the scenario file.

## A scenario

```yaml
name: trakko-clock-drift
duration_s: 600
vehicles:
  - id: V1
    provider: trakko
    route: fixtures/routes/a1-lyon-dijon.geojson
    speed_kmh: 80
    device_clock_offset_s: -2100     # 35 min behind
    assignment: {from: -1800, to: +7200}
expect:
  stage_counters:
    plausibility.kept: {min: 55, max: 62}
  position_events:
    all: {time_source: server}
  time_gap_p50_s: {min: 2000}
```

Routes are GeoJSON lines; the harness interpolates positions along them at the configured speed and interval. Twenty-two routes in `fixtures/routes/`, recorded from real trips with the vehicle ids removed.

## Scenario families

- **`normal-*`**: an hour of mixed traffic per provider. Baseline for the dedup rates in [[position-dedup-rules]] (the `DedupStageTest` reference replay is one of these).

- **`incident-*`**: one per incident. `incident-geolyx-replay` reproduces 6 h of 320 vehicles pushed in 20 minutes ([[incident-2025-11-geolyx-duplicate-flood]]); `incident-trakko-clock-drift` reproduces the February 2026 drift ([[incident-2026-02-trakko-timestamp-drift]]). Both fail on the pre-fix code and pass on the fixed one; they are the regression suite.

- **`hostile-*`**: bad HMAC, unknown vehicle refs, positions at (0,0), timestamps in 2038, batches of 50 000 positions, JSON with duplicated keys. All must be counted and rejected, none may crash a handler.

- **`load-2400`**: 2 400 positions per second for 20 minutes against a single gateway pod, from a mix of the three sources. Pass criteria: no message older than 60 s in the queue at the end, no 5xx, p95 stage latency under 5 s. Run before every MR that touches the pipeline (CI job `harness-load`, 25 minutes, on a dedicated runner).

- **`onboarding-*`**: the fixture set a new provider must supply ([[provider-onboarding-runbook]] step 5).

## Running it

```
make harness SCENARIO=incident-trakko-clock-drift        # one scenario, local stack
make harness-all                                         # everything but load, ~12 min
make harness SCENARIO=load-2400 GATEWAY_PODS=1           # load, needs the big runner
```

The local stack is the compose file with Postgres, Redis, Kafka and one gateway. `HARNESS_MODE=1` makes the gateway accept the fake providers' URLs from environment variables and disables the outbound call to the real platform.

## Limits

- No fake for the routing service or the ETA model; the ETA consumer is tested separately with recorded positions.

- Routes are interpolated, so the harness never produces the GPS noise of a real receiver in a city; the `hostile-*` family adds synthetic noise but it is not the same. Twice a real-world pattern (a phone bouncing between two cell towers' assisted positions) was not in the harness. The fix each time: record the real hour and add it as a `normal-*` fixture.
