---
name: testing-widget-vs-integration
description: Unit tests for sync and db (70 % coverage gate), widget tests only for screens with logic (pumpWidget with fake repositories), integration_test on the four device farm phones nightly, golden tests dropped in 2025
type: reference
status: active
verified: 2026-04-30
---

# Test strategy for the driver app

## Three layers, deliberately unbalanced

**Unit tests** (`test/sync`, `test/db`, `test/domain`): the bulk. `SyncCoordinator`, `ConflictResolver`, `LoadProjector`, the mutation outbox, the drift migrations. Pure Dart with an in-memory SQLite (`NativeDatabase.memory()`) and a fake HTTP client that replays recorded JSON fixtures from `test/fixtures/api/`. Coverage gate 70 % on `lib/sync` and `lib/db`, enforced in CI. Runs in 3 minutes.

The fixtures are real anonymised responses captured from staging with `tool/capture_fixture.dart`, refreshed when the API contract changes. A fixture older than 6 months fails a lint (`fixture_age_test.dart`), which forces us to look at drift between app and API.

**Widget tests** (`test/screens`): only for screens with conditional logic (the delivery screen's state machine, the login flow, the conflict banner). Not for layout. `pumpWidget` with fake repositories injected through Riverpod overrides. About 40 tests. A screen that only displays data does not get a widget test, the integration test covers it.

**Integration tests** (`integration_test/`): six scenarios, run on the four physical phones nightly (see [[ci-pipeline-mobile-runners]]):

1. Login online, kill network, unlock with PIN offline.

2. Receive an assignment through sync, open it.

3. Full delivery offline (pickup, arrive, POD photo, deliver), reconnect, check the server got everything in order.

4. Conflict: load reassigned while offline.

5. Push notification tap opens the right load.

6. Upgrade from the previous release: install previous APK, create pending mutations, install new APK, check the migration and the push.

Scenario 6 is the one that justifies the whole device farm. It caught the `pending_mutations.seq` index bug before release.

## Golden tests: dropped

We had 60 golden screenshot tests in 2024. Every font hinting change, every Flutter upgrade and every difference between the Linux runner and a macOS laptop made them fail, and people updated the goldens without looking. Removed in HF-1160 (September 2025). Visual review is done by a human on the debug APK posted in the merge request.

## What a test must not do

- Depend on wall clock: `Clock` is injected (`package:clock`, `withClock` in tests). See [[lesson-never-trust-device-clock]].

- Hit the real staging API. The fixtures exist for that.

- Sleep. `FakeAsync` or `pumpAndSettle`.

## Flakiness

Flaky integration tests are quarantined by moving them to `integration_test/quarantine/` with a ticket in the file header, and they run but do not block. Two are there now, both involving the Pixel's camera emulation. The rule is a quarantined test is fixed or deleted within a month.
