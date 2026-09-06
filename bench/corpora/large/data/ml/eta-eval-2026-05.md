---
name: eta-eval-2026-05
description: May 2026 ETA evaluation on 71 400 loads: 23 min MAE at 4 h, 81 % coverage, worst on Alpine crossings and new carriers, decisions
type: project
status: active
verified: 2026-06-05
---

Monthly evaluation of [[eta-model-overview]] on loads delivered between 2026-05-01 and 2026-05-31, computed by the [[backtesting-framework]] from the prediction log (`raw.ml_predictions_eta` in the warehouse) joined to the actual arrival (`core.loads.delivered_at`, driver-declared, cross-checked with the position trail within 500 m of the destination).

## Headline

| Horizon | Delivery p50 MAE | Pickup p50 MAE | p10 to p90 coverage |
|---|---|---|---|
| at award (median 19 h before) | 41 min | 33 min | 82 % |
| 12 h before | 34 min | 24 min | 81 % |
| 4 h before | 23 min | 17 min | 80 % |
| 1 h before | 11 min | 8 min | 79 % |

April was 24 / 42 / 81 %, so stable. The routing engine's driving time alone (no model) is at 67 minutes MAE at 4 hours, for reference.

## Segments

| Segment | Loads | MAE at 4 h | Note |
|---|---|---|---|
| domestic FR under 400 km | 18 200 | 16 min | |
| domestic PL | 12 900 | 19 min | |
| cross-border with 1 crossing | 24 100 | 25 min | |
| cross-border with 2 or more | 9 300 | 34 min | |
| Alpine crossing (Brenner, Mont-Blanc, Fréjus, Gotthard) | 2 100 | 58 min | queueing at the tunnels, not modelled |
| first load of a carrier (no history) | 1 900 | 49 min | carrier history features null, prior too optimistic |
| temperature-controlled | 6 800 | 21 min | slightly better, planned better |
| ADR | 2 300 | 27 min | |

## Errors that are not the model's

- 3.1 % of loads have a `delivered_at` more than 2 hours away from the last position near the destination (the driver declared late or early). These are excluded from the MAE above and counted separately; the model is evaluated against the position-based arrival when the two disagree by more than 30 minutes.
- 0.8 % have no position trail at all (app not running); excluded.

## Decisions

1. Alpine crossings: add a feature `alpine_queue_expected` from the hourly queue history of the four tunnels that we can derive from our own position trails (median stopped minutes within 5 km of the tunnel entrance by hour and weekday, 18 months of data). Ticket HF-2603, expected gain 15 to 20 minutes on that segment.
2. New carriers: the prior for the carrier history features moves from "average carrier" to "average new carrier" (p50 lateness of first loads is 22 minutes worse than average). Simple change in the feature store's default values, shipped 2026-06-03.
3. The coverage at 1 hour (79 %) is under target because the p10 is too late on short remaining trips; the quantile models will get a minimum spread rule. Not urgent.

## What was not changed

Retraining frequency stays weekly; a daily retrain was tested on the April data and changed the MAE by less than 0.5 minutes.
