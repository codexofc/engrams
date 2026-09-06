---
name: ops-prefers-kustomize-over-helm
description: The ops team keeps its own components as plain Kustomize bases and overlays, uses upstream Helm charts only through ArgoCD's Helm source with values in Git and never templates its own charts, so that every rendered manifest is diffable
type: user
status: active
verified: 2025-11-25
---

# Preference: Kustomize for ours, Helm only for upstream

Decided by the ops team in 2025 and written down because every new person asks.

- **Our components** (`halden-api`, `halden-web`, `live-gw`, PgBouncer, the outbox relay, the ingress snippets, the network policies) are plain YAML in `components/<name>/base/` with `overlays/<env>/` patches. No Helm chart of our own, ever. The reason is `kustomize build overlays/prod | kubectl diff -f -` shows exactly what will change, and a reviewer reads YAML, not templates.

- **Upstream software** (ingress-nginx, cert-manager, Longhorn, CloudNativePG, Cilium, the monitoring stack, ESO, Kyverno) is deployed from its official Helm chart through ArgoCD's Helm source, with a `values.yaml` per cluster in `clusters/<cluster>/<component>/`. The chart version is pinned in the Application. We do not fork charts. When a chart does not allow something, we add a Kustomize post-render patch (ArgoCD supports Helm plus Kustomize), which has been needed exactly three times.

- **No `helm install` from a laptop**, no `helm upgrade` by hand. If it is not in Git, it does not exist, and self-heal will remove it anyway (see [[argocd-app-of-apps]]).

- **Rendered manifests are committed** for our components: a CI job runs `kustomize build` for every overlay into `rendered/<cluster>/<namespace>/` and commits the result on the same MR. The diff of the rendered output is what reviewers look at. It doubles the size of some MRs and nobody has complained, because it caught a wrong patch target twice.

- **Kustomize features we allow**: `patches` with `target`, `configMapGenerator` with `disableNameSuffixHash: false` (the hash suffix triggers rollouts on config change, which we want), `images`, `replicas`, `namespace`. Features we avoid: `vars` and `replacements` (hard to read), remote bases over the network (everything is vendored in the repo).

- **Naming**: the overlay directory is the environment name, the Application is `<namespace>-<component>`, the rendered directory mirrors the cluster and namespace. A component that exists on both clusters has two Applications.

What the team explicitly does not want: a "platform chart" that templates all our components from one values file. It was tried in 2024, the values file reached 900 lines, and nobody could predict what a change would render. See [[registry-harbor-mirror]] for the image naming rule that came from the same period.
