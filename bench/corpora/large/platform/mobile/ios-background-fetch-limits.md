---
name: ios-background-fetch-limits
description: iOS gives the driver app at most a few background fetches per day and kills the sync isolate after 30 s, so background sync on iOS is best effort and the foreground pull is the real path
type: feedback
status: active
verified: 2025-11-14
---

# iOS background execution: what we actually get

Written after two weeks of instrumenting app 4.4 on 40 volunteer iPhones (HF-1195), because the team kept assuming iOS behaves like Android.

## Measured

- `BGAppRefreshTask` scheduled every 15 min: executed 3 to 8 times per day per device. Median gap between executions 2 h 10 min. iOS learns the usage pattern and schedules around it, a driver who opens the app at 7:00 and 17:00 gets refreshes clustered around those times.

- Each execution gets about 30 s of wall time before `expirationHandler` fires. Our delta pull takes 1 to 9 s, so it fits, but a full initial sync (12 s on 3G) does not, and it must not run there.

- Silent pushes (`content-available`): delivered and executed in the background 60 % of the time. The rest is dropped when the device is in Low Power Mode, when the app was force-quit by the user (swiped up in the app switcher, then nothing wakes it until the next manual launch), or by APNs throttling after more than 2 or 3 per hour.

- `BGProcessingTask` (long running, requires charging and idle): runs once a night on devices that charge overnight, used for the local database purge. Never observed on devices that are not charging.

## Consequences in the design

- The foreground pull on app open is the path that is guaranteed. Every screen shows the age of the data ("Mis à jour il y a 4 min") so a driver knows to pull to refresh.

- The 15 min timer from [[sync-push-triggered-delta]] only runs while the process is alive. On iOS that means while the app is on screen or a few minutes after backgrounding.

- The outbox push after a delivery is tried immediately, and if the app is backgrounded while it is in flight we request `beginBackgroundTask`, which gives about 30 s more. A push that still cannot finish stays in the outbox for the next foreground.

- Nothing on iOS relies on the background for correctness, only for freshness.

## Force-quit

The main support case: a driver force-quits the app "to save battery" and then does not receive assignments until they open it. We added a one-time explanation dialog when we detect a cold start after a `willTerminate` (app 4.5), and support has a canned answer. No technical fix exists.

## Location is different

Background location updates through the `location` background mode are not subject to these limits: while a load is in transit, the app keeps running for GPS, see [[gps-tracking-battery-budget]], and the sync timer benefits from it as a side effect. This is why iOS sync during a delivery is fine and iOS sync while waiting for the next assignment is not.
