---
name: case-marchetti-pod-wrong-load-yard
description: April 2026, a Marchetti driver attached one shipper's POD to another's load in the same yard; first cross-org document move, new prompt in 4.9
type: project
status: active
verified: 2026-06-04
---

# Case: Marchetti Autotrasporti, one yard, two shippers, one POD

Fictional Italian carrier, 50 trucks, Business, `driver:sync` sub-tag `pod`. Two tickets on 2026-04-08, from two different shippers, about two loads delivered by the same driver at the same logistics platform.

## What happened

A driver delivered two loads at a shared warehouse, pallets for shipper A and pallets for shipper B, at two docks. He photographed the signed delivery notes and uploaded both from the load screen he had open, which was load A. Load A got two PODs, load B got none. Shipper B asked where its POD was. Shipper A asked why it had a delivery note with another company's name on it.

## What we did

- L2 confirmed with `taken_at` on the photos and the two `deliver` events: photo 2 was taken at dock B four minutes after the `deliver` on load B.

- `hfctl document move <document_id> --to-load <load B> --ticket ... --apply`, the first use of the command across two organisations. It worked as designed: same document id, `document.removed` webhook to A's integration, `document.available` to B's.

- Shipper A's account manager was told, because a user at A had seen B's delivery note (customer name, quantities). Not a personal data breach, commercial information about a third party visible for a few hours to one dispatcher. A did not care; we wrote the rule anyway.

## What we learned

- The rule now in the POD playbook: a document moved between two organisations triggers a note to the account manager of the org that saw it, every time, even when nobody complains. It costs a message.

- The driver did nothing unusual. Two deliveries in one yard is the normal case for platform logistics. App 4.9 (HF-3128) asks "Which load is this document for?" when the driver has two or more loads delivered in the last hour within 500 m, before the upload.

- The mobile team wanted to detect it from the photo's GPS metadata. Rejected: the two docks were 80 m apart, GPS in a yard is worse than that, and the question to the driver is cheaper.

## Figures

Between HF-3105's `document move` (March 2026) and June: 17 moves, 5 across organisations. Since 4.9 the "which load" prompt has been shown 1 400 times and the driver picked a load other than the open one 9 % of the time, which is 126 misattributions avoided in two months. The volume surprised everyone and is in the [[case-lessons-recurring-themes-2026-h1]] list under "the normal case is what the UI must handle".
