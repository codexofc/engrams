---
name: lesson-never-trust-device-clock
description: Device clocks are wrong on 2 % of driver phones (up to 11 days off), so every event carries device time plus monotonic uptime plus a server-computed offset, and the server assigns the authoritative timestamp on receipt
type: feedback
status: active
verified: 2025-12-15
---

# Never trust the device clock

## What we saw

In September 2025 a shipper disputed a delivery time: the app said 14:02, the dock's own log said 16:35. The driver's phone had automatic time disabled and was 2 h 33 min behind. A scan of one week of mutations gave 2.1 % of devices with a clock more than 5 minutes off from the server at push time, and 12 devices more than a day off (the record is 11 days, a phone that had been reset without a SIM).

The mutation payload carried `occurred_at` from `DateTime.now()`, and the API stored it as the event time. That is the bug: a locally generated wall-clock time was treated as truth.

## The rule now (app 4.5, API HF-1205)

Every mutation and every GPS position carries three time fields:

- `device_at`: wall clock on the device, ISO 8601 UTC. Displayed to the driver, never used for ordering or disputes.

- `device_uptime_ms`: monotonic clock (`Stopwatch` started at process boot, plus the OS uptime at start on Android via `SystemClock.elapsedRealtime()`, `ProcessInfo.systemUptime` on iOS). Survives clock changes, not reboots.

- `pushed_at`: set by the server on receipt.

The server computes the authoritative `occurred_at` as `pushed_at - (uptime_at_push - device_uptime_ms)` when the device sends its current uptime in the push request header `X-Device-Uptime-Ms`, and both the mutation's uptime and the header are from the same boot (a `boot_id` is included, random UUID generated once per process start). When they are not (reboot in between), the server falls back to `device_at` corrected by the last known offset for that installation, stored in `device_clock_offsets`. When there is no known offset either, `device_at` is used as is and the event is flagged `time_source: device_uncorrected`, which the web timeline shows with a small warning icon.

For a mutation pushed right away, the correction is exact to the second. For one pushed after 4 h offline, the monotonic path is still exact unless the phone rebooted. In practice 97 % of events are corrected through uptime, 2.5 % through the stored offset, 0.5 % uncorrected.

## How to apply it elsewhere

- Ordering of events from one device: use `seq` from the outbox, never timestamps. See the sync note.

- Ordering of events from different devices: server `pushed_at`, accept that it is not the real order.

- Anything legal (delivery time on a POD, waiting time invoicing): the corrected `occurred_at`, with `time_source` visible in the document.

- Tests: inject `Clock`, and have at least one test where the device clock is 3 days ahead.

- Expiry decisions (is the access token still valid, is the load window past) are made against server time received in the `Date` header of the last response, not the device time. `ServerClock.now()` in the app is device time plus the last measured offset.

The same idea, in the GPS pipeline: [[gps-tracking-battery-budget]].
