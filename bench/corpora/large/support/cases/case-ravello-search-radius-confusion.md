---
name: case-ravello-search-radius-confusion
description: June 2026 feedback from Ravello Beverages and three carriers: the 150 km radius measured from a default point nobody understood, UI changed
type: feedback
status: active
verified: 2026-08-06
---

# Feedback: Ravello Beverages and the radius nobody understood

Fictional Italian shipper (beverages), Business, plus three small carriers who wrote in the same fortnight. June 2026. Not a bug, a feedback note because everyone was right and the screen was wrong.

## What they said

Ravello: "We publish loads from our plant near Verona every day, we know four carriers who work that area, none of them sees our loads in the app. Your search is broken."

Carrier 1 (fictional, one truck): "I search from home, I get nothing within 150 km, then a colleague shows me the same load on his phone."

Carrier 2: "The radius is 150 km but my truck is parked 200 km from home this week."

Carrier 3: "I set the radius to 500 km, I get 900 results and the one I want is on page 12."

## What was actually happening

The carrier search takes a *search point* (a city the carrier types, or their current GPS position if they allow it) and a radius, default 150 km. The point defaults to the org's registered address, which for many small carriers is an accountant's office or the owner's home, not where the truck is. Carrier 1 had the accountant's address 170 km from Verona. Carrier 2 understood it correctly and was just asking for a per-week override. Carrier 3 widened the radius instead of moving the point and drowned.

Ravello's loads were indexed and findable; the [[case-lessons-recurring-themes-2026-h1]] note counts this case under "the customer describes a symptom in the words of the wrong system".

## What we changed

- The search screen shows the search point on the map with the radius circle, and a one-tap "Use my position". Before, the point was a text field with the city name and no map. HF-3220, shipped in app 4.10 in August.

- The default point is the last search point, not the registered address, once the carrier has searched once.

- Result sorting inside the radius favours distance from the point over recency when the radius is above 300 km, so carrier 3's load moves up (a relevance rule change on the search side, cross-referenced in their notes).

- Shippers get a "who can see this load" panel: number of carriers whose saved searches match the load. Ravello saw "38 carriers" the day it shipped and stopped worrying.

## What we learned

- Four people described four symptoms of one design choice. The triage found it because a support agent put the tickets next to each other; no single one would have been escalated.

- A default value that is right for a fleet of 100 (registered address is the depot) is wrong for a fleet of one.

- "Your search is broken" from a shipper means "carriers do not find me", which is a carrier-side question. Support now asks the shipper for the names of the carriers who should see the load, and checks their saved searches with the read-only impersonation.
