---
name: schema-drift-monitor
description: schema rules compare a table's columns and types to the marmot model's declared set, after every run and hourly; missing or retyped column pages
type: reference
status: active
verified: 2026-05-19
---

# Schema drift

## What it checks

A `schema` rule ([[dq-framework-overview]]) reads the table's columns from `system.columns` and compares them to the declared set. The declared set comes from the marmot model's `columns:` block for `core` and `mart` tables (generated rules), or from an explicit list in the rule for tables where the list is the point.

| Finding | Severity | Meaning |
|---|---|---|
| column in the table, not declared | warn | a migration added something nobody documented, or the model's metadata is stale |
| column declared, not in the table | page | the model will fail or has failed, downstream rules will `error` |
| column present with a different type | page | a silent cast is happening somewhere |
| column order changed | info | cosmetic, noted for `INSERT` without column lists, which are forbidden anyway |

The `page` cases are the ones that turn into `error` results on other rules ([[dq-severity-and-paging]] treats `error` as `warn` for the data team); catching them at the schema level gives one alert that names the cause instead of fifteen that name symptoms.

## Allowlist mode

`mode: allowlist` means "these columns and no others", `warn` becomes `page` for an unexpected column. Used on five tables where a new column is a data governance question before it is a technical one:

- `core.carriers`, `core.users_dim`, `core.carrier_contacts`: no personal data beyond what is listed. The CDC connector filters columns at the source; this rule is the second net, on the destination.

- `core.invoices` and `core.payments`: finance asked that any new column on money tables be reviewed by them before it exists in a mart.

`carriers_no_pii_columns` in the [[dq-rule-catalog-core]] is one of these. It has fired once, in 2026-02, when a migration added `carriers.contact_phone_e164` to the source table and the CDC filter for `carriers` had `include: all` (the filter on `users` was strict, the one on `carriers` was not). The column reached `core.carriers` for 40 minutes before the rule paged and the CDC filter was fixed. Nothing read it in those 40 minutes; the point is that we know that.

## Schedule

After every marmot model run for the model's own table (post-hook), plus an hourly sweep over every table with a `schema` rule regardless of runs, because a migration on the source does not trigger a model run and the CDC connector propagates new columns within seconds ([[freshness-monitors]] has the same "not tied to a run" logic for the same reason).

## Where declared types come from

The marmot model declares `columns:` with a ClickHouse type each. A model without a `columns:` block gets no generated schema rule and a lint warning; 62 of 71 `core` models have one, the 9 without are `ops` helper tables. The declared type is compared exactly (`Nullable(String)` is not `String`; `DateTime64(3, 'UTC')` is not `DateTime`), which has been noisy exactly once, when a model was changed to `LowCardinality(String)` for a status column and the rule paged. The rule was right: the change was intentional and the `columns:` block had not been updated, and a downstream `accepted_values` rule with a literal `IN (...)` against a `LowCardinality` column had started to run 10 times slower for an unrelated reason. Both fixed in the same MR.

## What it does not check

- Source (PostgreSQL) schema against the CDC topic schema: that is the streaming side's `cdc-schema-check` in the CI of migrations.

- Semantic drift, a column that keeps its name and type and changes meaning (minutes to seconds, local to UTC). No schema rule sees that; the unit rules on the streaming side and the invariant rules here ([[incident-2026-06-timezone-shift-status-history]] for the timezone case) are the defence.

- `raw` tables. Their schema follows the topic schema by construction, and `ingest-svc` records type violations in `raw._skipped`, which has its own `warn` when non-empty.

## Numbers (May 2026)

- 71 `schema` rules (62 generated, 9 explicit).

- Findings in the month: 4 `warn` (undeclared columns, all metadata lag after migrations, fixed by updating `columns:`), 1 `page` (a `mart` model renamed a column and a downstream `not_null` rule would have `error`ed at 06:00; the schema rule paged at 22:10 the evening before, right after the deploy).

- Runtime: 20 to 40 ms per table, `system.columns` is small. All 71 in 3 s hourly.
