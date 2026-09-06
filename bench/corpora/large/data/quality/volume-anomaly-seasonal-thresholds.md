---
name: volume-anomaly-seasonal-thresholds
description: Anomaly rules compare a daily metric to a same-weekday band over 8 weeks (median ± 3 MAD) per segment, with holidays and an explicit no-history state
type: reference
status: active
verified: 2026-05-14
---

# Volume anomaly rules

## The problem they solve

Row counts compared to yesterday ([[legacy-row-count-checks]]) fail in both directions: a Monday is 40 % above a Sunday and looks like an anomaly, and a 1.3 % duplicate over a growing month looks like nothing. An `anomaly` rule ([[dq-framework-overview]]) asks a narrower question: is today's value of this metric, for this segment, where the same weekday has been recently?

## The band

For a metric `m` on day `d` and segment `s` (country, currency, carrier tier, whatever the rule declares):

1. Take the values of `m` for `s` on the same weekday over the previous 8 weeks (8 points). Drop days flagged in `dim.holidays` for the segment's country and, for pan-European metrics, days that are holidays in more than 3 of the top-5 countries.

2. Compute the median and the median absolute deviation (MAD) of the remaining points. Band is median ± 3 × 1.4826 × MAD, floored at ± 10 % of the median so that a very stable series does not produce a band of zero width.

3. If fewer than 4 points remain, the state is `no_history` and the rule emits a `warn` saying so once, then `info` until history exists. This state was added after the SEK episode ([[missed-2026-04-sek-invoices-summed-as-eur]]): a new currency with a non-zero amount is exactly the case where "not enough history" must be said out loud, not swallowed.

4. Today is compared to the band. Outside is `warn`. There is no `page` anomaly rule and there will not be: a volume shift is a reason to look, never a certainty that something is wrong, and paging on it would train people to ignore it.

Median and MAD rather than mean and standard deviation because one bad day in the 8 (an outage, a replay) would otherwise widen the band for two months. The tail note of the duplicate loads episode has the 1.3 % that motivated all this.

## The 12 rules (May 2026)

| Rule | Metric | Segment | Owner |
|---|---|---|---|
| `loads_published_daily` | count of loads published | pickup country | dispatch |
| `bids_daily_volume` | count of bids | pickup country | pricing |
| `bids_median_amount_daily` | median bid amount | lane cluster | pricing |
| `loads_assigned_daily` | count assigned | pickup country | dispatch |
| `deliveries_daily` | count delivered | delivery country | dispatch |
| `invoices_issued_daily` | count issued | currency | billing |
| `revenue_daily_by_currency` | sum of invoice amounts | currency | billing |
| `invoices_overdue_daily_by_country` | count newly overdue | shipper country | billing |
| `payouts_daily` | sum of payouts | currency | billing |
| `driver_positions_hourly` | count of positions per hour | none, 24 points a day | data |
| `eta_predictions_daily` | count of predictions | none | ml |
| `carrier_signups_daily` | count of new carriers | country | product |

`driver_positions_hourly` is the one hourly rule; its band is built from the same hour of the same weekday, and its purpose is to see the driver app breaking on one platform version (a 30 % drop at 07:00 on a Tuesday is a release, and it has been, twice).

## What they have caught

- 2026-02-20: `invoices_overdue_daily_by_country` +18 % on the total. This was the rule's predecessor, on the total not per country, and the alert was dismissed as month-end. After the March replay on the streaming side it was split per country; the per-country version, replayed on the February data, shows Poland at +210 % and Germany at +180 % on the 20th, and France flat. It would not have been dismissed.

- 2026-03-31: `bids_median_amount_daily` for lane cluster `ES-FR-north` at −35 %: a carrier's API client posting bids with a unit error (amount in tens of euros). Pricing contacted the carrier; the bids were withdrawn.

- 2026-05-05: `carrier_signups_daily` for Romania at +400 %: a fraud ring creating carrier accounts. Product and the KYC flow caught most at Verifid; the rule was 6 hours earlier than the KYC review queue noticing.

- 2026-05-19: `driver_positions_hourly` 06:00 to 07:00 at −40 %: the broker disk incident, positions buffered on phones, arrived later. Noise in that case, but a correct observation.

## What they do not do

- Forecast. A band from 8 same-weekdays is not a model of the business; a real trend (Sweden opening, a big shipper leaving) produces `warn` for a couple of weeks until the band moves. The owner acknowledges those with a `dq-runner silence --reason "Sweden ramp-up"` and a 14-day limit. Two of those have happened.

- Detect a shift inside the band. A 5 % persistent over-count sits in the band forever. That is what invariants ([[dq-rule-catalog-core]]) are for; the anomaly rules are the net under the invariants, not a replacement.

## Cost

Each rule is one query over 8 weeks of a `mart` table, 40 to 300 ms. All 12 run at 06:15 and are done by 06:16. They were the cheapest rules to keep in the May pruning ([[dq-checks-runtime-cost]]).
