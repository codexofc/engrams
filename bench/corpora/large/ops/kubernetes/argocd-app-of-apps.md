---
name: argocd-app-of-apps
description: ArgoCD on ops-tools manages both clusters through an app-of-apps in halden-infra, one Application per component and environment, Kustomize overlays, auto-sync with self-heal for staging and ops, manual sync for platform-prod API
type: reference
status: active
verified: 2026-04-27
---

# ArgoCD : app-of-apps

ArgoCD tourne sur `ops-tools` et gère les deux clusters (`hf-main` est enregistré comme cluster distant). Tout ce qui est déployé vient du dépôt `halden-infra`, branche `main`.

## Arborescence

```
halden-infra/
  apps/                      # l'app-of-apps : une Application par ligne
    hf-main/
      platform-prod-api.yaml
      platform-prod-web.yaml
      platform-staging-api.yaml
      ...
    ops-tools/
      argocd.yaml
      monitoring.yaml
  components/<name>/base/    # Kustomize base par composant
  components/<name>/overlays/<env>/
  clusters/<cluster>/        # cilium, longhorn, cert-manager, ... (voir la note de préférence Kustomize)
```

L'Application racine `root` pointe sur `apps/` et crée les autres. Ajouter un composant = ajouter un fichier dans `apps/`, et la MR le montre.

## Politiques de synchronisation

| Cible | Auto-sync | Self-heal | Prune |
|---|---|---|---|
| `platform-staging-*` | oui | oui | oui |
| `platform-prod-web`, `platform-prod-live-gw` | oui | oui | oui |
| `platform-prod-api` (API, workers, migrations) | **non** | oui | non |
| `ops` et `kube-system` sur les deux clusters | oui | oui | non |

`platform-prod-api` en synchronisation manuelle : la personne qui promeut vérifie le diff dans l'interface (surtout le Job de migration, hook `PreSync`) et clique. Le `prune` est désactivé sur la prod API parce qu'une ressource supprimée du dépôt par erreur ne doit pas disparaître de la prod sans qu'on la voie dans le diff.

Le self-heal partout signifie qu'un `kubectl edit` est annulé en 3 minutes. Pour une intervention d'urgence, on désactive le self-heal sur l'Application concernée depuis l'interface, on intervient, on met le dépôt à jour, on réactive. Ne pas oublier la dernière étape, voir [[incident-2026-06-argocd-sync-loop]].

## Images

Le tag d'image de chaque composant est dans l'overlay (`images:` de Kustomize). Le pipeline de release de chaque dépôt applicatif ouvre une MR sur `halden-infra` qui change cette ligne (`kustomize edit set image`), fusionnée automatiquement pour staging, par un humain pour la prod. On n'utilise pas l'image updater d'ArgoCD : on veut le tag dans Git, et un diff lisible.

## Vagues et hooks

- `PreSync` : Job de migration de l'API, avec `hook-delete-policy: BeforeHookCreation` pour qu'un ancien Job échoué ne bloque pas le suivant.

- `sync-wave` : `-1` pour les Secrets et ConfigMaps externes, `0` pour les Deployments, `1` pour les Ingress, `2` pour les CronJobs. Un Ingress qui arrive avant son Service produit des 503 pendant quelques secondes, d'où l'ordre.

- `PostSync` : un Job de vérification qui appelle `/health` de la nouvelle version et échoue la synchronisation si ce n'est pas 200 en 2 minutes. ArgoCD marque alors la sync en échec, les anciens pods sont encore là (le Deployment n'a pas fini son rollout), et on décide.

## Accès

SSO via le fournisseur d'identité, groupes : `ops-admin` (tout), `platform-dev` (lecture partout, sync sur staging), `platform-release` (sync sur `platform-prod-*`). Les tokens de projet pour les pipelines ont un rôle limité à `get` et `sync` sur leurs Applications.

## Ce qu'on surveille

`argocd_app_info{sync_status!="Synced"}` pendant plus de 30 minutes alerte en `warn`, sauf pour `platform-prod-api` qui est souvent `OutOfSync` entre la fusion de la MR d'image et la promotion. `argocd_app_sync_total{phase="Failed"}` alerte en `page` pour la prod.

Voir aussi [[ops-prefers-kustomize-over-helm]] pour la raison de l'arborescence.
