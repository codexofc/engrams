---
name: ml-price-suggestion-training
description: How the ML team builds the training set and backtest for pricing's price suggestion model, filters, features and the handover
type: project
status: active
verified: 2026-05-21
---

## Who owns what

The price suggestion model is owned by the pricing team (it lives in `pricing-svc`, they decide the features and the blend with the cluster median). The ML team owns the training set construction, the feature store entries the model uses, the backtest, and the registration in the model registry ([[model-registry-conventions]], name `price-suggestion-v2`). This split exists because the training set is where leakage and data quality problems live, and that is our job.

## Training set

Built weekly by `build_training_set(price_suggestion_spec)` from the warehouse:

- Rows: awarded loads from the last 180 days, one row per load, target `log(awarded_amount_eur_cents / distance_km)` where the awarded amount excludes the surcharges stored with the bid.
- Filters: `breakdown_consistent = true` on the accepted bid (about 4 % of accepted bids are dropped here; the carrier edited the breakdown into something that does not add up), `distance_km >= 20`, currency conversion at the rate of the bid date, no internal accounts, no loads with `max_price_cents` (the shipper cap is excluded on purpose, the pricing team's rule).
- Features from the feature store ([[feature-store-design]]) at the time of posting: lane cluster medians, imbalance ratios, active carriers near origin, shipper award ratio, fuel index, calendar. Point-in-time as always; the shipper award ratio is computed on loads awarded before the posting time, which matters because the shipper's own current load would otherwise leak its outcome.

About 900 000 rows, 31 features, 6 minutes to build.

## Evaluation

Backtest ([[backtesting-framework]]) with a time split: train on 180 days, test on the following 14, rolling over 12 weeks. Metrics: MAPE on the awarded price (not on the log target; pricing thinks in euros), median absolute error in EUR, share of loads within 10 %, all by lane confidence segment. May 2026: MAPE 9.8 %, and the segment table is in the pricing team's model note.

The naive baseline is the cluster median per km times distance. The model must beat it on medium and low confidence clusters; on high confidence it roughly ties, which is why pricing blends rather than replaces.

## Handover

The trained booster goes to `models/price-suggestion-v2/<version>/` with its card and `features.lock`; `pricing-svc` loads it from the `current` pointer, and its own loader validates the model on 100 reference loads before use. Promotion is automatic within a MAPE tolerance of +0.5 points; it was refused twice in 2026 after warehouse ingestion delays left the last training week half empty, which the backtest caught as a sudden drop in row count per day (the `row_count_delta` guard in the spec).

## Open issue

The surcharge exclusion depends on the carrier's stored breakdown. The 4 % dropped rows are not random: they over-represent small fleets who type a round number. We measured the effect by training with and without a corrected version (breakdown recomputed from the quote) and the MAPE difference is 0.2 points, so it stays as is, documented in the model card's `known_limits`.
