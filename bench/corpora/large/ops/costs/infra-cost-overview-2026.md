---
name: infra-cost-overview-2026
description: Infrastructure and vendor run cost in H1 2026: 96 000 EUR a month all-in, colocation and amortisation 41 %, SaaS vendors 33 %, per-line table and trend
type: reference
status: active
verified: 2026-07-08
---

# What the platform costs to run, 2026

## Scope

Everything that appears on an invoice for running Halden Freight's systems: the datacentre, the hardware (as monthly amortisation over the 4-year cycle), software and SaaS vendors that the platform depends on, and the cloud provider we use around the edges (Skyvale: CDN, DNS, a status page, burst VMs for staging and CI). Not counted: salaries, Payla's payment processing fees (a percentage of transaction volume, a cost of revenue owned by finance, reconciled by billing), and office IT.

The predecessor of this table was a spreadsheet ([[legacy-cost-spreadsheet]]); the numbers below come from the monthly export of `finops/costs.yaml` plus the vendor invoices, reconciled as described in [[vendor-invoices-reconciliation]].

## Monthly cost, average January to June 2026

### By line

| Line | EUR / month | Share | Trend vs Q4 2025 |
|---|---|---|---|
| colocation: two racks, power, cooling, cross-connects, remote hands | 11 800 | 12 % | flat |
| hardware amortisation: 31 servers, 2 Stashbox appliances, 5 torrent nodes, network, 4-year straight line | 24 600 | 26 % | +1 200 (the two GPU nodes for OCR) |
| offsite copy contract | 1 900 | 2 % | flat |
| Skyvale: CDN and egress | 3 100 | 3 % | −4 200 ([[egress-finding-map-tiles-2025-11]]) |
| Skyvale: DNS, status page, burst VMs (staging, CI runners) | 2 400 | 3 % | −1 800 ([[staging-environment-cost-cut]]) |
| Skyvale: committed plan discount | −900 | | since January ([[reserved-capacity-decision-2026-01]]) |

### By line, continued: vendors

| Line | EUR / month | Share | Trend vs Q4 2025 |
|---|---|---|---|
| Courrix (e-mail) | 4 200 | 4 % | +600 (dedicated IP) |
| Bipline (SMS) | 11 200 | 12 % | −2 100 (push-first fallback) |
| Verifid (KYC) | 6 800 | 7 % | +900 (volume) |
| map and routing data provider | 5 600 | 6 % | flat |

### By line, end: licences, internal lines, total

| Line | EUR / month | Share | Trend vs Q4 2025 |
|---|---|---|---|
| software licences: monitoring, an APM, the ClickHouse support contract, the Stashbox support contract, misc | 9 400 | 10 % | flat |
| logging and traces storage (internal, marginal disk and object) | 700 | 1 % | −1 900 ([[logging-cost-reduction-2026]]) |
| torrent nodes power and amortisation (already in the lines above, shown for the chargeback) | (3 800) | | |
| warehouse (already in the lines above; the data team's own figure is 14 200 including their chargeback share) | (9 100) | | |
| everything else under 500 EUR (certificates, domains, a font licence, two small SaaS tools) | 1 400 | 1 % | |
| **total** | **96 000** | | **−7 300 vs Q4 2025 (103 300)** |

Lines in parentheses are inside other lines and shown so that the team-level numbers of [[per-service-cost-table-q2-2026]] can be traced back.

## What moves the number

- **Hardware cycle.** 24 600 a month is the 2024 to 2028 fleet. Adding a node is 550 EUR a month for 4 years; the rightsizing of Q4 2025 ([[rightsizing-2025-q4-requests-limits]]) avoided four of those. The two storage expansion drawers planned for September add 460 a month.

- **SMS.** The single most variable line. 11 200 in H1 against 13 300 in Q4 2025, entirely from the push-first fallback on driver assignments. Per-country routing decides the unit price.

- **KYC.** Verifid bills per verification; carrier signups drive it. The May fraud ring cost 800 EUR of verifications that were then refused.

- **Egress.** Was 7 300 in October 2025, is 3 100. The tiles finding.

## What does not move it

Traffic to the API. The bare-metal cluster has 35 % average CPU headroom and the marginal cost of a request is the electricity, which is inside the colocation line and has not changed with volume in two years. This is the main reason the numbers above are stable month to month while the business grew 30 %: we bought capacity in 2024 and are filling it.

## Per user, per load

96 000 EUR a month for about 380 000 active users and 210 000 loads a month: 0.25 EUR per user-month, 0.46 EUR per load. The per-load figure was 0.62 in Q4 2025. It is the number the finance side asks for, and it is mostly a volume effect on a fixed base.

## How the table is produced

`finops/costs.yaml` in `halden-infra` lists every line with its source (an invoice, a chargeback formula, an amortisation schedule), and `finops-report --month 2026-06` renders the table and the per-service split. Invoice amounts are typed in by hand on the first working day of the month, from the PDF, by the person on the [[costs-reviewer-preferences]] rota; the reconciliation against usage is the same person's job that morning. The anomaly detection ([[cost-anomaly-alerts]]) runs on the usage side daily so that the invoice is rarely the first news.

## Review

Monthly, one hour, the platform lead, the data lead and someone from finance ([[finops-monthly-review-feedback]] has what the review has learned). Quarterly, the same table goes to the management review with the trend and one paragraph per line that moved more than 10 %.
