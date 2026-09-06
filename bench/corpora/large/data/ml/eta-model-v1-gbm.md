---
name: eta-model-v1-gbm
description: The first ETA model (2024 to January 2026), a point-estimate gradient boosted model on 21 features, 38 min MAE, replaced by eta-v3
type: reference
status: archived
superseded_by: [[eta-model-overview]]
verified: 2025-10-15
---

`eta-v1` ran from mid-2024 to 2026-01-20. One gradient boosted model, point estimate, target `arrival_minutes_from_now` in absolute minutes, 21 features (routing time, distance, hour, day of week, carrier lateness, vehicle type, country pair, a few position features).

Known figures at retirement: delivery MAE 38 minutes at 4 hours before arrival, 62 minutes at award. No interval, so the delay alert used p50 plus a fixed 45-minute margin, which fired too often on short trips and too rarely on long ones.

What v3 ([[eta-model-overview]]) changed and why:

- Quantile regression (p10, p50, p90) instead of a point estimate, so that the alert margin follows the uncertainty of the trip.
- Log of remaining minutes as target; the absolute target made long trips dominate the loss.
- The driver state features (`remaining_driving_minutes` and the rest, see [[eta-features]]) came with the driver app 4.5 in late 2025; v1 never had them and could not predict mandatory rests, which were the largest error source (a rest is 9 or 11 hours).
- Feature store with point-in-time joins; v1's training set had the small leakage on carrier lateness described in [[training-data-leakage-lesson]].

`eta-v2` existed for three weeks in December 2025 as v1 plus the driver features, with the absolute target; it was at 29 minutes MAE and was superseded by v3 before full rollout. Its files are in the registry under `keep` because it was serving during the last week of December, and the history log needs it.
