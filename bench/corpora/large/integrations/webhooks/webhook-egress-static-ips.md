---
name: webhook-egress-static-ips
description: Outbound webhooks leave through two fixed NAT addresses published by GET /v2/webhooks/egress-ips, pinned in the cluster, changed with 30 days notice
type: reference
status: active
verified: 2026-03-26
---

# Egress addresses for outbound webhooks

Customers allowlist us on their firewalls. For that to work, every webhook request must come from a small, stable set of public addresses, and any change must be announced well ahead.

## The addresses

Two IPv4 addresses, one per NAT gateway in the two availability zones the relay runs in. They are published:

- on the docs site, integrator guide, section "Securing your endpoint";

- by the API, `GET /v2/webhooks/egress-ips`, unauthenticated, returns `{ "ipv4": [...], "ipv6": [], "valid_from": "...", "next_change": null }`. Integrators can poll it from their deployment scripts; two do.

No IPv6 yet. The relay pods egress through IPv4 NAT only; when we add IPv6 egress the list will carry both, with a 60-day notice.

The addresses themselves are not in this note on purpose; the API is the source of truth and a copy here would go stale.

## How they are pinned

The relay Deployment has a node selector and a network policy that force egress through the two NAT gateways (one per zone). The gateways have Elastic-style reserved addresses that survive gateway replacement. Nothing else uses these gateways: the API pods, the workers and the indexers egress through other NATs, so the webhook addresses are "webhooks only" and a customer allowlisting them is not implicitly allowing anything else we run.

A check in CI (`infra/tests/egress_ips_test.sh`) asserts that the addresses in the docs, in the API's config and in the Terraform state are identical. It failed once, in the right direction (someone updated Terraform for a gateway rebuild and forgot the docs), and blocked the merge.

## Changing them

We changed once, 2026-01-20, when the cluster moved region. The process, now written down (HF-3070):

1. New addresses reserved and added to the API's list with `valid_from` 30 days ahead; `next_change` set.

2. E-mail to all admins of organisations with an active subscription, 30 days and 7 days before: "add these, keep the old ones until <date>".

3. On the day, relay egress switched; old addresses kept reserved but unused for 30 more days, then released.

4. Deliveries failing with `connect_timeout` from hosts that succeeded before the switch are flagged in the metrics dashboard ([[webhook-delivery-metrics-and-slo]]) for two weeks; support contacts those customers. In January, 9 customers had not updated their firewall; all fixed within three days of the switch.

## What we do not do

- **Publish a range.** A /24 would be easier to allowlist but we do not own one exclusively, and "allow the whole cloud provider range" is what some customers would do, which defeats the purpose.

- **Proxy through a third-party static-IP service.** Considered for simplicity; rejected because a third party would see every payload, and the [[webhook-mtls-request-declined]] discussion showed customers care about who touches the bytes.

- **Per-customer addresses.** One integrator asked to have "their own" source address. No: the addresses identify us, the signature identifies the subscription.

## Support

`hfctl webhooks egress-ips` prints the current list and `next_change`. When a customer says "your requests are blocked by our firewall", the answer is this list plus the reminder that a `connect_timeout` in `last_error` is what a firewall drop looks like from our side.
