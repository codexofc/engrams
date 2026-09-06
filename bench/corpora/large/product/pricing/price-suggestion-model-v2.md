---
name: price-suggestion-model-v2
description: v2 price suggestion: gradient boosted, 31 features, MAPE 9.8 % against 13.1 % for v1, weekly retrain, blend with cluster median
type: project
status: active
verified: 2026-06-30
---

## Role

The suggestion model predicts the awarded price of a load before any bid. It feeds the quote (see [[bid-engine-architecture]]) as the base price for low-confidence clusters and as a correction term for the others ([[pricing-lane-clusters]]). The previous version is described in [[price-suggestion-model-v1]].

## Model

Gradient boosted trees, 31 features, target `log(awarded_price_excluding_surcharges / distance_km)`. Predicting the per-km price in log space made the residuals homoscedastic across distances, which v1 never managed with an absolute target.

Main features by importance (June 2026 retrain):

1. cluster median per km, 90 days
2. distance band and exact distance
3. vehicle type
4. day of week of pickup and hours until pickup
5. `imbalance_ratio` of the lane (loads A to B over B to A)
6. fuel index of the month
7. number of active carriers with a base within 100 km of the origin in the last 30 days
8. shipper's historical award ratio (awarded price over suggestion on their past loads)
9. week of year (seasonality, agriculture in summer, retail in November)

Features 7 and 8 come from the feature store maintained by the data team; the training set is built from the warehouse's bids fact table, not from the production database.

## Evaluation

Holdout of the last two weeks before each retrain. June 2026: MAPE 9.8 % on awarded price, median absolute error 41 EUR, 71 % of loads within 10 %. v1 on the same weeks: 13.1 %. By segment, MAPE is 8.1 % on high-confidence clusters, 11.4 % medium, 17.9 % low.

The model is only worth using where it beats the cluster median: on high-confidence clusters the median alone is at 9.2 % and the model at 8.1 %, hence the blend weights (median 60 %, model 40 % on high, reversed on low).

## Serving

Exported as a serialised booster, loaded by `pricing-svc` at startup and hot-reloaded when `models/price_suggestion/current` changes. Inference 2 ms. If the model file is missing or fails validation (the loader scores 100 reference loads and refuses the model if the MAPE on them exceeds 15 %), the quote uses the cluster median alone and logs `price_model.fallback`.

## Retraining

Sunday 02:00, on the last 180 days of awarded loads, about 900 000 rows. Training 14 minutes. The new model is written to `models/price_suggestion/candidate`, promoted to `current` automatically if its holdout MAPE is within 0.5 points of the previous one, otherwise pricing gets an alert and promotes by hand. Automatic promotion was refused twice in 2026, both times after a warehouse ingestion delay left the last week half empty.

## Open work

- Excluding the surcharges from the target relies on the stored breakdown of each bid, which carriers can edit; about 4 % of bids have an inconsistent breakdown and are dropped from training.
- The shipper award ratio feature leaks a little (a shipper who always accepts the cheapest bid lowers the target and the feature together). Measured leakage effect under 0.3 points of MAPE, accepted for now.

## Blend with the cluster median, exact rule

The quote's base price is `w * model + (1 - w) * median` with `w` by confidence: `high` 0.4, `medium` 0.5, `low` 0.6, and `1.0` when the cluster has no median at all (fewer than 3 awarded loads in 90 days, 9 % of clusters, 0.8 % of loads). The weights were chosen on the backtest by grid over {0.2, 0.4, 0.5, 0.6, 0.8}; the differences between neighbouring weights are under 0.2 points of MAPE, so the exact values are not precious and nobody should spend a week on them again.

The blend is applied on the per-km price in log space, then multiplied by distance, then floored (1.05 EUR/km under 150 km, 180 EUR absolute), then surcharges are added. The floor is applied after the blend on purpose: on very short lanes the model predicts a per-km price that is realistic for a 300 km trip and absurd for 40 km.

## Explanation shown to shippers and carriers

Three factors, computed by the contribution of feature groups to the prediction relative to the cluster median: distance band and vehicle, lane balance, timing (day of week, hours to pickup). Rendered as a sentence: "Suggestion above the usual price for this lane: few carriers positioned near the origin, pickup on a Monday." The wording comes from `explanations.yaml`, 18 templates, translated into 5 languages. Carriers rate the explanation useful 71 % of the time (thumbs on the quote panel, 40 000 votes since March).

## Retrain history that matters

- 2026-01-18: first v2 in production, MAPE 10.6 %.
- 2026-02-08: added `imbalance_ratio` and the active-carriers feature, MAPE 10.0 %.
- 2026-03-15: training set filtered to `breakdown_consistent` bids, MAPE 9.9 %, and the p99 error halved (the inconsistent breakdowns were mostly outliers).
- 2026-04-12: automatic promotion refused, holdout MAPE 11.8 %: the warehouse had a half-empty last week after an ingestion delay. Manual promotion skipped, previous model kept.
- 2026-05-24: same refusal, same cause; the ML team's backtest now guards on rows per day.
- 2026-06-14: MAPE 9.8 %, current.

## What would make us retire it

- If the cluster median alone got within 0.5 points of the model on medium clusters, the model would only be worth keeping for low clusters, and a simpler nearest-neighbour cluster average might do. Checked at each monthly review; the gap is 2.3 points in June.
- If the pricing team stops being able to explain a quote to a shipper in one sentence. That was the reason v1 lasted so long ([[price-suggestion-model-v1]]).
