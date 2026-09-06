---
name: quality-owner-preferences
description: How the engineer who runs data quality works: invariants first, rules owned by domains, every page rule has a story, pruning is a feature, not a gate
type: user
status: active
verified: 2026-07-07
---

Habits of the person on the data team who owns `dq-runner` and runs the monthly review, as applied in 2026. One person, with a named backup.

- **Invariants first.** A new table gets its primary key uniqueness and its conditional not-nulls before any volume rule. The plausibility checks come last and are `warn` at most ([[volume-anomaly-seasonal-thresholds]]).

- **Domains own their rules.** The data team owns the runner, the routing, the generated rules and the review; the dispatch, pricing, billing, ML and product owners write and receive their own rules ([[dq-severity-and-paging]]). A rule the domain does not understand is a rule the domain will silence.

- **Every `page` rule has a story or a reason.** Either an `origin: HF-xxxx` pointing at the incident it came from, or a one-line reason in the description that a new hire would accept. A `page` rule with neither is downgraded at the next review.

- **Pruning is a feature.** A rule that has never failed, has no origin and costs more than a second is deleted or made `info`. The May 2026 pass ([[dq-checks-runtime-cost]]) removed 60 rules and nobody missed them. Fewer rules that mean something beat more rules that decorate a dashboard.

- **No rule without a runbook**, for `page`. The lint enforces it. The runbook starts with the query on `sample_keys`.

- **The review is the product.** `dq-runner` is 2 800 lines and could be rewritten in a month. The monthly hour where each domain looks at its failures, its silences and its costs ([[dq-rules-review-feedback]]) is what makes the rules mean something, and it is protected on every calendar.

- **A miss is written up as carefully as a catch.** The two misses ([[missed-2025-11-duplicate-loads-after-replay]], [[missed-2026-04-sek-invoices-summed-as-eur]]) have longer notes than the catches, on purpose. The catches confirm the rules; the misses write the next ones.

- **Cost is watched weekly.** The runner's total warehouse time per day is a panel and a `warn` at 30 minutes. Quality that costs 10 % of the warehouse is quality nobody will fund.

- **English rule names and yaml keys, French or English everywhere else.** Descriptions in the language of the owner.

- **Not a gate.** `dq-runner` does not block a marmot model from publishing. A failed rule alerts a person who decides. Blocking on quality was tried in the first month and produced a Monday morning with no dashboards over a `warn`-grade rule; the decision belongs to a human with the runbook open.
