---
name: ingress-traefik-legacy
description: Historical Traefik 2 ingress that shipped with RKE2, replaced by ingress-nginx in September 2025 after the certificate expiry incident and the lack of a JSON access log with trace ids
type: reference
status: archived
superseded_by: [[ingress-nginx-config]]
verified: 2025-09-20
---

# Traefik ingress (until September 2025)

RKE2 ships Traefik 2 as its default ingress and that is what the cluster ran from its creation until the migration to [[ingress-nginx-config]] (ticket HF-1105, done 2025-09-14 in a 20 minute window with a DNS switch to the new VIP).

What pushed the change:

- Traefik's own ACME resolver stored certificates in a single `acme.json` on a Longhorn volume, and a Longhorn rebuild during a node replacement left the file locked. Renewals silently failed for 3 weeks, see [[incident-2025-10-ingress-cert-expired]] for the consequence. Moving certificate issuance to cert-manager was the fix, and at that point the ingress choice was open again.

- No JSON access log with the request's `traceparent`, which we needed to join ingress and application logs.

- The `IngressRoute` CRD versus plain `Ingress` split: half the routes were CRDs, half were Ingress objects, and every new person asked which to use.

- Sticky sessions for the live gateway required a middleware chain that nobody understood after the person who wrote it left.

Nothing was wrong with Traefik's performance. The reasons were operational.
