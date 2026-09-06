---
name: telematics-costs-per-provider
description: Monthly cost per vehicle in 2026: app 0.40 EUR, Trakko 6.10 EUR (resold at 9), Geolyx 1.60 EUR (free to the carrier); tracking is cost-neutral overall
type: reference
status: active
verified: 2026-06-04
---

# What tracking costs, per vehicle, per month

Figures for the first half of 2026, used for the Geolyx renewal ([[geolyx-contract-renewal-2026]]) and for the yearly pricing review. All amounts exclusive of VAT, per tracked vehicle, per month.

### Cost table

| Source | Provider fee | Hardware | Our infrastructure | Total cost | What the carrier pays us |
|---|---|---|---|---|---|
| Driver app | 0 | 0 | 0.40 | **0.40** | 0 (included) |
| Trakko unit | 4.50 | 1.60 | 0.40 (higher raw volume, but dedup at the door) | **6.10** (about 6.50 with support time) | 9.00 |
| Geolyx | 1.20 | 0 | 0.40 | **1.60** | 0 (included) |

Infrastructure: the telematics gateway (3 pods), its Kafka topic share, the `position_events` storage at 90 days, and the raw bucket at 7 days, divided by 2 900 vehicles: 0.40 EUR. The app's share is the same as the others because the cost is in the pipeline, not in the source.

## Trakko

- **4.50 EUR** per active unit per month, the subscription price after the 2025 renegotiation (was 5.20). Active means "sent at least one position this month", so a unit in a parked truck for a month costs nothing; 6 % of units are inactive in any given month.

- **Hardware**: 96 EUR per unit including installation by Trakko's partner, amortised over 5 years: 1.60 EUR. Failure rate so far 3 % per year, covered by warranty for 3 years (see the clock drift replacement in [[incident-2026-02-trakko-timestamp-drift]]).

- **Price to carrier**: 9.00 EUR per vehicle per month, 12 month commitment, unit remains ours. Margin about 2.50 EUR, which pays for support: Trakko-related tickets are 20 % of telematics support volume for 27 % of vehicles, mostly installation and mapping ([[tracker-vehicle-mapping]]).

- The retention amendment (compliance project) did not change the price; their first offer did (plus 15 %), and we said no, and they dropped it.

## Geolyx

- **1.20 EUR** per vehicle per month under the partner agreement, billed to us on vehicles that sent at least one position via our subscription. Their pricing to the carrier is separate and not our business.

- Free to the carrier because it costs us little and it removes the reason to install a Trakko unit in a truck that already has telematics. Also the argument for keeping the fee low in the renewal.

## The app

- Costs us the infrastructure share only. Costs the carrier a phone and a data plan they already have. The mobile team's battery work is what makes it acceptable to drivers.

- 62 % of vehicles, and rising 1 point per month, mostly small carriers onboarding.

## Comparison figures

- Total telematics cost in May 2026: about 6 900 EUR (2 900 vehicles). Revenue from Trakko resale: about 7 000 EUR. Tracking as a whole is cost-neutral, which is the agreed target: the value is in the dispatch and ETA product, not in selling boxes.

- Cost per kept position: 0.02 cents. Nobody optimises for this; it is here so that "let's store every position forever" can be answered with a number as well as with the compliance argument.

## Rejected

- A third provider proposing 0.80 EUR per vehicle with continuous tracking data resale as part of their model. The compliance conditions (see [[telematics-providers-overview]] and the compliance project's sub-processor note) excluded them regardless of price.

- Buying Trakko units outright without the subscription and hosting the data ourselves. Their units only talk to their platform.
