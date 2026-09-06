---
name: case-bosque-fretzone-duplicate-loads
description: March 2026, Bosque Logistica bid on a load twice (direct and Fretzone copy), both accepted by two dispatchers; origin of the dedup rule HF-3210
type: project
status: active
verified: 2026-05-22
---

# Case: Bosque Logistica and the load that existed twice

Fictional Spanish carrier, 30 trucks, Starter. `load:duplicate`, ticket 2026-03-17, then a shipper ticket the next day.

## What happened

A shipper published a load on Halden and, out of habit, also on Fretzone. Our Fretzone sync pulled the Fretzone listing as a partner load (source `fretzone`, visibility `PARTNER_ONLY` in the other direction) and showed it in the carrier search next to the direct one. Same route, same dates, two cards. Bosque bid on both, slightly different prices, because their dispatcher assumed two shipments.

The shipper had two dispatchers. One accepted the direct bid in our web app. The other accepted the Fretzone bid on Fretzone, which came back to us as an acceptance on the partner load. Bosque saw two `DISPATCHED` loads and sent two trucks the next morning. One left empty.

## What we did

- Support confirmed the chain with `hfctl partner refs` on both loads and `hfctl load events`. Both acceptances were legitimate from the system's point of view.

- The shipper cancelled the partner-side load; the cancellation fee for a `DISPATCHED` cancel applied on our side for the direct one, nothing for the partner one (Fretzone's rules, not ours). Bosque asked for the empty run to be compensated; that was between them and the shipper, and the shipper paid it outside the platform.

- HF-3210: the sync must not import a partner listing that matches a direct load of the same shipper on `(org_id, reference, pickup_date)`, and when the reference is missing, on `(org_id, pickup postcode, delivery postcode, pickup_date)` with a warning to the shipper. The shipper's org on Fretzone is matched to ours through the partner account link they set up when enabling the integration. In progress as of this note, the dedup by reference is live since May, the fuzzy match is being tested.

- Until HF-3210 is complete, `hfctl partner unlink <load_id>` removes the pulled copy from our search (L2).

## What we learned

- A shipper who publishes in two places is the normal case, not an abuse. The sync must expect it.

- Two acceptances by two dispatchers of the same company is also normal. We considered a warning "your colleague accepted a bid on a similar load two minutes ago" and put it in the backlog (HF-3215).

- Starter carriers absorb these costs alone. Bosque did not churn, but they asked, reasonably, why we showed the same load twice. We had no good answer that day.

## Figures

Before the dedup by reference: about 90 partner-pulled loads per week matched a direct load of the same shipper. After: 11 per week, all without a reference on the partner side. Double acceptances found in the previous six months by a warehouse query: 7. This is the most expensive line in [[case-lessons-recurring-themes-2026-h1]].
