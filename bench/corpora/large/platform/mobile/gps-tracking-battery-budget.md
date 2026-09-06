---
name: gps-tracking-battery-budget
description: Tracking targets under 8 % battery per 8 h shift, achieved with adaptive intervals (10 s moving, 120 s stopped, off outside an active load), batched uploads every 60 s and a distance filter of 25 m
type: project
status: active
verified: 2026-05-27
---

# GPS tracking and the battery budget

The driver app sends positions so dispatchers see trucks on the map and shippers get ETAs. The constraint from the first pilot in 2024: drivers uninstall an app that drains their phone. The budget we set and still hold: **under 8 % of battery per 8 hour shift** on a mid-range Android from 2022, screen off, with tracking active the whole time.

## Where the budget goes (measured on Galaxy A25, app 4.9, May 2026)

| Item | Battery per 8 h |
|---|---|
| GPS fixes at the adaptive interval | 3.9 % |
| Upload batches (radio wake-ups) | 1.4 % |
| Foreground service and notification | 0.6 % |
| Everything else (sync, UI when opened) | 0.8 % |
| Total | 6.7 % |

Measured with `adb shell dumpsys batterystats` after a reset, over three real shifts of two volunteer drivers. Not a lab number. The first version in 2024 was at 19 %.

## Rules in `TrackingController`

- **Tracking only during an active load.** Starts at `pickup`, stops at `deliver` of the last stop or when the load leaves the driver. Outside a load, zero GPS. Dispatchers wanted "always on" for empty trucks and were refused, drivers are not employees of Halden and the works councils of two carriers asked about it explicitly.

- **Adaptive interval**: 10 s while speed is above 8 km/h, 120 s once speed has been under 3 km/h for 2 minutes, back to 10 s on the first fix above 8 km/h. The stopped state is where a loading dock wait of 90 minutes would otherwise burn the budget.

- **Distance filter 25 m**: fixes closer than that to the last accepted one are dropped before they reach the upload queue. Removes GPS jitter at a standstill without turning off the receiver (turning it off and on costs more than keeping it warm at a low rate).

- **Accuracy**: `LocationAccuracy.high` on Android (fused provider), `kCLLocationAccuracyNearestTenMeters` on iOS. `best` on iOS doubled the cost for nothing visible on a map at city scale.

- **Batched upload every 60 s**, or when 30 positions are queued, or at the `deliver` mutation. `POST /internal/mobile/positions` with an array. Positions older than 24 h in the queue are dropped, the server does not want them anyway. The server-side limiter accepts at most one position per 10 s per driver and silently drops the rest, so a batch of 6 for 60 s is the expected shape.

## Foreground service (Android) and background modes (iOS)

Android: `TrackingForegroundService` with `foregroundServiceType="location"`, a persistent notification "Suivi en cours pour L-2026-004512" with a stop button. Required by Android 14 and it is honest to the driver. Some manufacturers kill it anyway, see [[device-quirk-samsung-battery-optim]].

iOS: `location` background mode with `allowsBackgroundLocationUpdates = true` and `pausesLocationUpdatesAutomatically = false`. iOS shows the blue bar. The main loss on iOS is drivers who choose "Allow once" for location and never see the prompt again, about 8 % of iOS installs, handled by a settings deep link on the tracking screen.

## What the server does with it

Positions land in the data platform's tracking topic, not in the API's PostgreSQL. The API only keeps `loads.last_position` (lat, lng, at, speed) updated by the same endpoint, which is what the dispatch map reads. History is a data platform concern.

## Things we tried and dropped

- **Geofencing to detect arrival** instead of speed: unreliable at dock entrances behind gates, and geofence callbacks on Android arrive 2 to 10 minutes late. Arrival is confirmed by the driver tapping "Arrivé", the position at that moment is attached to the event.

- **Activity recognition** (in vehicle / still) to drive the interval: cheaper than GPS in theory, but the API had 30 to 60 s of lag and produced false "still" while driving in slow traffic. The speed rule is dumber and works.

- **Significant location change** on iOS as the only source: 500 m granularity, useless for ETAs.

## Alerting

If a driver in `IN_TRANSIT` has not sent a position for 20 minutes, the dispatch board shows the truck grey with "Dernière position il y a 23 min". This is the signal that drives the Samsung investigation, the kill detection lives on the device, see [[device-quirk-samsung-battery-optim]].

Related: [[lesson-never-trust-device-clock]] for why each position carries both the device time and the monotonic uptime.

## Volume and what the server does at the edge

Numbers from May 2026, weekdays: 1 100 to 1 400 drivers in transit at peak, 1.4 M accepted positions per day, 120 000 batches per day. Each batch is a JSON array of 3 to 8 positions, about 900 bytes, so the whole tracking traffic is around 110 MB a day inbound, which is less than the sync traffic before the delta pull existed.

`POST /internal/mobile/positions` does four things and nothing else: validate the array (max 60 entries, timestamps within the last 24 h and not in the future by more than 5 minutes after clock correction), drop entries closer than 10 s to the previous accepted one for that driver (kept in Redis under `pos:last:<driver>` with a 1 h TTL), update `loads.last_position` with the newest accepted entry in one `UPDATE` (no `SELECT` first, the load id is in the batch), and publish the accepted entries to the data platform topic through the Messenger `async` transport in a single message per batch. It answers 200 with `{ "accepted": n, "dropped": m }`, and the app logs the dropped count only when it is above half the batch, which is the signal of a duplicated upload.

The p99 of the endpoint is 18 ms. It was 90 ms when it did a `SELECT` on the load to check the driver was assigned. The assignment check now uses the JWT's `sub` and a Redis set `assigned:<driver>` refreshed by the assignment handler, and a position for a load the driver is not assigned to is dropped with a log line, not an error. This is the one endpoint where we accepted a cache-based authorisation check, because the worst case is a position recorded on a load the driver no longer has, which the dispatcher can see and ignore.

Retention: `loads.last_position` is overwritten, the data platform keeps 400 days of history for the ETA model and for disputes (waiting time invoicing needs the arrival time, and the position trail is what proves it). A driver can ask for their trail to be deleted after a load is invoiced, which happened twice, and it is a manual data platform operation.

## Calibration knob

`TrackingConfig` in the app reads `values.tracking_moving_interval_s` (default 10) and `values.tracking_stopped_interval_s` (default 120) from the remote config, so the intervals can be changed for one carrier without a release. It was used once, for a carrier doing urban courier work where 120 s at a stop made the map look frozen: 60 s for them, with a measured 1.1 % extra battery per shift.
