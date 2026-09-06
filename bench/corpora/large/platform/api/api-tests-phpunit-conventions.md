---
name: api-tests-phpunit-conventions
description: PHPUnit layout on halden-api (Unit, Integration with a real PostgreSQL in CI, Api via WebTestCase), fixtures through builders not Alice, transaction rollback per test, 11 min CI budget
type: reference
status: active
verified: 2026-06-02
---

# Test conventions (halden-api)

## Three suites, three directories

- `tests/Unit`: no container, no database, no clock. Pure PHP. Runs in about 20 s. Must stay under 30 s or the pre-push hook becomes annoying and people disable it.
- `tests/Integration`: booted kernel, real PostgreSQL 16 (the CI service container, same major as prod), real Redis. Repository queries, Messenger handlers, anything with SQL.
- `tests/Api`: `WebTestCase`, HTTP in and JSON out, asserts on status code and body. This is where the error envelope and pagination contracts are tested.

No SQLite. We had SQLite for unit-ish repository tests until 2024 and every PostgreSQL-specific query (`@@`, `ON CONFLICT`, row comparisons, `timestamptz`) was untestable. If a test needs SQL it needs PostgreSQL.

## Database per test

`App\Tests\DatabaseTestCase` wraps each test in a transaction and rolls it back in `tearDown()`. This is 10x faster than truncating tables. Two caveats:

- Anything that commits explicitly (the invoice number allocator, the outbox) escapes the rollback. Those tests extend `CommittingDatabaseTestCase` which truncates the specific tables it declares in `protected static array $tables`.
- `CREATE INDEX CONCURRENTLY` is not testable inside a transaction, so migrations are tested by the `migrations-up-to-date` CI job against a restored dump, not by PHPUnit. See [[doctrine-migration-workflow]].

## Fixtures: builders, not Alice

`App\Tests\Builder\LoadBuilder::aLoad()->open()->from('Lyon')->to('Milano')->withBids(3)->build($em)`. Each builder produces a valid entity with sensible defaults and only the fields you name differ. Alice YAML fixtures were removed in HF-1050 because every test depended on the same 200-line YAML and changing one line broke thirty tests.

Reference data (countries, vehicle types, cancellation reasons) is loaded once per suite by `ReferenceDataLoader`, it is not a fixture.

## Clock

`Psr\Clock\ClockInterface` everywhere, `Symfony\Component\Clock\MockClock` in tests. `new \DateTimeImmutable()` in `src/` fails PHPStan (custom rule `NoNativeDateTimeRule`). This is what makes the invoice year boundary and the bid expiry testable. See [[timezone-handling-utc-rule]].

## Time budget

CI target is 11 minutes for the full pipeline on a PR: lint 1 min, unit 30 s, integration 4 min, api 3 min, the rest is image build. The `@group concurrency` and `@group slow` tests run only on `main` and nightly. When integration crosses 5 minutes we split it into two parallel jobs by directory, which happened once already in April 2026.

## Assertions we have and you should use

- `assertMaxQueries()` from [[n-plus-one-loads-list-fix]].
- `assertErrorEnvelope($response, 409, 'load-not-biddable')`.
- `assertMessageDispatched(GenerateInvoicePdf::class)` on the in-memory transport, all transports are `in-memory://` in the `test` env.
- `assertEventuallyEquals()` for the two tests that involve a real worker process. Polls up to 3 s. Use rarely.
