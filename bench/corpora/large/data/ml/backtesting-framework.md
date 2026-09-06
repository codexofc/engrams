---
name: backtesting-framework
description: mlkit backtest runs rolling time splits from a spec with guards and segments, the only evaluation the promotion rule accepts
type: reference
status: active
verified: 2026-02-23
---

## What it is

`mlkit backtest <spec.yaml>` evaluates a model the way it will be used: train up to a date, predict the following window, move forward, repeat. It is the only evaluation that the promotion rule of [[model-registry-conventions]] accepts, and the monthly evaluations such as [[eta-eval-2026-05]] are backtests on the deployed model's prediction log rather than on retrained candidates.

## Spec

```
model: eta-v3
mode: candidate            # or: production_log
train_window_days: 120
horizon_days: 14
step_days: 7
windows: 12
target: log_remaining_minutes
metrics: [mae_minutes, coverage_p10_p90, mae_by_horizon]
baseline: routing_time_plus_margin
segments: [country_pair_band, distance_band, driver_app_version_band, carrier_is_new, alpine_crossing]
guards:
  row_count_delta_max: 0.3
  suspicious_gain_ratio: 0.5
```

- `mode: candidate`: trains the model at each split from the training set built by `build_training_set` ([[feature-store-design]]) and evaluates on the following window. Used before promotion.
- `mode: production_log`: no training; reads the prediction log (`raw.ml_predictions_<model>`) and the actual outcomes, and computes the same metrics and segments. Used for the monthly evaluation and the drift monitoring.

## Guards

- `row_count_delta_max`: a split whose training rows per day differ by more than 30 % from the previous split's fails the run. This is what caught the two half-empty weeks after warehouse ingestion delays, and refused the automatic promotion of the price suggestion model twice.
- `suspicious_gain_ratio`: a candidate beating the baseline by more than 50 % relative is flagged `suspicious_gain` and cannot be promoted without a second person's note in the card ([[training-data-leakage-lesson]]).
- Every metric is reported with the baseline on the same rows, and with the standard deviation across windows.

## Outputs

- `eval.json`: metrics per window and averaged, baseline included, guards' results. Copied into the model directory at registration.
- `segments.csv`: metric per segment value with row counts. Segments under 500 rows are reported but marked `low_n`.
- A short markdown summary posted to the ML channel with the diff against the current production model when `mode: candidate`.

## Cost and where it runs

An ETA candidate backtest with 12 windows trains 12 times 6 boosters on about 4 M rows: 55 minutes on the ML batch host (16 vCPU). Demand: 12 windows times 14 horizons, 20 minutes. The `production_log` mode is a few warehouse queries, 2 minutes.

Runs on the `ml-batch` host from the weekly retrain scheduler, or by hand. Results are kept 1 year under `evals/<model>/<date>/`.

## What it refuses

- A spec without `baseline`.
- A spec whose target is not the one in the model card.
- Random splits. There is no option for it; `step_days` and `windows` are the only way to cut, and they cut by time.
