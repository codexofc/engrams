---
name: telematics-consent-and-masking
description: WindowStage keeps a position only during an active assignment plus 30 min before pickup, drops 55 % of Trakko input, 10 min lookback buffer, opt-out flag
type: project
status: active
verified: 2026-03-06
---

# Assignment-window filtering at ingestion (HF-2094)

The compliance project decided that a position of a named driver outside a job is surveillance we have no purpose for. For the driver app, the mobile team simply stopped collecting outside the window. For hardware trackers the unit reports continuously by construction, so the rule has to be applied where positions enter: `WindowStage` in the pipeline ([[position-ingestion-pipeline]]). Deployed 2026-01-20.

## The rule

A raw position for `vehicle_id` at `device_ts` is kept if there exists an assignment for that vehicle such that:

```
assignment.started_at (or planned_pickup_from - 30 min, whichever is earlier)
  <= device_ts <=
assignment.completed_at (or now, if still running)
```

Otherwise the position is dropped: not stored, not even in the 7 day raw bucket. `outcome=outside_window` is counted per provider, and that counter is the only trace that the position ever existed.

The 30 minutes before the pickup slot exist so that the approach to the pickup site is visible and the arrival geofence ([[geofence-arrival-detection]]) can fire. It was 60 minutes in the first draft; the compliance review asked for the smallest value that covers the approach, and 30 covers 94 % of observed approaches (the rest are long-haul trucks that start their day far away, and for those the ETA before pickup is not a service we promised).

## The index

`AssignmentWindowIndex` holds, in memory per gateway pod, the windows for all vehicles for the last 2 h and the next 48 h, loaded from the platform's `GET /internal/assignments/windows?from=&to=` every 60 s (about 12 000 windows, under 5 MB). A position is checked against the index with a binary search per vehicle. A window that just started may be missing for up to 60 s.

## The late-start problem

A driver presses "start" at 07:04 for a job the planner created at 07:00 with pickup at 08:00. Positions from 07:00 to 07:04 arrived before the window existed and were dropped. Under the rule they should have been dropped anyway (30 min before 08:00 is 07:30). But a planner who creates an assignment at 09:00 for a truck that has been driving with the goods since 08:30 (it happens: paperwork after the fact) loses 08:30 to 09:00 for good, and the geofence never sees the pickup arrival.

Mitigation: a **10 minute lookback buffer**. Positions that fall outside every window are held in a Redis list per vehicle for 10 minutes (`pending:{vehicle_id}`, capped at 120 entries) and re-checked when the index refreshes; if a window now covers them, they proceed. After 10 minutes they are gone. Ten minutes is a compromise: it covers the common "start pressed a few minutes late" case (recovers 3 % of otherwise-dropped positions) and it keeps the maximum retention of an unjustified position to 10 minutes in a volatile store. The compliance rota signed off on the buffer in writing; it is in the balancing test annex.

## Opt-out

`users.tracking_opted_out` (driver-level, set through the DSAR process) is loaded with the windows: a window whose assigned driver has opted out is excluded from the index, so positions from any source for that vehicle during that assignment are dropped. The assignment shows `tracking = 'manual'` to the shipper. Three drivers in March 2026.

## Numbers

Trakko: 55 % of raw positions dropped as outside window (the units run 24/7, jobs cover about half the driving day). Geolyx: 38 % (fleets that are mostly on our platform). App: under 1 % (the app already stops collecting; the residual is the upload of a batch that straddles the job end).

## What this does not do

- It does not filter by geography (no "only inside the EU" rule); the window is the criterion.

- It does not tell the carrier's own Geolyx account anything; their continuous tracking is their business under their contract.

- It does not apply to `unmapped_positions` (the last position per unknown tracker kept for the mapping screen, see [[tracker-vehicle-mapping]]): one position per tracker, no history, 30 day TTL, reviewed with compliance as acceptable.
