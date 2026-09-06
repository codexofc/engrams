---
name: case-oyelaran-erasure-active-load
description: June 2026, a former Oyelaran Haulage driver asked for erasure while still assigned to an IN_TRANSIT load; blocker became a carrier notification
type: project
status: active
verified: 2026-08-03
---

# Case: Oyelaran Haulage, erasure request with a truck on the road

Fictional carrier registered in Ireland, 15 trucks, Starter. `other:gdpr`, ticket 2026-06-10 from the driver himself, in English.

## What happened

A driver left the company at the end of May. He asked us, by e-mail from the phone number on his account, to delete his data. Identity check passed by SMS code the same day. `hfctl gdpr check` reported a blocker: he was still the assigned driver of a load `IN_TRANSIT` since 2 June. The load was being driven by someone else; the carrier had never reassigned it in the back-office, and the new driver was reporting positions from the old driver's pool phone, still logged in as him.

So the "blocker" was a data quality problem at the carrier, and the person asking for erasure was still, in our records, driving a truck.

## What we did

- Support told the driver the request was accepted and would proceed once the carrier corrected its assignment, within the 30 days.

- Support wrote to the carrier admin: the load must be reassigned to the actual driver, and the pool phone must be logged out of the former driver's account. The carrier did it in two days, which also fixed the GPS attribution and the future POD's `uploaded_by`.

- Erasure ran on 2026-06-15. The positions recorded between 2 and 12 June had been attributed to the former driver; after reassignment they belong to the load, not to a driver, so nothing personal remained on them.

## What we learned

- An erasure blocker on an active load is also a signal that the carrier's records are wrong. The playbook step now says: when the blocker is an active load and the requester says they left, write to the carrier the same day, do not wait.

- Pool phones left logged in as a departed driver are a recurring pattern (the Kowalczyk and Sandoval cases too). HF-3190 adds a carrier back-office view "drivers with no activity for 14 days but with an active session", with a one-click revoke. Shipped July.

- The former driver could see, on the pool phone if he had kept it, the loads of his former employer. He did not have the phone; had he had it, this would have been a different case. The single-device rule limits it, the 14-day view closes it.

## Figures

Since HF-3190, carriers revoked 212 stale sessions in the first month. Four erasure requests in July 2026, none blocked. See [[case-lessons-recurring-themes-2026-h1]].
