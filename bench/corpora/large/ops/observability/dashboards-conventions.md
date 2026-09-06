---
name: dashboards-conventions
description: Grafana dashboards are provisioned from JSON in Git under dashboards/<team>/, each has a home row with the four golden signals, uses recording rules, a fixed uid, and the templating variables env and namespace, no ad hoc dashboards survive a week
type: reference
status: active
verified: 2026-03-17
---

# Conventions des dashboards Grafana

## Provisionnement

Les dashboards sont des fichiers JSON dans `halden-infra/dashboards/<team>/<name>.json`, chargés par le sidecar de provisionnement de Grafana depuis un ConfigMap généré par Kustomize. Un dashboard modifié dans l'interface n'est pas sauvegardé (l'interface est en lecture seule sur les dashboards provisionnés) : on exporte le JSON, on ouvre une MR. Ça freine les modifications à chaud, et c'est voulu.

Les dashboards créés à la main dans le dossier `Scratch` sont supprimés chaque lundi par un script. Ce qui mérite de rester passe par Git.

## Structure obligatoire

- **`uid` fixe** et lisible (`hf-api-overview`, `hf-postgres`, `hf-mobile-sync`), jamais l'uid généré, pour que les liens dans les runbooks et les alertes ne cassent pas.

- **Variables** `env` (`platform-prod`, `platform-staging`) et, si pertinent, `route`, `node`, `pool`. Les variables sont alimentées par `label_values()` sur une règle d'enregistrement, pas sur une série brute.

- **Première rangée "Home"** avec les quatre signaux de la chose observée : trafic, erreurs, latence, saturation. Sur l'API : requêtes par seconde par route, ratio d'erreurs, p50/p95/p99, workers php-fpm actifs. Sur PostgreSQL : transactions par seconde, erreurs et rollbacks, latence des requêtes, connexions et lag. Cette rangée utilise les règles d'enregistrement de [[prometheus-stack-layout]] pour être rapide.

- **Rangées suivantes** repliées par défaut, par sous-système.

- **Liens** en haut vers le runbook du composant et vers les dashboards voisins (API → PostgreSQL → PgBouncer).

- **Annotations** : les déploiements (depuis les événements ArgoCD via une source de données Loki sur les logs du contrôleur) et les incidents (annotation manuelle avec le lien vers la note). Les déploiements en annotation ont expliqué un bon tiers des pics de latence en un coup d'œil.

## Règles de présentation

- Unités toujours définies (`s`, `reqps`, `bytes`, `percentunit`). Un panneau sans unité est renvoyé en revue.

- Pas de `stat` géant en couleur pour une valeur qui n'a pas de seuil défini. Le vert et le rouge veulent dire quelque chose ou ne sont pas là.

- Le seuil d'alerte est dessiné sur le graphe (ligne de seuil) pour les panneaux qui correspondent à une règle `page`, avec le nom de la règle dans la description.

- Maximum 12 panneaux visibles à l'ouverture. Au-delà, une rangée repliée.

- Les requêtes `topk(10, ...)` pour tout ce qui est "par pod" ou "par route", jamais une série par pod sur 45 pods.

- Intervalle minimum `30s` sur les panneaux qui interrogent des règles à 5 minutes, sinon les courbes en escalier trompent.

## Dashboards de référence

| uid | Contenu | Propriétaire |
|---|---|---|
| `hf-api-overview` | l'API, par route | platform |
| `hf-postgres` | PostgreSQL et PgBouncer, seq scans par table depuis l'incident de février | platform + ops |
| `hf-mobile-sync` | synchronisation, pushs, crash-free | mobile |
| `hf-ingress` | par hôte et par ingress, WAF | ops |
| `hf-cluster-capacity` | requests vs allocatable, par pool | ops |
| `hf-etcd` | taille vs quota en premier panneau | ops |
| `hf-slo` | budgets d'erreur, voir [[slo-api-latency]] | platform |
| `hf-logs-volume` | ingestion Loki par flux, voir [[log-volume-finding-mobile-sync]] | ops |

## Revue

Une fois par trimestre, chaque équipe ouvre ses dashboards pendant 30 minutes et supprime les panneaux que personne n'a regardés (Grafana a les statistiques de consultation). En mars 2026 : 18 panneaux supprimés, 2 dashboards fusionnés.

## Le provisionnement et ses pièges

Le sidecar de Grafana surveille les ConfigMaps portant le label `grafana_dashboard: "1"` dans le namespace `monitoring` et écrit leur contenu dans `/var/lib/grafana/dashboards/<dossier>/`. Le dossier Grafana vient de l'annotation `grafana_folder` sur le ConfigMap, et Kustomize la pose à partir du nom du répertoire dans Git (`dashboards/platform/` → dossier "Platform"). Ce qui a mordu :

- Un ConfigMap est limité à 1 Mo. Le dashboard `hf-postgres` a atteint 600 Ko avec ses 80 panneaux et a été découpé en `hf-postgres` et `hf-postgres-tables`. La règle des 12 panneaux visibles ne limite pas la taille du JSON, les rangées repliées comptent.

- Deux fichiers avec le même `uid` : le second écrase le premier sans erreur dans les logs du sidecar, et le dashboard qui disparaît est celui dont le fichier est lu en premier, c'est-à-dire aléatoire. Le lint CI (`scripts/lint-dashboards.sh`) vérifie l'unicité des `uid` et des `title` dans un même dossier.

- L'`uid` de la source de données dans le JSON exporté est celui de l'instance où l'export a été fait. Si quelqu'un exporte depuis son Grafana local, l'uid ne correspond à rien en prod et les panneaux affichent "datasource not found". Le lint remplace tout `datasource.uid` par la variable `${DS_PROMETHEUS}` ou `${DS_LOKI}` selon le type, et Grafana les résout au chargement.

- La version du modèle JSON (`schemaVersion`) avance avec Grafana. Un dashboard exporté d'une version plus récente que celle déployée est chargé mais certains panneaux se cassent silencieusement. Le lint refuse un `schemaVersion` supérieur à celui de la version de Grafana déployée, lue dans le `values.yaml` du chart.

- Le rechargement : le sidecar détecte le changement de ConfigMap en 10 à 60 secondes (le délai de propagation des volumes ConfigMap), et Grafana relit le fichier à la prochaine ouverture. Une MR fusionnée est visible en prod en 3 à 5 minutes avec la synchronisation ArgoCD, et les gens qui rechargent la page 30 secondes après la fusion pensent que ça ne marche pas.

## Ce qu'un nouveau dashboard doit contenir avant la revue

Le modèle `dashboards/_template.json` : les variables `env` et `namespace`, la rangée "Home" vide avec ses quatre emplacements nommés, les liens vers le runbook et les voisins, l'annotation des déploiements. Copier le modèle prend 2 minutes et évite les trois allers-retours de revue habituels (pas d'unité, pas de variable `env`, pas de lien).
