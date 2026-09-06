---
name: feature-store-design
description: Feature store: offline feat tables with point-in-time joins and an online key-value store, both generated from one registry
type: reference
status: active
verified: 2026-03-31
---

## Principle

One definition per feature, in `feature_registry.yaml`: name, entity (`load`, `carrier`, `driver`, `lane_cluster`, `shipper_site`), type, unit, the SQL that computes it, the freshness (`daily`, `10min`, `at_event`), the default value when missing, and the owner. Everything else is generated from the registry: the warehouse tables, the online store loaders, the feature vector assembly at inference, the documentation.

## Offline

Tables in the `feat` database of the warehouse, one per group and entity: `feat.eta_carrier_daily` (`carrier_id`, `date`, 7 columns), `feat.eta_load` (computed once at award, `load_id`), `feat.demand_lane_daily`, `feat.price_shipper_daily`, and so on. Built by marmot models generated from the registry's SQL, on the data team's schedule.

Training sets are built with point-in-time joins: for a training row at time `t`, the feature value is the one with `date < toDate(t)` for daily features, or `computed_at <= t` for event features. The join helper `build_training_set(spec)` in the `mlkit` package does it and refuses a spec without a time column, which is how the leakage of [[training-data-leakage-lesson]] became impossible to reintroduce by accident.

## Online

A key-value store keyed by `<entity>:<id>` with a JSON of the latest feature values, loaded by the same models after each run (`feat_sync`, which reads the last partition and writes the changed keys, about 200 000 writes a day). Latency 0.4 ms p50 per lookup; the ETA feature vector needs 4 lookups (load, carrier, driver, shipper site), done in parallel, see [[inference-service-latency]].

Online values are at most 10 minutes old for `10min` features (driver state) and up to 24 hours for `daily` ones, which matches what the models were trained on: a daily feature at inference is yesterday's value, exactly as in training. An online value fresher than what training saw would be a skew, not an improvement.

## Defaults

Every feature has a default in the registry, used when the key is missing (new carrier, new site). Defaults are not zeros: `carrier_lateness_p50_90d` defaults to the p50 of first loads of new carriers (22 minutes worse than average since the June 2026 change from [[eta-eval-2026-05]]). The default is also applied in training when the point-in-time join finds nothing, so the model has seen the default.

## Skew monitoring

Every inference logs the feature vector with the prediction (`raw.ml_predictions_*`, 180 days retention). Weekly, `mlkit skew` recomputes the offline features for a sample of 5 000 logged predictions and compares: a feature with more than 1 % of mismatches (beyond freshness) is a bug. It has found two: a unit mismatch (minutes offline, seconds online) on `shipper_site_avg_loading_minutes` in December 2025, and a null-versus-default difference on `carrier_lane_familiarity`.

## What the store is not

Not a place for raw data or for features of a single model. A feature enters the registry when a second model wants it or when it is used online; a one-off experiment computes what it needs in a notebook from the warehouse. Conventions for registering a model that uses features are in [[model-registry-conventions]].

## Registry entry, an example

```
- name: carrier_lateness_p50_90d
  entity: carrier
  group: eta
  type: float
  unit: minutes
  freshness: daily
  default: 22.0        # p50 of first loads of new carriers, from eta-eval-2026-05
  range: [-120, 600]
  owner: ml
  sql: |
    SELECT carrier_id, toDate(delivered_at) AS date,
           quantile(0.5)(dateDiff('minute', delivery_window_end, delivered_at)) AS value
    FROM core.loads_current
    WHERE delivered_at >= date - INTERVAL 90 DAY AND delivered_at < date
    GROUP BY carrier_id, date
  description: median delivery lateness in minutes over the previous 90 days, negative when early
```

`mlkit registry check` validates the file: every feature has a default inside its range, the SQL references only `core` or `feat` tables, the `sql` has the entity id and `date` (or `computed_at`) columns, and no two features share a name. The generated marmot model for the group is `feat.eta_carrier_daily`, one column per feature of the group and entity.

## Offline table sizes and refresh

| Table | Rows | Refresh | Time |
|---|---|---|---|
| `feat.eta_carrier_daily` | 26 M (71 k carriers × 365 days) | daily 03:00 | 4 min |
| `feat.eta_load` | 3.2 M | at award, by the 10-minute run | 20 s |
| `feat.eta_driver_10min` | 480 M (180 days) | every 10 min | 30 s per run |
| `feat.eta_site_daily` | 9 M | daily | 2 min |
| `feat.demand_lane_daily` | 1.6 M | daily | 1 min |
| `feat.price_shipper_daily` | 7 M | daily | 1 min |

The driver 10-minute table is the big one and is why `feat` has a 180-day retention while the others follow `core`'s 5 years.

## Online store keys and sizes

Keys are `<entity>:<id>` (`carrier:8812`, `load:99120033`, `site:FR-69007-0042`, `driver:d_4410`). One JSON per key with the latest value of every feature of that entity across groups, about 600 bytes per carrier, 300 per load. 71 k carrier keys, 40 k in-progress load keys, 60 k driver keys, 45 k site keys, 180 MB total. Loads are deleted from the online store 48 hours after delivery by `feat_sync --prune`.

## Backfilling a new feature

1. Add it to the registry with a default and a range.
2. `mlkit registry check`, merge.
3. `marmot backfill --model feat.<group>_<entity>_daily --from <start>` computes the history (a 365-day carrier feature: 4 minutes per month of partition, about 50 minutes). The data team's backfill runbook applies.
4. `feat_sync --full` pushes the latest values online.
5. Retrain the model that wants it; the training set builder picks it up from the registry, and `features.lock` in the model card records the registry hash.

Point 5 is what makes a feature change safe: a model trained before the feature existed keeps a `features.lock` that does not mention it, and `ml-infer` assembles that model's vector without it.
