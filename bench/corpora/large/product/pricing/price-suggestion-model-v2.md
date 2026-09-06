---
name: price-suggestion-model-v2
description: The v2 price suggestion is a gradient boosted model on 31 features served by pricing-svc, MAPE 9.8 % on awarded price against 13.1 % for v1, retrained weekly on Sunday night, with a per-cluster fallback to the median
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
