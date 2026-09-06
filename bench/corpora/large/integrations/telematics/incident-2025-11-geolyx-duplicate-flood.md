---
name: incident-2025-11-geolyx-duplicate-flood
description: On 2025-11-24 Geolyx replayed 6 h of positions in 20 min (2 400/s); Redis eviction killed the dedup cache and 180 000 duplicate rows led to the unique index
type: project
status: active
verified: 2025-12-18
---

# Rafale de positions Geolyx après leur panne (novembre 2025)

Ticket HF-2076. Incident d'intégration, sans exposition de données, qui a changé le dimensionnement du pipeline et fait ajouter l'index unique.

## Chronologie (UTC)

- **2025-11-24 02:10 à 08:05** : panne côté Geolyx (leur page de statut : « délai de traitement des positions »). Pendant six heures, aucun webhook ne nous arrive pour les 320 véhicules alors connectés via Geolyx. Nos alertes « pas de position depuis 15 minutes » se déclenchent pour ces véhicules à partir de 02:25 ; l'astreinte constate la panne fournisseur et attend, c'est le comportement prévu.

- **08:05** : Geolyx redémarre et **rejoue tout le tampon** : les six heures de positions de 320 véhicules, en lots de 500, aussi vite que notre endpoint répond. 12 millions de positions en 20 minutes, pic à 2 400 requêtes par seconde sur `/telematics/geolyx/positions` (voir [[geolyx-push-webhook-format]]).

- **08:05 à 08:12** : le contrôleur répond 200 dès la mise en file ; la file interne (Redis Streams à l'époque) grossit à 9 millions de messages. Les 4 handlers traitent à environ 1 000 par seconde. Rien ne casse à cet endroit.

- **08:09** : l'instance Redis partagée par la file, le cache de déduplication et `lastpos:` atteint `maxmemory` (2 Go). Politique d'éviction `allkeys-lru` : les clés `dedup:{vehicle}` (voir [[position-dedup-rules]]), peu consultées pendant la rafale par rapport aux messages de la file, sont évincées en premier.

- **08:09 à 08:31** : sans le cache, les positions déjà vues (Geolyx avait réémis aussi les lots des 20 minutes précédant la panne, qu'on avait déjà traités) passent l'étape de déduplication et sont écrites. 180 000 lignes en double dans `position_events` sur la partition du jour.

- **08:31** : file vidée, situation normale. Les ETA des 320 véhicules ont été recalculées 6 heures de positions d'un coup, sans effet visible : le consommateur ETA ne garde que la dernière par mission.

- **09:40** : un analyste de la plateforme data signale des doublons dans son import du matin. C'est ainsi qu'on l'apprend ; rien chez nous n'avait alerté sur les doublons, seulement sur la latence de la file (alerte à 08:07, acquittée comme « rafale fournisseur, on absorbe »).

## Causes

1. Geolyx rejoue son tampon sans lissage ni signalement (`sent_at` très différent de `ts`, mais on ne regardait pas).

2. Une seule instance Redis pour trois usages avec des profils différents, et une politique d'éviction qui a sacrifié le cache le plus important pendant l'incident.

3. Aucune contrainte en base : la déduplication reposait entièrement sur le cache.

## Correctifs

- **Index unique** `(vehicle_id, ts, provider)` sur chaque partition de `position_events`, créé `CONCURRENTLY` sur les partitions existantes après suppression des 180 000 doublons (garder la ligne au plus petit `id`). Livré le 2025-11-27. Le `COPY` par lot bascule en `INSERT ... ON CONFLICT DO NOTHING` ligne à ligne quand un lot échoue sur la contrainte. Voir [[positions-table-partitioning]].

- **Redis séparé** pour la file (qui a ensuite migré vers Kafka en janvier 2026, voir [[position-ingestion-pipeline]]) et pour les caches, avec `maxmemory-policy noeviction` sur les caches et une alerte à 70 % de mémoire.

- **Détection de rejeu** : si `sent_at - ts > 10 min` pour la majorité d'un lot, le lot est marqué `replay = true`, traité par une file à basse priorité, et compté dans `telematics_replay_positions_total`. Le trafic temps réel des autres abonnements n'attend plus derrière un rejeu.

- **Demande à Geolyx** de lisser les rejeux : refusée (« comportement standard »), mais ils ont accepté d'ajouter un en-tête `X-Geolyx-Replay: true` sur les lots rejoués, livré en janvier 2026, qu'on utilise en plus de l'heuristique sur `sent_at`.

- **Test de charge** dans le harnais ([[telematics-integration-test-harness]]) : le scénario « 6 h de 320 véhicules en 20 minutes » est rejoué avant chaque changement du pipeline.

## Ce qu'on retient

Une déduplication qui dépend d'un cache est une déduplication qui dépend de la mémoire disponible au pire moment. La contrainte en base coûte un index de plus par partition et rend le bug impossible ; c'est le même raisonnement que la plateforme a appliqué aux offres en double.
