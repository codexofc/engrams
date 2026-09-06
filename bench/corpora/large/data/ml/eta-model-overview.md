---
name: eta-model-overview
description: eta-v3 predicts pickup and delivery arrival as p10, p50, p90 with 47 features, 23 min MAE at 4 h, served at 6 ms, weekly retrain
type: reference
status: active
verified: 2026-06-17
---

## What it predicts

For an awarded load, the time of arrival at pickup and at delivery. Consumers: the dispatch tool (shows the driver's expected arrival to the shipper), the driver app (rest planning), the bid ranking (`eta_fit` component, uses the pickup ETA of a carrier's declared position), and the delay alerts (a p50 later than the window end plus 30 minutes triggers a notification to the shipper).

## Model

`eta-v3`, in production since 2026-01-20, replacing [[eta-model-v1-gbm]] (v2 was a short-lived variant, never fully rolled out). Gradient boosted trees with quantile loss, three models per target (p10, p50, p90), so six boosters. Target: `log(remaining_minutes)` from the prediction time to the arrival, which keeps the relative error stable between a 2-hour and a 30-hour trip.

Features (47) are listed in [[eta-features]]; the important ones are the routing engine's driving time, the driver's remaining driving time under the rest rules (from the tachograph-like state the driver app reports), the hour and day of departure, the border crossings on the route, and the carrier's historical lateness.

The prediction is re-issued every 10 minutes while the load is in progress, with the current position; the "static" prediction at award time is a separate call with no position (`context = 'at_award'`) and is what the bid ranking uses.

## Accuracy (May 2026, see [[eta-eval-2026-05]] for details)

- Delivery p50 MAE: 23 minutes at 4 hours from arrival, 41 minutes at award time (median 19 hours before).
- p10 to p90 coverage: 81 % (target 80 %).
- Pickup p50 MAE: 17 minutes at 2 hours before.

## Serving

`ml-infer` (Rust wrapper around the booster runtime) serves `POST /v1/eta` with the feature vector assembled by the feature store's online lookup ([[feature-store-design]]). Latency 6 ms p50, 21 ms p99 including the feature lookups; the routing time is not computed at inference, it is a feature already stored with the load. See [[inference-service-latency]].

## Retraining

Weekly, Monday 03:00, on the last 120 days of completed loads with at least 3 position reports, about 380 000 loads and 4.1 M prediction points (one per 10-minute tick, sampled 1 in 4). The [[backtesting-framework]] evaluates the candidate on the last 14 days before promotion; promotion is automatic if MAE and coverage are within tolerance, manual otherwise. The drift monitoring that led to the February incident is in [[drift-incident-eta-2026-02]].

## What it does not do

- No prediction for loads without a route (multi-stop loads where stops are not ordered): 2 % of loads, they get the routing time plus a fixed margin and a `low_confidence` flag.
- No traffic feed. Tried in 2025 with a commercial traffic API, gained 2 minutes of MAE for 1 800 EUR a month; dropped. The hour-of-day and day-of-week features capture recurring congestion, which is most of it for long-haul.

## Delay alerts, the rule in full

An in-progress load gets a delay alert to the shipper when `p50_delivery > delivery_window_end + 30 min` for two consecutive predictions (20 minutes apart), and the alert is cleared when `p50` comes back inside the window plus 15 minutes. The two-tick rule and the hysteresis come from the March 2026 review: single-tick alerts flip-flopped on drivers taking a 45-minute break, 18 % of alerts were cleared within 30 minutes, and shippers asked us to stop.

The message includes the p10 to p90 range rounded to 15 minutes ("expected between 16:15 and 18:00") rather than a single time, since shippers plan a dock slot and the range is what they need. 4 100 alerts in May 2026, 71 % confirmed late by the actual arrival, 22 % arrived inside the window after all, 7 % unknown (no position at destination).

## Pickup ETA and the bid ranking

The `eta_fit` component of the pricing team's bid ranking uses the pickup prediction at award time with the carrier's declared position and availability, not a live position. It is the least accurate use of the model (33 minutes MAE, see [[eta-eval-2026-05]]) and the ranking weight of 0.15 reflects it. The declared position is stale for 40 % of carriers (last updated more than 24 hours before); when it is, the feature vector uses the carrier's home base and the prediction carries `low_confidence`, which the ranking maps to `eta_fit = 0.5` rather than trusting a number.

## Serving contract with dispatch

- `POST /v1/eta` with `load_id`, `context` (`at_award`, `in_progress`), and optionally `position` and `driver_state`. Response: `p10`, `p50`, `p90` for pickup and delivery as UTC timestamps, `confidence` (`normal`, `low`), `model_version`, `features_age_s`.
- Dispatch stores every response in `load_eta_history` and displays the latest; the warehouse gets them through the prediction log topic.
- If `ml-infer` is down, dispatch shows the routing engine's time plus a margin of 20 % with a dashed style, and no delay alerts are sent (a false alert is worse than none). `ml-infer` availability in the first half of 2026: 99.94 %, the main incident being the connection pool episode in [[inference-service-latency]].

## Known biases

- Systematic optimism of about 6 minutes on loads picked up on Friday afternoon, because the training set's Friday loads are dominated by short domestic trips and the long ones that cross the Sunday ban are rarer. The `weekend_driving_ban_hours` feature ([[holiday-calendar-feature]]) corrects most of it; the residual is accepted.
- Pessimism of 4 minutes on repeat lanes for a carrier (high `carrier_lane_familiarity`), because the historical lateness feature includes their early days on the lane. A decay on the lateness window was tried and gained 1 minute; not shipped, too small for the added feature.
- No bias by country of the carrier after the new-carrier prior change of June 2026; before it, PL carriers were predicted 5 minutes too early on their first loads.
