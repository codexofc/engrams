---
name: incident-2026-02-prometheus-oom
description: Feb 2026, the central Prometheus was OOM-killed 6 times in 2 hours after a mobile telemetry label (device_model plus os_build) pushed active series from 1.6 M to 3.9 M, 2 h of metrics gap, fixed with relabel drops and a series-count alert
type: project
status: active
verified: 2026-02-27
---

# Incident 2026-02-24 : Prometheus central en OOM

## Impact

De 15 h 20 à 17 h 30, le Prometheus central a redémarré six fois. Chaque redémarrage prend 8 à 12 minutes (relecture du WAL de 2 h), pendant lesquels aucune règle n'est évaluée et aucune alerte ne peut partir. Les agents ont gardé leurs 2 h de WAL local et ont rejoué à la reprise, donc les données sont complètes après coup, mais pendant deux heures on était aveugles sur les alertes. Aucun incident applicatif pendant la fenêtre, par chance.

## Chronologie

- 14 h 50 : déploiement de l'app chauffeur 4.8.1 à 100 % (Android), qui ajoute à la télémétrie poussée les labels `device_model` et `os_build` sur les métriques `hf_mobile_*` (l'idée était de trouver les modèles à problème pour le suivi GPS, ce qui est une bonne idée avec la mauvaise méthode).

- 15 h 05 : les séries actives passent de 1,6 M à 3,9 M en quinze minutes (3 400 appareils × 40 séries × modèles et builds distincts).

- 15 h 20 : premier OOM du Prometheus central (limite 28 Go, request 24 Go).

- 15 h 32 : redémarrage terminé, remplissage de la mémoire, OOM à 15 h 41. Et ainsi de suite.

- 15 h 50 : l'astreinte ops voit `Watchdog` manquant sur le service de paging (le dead-man's switch de [[alertmanager-routing-oncall]], qui a fait exactement son travail).

- 16 h 10 : cause identifiée sur le dashboard des séries par job (qui tournait sur l'instance encore vivante entre deux redémarrages) : `job="mobile-telemetry"` à 2,4 M.

- 16 h 25 : `metric_relabel_configs` ajouté sur la passerelle de télémétrie pour supprimer `device_model` et `os_build`, déployé.

- 16 h 40 : les séries redescendent au rythme de l'expiration des anciennes (5 minutes sans échantillon). Prometheus tient à partir de 17 h 30 sans redémarrage.

## Cause

Une cardinalité non bornée dans un label, poussée par un client qu'on ne contrôle qu'en déployant une app. `device_model` a 380 valeurs distinctes, `os_build` en a 1 100, et le produit des deux sur 40 métriques a fait le reste. Personne n'avait fait la multiplication en revue, et rien ne la faisait à notre place.

## Ce qui a changé

1. La passerelle de télémétrie mobile (le service qui reçoit les métriques poussées par l'app et les expose au scrape) a une **liste blanche de labels** : `app_version`, `platform`, `os_major`, `carrier_id`, `has_gms`. Tout autre label est supprimé à l'entrée. Le modèle d'appareil, quand on en a besoin, est dans les événements de télémétrie envoyés à Loki (voir [[log-labels-cardinality-rule]] : c'est un champ du JSON, pas un label), où la cardinalité n'a pas le même coût.

2. Alerte `PrometheusSeriesCountHigh` : `prometheus_tsdb_head_series > 2.6M` en `warn`, `> 3.2M` en `page`. Et par job : `topk(5, count by (job) ({__name__=~".+"}))` sur le dashboard, avec une alerte si un job double en une heure.

3. Limite mémoire de Prometheus montée à 40 Go (request 32) sur un nœud d'`ops-tools` qui a été passé à 64 Go de RAM, ce qui donne de la marge pour une nouvelle bêtise sans que ce soit une solution.

4. `sample_limit: 50000` par cible sur la passerelle de télémétrie dans son `ServiceMonitor`, pour qu'un scrape trop gros échoue plutôt que d'être ingéré.

5. Une seconde réplique du Prometheus central était prévue et n'est toujours pas là : les deux répliques auraient sauté ensemble, ça n'aurait rien changé. Elle reste sur la liste pour les redémarrages planifiés, voir [[prometheus-stack-layout]].

## Ce qu'on retient

Un label est un multiplicateur. Avant d'en ajouter un, compter les valeurs distinctes et multiplier par le nombre de séries qui le portent. Au-dessus de 10 000 séries pour un seul label ajouté, c'est un événement de log, pas un label de métrique. Et le dead-man's switch a payé sa mise en place ce jour-là.

## Le calcul qu'on aurait dû faire, et le budget par équipe

L'arithmétique de ce jour-là, écrite pour la prochaine fois : 40 métriques × 3 400 appareils × (380 modèles répartis, mais chaque appareil n'en a qu'un) donne 136 000 séries si `device_model` seul est ajouté, parce que le label ne multiplie que par le nombre de valeurs *par série existante*, pas par le nombre total de valeurs. Ce qui a fait exploser, c'est l'ancienne clé de série : avant 4.8.1 les métriques mobiles étaient agrégées par la passerelle par `app_version` et `platform` (une dizaine de séries par métrique), et 4.8.1 a fait passer la passerelle en mode "une série par appareil" pour pouvoir porter le modèle. C'est donc le passage de 400 séries à 136 000, puis × `os_build` distinct par appareil pour arriver à 2,4 M en comptant les anciennes séries qui expiraient lentement. La leçon exacte : ce n'est pas le label qui coûte, c'est le changement de granularité de la série qui vient avec.

Depuis, chaque équipe a un budget de séries, affiché sur le dashboard `hf-cluster-capacity` et vérifié par une alerte `warn` quand il est dépassé de 20 % :

| Équipe (label `team` sur les jobs) | Budget | Utilisé (mai 2026) |
|---|---|---|
| platform (API, workers, PgBouncer, live-gw) | 500 000 | 350 000 |
| ops (Kubernetes, nœuds, Cilium, stockage, ingress) | 1 400 000 | 1 300 000 |
| mobile (passerelle de télémétrie) | 150 000 | 120 000 |
| data | 200 000 | 90 000 |

Le budget `ops` est le plus tendu à cause de Hubble et de cAdvisor, et c'est là que la prochaine campagne de `metric_relabel_configs` aura lieu.

La revue d'une MR qui ajoute un label sur une métrique a maintenant trois questions dans le modèle : combien de valeurs distinctes ce label a-t-il en production (avec la requête qui le prouve), par combien de séries existantes est-il multiplié, et est-ce que la question à laquelle il répond ne serait pas mieux servie par un champ dans un événement de log. Depuis février, deux labels ont été refusés en revue sur ces questions (`route` sur les métriques de PgBouncer, `org_id` sur les métriques de webhook) et remplacés par des champs de log, ce qui a coûté zéro série.
