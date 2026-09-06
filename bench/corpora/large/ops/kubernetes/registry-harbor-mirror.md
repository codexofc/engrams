---
name: registry-harbor-mirror
description: The container registry on ops-tools (registry.hf.internal) hosts our images and proxies public registries with a 400 GB cache, RKE2 nodes are configured with registries.yaml mirrors so no node pulls from the internet directly, images are signed and scanned
type: reference
status: active
verified: 2026-02-06
---

# Registry and pull-through mirror

`registry.hf.internal` on `ops-tools`, backed by a 400 GB Longhorn volume (excluded from Velero, see [[backup-velero-schedule]]) and the object store for the image blobs of our own projects.

## Projects

- `hf/` : our images (`hf/halden-api`, `hf/halden-web`, `hf/live-gw`, `hf/flutter-ci`, `hf/php-ci`). Pushed by CI with a robot account per repository, scoped to push on its own project only. Retention: keep the last 30 tags per repository plus every tag referenced by a Git tag in `halden-infra` (a nightly job computes the list).

- `proxy-dockerhub/`, `proxy-ghcr/`, `proxy-quay/` : pull-through caches. A node pulling `docker.io/library/postgres:16` actually pulls `registry.hf.internal/proxy-dockerhub/library/postgres:16`, transparently, see below. Cache retention 30 days since last pull.

- `proxy-rke2/` for the RKE2 system images, so a cluster rebuild does not depend on the internet.

## Node configuration

`/etc/rancher/rke2/registries.yaml` on every node, rendered by the provisioning playbook:

```yaml
mirrors:
  docker.io:
    endpoint: ["https://registry.hf.internal/v2/proxy-dockerhub"]
  ghcr.io:
    endpoint: ["https://registry.hf.internal/v2/proxy-ghcr"]
  quay.io:
    endpoint: ["https://registry.hf.internal/v2/proxy-quay"]
configs:
  registry.hf.internal:
    tls: { ca_file: /etc/rancher/rke2/hf-internal-ca.crt }
```

No node has direct egress to public registries; the network firewall blocks it, and the mirror is the only path. This was tested by accident when the mirror was down for 20 minutes in January 2026: every new pod stuck in `ImagePullBackOff`. Running pods were unaffected. The mirror is now 2 replicas with the volume in `ReadWriteMany` through Longhorn's NFS mode, which is slower but survives one replica.

The certificate for `registry.hf.internal` comes from the internal CA, see [[cert-manager-and-letsencrypt]].

## Signing and scanning

- CI signs every `hf/*` image with a keyless signature (the CI OIDC identity) and the admission policy on `hf-main` verifies the signature for the `platform-prod` namespace. Unsigned image in prod = pod rejected. Staging only audits.

- Vulnerability scan on push, results in the registry UI. The policy blocks a deploy to prod only on `critical` with a fix available, which happens about once a month, usually a base image. The rebuild is a matter of bumping the `FROM` and letting CI run.

## Rate limits and quotas

Before the mirror (2024) we hit the anonymous pull limit of the public registry twice during cluster rebuilds. The mirror authenticates to the upstreams with a paid account for one of them, and caches everything, so a rebuild pulls from upstream once per image.

Storage quota per proxy project: 150 GB. When it is full, the oldest cached blobs go first. The `proxy-dockerhub` project sits at 120 GB.

## Monitoring

`registry_storage_used_bytes` on the storage dashboard, `probe_success` from the blackbox exporter on `/v2/` every minute (page on 5 minutes down, since a down mirror blocks every deploy and every node reboot), and the scan job's failure count.

## Related

Images referenced in `halden-infra` overlays always use the mirror's hostname for our own images (`registry.hf.internal/hf/halden-api:<sha>`) and the public name for third-party images (the mirror rewrite is at the node level). Mixing them was a source of confusion, so the rule is written in the Kustomize README; see [[ops-prefers-kustomize-over-helm]].
