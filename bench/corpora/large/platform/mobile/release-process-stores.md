---
name: release-process-stores
description: Driver app releases every two weeks on Tuesday, staged rollout 5/20/50/100 on Android over 4 days, phased release on iOS, min-version enforcement via /internal/mobile/min-version, rollback is a forced update not a store rollback
type: reference
status: active
verified: 2026-06-16
---

# Release process of the driver app

## Cadence and naming

- A release every two weeks, cut on Monday, submitted on Tuesday morning. Hotfixes outside the cadence are allowed with two approvals, and happened 4 times in the last 12 months.

- Version `MAJOR.MINOR.PATCH` in `pubspec.yaml`, build number = CI pipeline id. The build number is what the stores compare, the version string is for humans and for `min-version`.

- Branch `release/4.9` cut from `main`. Only fixes cherry-picked into it after the cut. Tag `mobile-v4.9.0` on the commit that was submitted.

## Pipeline

See [[ci-pipeline-mobile-runners]] for the machines. The release job:

1. `flutter build appbundle --release --obfuscate --split-debug-info=build/symbols` and `flutter build ipa --release --obfuscate --split-debug-info=build/symbols`.

2. Symbols uploaded to the crash reporter, see [[crash-reporting-and-symbolication]].

3. Upload to Google Play internal track and to App Store Connect (TestFlight) with `fastlane`.

4. A human runs the smoke checklist on one physical Android and one physical iPhone from the internal track. The checklist lives in `docs/release-checklist.md` and takes about 40 minutes. The items that catch the most bugs: login with PIN while offline, complete a delivery with a POD photo while offline then reconnect, receive a push and tap it.

5. Promote.

## Rollout

Android: staged rollout on the production track, 5 % on Tuesday, 20 % on Wednesday, 50 % on Thursday, 100 % on Friday. Each step is manual and requires the crash-free rate of the new version to be above 99.5 % and no new crash signature above 20 occurrences. The number is checked on the crash dashboard, not felt.

iOS: phased release (Apple's 7-day curve), which we cannot control finely. Review takes 6 to 36 hours. We submit Tuesday and it usually goes live Wednesday.

## Forcing updates

`GET /internal/mobile/min-version` returns `{ "android": "4.6.0", "ios": "4.6.0", "message": {...} }`. The app checks on start and after each sync. Below the minimum, the app shows a blocking screen with a store link. Above the minimum but below the latest, a dismissible banner.

We raise the minimum only when the server contract changes in a way old versions cannot handle (last time: 4.6 for the sync cursor format). Between raises, at least three versions stay supported. Telemetry on 2026-06-01: 96 % of active devices on the latest or previous version, 3 % two versions behind, 1 % older (mostly devices with automatic updates off).

## Rollback

Stores do not roll back. If a release is bad:

- Android: halt the staged rollout (users who got it keep it), then ship a hotfix build with a higher build number.

- iOS: pause the phased release, submit a hotfix with expedited review (granted 2 out of 3 times).

- If the bad version corrupts data or floods the server (see [[incident-2025-12-sync-storm-after-release]]), raise `min-version` above it so affected devices are forced to the hotfix as soon as it is live.

## Feature flags

Anything risky ships behind a flag defaulting to off and is enabled progressively from the server, see [[feature-flags-mobile-remote-config]]. This is how we get a partial rollback without a build.

## What goes in the release notes

Store release notes are written by product in the six languages and are generic. The internal changelog (`CHANGELOG.md`, one line per PR with the ticket key) is what support reads.
