---
name: cost-anomaly-alerts
description: Daily anomaly checks on the usage behind every variable cost (SMS, e-mail, KYC, egress per host, VM hours, bucket growth), warn to owners, page only on SMS
type: reference
status: active
verified: 2026-05-12
---

# Cost anomaly alerts

## Principle

The invoice arrives monthly; the usage that produces it is visible daily. Every variable line of [[infra-cost-overview-2026]] has a usage metric we can read ourselves, and an anomaly rule on that metric, so that the person typing the invoice in ([[vendor-invoices-reconciliation]]) is rarely the first to know. The rules reuse the data quality team's anomaly runner (same seasonal band: same weekday over 8 weeks, median ± 3 MAD, floored at 10 %) against a small `finops.usage_daily` table in the warehouse fed by a nightly job.

## The rules

### Usage metrics and thresholds

| Metric | Source | Segment | Severity | Owner |
|---|---|---|---|---|
| SMS segments | `notification_deliveries` (channel sms, sent) | country | `warn`; `page` above 3× the band upper bound | notifications |
| e-mail sends | `notification_deliveries` (channel email) | event group | warn | notifications |
| Verifid verifications | `kyc_checks` rows created | country | warn | onboarding |
| CDN egress | Skyvale usage API, per host | host | warn | platform (maps for `tiles.`) |
| Skyvale VM hours | Skyvale usage API | VM group (`ci`, `staging`) | warn | ops |
| object store growth | appliance bucket sizes, day over day | bucket | warn above 2× the band | storage |
| CPU requested | `kube_pod_container_resource_requests` sum | `hf.cost/service` | warn above +15 % day over day | service owner |
| map and routing API calls | provider usage endpoint | call type | warn | maps |
| Courrix dedicated IP volume | Courrix API | none | warn | notifications |

The `page` on SMS is the only page: a 3× SMS day is 1 000 EUR by lunchtime and the January OTP loop on the notifications side is what it looks like. Everything else can be a morning conversation.

## What the rules have found

- 2025-11 (before the runner, found by hand): the tiles egress, 79 % of the CDN bill ([[egress-finding-map-tiles-2025-11]]). The per-host egress rule exists because of it.

- 2026-01-20: SMS segments for Poland at 8× the band at 07:00, `page`. The OTP retry loop. The notifications team was already on it from their own queue alert; this fired 4 minutes later and would have been the first signal on a day when the queue alert was still `warn`.

- 2026-02-11: Skyvale VM hours for `ci` at 2.4×: a CI pipeline stuck in a retry loop on a flaky test, 30 runners for 14 hours overnight. 60 EUR, and a runner timeout that did not exist before.

- 2026-03-14: object store growth on `hf-warehouse-cold-prod` at 40× the band. The planned part rewrite, announced. Acknowledged in a minute; the rule has no way to know a change is planned, and we prefer acknowledging planned events to missing unplanned ones.

- 2026-05-05: Verifid verifications for Romania at 6×: the carrier signup fraud ring. Same event the data quality signup rule caught; this one costs money per event, so onboarding turned on the pre-KYC duplicate check the same day and the verifications the ring triggered stopped at 800 EUR.

- 2026-05-19: `api` CPU requested +22 %: a team scaled a Deployment from 12 to 15 replicas for a load test and left it. Back to 12 the next morning.

## What they do not do

- Predict the invoice. They say "today is not like the last eight of these weekdays". The reconciliation does the money.

- Cover the fixed lines. Colocation and amortisation do not move daily; they move when a purchase order is signed, which is a review topic ([[finops-monthly-review-feedback]]), not an alert.

- Cover Payla fees, which are finance's.

## Output

A daily message at 07:30 in the finops channel listing every metric outside its band with the observed value, the band, and the owner tagged; nothing when nothing is out of band (about two days in three). The `page` goes through the notifications team's pager. Findings and acknowledgements are appended to `finops/anomalies.md` by the owner in one line, which is what the monthly review reads first.

## Cost of the alerts

The nightly job reads seven sources and writes about 200 rows a day. Under a minute of warehouse time. The Skyvale and Courrix usage APIs are polled once a day; both are free on our plans.
