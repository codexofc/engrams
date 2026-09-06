---
name: ml-team-preferences
description: ML team preferences: gradient boosted by default, no per-driver features, naive baseline everywhere, cards before promotion
type: user
status: active
verified: 2026-03-02
---

Preferences of the ML team (three people), as applied in 2026.

- **Gradient boosted trees by default.** Every production model is one. A different family needs a backtest showing a gain that matters to the consumer, not to the metric alone. Neural approaches were tried on ETA in 2025 and were 1 minute better at 20 times the serving cost.

- **No per-driver features.** Carrier-level yes, driver-level no, even though `driver_id` would improve ETA on repeat drivers. Reason: a driver's lateness score would end up in a carrier's HR decisions, and the works council question in Germany was enough to settle it. The driver state features (remaining driving time) are about the trip, not the person, and are never stored per driver beyond the trip. See [[eta-features]].

- **A naive baseline in every evaluation**, and the model must beat it. From [[training-data-leakage-lesson]].

- **Every served model has a card** ([[model-registry-conventions]]); the card is written before the model is promoted, not after.

- **Notebooks are for finding out, never for producing.** A feature computed in a notebook enters the feature registry with its SQL before any model uses it in production.

- **Weekly retrains, not daily**, unless the backtest shows a gain above the promotion tolerance; a retrain that changes nothing is noise in the history log.

- **The consuming team pairs on the evaluation.** Pricing sits with us on the price suggestion backtest, dispatch on the ETA segments. A metric the consumer does not understand is not the primary metric.

- **Plain services we can read end to end.** `ml-infer` is 3 000 lines of Rust; a managed model server would hide the part that breaks ([[inference-service-latency]]).

- **French or English**, whoever writes; feature names, column names and model names in English.

- **Meetings**: model review on the first Monday of the month (one hour, every served model's current metrics against last month), and an incident review within a week of any drift alert that was real.

