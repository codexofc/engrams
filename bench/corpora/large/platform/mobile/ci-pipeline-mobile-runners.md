---
name: ci-pipeline-mobile-runners
description: Mobile CI runs on two self-hosted macOS runners (mac-runner-01 and -02, Apple silicon) for iOS builds and on the cluster's Linux runners for analysis, tests and Android, full pipeline 22 min, device farm is four physical phones on a USB hub
type: reference
status: active
verified: 2026-05-13
---

# Mobile CI pipeline

## Runners

- **Linux** (Kubernetes runners of the platform, image `hf/flutter-ci:3.27`): `flutter analyze`, `flutter test`, Android build. 4 CPU, 8 GB. Android build takes 7 min with the Gradle cache warmed.

- **macOS**: two Mac minis in the office, `mac-runner-01` and `mac-runner-02`, Apple silicon, 16 GB, registered as tagged runners `macos-arm`. iOS build takes 9 min. Xcode version pinned per branch in `ci/xcode-version.txt` and selected with `xcode-select` at job start, so a release branch keeps building after `main` moves to a newer Xcode.

- Both Macs also run the device tests, see below. If both are down, iOS builds queue, there is no cloud fallback. The last time both were down at once was a power cut in March 2026, 5 hours.

## Stages (merge request)

1. `analyze`: `flutter analyze --fatal-infos` and `dart format --set-exit-if-changed`. 1 min.

2. `test`: `flutter test --coverage`, coverage threshold 70 % on `lib/sync` and `lib/db` only, no threshold elsewhere. 3 min. See [[testing-widget-vs-integration]].

3. `build-android`: debug APK, artifact kept 3 days, link posted in the MR for testers. 7 min.

4. `build-ios`: only when `ios/` or `pubspec.lock` changed, or on release branches. 9 min.

5. `size-check`: compares the release APK size with `main`, fails above +1 MB without the `size-ok` label. See [[app-size-budget]].

Total for a typical MR: 12 min (no iOS build). Release branch: 22 min.

## Device farm

Four physical phones on a powered USB hub attached to `mac-runner-01`: Galaxy A25 (Android 14), Pixel 6a (Android 15), a Huawei P40 lite without Google services, iPhone 12 (iOS 17). The job `device-smoke` runs the integration tests in `integration_test/` on all four, nightly on `main` and on demand with the `run-devices` label. 25 min. The A25 is the one that finds the bugs, see [[flutter-upgrade-3-27]].

Phones are kept at 80 % charge with a smart plug schedule, because two batteries swelled in 2025 from being at 100 % permanently.

## Secrets

Signing keys (Android keystore, iOS certificates and provisioning profiles) are in the CI secret store, injected as files at job start and deleted at job end. The iOS profiles are managed with `fastlane match` against a private repository. Nobody has the keystore on a laptop, and the recovery procedure (if the CI secret store is lost) is in the ops vault runbook.

## Release job

Manual trigger on `release/*` branches, runs the two builds with `--obfuscate`, uploads symbols and bundles. See [[release-process-stores]].
