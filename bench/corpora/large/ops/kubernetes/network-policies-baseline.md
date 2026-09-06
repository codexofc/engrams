---
name: network-policies-baseline
description: Every namespace on hf-main has a default-deny ingress and egress CiliumNetworkPolicy, with explicit allows per workload (API to PostgreSQL, RabbitMQ, Redis, object store, DNS, and named external FQDNs), staging cannot reach prod services
type: reference
status: active
verified: 2026-03-20
---

# Network policies: the baseline

Enforced by Cilium. Every namespace created by the app-of-apps gets the baseline from `components/netpol/base/`, and each workload adds its own allows in its component.

## Default deny

```yaml
apiVersion: cilium.io/v2
kind: CiliumNetworkPolicy
metadata: { name: default-deny }
spec:
  endpointSelector: {}
  ingress: []
  egress:
    - toEndpoints:
        - matchLabels: { k8s:io.kubernetes.pod.namespace: kube-system, k8s-app: kube-dns }
      toPorts: [{ ports: [{ port: "53", protocol: UDP }, { port: "53", protocol: TCP }] }]
```

DNS is allowed by default because a policy without it fails in ways that look like everything else. Everything else is denied until a workload policy allows it.

## What the API is allowed to reach

From `components/api/base/netpol.yaml`, egress from pods `app=halden-api` and the workers:

- `hf-postgres-rw` and `-ro` (through the PgBouncer pods, which have their own policy to PostgreSQL on 5432)

- RabbitMQ on 5672, Redis on 6379 (both in the same namespace)

- The object store appliance: `toCIDR: 10.20.0.60/32` on 443

- External FQDNs via Cilium's `toFQDNs`: the e-mail provider API, the push providers (FCM and APNs hostnames), the DNS provider API used by nothing in the API (that one is cert-manager's, listed here as a reminder that it is not the API's), and the routing service (`routing-svc.data.svc` in cluster, no FQDN needed)

- Webhook deliveries: the outbox relay pod (`app=halden-api-outbox`) has `egress: toCIDR 0.0.0.0/0 except RFC1918 and the cluster CIDRs` on 443 and 80, because subscriber URLs are arbitrary. It is the only pod in the namespace with open egress, and it runs with `readOnlyRootFilesystem` and no service account token.

Ingress to the API pods: only from the ingress controller pods (label `app.kubernetes.io/name=ingress-nginx` in `ingress-nginx` namespace) on 9000 (php-fpm behind the nginx sidecar on 8080), and from the Prometheus agent on 9090 for metrics.

## Staging isolation

`platform-staging` policies use the same files with a Kustomize patch, and additionally a cluster-wide `CiliumClusterwideNetworkPolicy` denies any traffic from `platform-staging` to `platform-prod` and the reverse. A staging pod that tries to reach `hf-postgres-rw.platform-prod.svc` is dropped, and the drop is logged by Hubble with the policy name.

## Visibility

`hubble observe --verdict DROPPED -n platform-prod` is the first command when something "cannot connect". Hubble flow logs are exported to Loki with a 3 day retention (they are voluminous, see the observability project's log volume note). The dashboard "Policy drops by source pod" is what shows a missing allow after a new component is deployed.

## Rules for adding a policy

- One `CiliumNetworkPolicy` per workload, named after it, in its component directory. Not in the baseline.

- `toFQDNs` for external hosts, never `toCIDR` for anything outside our racks. Provider IPs change.

- A policy change is applied to staging first and left for a day. Drops show in Hubble before they show as incidents.

- No `endpointSelector: {}` with allows in a workload policy. That is the baseline's job.

Related: [[rke2-cis-hardening]] for the other admission and runtime constraints, [[secrets-external-secrets-vault]] for how the secrets agent is allowed out.
