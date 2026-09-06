---
name: dq-framework-overview
description: Data quality rules are yaml files run by dq-runner after every marmot model, results in dq.results, 310 rules on 62 tables in June 2026, five rule kinds (not_null, unique, accepted_values, freshness, reconciliation, anomaly), severity decides paging, owned by the data team with rules owned by domains
type: reference
status: active
verified: 2026-06-26
---

# The data quality framework

## What it is

`dq-runner` is a 2 800-line Rust binary in the `data-platform` repository that reads rule files from `dq/rules/<domain>/*.yaml`, runs each rule as a query against the warehouse, writes one row per rule execution to `dq.results`, and emits alerts according to severity ([[dq-severity-and-paging]]). It is invoked by marmot as a post-hook after each model run (`marmot run --select core.loads` then `dq-runner --for core.loads`), and on a schedule for rules that are not tied to a model (the reconciliations, the freshness monitors).

It replaced a set of ad hoc row-count checks ([[legacy-row-count-checks]]) in October 2025 after the duplicate loads episode ([[missed-2025-11-duplicate-loads-after-replay]]) showed that "the row count looked fine" is not a quality check.

## Rule kinds

| Kind | What it checks | Example |
|---|---|---|
| `not_null` | a column has no nulls (or fewer than a tolerance) | `core.loads.carrier_id` where `status >= 'assigned'` |
| `unique` | a set of columns is unique | `core.bids (bid_id, version)` |
| `accepted_values` | a column is in a list or a reference table | `core.invoices.currency IN dim.currencies` |
| `freshness` | the max of a timestamp column is recent | `core.load_status_history.inserted_at > now() - 15 min` ([[freshness-monitors]]) |
| `reconciliation` | two aggregates agree within a tolerance | invoices total vs Payla settlements ([[reconciliation-payla-settlements]]) |
| `anomaly` | a metric is within its seasonal band | daily `overdue` count per country ([[volume-anomaly-seasonal-thresholds]]) |
| `schema` | a table's columns match the declared set | ([[schema-drift-monitor]]) |

Every rule is a yaml document:

```
- name: loads_assigned_have_carrier
  kind: not_null
  table: core.loads
  column: carrier_id
  where: "status IN ('assigned','in_transit','delivered')"
  tolerance: 0
  severity: page
  owner: dispatch
  description: an assigned load always has a carrier, a null here is a CDC or projection bug
  runbook: dq/runbooks/loads_assigned_have_carrier.md
```

`dq-runner lint` refuses a rule without `owner`, `severity` and `description`, and a `page` rule without `runbook`. The rule catalogue for the core tables is in [[dq-rule-catalog-core]].

## Results

`dq.results(run_id, rule_name, ran_at, table, status, observed, threshold, sample_keys, duration_ms)`, ClickHouse, 400 days. `status` is `pass`, `warn`, `fail`, `error` (the query itself failed, which is its own alert). `observed` is the number the rule computed (nulls found, duplicates found, seconds stale, difference in cents), `threshold` what it was compared to, `sample_keys` up to 20 primary keys of offending rows so that the runbook can start from concrete cases. 1.4 M rows in June 2026, 40 000 a day.

The `Data quality / Overview` board reads it: pass rate per domain per day, rules currently failing, longest-failing rule, execution time per rule (the cost note, [[dq-checks-runtime-cost]], came from that panel). Per-domain scorecards ([[dq-scorecards-per-domain]]) aggregate it monthly.

## Ownership

The data team owns `dq-runner`, the result table, the alert routing and the framework rules (`schema`, `freshness` on every `core` table). Domains own their rules: `dispatch`, `pricing`, `billing`, `ml`, `product`. A rule's alert goes to its owner's channel or pager; the data team sees the aggregate. A new `core` table gets `schema` and `freshness` rules automatically from the marmot model's metadata, and the domain owner is asked in the model's MR to add at least one `not_null` and one `unique` rule, which `dq-runner lint --strict` enforces for tables tagged `tier: 1`.

## What a rule is not

- Not a unit test of the transformation. A marmot model's SQL has its own tests in the model's repository (fixtures in, expected rows out). A dq rule runs on production data after the model and says whether the data is the shape the business expects, whatever produced it.

- Not a fix. A rule that finds duplicates does not delete them. The runbook says who does what.

- Not free. 310 rules cost 22 minutes of warehouse CPU a day; the cost note has the pruning done in May 2026.

## Numbers (June 2026)

| | Value |
|---|---|
| rules | 310 (94 `not_null`, 71 `unique`, 48 `accepted_values`, 62 `freshness`, 14 `reconciliation`, 12 `anomaly`, 9 `schema`) |
| tables covered | 62 of 71 in `core`, 9 of 20 in `mart`, 0 in `raw` by design |
| pass rate, month | 99.62 % |
| `page` rules | 41 |
| pages in the month | 3 |
| `warn` alerts in the month | 38 |
| rules added in the month | 11, of which 6 after an incident |

## Rules that came from incidents

The half of the catalogue that earns its keep. Each entry in `dq/rules/` may carry an `origin: HF-4xxx` field, and 140 of the 310 rules have one. The ones with the longest stories are the null carrier ids ([[caught-2026-01-null-carrier-ids]], a catch), the duplicate loads and the SEK invoices ([[missed-2026-04-sek-invoices-summed-as-eur]], both misses that produced rules), and the timezone shift ([[incident-2026-06-timezone-shift-status-history]], caught by a rule written for something else). The [[dq-rules-review-feedback]] note has what the team learned about writing rules that fire for the right reason, and the [[quality-owner-preferences]] note has the habits of the person who runs the monthly review.
