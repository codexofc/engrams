---
name: webhook-endpoint-url-validation-ssrf
description: Subscription URL validation against SSRF: https only, private ranges refused, DNS resolved again at each delivery, no redirects, 4 KB body cap
type: reference
status: active
verified: 2026-04-14
---

# Webhook URL validation (anti-SSRF)

The relay is a process inside our network that makes HTTP requests to URLs chosen by customers. Without checks, a customer could point a subscription at `http://admin-api.hf.internal/...` or at the cloud metadata address and read the response through the "response body" shown in the delivery viewer. These are the rules, implemented in `WebhookUrlPolicy` and applied at creation, at `PATCH`, and at every delivery attempt.

## At creation and patch

- Scheme must be `https`. `http` is refused with `url_not_https`. No exception for staging; staging endpoints get a certificate like everyone else, and the staging environment uses a sink URL when `staging_allowed` is false anyway.

- Host must be a DNS name, not an IP literal. `url_ip_literal`.

- Port must be 443 or 8443. Others are refused. Two integrators asked for custom ports in 2026; both had a reverse proxy available and used it.

- The name is resolved (A and AAAA). Every resolved address must be public: refused if any is in `10/8`, `172.16/12`, `192.168/16`, `127/8`, `169.254/16`, `100.64/10`, `::1`, `fc00::/7`, `fe80::/10`, or the cloud provider's metadata range. `url_private_address`. If resolution fails, we accept the URL with a warning field (`url_unreachable_hint`), because integrators create subscriptions before their DNS is live; the address check runs again at delivery.

- Path and query are free. Userinfo (`https://user:pass@host/`) is refused, `url_userinfo`; the relay would not send it anyway and it is a sign of a copy-paste from somewhere else.

- Our own domains (`*.halden.example`, `*.hf.internal`) are refused whatever they resolve to.

## At every delivery

The relay resolves the host again, checks the same ranges, and **connects to the resolved address** while sending the original host name for TLS and the `Host` header. It does not let the HTTP client resolve on its own. This closes DNS rebinding: a name that resolved to a public address at creation and to `10.0.0.5` an hour later is refused at that delivery with `last_error = url_private_address`, and the delivery is marked `DEAD` immediately (no retry, the URL is hostile or broken).

We added the second resolution after a security review in February 2026 (HF-3096) demonstrated the rebinding case on staging with a DNS name under the reviewer's control. Before that, only the creation check existed.

## Redirects

Not followed. A 3xx counts as a failed attempt with `http_3xx` and goes through the normal retries ([[webhook-retry-schedule-v2]]). Following a redirect would send a signed body to a URL we never validated.

## Response handling

The response body is stored for the delivery viewer, truncated to 4 KB, and only when the status is not 2xx. A 2xx body is discarded. This limits what a hostile URL could exfiltrate through us to 4 KB of an error page from a host we already refuse to connect to.

## Egress

The relay pods run with a network policy that only allows egress to the internet through the NAT gateways (the two addresses in [[webhook-egress-static-ips]]) and to the database. Even if a check were bypassed, the pod cannot reach the cluster's internal services. Belt and braces.

## What customers see

The web screen validates the URL with the same policy on blur and shows the error in words. Support sees the code in `hfctl webhooks subs`. The most frequent refusal is `url_not_https` (people paste their local development URL), then `url_private_address` from integrators who test from an office network and give us the office reverse proxy's internal name.
