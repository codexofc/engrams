---
name: search-relevance-rules-v1
description: First carrier search ranking (March to July 2026): raw price per km and a 6-hour freshness decay, and the two problems that led to v2
type: reference
status: archived
superseded_by: [[search-relevance-rules-v2]]
verified: 2026-04-15
---

# Relevance rules, version 1 (March to July 2026)

The ranking that shipped with haystack in March 2026, replaced by [[search-relevance-rules-v2]] on 2026-07-08. Kept to read the A/B results and the feedback of the period.

## Score

`function_score`, `score_mode: sum`, `boost_mode: replace`, four components:

- **distance**: gauss decay from the search point, scale = radius / 2, weight 3.0

- **pickup date**: gauss decay on `pickup.window_start`, scale = 2 days, weight 2.0

- **price per km**: `price_per_km_eur_cents` normalised by the 95th percentile of the day across all loads, clipped to [0, 1], weight 2.0

- **freshness**: exp decay on `published_at`, scale = 6 h, weight 2.0

Same filter stage as v2 except the fit and the 300 km rule did not exist.

## What went wrong

**Price normalised globally.** A 900 km load at 1.60 EUR/km scored higher on price than a 120 km load at 1.30 EUR/km, always, everywhere. But 1.30 on a short regional lane is a good price and 1.60 on a long international one is average. Carriers doing regional work saw the top of their list filled with long loads they would never take. Measured in the June A/B: 14 % of top-3 results were loads over 500 km for carriers whose saved searches were all under 250 km.

**Freshness too aggressive.** Scale 6 h with weight 2.0 meant a load published 10 minutes ago outscored a better-fitting one published 4 hours ago by a margin larger than the distance component could compensate within 50 km. Carriers refreshing every few minutes saw the list reorder each time. The [[search-carrier-feedback-too-many-results]] note has the complaints.

**No notion of the carrier.** Two carriers with the same search point and radius got the same list, whatever their fleet history or saved searches. The fit component in v2 is the answer.

## What was fine

Distance and date components, kept unchanged. The filter stage, kept. The test cases format (`tests/ranking/cases/`), kept and extended from 31 to 63 cases.
