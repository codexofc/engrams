---
name: model-registry-conventions
description: Models live in the models bucket per name and version with card, features.lock and eval, promoted by a current pointer
type: reference
status: active
verified: 2026-02-16
---

## Layout

Object store bucket `models`, one prefix per model name and version:

```
models/eta-v3/2026-06-15T03/
  model.bin            the serialised boosters (six for eta, one per quantile and target)
  card.yaml            what it is, see below
  features.lock        the feature registry entries used, with their version hash
  eval.json            metrics from the backtest that promoted it
models/eta-v3/current  a small file containing the version path above
```

`ml-infer` and `pricing-svc` watch the `current` pointer and hot-reload the model when it changes. Rolling back is writing the previous path into `current`. No model file is ever overwritten; old versions are deleted after 180 days except the ones tagged `keep` (the version in production during an incident is always tagged).

## Naming

`<purpose>-v<major>`: `eta-v3`, `demand-v2`, `price-suggestion-v2`, `rest-detect-v1`. The major changes when the target, the model family or the feature set changes in a way that makes metrics incomparable. A retrain is a new version under the same major, named by the training timestamp.

## The card

`card.yaml` is mandatory and validated by `mlkit register`:

- `name`, `version`, `owner` (a team), `created_at`
- `purpose`: one sentence
- `target`: the exact definition (column and transformation)
- `training_window`: from, to, row count, filters
- `features`: the registry names (must match `features.lock`)
- `metrics`: the backtest figures ([[backtesting-framework]]) with the evaluation window
- `promotion`: `auto` or `manual`, with the tolerance rule applied
- `known_limits`: free text, at least one line (the loads without a route for ETA, the clusters under 6 months for demand)

A model without a card cannot be pointed to by `current`; the registration command refuses it.

## Promotion

`mlkit promote <name> <version>` runs the backtest on the last 14 days, compares with the current version's metrics on the same window, and:

- promotes automatically if every primary metric is within the tolerance in the card (ETA: MAE within +1 minute and coverage within ±2 points; demand: WAPE within +1 point);
- otherwise leaves the candidate in place and posts the comparison to the ML channel for a manual decision.

Every promotion, automatic or manual, writes a line in `models/<name>/history.log` with who or what promoted and the metrics diff.

## Serving contract

A served model declares its input as the list in `features.lock`; `ml-infer` assembles the vector from the feature store ([[feature-store-design]]) in that order and refuses to load a model whose lock references a feature missing from the registry. A feature renamed in the registry therefore breaks the load of old models, which is intended: the rename must ship with a retrain.

## What is not here

Notebooks, experiments and one-off models live in the `ml-experiments` repository with a README per experiment; they are not registered. The registry is for what runs in production.
