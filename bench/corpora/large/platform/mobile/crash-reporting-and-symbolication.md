---
name: crash-reporting-and-symbolication
description: Crashes and non-fatal errors go to the self-hosted crash reporter (glitchtip-like, in the ops cluster), Dart symbols and native dSYM/so files uploaded per build by CI, crash-free target 99.5 %, ANRs tracked separately
type: project
status: active
verified: 2026-06-05
---

# Crash reporting

## Where

Self-hosted error tracker running in the ops cluster, project `driver-app`. Chosen over the hosted offerings because driver phone numbers appear in breadcrumbs and legal preferred the data to stay on our infrastructure. The SDK is the Flutter package of that tracker, initialised in `main()` before `runApp`, with `FlutterError.onError` and `PlatformDispatcher.instance.onError` both hooked.

## Symbolication

Release builds use `--obfuscate --split-debug-info=build/symbols`. Without the symbol files a Dart stack trace is a list of hex offsets. CI uploads, per build number:

- `build/symbols/app.android-arm64.symbols` and the arm and x64 variants

- `build/symbols/app.ios-arm64.symbols`

- The native `.so` files of the Android build (for crashes inside plugins) and the iOS `dSYM` bundle

The upload step is the one that must not be skipped: a release whose symbols were not uploaded produces unreadable crashes for its whole life. It happened with 4.3.1 (a hotfix built by hand on a laptop) and we could not read a single crash of that version. The rule since then: no build outside CI, see [[ci-pipeline-mobile-runners]].

## What is reported

- Fatal crashes.

- Non-fatal errors passed explicitly with `reportError(e, stack, context: {...})`. Used in the sync layer for every unexpected server response and in the upload pipeline, so we see failures that do not crash. The volume is around 400 per day, mostly network timeouts, which are grouped by their `context.kind`.

- Breadcrumbs: screen navigations, sync start and end, mutation push results. Never the mutation payload (it contains addresses).

- User context: `installation_id`, `carrier_id`, app version, OS version, device model. Not the driver id, not the phone number.

## ANRs

Android "Application Not Responding" is not a crash and the SDK does not catch it. We read the ANR rate from the Play Console once a week. It was 0.9 % in October 2025 (bad, the threshold for store visibility penalties is 0.47 %) and dropped to 0.2 % after moving the image preparation to an isolate, see [[pod-photo-compression]].

## Targets and rollout gate

Crash-free sessions above 99.5 % for a version to progress through the staged rollout, see [[release-process-stores]]. Current: 99.7 % on 4.9. The top crash signature of the last quarter was a `PlatformException` from the camera plugin on Android 11 devices when storage is full, handled since 4.9 with a message asking to free space.

## Alerting

A new crash signature seen on more than 20 installations in an hour pages the mobile on-call through the same alert routing as the platform. Configured as a webhook from the tracker to the alert manager.
