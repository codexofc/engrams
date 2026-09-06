---
name: telematics-providers-overview
description: Positions come from the driver app (62 %), Trakko trackers polled every 30 s (27 %) and Geolyx push webhooks (11 %), normalised by the telematics gateway
type: reference
status: active
verified: 2026-06-16
---

# Telematics sources: the map

The telematics gateway (`hf-telematics-gw`, its own deployment in the `integrations` namespace) turns everything that says "vehicle X was at (lat, lon) at time T" into one shape and hands it to the platform. This note is the overview; each source and each processing step has its own note.

## Three sources

| Source | Mechanism | Share of vehicles (June 2026) | Median latency to platform |
|---|---|---|---|
| Driver mobile app | batched HTTPS uploads every 60 s while an assignment is active | 62 % | 45 s |
| **Trakko** hardware trackers | we poll their REST API every 30 s per fleet, see [[trakko-api-contract-quirks]] | 27 % | 40 s |
| **Geolyx** fleet telematics | they push webhooks to us on every position, see [[geolyx-push-webhook-format]] | 11 % | 8 s |

The app covers small carriers and owner-drivers who have nothing else. Trakko is our own offer for carriers who want a fixed unit in the truck (we buy the units, Trakko hosts the data, the carrier pays us a monthly fee per vehicle). Geolyx is what mid-size fleets already have; we integrate with their existing account.

A vehicle can have more than one source at once (a Trakko unit and a driver with the app). Which one wins is in [[position-source-priority]].

## The pipeline in one paragraph

Source adapters produce `RawPosition` records (provider, provider vehicle id, device timestamp, received timestamp, lat, lon, heading, speed, accuracy, raw payload reference). The gateway maps the provider vehicle id to our `vehicle_id` ([[tracker-vehicle-mapping]]), drops positions outside an active assignment window ([[telematics-consent-and-masking]]), deduplicates ([[position-dedup-rules]]), writes to `position_events` ([[positions-table-partitioning]]), publishes to the `positions` topic for the ETA and geofence consumers ([[eta-feed-publication]], [[geofence-arrival-detection]]). Details in [[position-ingestion-pipeline]].

## Volumes

June 2026, per day: 41 million raw positions received, 9.2 million written after window filtering and dedup. The app produces most raw volume (10 s interval when moving); Trakko produces the most duplicates (their API returns a sliding window); Geolyx produces the least but with the biggest bursts (see [[incident-2025-11-geolyx-duplicate-flood]]).

## Ownership

The integrations team owns the gateway, the adapters and the mapping. The mobile team owns the app-side collection (interval, battery budget, offline queue); the gateway only sees the upload endpoint. The data platform consumes `position_events` for ETA models and gets nothing else. Compliance owns the retention rule (90 days raw); the gateway implements the window filter because that is where positions enter.

## What is not here

- No video telematics, no driver behaviour scoring, no fuel data. Two providers offered these; the compliance position on driver data (their project has the note) is that we do not collect what we have no purpose for, and the answer has been the same three times.

- No direct CAN bus or OBD integration. Trakko's units read odometer and ignition state and we take those two fields, nothing else.

- No historical import when a provider is connected. Tracking starts at connection.

Cost per source is in [[telematics-costs-per-provider]]; how to connect a new provider is in [[provider-onboarding-runbook]]; the test harness that lets us develop without live trucks is in [[telematics-integration-test-harness]].

## How the sources compare in practice

Measured over May 2026 on the 310 vehicles that had two sources at once, which is the only fair comparison.

| Metric | App | Trakko | Geolyx |
|---|---|---|---|
| Median interval between kept positions while moving | 10 s | 30 s | 15 s |
| Median accuracy | 8 m | 15 m (assumed) | 6 m |
| Positions lost in tunnels and blank zones, then recovered | recovered (device queue, 60 s batches) | recovered (unit buffer, 90 s) | recovered (Geolyx buffer, 24 h) |
| Wrong vehicle rate (mapping) | 0.2 % (driver picked the wrong truck) | 0.9 % (unit moved) | 0.1 % |
| Clock trust | phone, network-synced | device, see the drift incident | Geolyx server |
| Cost per vehicle per month | 0.40 EUR | 6.10 EUR | 1.60 EUR |

The app wins on everything but "the driver has to do something". Trakko exists for carriers whose drivers will not or cannot use a phone app (owner-drivers with a personal phone they refuse to use for work are the main case), and Geolyx exists because installing a second box in a truck that already has one is absurd.

## What the platform sees

The platform API never talks to a provider. It reads `position_events` through the gateway's internal endpoints (`GET /internal/vehicles/{id}/last-position`, `GET /internal/assignments/{id}/positions`) and consumes the `positions_selected` topic for live updates. This boundary is deliberate: the provider adapters have changed four times in eighteen months, the internal contract has not changed once.

## Failure modes by source

- App: the driver's phone dies or the app is killed by the OS. Detected by "no position for 15 minutes" and by the mobile team's heartbeat. Recovery: none from our side; the dispatcher calls.

- Trakko: their API slows down or a fleet key expires. Detected by poll latency and 401 counters per fleet. Recovery: the poller backs off and catches up through the history endpoint within the rate limit.

- Geolyx: their platform stops pushing. Detected by "no batch for any subscription for 5 minutes" (they always have some vehicle moving). Recovery: they replay, we absorb at low priority since November 2025.

The one failure mode that is ours alone is the gateway itself being down. Adapters do not buffer on our side: Trakko's window and Geolyx's retries buffer for us, and the app queues on the device. A gateway outage under 5 minutes loses nothing; longer than that, Trakko positions beyond the 5 minute window need the history catch-up, which is automatic.
