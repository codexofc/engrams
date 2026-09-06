---
name: pricing-experiment-guidelines
description: How pricing experiments are run, randomise on the unit that receives the treatment, six weeks minimum, initial open bids only, pre-registered metrics in the ticket, and the two metrics that must always be paired
type: feedback
status: active
verified: 2026-04-22
---

Written after [[experiment-min-bid-floor]] and [[experiment-anchor-price-2026-03]], which taught the same lessons from both sides.

## Before starting

- Write the hypothesis, the unit of randomisation, the primary metric, the secondary metrics and the stop criteria in the ticket before the flag goes live. An experiment whose metrics are chosen after seeing the data is a story, not an experiment.
- Randomise on the unit that receives the treatment. If the change is in what a carrier sees, hash on `carrier_id`; if it is in how a load behaves, hash on `load_id`. The hash is stable (`murmur3(unit_id + experiment_name) % 100`), not per session, otherwise the same carrier sees both variants in a day and the analysis is meaningless.
- Six weeks minimum. Freight has a weekly cycle (Monday posting peak, Friday bidding lull) and a monthly one (fuel index on the 3rd, see [[fuel-surcharge-index]], month-end invoicing). Four weeks caught the fuel change on only one side once.
- Exclude an entity if something else is changing there (PL during the KSeF switch, for example). Say so in the ticket.

## During

- Look at the guardrail metrics weekly, not at the primary. Guardrails: fill rate (loads with at least one bid), award rate, dispute rate. Stop if any drops by more than 2 points against control for two consecutive weeks.
- Do not peek at the primary metric to decide when to stop. Decide the duration in advance.

## Analysis

- Initial open bids only. Re-bids after being outbid are reactions to other carriers, not to the treatment.
- Always pair a "quality" metric with a "volume" metric. Junk flags with bids per load. Awarded price with fill rate. Anything that only measures complaints can be improved by suppressing the thing people complain about.
- Segment by fleet size (1 to 3, 4 to 20, over 20 trucks) and by lane confidence ([[pricing-lane-clusters]]). Both experiments so far had effects that reversed across a segment.
- Report medians for prices and ratios; bid amounts are heavy-tailed and one 40 000 EUR exceptional convoy moves a mean.

## After

- Decision in the ticket: ship, reject, or ship with a condition. Remove the flag from the code within two weeks of the decision.
- Put the ongoing metric in the pricing dashboard with the threshold that would reopen the question, as done for the anchor (prices back above 1.03 of suggestion).
- Add the lessons here, not in a new note, unless they are specific enough to deserve one.
