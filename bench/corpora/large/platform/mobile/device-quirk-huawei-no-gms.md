---
name: device-quirk-huawei-no-gms
description: About 4 % of Android drivers run Huawei devices without Google services, no FCM and no Play Store, supported through an APK download page, the 15 min sync timer and a maps fallback, decided not to build an HMS variant
type: project
status: active
verified: 2025-12-09
---

# Huawei without Google services

Telemetry (November 2025): 140 active devices out of 3 400 Android are Huawei or Honor models without Google Mobile Services (P40, Mate 30, nova 7). Mostly Polish and Romanian carriers who bought them cheaply in 2021 and 2022.

## What does not work on them

- Play Store: they cannot install or update the app from the store.

- FCM: no push at all, neither silent sync triggers nor alerts.

- Google Maps SDK: the map view is blank.

- Play Integrity: not available, and we do not require it (see below).

## What we do (HF-1310)

- A direct APK download page (`https://app.halden.example/driver/android`) with the same signed build as the store, updated by the release job. The app checks `min-version` like the others and the update banner links to that page instead of the store when `GoogleApiAvailability` reports services missing. Sideloading a 60 MB APK over mobile data is not great, but it works.

- Sync: the 15 min timer in [[offline-sync-architecture]] is the only trigger. We shortened it to 5 min when GMS is absent. Battery impact measured at about 2 % per 8 h shift, acceptable.

- Alerts: replaced by an in-app inbox polled with the sync. Assignments reach these drivers up to 5 minutes late, and their dispatchers were told.

- Map: the app uses the tile-based map widget (same as the web front's map library, MapLibre) instead of the Google Maps plugin when GMS is absent. It was already the fallback for the iOS build variant used in testing, so the cost was small.

## What we decided not to do

An HMS (Huawei Mobile Services) build variant with Huawei Push Kit and AppGallery. Estimated at 3 weeks plus ongoing maintenance of a second push pipeline on the API side, for 4 % of Android devices that shrink every quarter (they were 7 % a year earlier). Product agreed to revisit only if the share goes back up.

## Detection

`DeviceCapabilities.hasGms` is computed once at startup from `GoogleApiAvailability.isGooglePlayServicesAvailable()` and cached. Everything that depends on GMS reads that flag, never checks the manufacturer, because some Huawei devices sold in 2019 do have GMS.

See also [[device-quirk-samsung-battery-optim]].
