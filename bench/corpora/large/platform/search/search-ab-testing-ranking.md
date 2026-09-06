---
name: search-ab-testing-ranking
description: Evaluating ranking changes: offline replay of yesterday's searches, then a two-week A/B by carrier org on bids per search; the June 2026 v2 experiment
type: project
status: active
verified: 2026-07-30
---

# Evaluating ranking changes

A ranking change cannot be judged by looking at a few searches. We use two tools, in order: an offline replay to reject bad ideas cheaply, then an online A/B to confirm the survivors.

## Offline replay

`ranking-replay --weights candidate.yaml --day 2026-06-10` reads the search log of the day (query, carrier profile, the 20 results returned, which result the carrier clicked and whether they bid) and re-scores the returned documents with the candidate weights. It reports:

- mean reciprocal rank of the clicked load, current versus candidate

- share of searches where the clicked load moves up, down, or stays

- share of searches where the bid load (when there was a bid) is in the top 5

It is a proxy with a known bias: it can only reorder what was shown; a load the current ranking hid on page 3 is not in the log. Good for rejecting a change that pushes clicked loads down, bad for measuring a change that surfaces new things. The June replay of v2 candidates showed the clicked load's MRR going from 0.41 to 0.46, enough to go online.

## Online A/B

- **Unit**: carrier organisation, hashed into arm A or B with a per-experiment salt. Per org, not per user, so two dispatchers in the same company see the same ranking and do not compare screens.

- **Duration**: two full weeks minimum, to cover two Monday imports and two Friday afternoons.

- **Primary metric**: bids per search session (a session is a carrier's searches within 30 minutes). It is what the marketplace needs; clicks are cheap.

- **Secondary**: bid acceptance rate (are the bids we induce any good to shippers), time to first bid on new loads.

- **Guardrails**: zero-result rate must not rise by more than 0.5 points; p99 latency must not rise by more than 20 ms ([[search-query-latency-slo]]); support tickets tagged `load:search` from arm B must not exceed arm A by more than 5.

- **Implementation**: the search API loads two weights files (`ranking/v2.yaml`, `ranking/experiment-<id>.yaml`), picks per org, logs the arm with every search. Assignment and results live in the warehouse (`marts.search_experiments`), computed daily.

## June 2026 experiment: v2 versus v1

2026-06-08 to 06-22, 3 100 carrier orgs per arm.

- Bids per session: A (v1) 0.71, B (v2) 0.78, +9.8 %, confidence interval excluding zero.

- Wide searches (radius over 300 km): +18 % in B; narrow: +4 %. The 300 km rule did most of the work.

- Bid acceptance rate: 23.1 % versus 23.4 %, no difference. The extra bids were as good as the others.

- Zero-result rate: unchanged (filters were identical). p99: +6 ms in B (the fit computation).

- Support tickets: 4 in A, 3 in B.

Shipped as [[search-relevance-rules-v2]] to everyone on 07-08. The list reshuffling complaints ([[search-carrier-feedback-too-many-results]]) dropped in the weeks after, which the experiment did not measure but the support triage did.

## Running one

1. Ticket with the hypothesis and the metric you expect to move.

2. Offline replay on three different days (a Monday, a Wednesday, a Friday). Reject if MRR drops.

3. Weights file in `ranking/`, PR reviewed by the other person in the pair, test cases updated if the expected order changes.

4. Two weeks online, daily look at guardrails only (no peeking at the primary metric, the dashboard hides it until day 14).

5. Decision in the ticket, weights merged into the versioned file or deleted.

One experiment at a time. Two would need orthogonal assignment and we do not have the volume.

## What we refuse to test

- Paid placement. Not a ranking question.

- Showing fewer results to induce urgency. Rejected in the design review; a marketplace that hides supply from its carriers is not one we want to run.

- Personalised prices. The price shown is the shipper's, full stop.

- Anything by user rather than by org (colleagues would compare screens and support would get "why does he see it and not me").
