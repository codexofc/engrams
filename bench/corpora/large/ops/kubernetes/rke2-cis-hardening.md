---
name: rke2-cis-hardening
description: hf-main runs with the RKE2 CIS profile enabled, Pod Security Admission restricted on platform namespaces (baseline on data), audit logging to Loki, no privileged pods outside ops, and the list of exemptions with their reason
type: project
status: active
verified: 2026-03-04
---

# Durcissement CIS de RKE2 (HF-INFRA-350)

Fait en janvier 2026 pour un questionnaire de sécurité client, et parce que ça aurait dû l'être avant.

## Ce qui est activé

- `profile: cis` dans la configuration RKE2 des serveurs et agents. Ça pose les paramètres kubelet et API server recommandés, exige un utilisateur `etcd` dédié, et applique les `sysctl` de protection (`vm.panic_on_oom=0`, `kernel.panic=10`, etc.). Le playbook de provisionnement crée l'utilisateur et applique le `60-rke2-cis.conf` avant l'installation.

- **Pod Security Admission** par namespace via labels :
  - `platform-prod`, `platform-staging`, `ops` : `enforce: restricted`, `warn: restricted`.
  - `data` : `enforce: baseline`, `warn: restricted` (le loader de l'entrepôt a besoin d'un `hostPath` en lecture, en cours de retrait).
  - `kube-system`, `longhorn-system`, `cilium` : `privileged`, avec la liste des DaemonSets qui l'exigent dans le README.

- **Audit log** de l'API server activé avec la politique `audit-policy.yaml` de RKE2 (niveau `Metadata` par défaut, `RequestResponse` pour les Secrets, les RBAC et les `exec`), envoyé à Loki par l'agent de logs avec 90 jours de rétention.

- `kube-apiserver-arg: anonymous-auth=false`, `authorization-mode=Node,RBAC`, `admission-control-config-file` avec `EventRateLimit`.

- `protect-kernel-defaults: true` sur kubelet, ce qui a fait échouer le démarrage sur les nœuds OCR jusqu'à ce que leurs `sysctl` soient alignés, voir [[ocr-workers-node-pool]].

## Ce que `restricted` a cassé

- Le sidecar nginx de l'API écoutait sur le port 80 en tant que root. Passé en port 8080, utilisateur 101, `runAsNonRoot: true`, `allowPrivilegeEscalation: false`, `capabilities: drop: [ALL]`, `seccompProfile: RuntimeDefault`. Les manifestes de tous nos composants portent ce bloc `securityContext` depuis une base Kustomize commune (`components/_common/securitycontext.yaml`).

- Les workers PHP écrivaient dans `/tmp` avec `readOnlyRootFilesystem: true` : un `emptyDir` monté sur `/tmp` et `/var/run`.

- `live-gw` en Node.js voulait `NET_BIND_SERVICE` pour le port 443 interne : passé en 8443.

- Les CronJobs de maintenance tournaient avec l'image de l'API et l'utilisateur root par défaut. Corrigé par le même bloc.

Rien n'a cassé côté fonctionnel après ces changements. Deux semaines en staging en `warn` avant le passage en `enforce` ont listé tout ce qui manquait.

## Exemptions documentées

| Objet | Namespace | Pourquoi |
|---|---|---|
| Longhorn `instance-manager`, `csi-*` | `longhorn-system` | accès aux périphériques bloc |
| Cilium agent | `kube-system` | eBPF, réseau hôte |
| Node exporter, log agent | `ops` | `hostPath` en lecture sur `/proc`, `/var/log` |
| NodeLocal DNSCache | `kube-system` | réseau hôte, voir [[cluster-dns-coredns-tuning]] |
| Device plugin GPU | `kube-system` | accès `/dev/nvidia*` |
| Loader entrepôt | `data` | `hostPath` en lecture, ticket de retrait HF-INFRA-360 |

Toute exemption est un `PodSecurity` label sur un namespace dédié ou une exception Kyverno nommée avec un ticket. Une exemption sans ticket est refusée en revue.

## Vérification

`kube-bench` avec le profil `rke2-cis-1.9` tourne chaque nuit comme Job sur un nœud de contrôle et un nœud agent, résultats dans Loki. Score actuel : 0 `FAIL`, 4 `WARN` (tous sur des contrôles manuels de rotation de certificats que RKE2 gère lui-même). Un `FAIL` nouveau alerte en `warn` le matin.

## Ce qui reste

- Le namespace `data` en `baseline` tant que le loader n'est pas réécrit.

- La rotation des certificats des nœuds est automatique dans RKE2 mais pas surveillée : une alerte sur `kubelet_certificate_manager_client_expiration_renew_errors` est à ajouter.

- Les politiques réseau ([[network-policies-baseline]]) sont l'autre moitié du durcissement et ont été faites avant.
