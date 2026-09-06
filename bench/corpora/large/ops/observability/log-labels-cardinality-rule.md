---
name: log-labels-cardinality-rule
description: Loki stream labels are limited to namespace, app, container, level and source, everything else stays in the JSON line and is queried with the json parser, trace_id and route are never labels, enforced by the collection agent's pipeline
type: feedback
status: active
verified: 2026-03-23
---

# Règle : peu de labels Loki, tout le reste dans la ligne

## La règle

Un flux Loki est défini par ses labels, et Loki tient tant que le nombre de flux reste raisonnable (voir la limite `max_streams_per_user` dans [[loki-retention-and-volume]]). Les labels autorisés :

- `namespace`, `app`, `container` (posés par l'agent depuis les métadonnées Kubernetes)

- `level` (extrait de la ligne JSON, valeurs `debug`, `info`, `warn`, `error`, une valeur inconnue devient `unknown`)

- `source` (`app`, `ingress`, `audit`, `hubble`, `system`, `mobile`)

- `node` uniquement pour `source="system"`

Et c'est tout. `pod` n'est pas un label : 45 pods d'API qui redémarrent à chaque déploiement, c'est 45 nouveaux flux par déploiement, pour une information (`pod_name`) qui est dans la ligne. `route`, `trace_id`, `driver_id`, `org_id`, `status` : jamais.

Le pipeline de l'agent (`clusters/hf-main/logging/pipeline.yaml`) a une étape `labeldrop` finale avec une liste blanche : tout label qui n'est pas dans la liste est supprimé avant l'envoi. Ajouter un label demande donc une MR sur ce fichier, ce qui est le moment où on pose la question de la cardinalité.

## Comment interroger sans label

```
{namespace="platform-prod", app="halden-api"} | json | trace_id="0a1f3c9e8b7d4e2f"
{namespace="platform-prod", app="halden-api", level="error"} | json | route="api_loads_search" | duration_ms > 1000
```

Le parseur `json` extrait les champs à la volée. Sur 14 jours de logs API, une requête par `trace_id` prend 2 à 6 secondes, parce que Loki doit lire les chunks. C'est acceptable pour une investigation. Ce qui le rendrait rapide, un index sur `trace_id`, coûterait un flux par requête HTTP, ce qui n'est pas un compromis, c'est un suicide.

Pour les champs qu'on filtre souvent, Loki a les "structured metadata" (depuis la v3) : `trace_id` et `route` y sont mis par l'agent, ce qui permet `| trace_id="..."` sans le parseur `json` et un filtrage plus rapide sans créer de flux. C'est le bon endroit pour la cardinalité élevée.

## Comment l'appliquer

- Un nouveau champ dans un log = un champ JSON. Pas de discussion.

- Une nouvelle dimension sur laquelle on veut *agréger* souvent (un `sum by (x)`) : c'est une métrique, pas un log, voir [[log-volume-finding-mobile-sync]].

- Une dimension sur laquelle on veut *filtrer* vite dans les logs : structured metadata, pas label.

- Le format de ligne est du JSON à plat, une ligne par événement, clés en snake_case, `message` en anglais, toujours `level` et `trace_id` quand il y en a un. Le logger Monolog de l'API a un formatter qui l'impose, le front et le mobile envoient au collecteur d'événements qui reformate.

## Ce que ça a évité

Avant cette règle (2024), `pod` et `route` étaient des labels. 30 000 flux, des writers qui tombaient à chaque déploiement de l'API, et des requêtes qui timeoutaient parce que le sélecteur touchait des milliers de flux minuscules. Après : 18 000 flux (dont 12 000 pour Hubble, qui a `source` et `namespace` seulement), et aucun problème de flux depuis. La même leçon du côté des métriques est dans [[incident-2026-02-prometheus-oom]] : un label est un multiplicateur, dans les deux systèmes.
