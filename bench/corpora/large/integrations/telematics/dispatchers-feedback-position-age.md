---
name: dispatchers-feedback-position-age
description: Dispatchers trust the map since markers show position age (green under 2 min, amber to 15, grey beyond) and ETAs are rounded to 5 min with a band
type: feedback
status: active
verified: 2026-04-02
---

# What dispatchers told us about position freshness

Sessions in February and March 2026 with 11 dispatchers from 5 carriers and 2 shippers, after the pipeline rewrite ([[position-ingestion-pipeline]]) brought latency from 12 minutes to under a minute. We expected "the map is faster, great". We got something more specific.

## Findings

**1. Latency mattered less than knowing the latency.** With the old pipeline, dispatchers had learned that the map was "about 10 minutes behind" and mentally corrected. With the new one, most positions were fresh but some (Trakko poll gaps, tunnels, a phone in a drawer) were not, and nothing told them which. Two dispatchers said they trusted the old map *more* because its lag was predictable. What fixed it: the marker colour by age. Green under 2 minutes, amber up to 15, grey beyond, with the age in the tooltip ("il y a 4 min"). After the change, "where is my truck really" calls to drivers dropped, by the dispatchers' own count, from "several a day" to "when it goes grey".

**2. Exact ETAs that moved every minute were read as unreliable.** When the ETA said 14:37, then 14:34, then 14:39, dispatchers concluded "the system does not know" and called the driver. The ETA publication policy ([[eta-feed-publication]]) reduced webhook noise; on the screen we now round to 5 minutes and show a band ("14:30 to 15:00"), and the band width comes from the model's confidence. Dispatchers described the band as "honest". Nobody asked for the exact minute back.

**3. A stale ETA is worse than no ETA.** An ETA computed from a position 40 minutes old, shown with the same styling as a fresh one, is what caused the false-late panic in the Trakko clock incident ([[incident-2026-02-trakko-timestamp-drift]]). The `stale` status now greys the ETA and shows "last position 40 min ago" instead of a time.

**4. Arrival detection changed the conversation with shippers.** Dispatchers used to argue about arrival times with shipper contacts by phone. With geofence arrivals ([[geofence-arrival-detection]]) they now send the detected time and the shipper's own tracking page shows the same figure. Three dispatchers said this was the single most useful change of the year. One said it created a new argument ("your geofence is wrong for our site"), which the per-site radius handles.

**5. They do not want the raw track.** We offered a "show the last 2 hours of positions as a line" feature. Two tried it, none kept it on. What they want is: where is it now, how old is that, when will it arrive, did it arrive. The compliance position that we should not keep or show fine traces turned out to be aligned with what the users actually use.

## What we changed

- Marker colour by age and the age in the tooltip (HF-2128, February 2026).

- ETA rounded to 5 minutes with a confidence band; `stale` and `manual` states styled distinctly (HF-2131).

- Removed the "track line" experiment.

- Per-site geofence radius editable by the carrier's dispatcher, not only by support.

## What we did not change

- The Trakko poll interval (30 s). Dispatchers asked for "faster" for Trakko trucks; the poll interval is bounded by the provider's rate limit and 30 s is already fine when the age is visible. The real answer for those carriers is the app alongside the unit ([[position-source-priority]]).

- We did not add sound or push alerts on "went grey". Dispatchers look at the map all day; a grey marker is enough, and the 30 minute no-position notification exists for when they are not looking.

## Quote

"Avant je regardais la carte et j'appelais le chauffeur pour vérifier. Maintenant je regarde la couleur et j'appelle seulement si c'est gris." (dispatcher, 40 trucks, March 2026).
