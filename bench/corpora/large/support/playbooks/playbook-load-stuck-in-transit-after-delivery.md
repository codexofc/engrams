---
name: playbook-load-stuck-in-transit-after-delivery
description: Playbook for a load still IN_TRANSIT after the goods were delivered: check the driver outbox, the deliver event in load_events, POD presence, and the two ways out (driver resync or carrier admin closing from the web)
type: reference
status: active
verified: 2026-06-30
---

# Load still `IN_TRANSIT`, goods already delivered

Category `load:stuck`. The shipper says the truck unloaded yesterday, the load is still `IN_TRANSIT`, and the invoice cannot be raised because `invoice` needs `DELIVERED`. Usually a sync problem, sometimes a driver who forgot.

## Checks

1. `hfctl load get <load_id>`: confirm `IN_TRANSIT`, note `driver_id`, `delivery_window_end`.

2. `hfctl load events <load_id>`: is there a `deliver` event? If yes but the status is `IN_TRANSIT`, stop and escalate to L2 immediately, that is a state inconsistency we have seen once (HF-3177) and it must not be touched by hand.

3. `hfctl driver sync-status <driver_id>`: `outbox_depth > 0` and a recent `last_seen_at` means the delivery is sitting in the phone. The app pushes when it gets network, and the delivery event carries the real timestamp from the phone, so the timeline will be right once it arrives. Send macro `driver-open-app-sync` to the carrier: open the app on network, wait for the sync icon.

4. `hfctl driver sync-status` shows `outbox_depth = 0` and a recent push: the driver never pressed "Delivered". Macro `driver-confirm-delivery` to the carrier. The driver can confirm late, the app asks for the actual delivery time when the confirmation is more than 2 h after the GPS shows the truck stopped at the delivery address.

5. No device or last seen more than 48 h ago: the driver phone is gone (broken, replaced, left the company). Go to the carrier admin path below.

## Carrier admin path

Since HF-3118 (March 2026) a carrier admin can confirm delivery from the web back-office on behalf of a driver, with a mandatory reason and a delivery time. It writes a `deliver` event with `actor_type = carrier_admin`. Macro `carrier-admin-confirm-delivery`. This exists precisely so that support never has to run the transition.

If the carrier admin cannot find the button: the carrier must be on the Business plan or above, or the load must be older than 72 h past `delivery_window_end` (the button appears for everyone then). Check `hfctl org get <carrier_org_id>`.

## POD

A delivery confirmed without a POD is valid. The shipper may refuse to pay until the POD arrives, which is a commercial matter. Check `hfctl load get` field `documents`, and if the POD is on the phone but not uploaded, see [[playbook-pod-missing-or-wrong-load]].

## What support does not do

We do not run `deliver`. We do not edit `load_events`. If neither the driver nor the carrier admin can act (carrier gone bankrupt, happened twice), escalate to L2 with the shipper's written statement of delivery and the ticket; the backend does it with an HF ticket and the audit log.

## Related

The opposite problem, a load marked `DELIVERED` by mistake, is [[playbook-undeliver-wrong-delivered]]. The stuck-before-pickup case is [[playbook-load-stuck-dispatched-no-pickup]].
