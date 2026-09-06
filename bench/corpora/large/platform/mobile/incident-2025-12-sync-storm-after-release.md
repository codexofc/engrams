---
name: incident-2025-12-sync-storm-after-release
description: Dec 2025, app 4.5.2 retried sync without backoff on a 422 invalid_cursor, 2 800 devices hammered the API at 40 req/s each for 25 minutes, fixed by min-version bump and a backoff rule on all 4xx
type: project
status: active
verified: 2025-12-23
---

# Incident 2025-12-09 : tempête de synchronisation après la 4.5.2

## Résumé

La 4.5.2 (correctif mineur) a été poussée à 100 % du parc Android le mardi 9 décembre à 8 h 40. À 9 h 05, l'API recevait 110 000 requêtes par minute sur `/internal/mobile/sync` (normal : 70 par minute). Le rate limiter a tenu, PgBouncer aussi, mais les 429 en masse ont saturé les pods php-fpm sur la sérialisation des enveloppes d'erreur et le p99 de toute l'API est passé à 4 s. Les dispatchers web ont vu des lenteurs pendant 25 minutes. Aucune donnée perdue.

## Chronologie

- 8 h 40 : passage à 100 % sur Android. Le déploiement par paliers avait été respecté (5 %, 20 %, 50 % les jours précédents) et rien d'anormal n'avait été vu. On comprendra pourquoi plus bas.

- 9 h 00 : le déploiement de l'API 2025.49 change le format du curseur de synchronisation (ajout de l'UUID, voir [[sync-push-triggered-delta]]). Prévu, annoncé, et l'API répond 422 `invalid_cursor` aux anciens curseurs, ce que l'app est censée traiter par une resynchronisation complète.

- 9 h 03 : `ApiRequestRateAnomaly` déclenche. Le trafic vient à 99 % de l'app 4.5.2 (en-tête `User-Agent`).

- 9 h 08 : hypothèse confirmée dans les logs : les appareils font `sync` → 422 → `sync` immédiatement → 422, en boucle, environ 40 requêtes par seconde par appareil.

- 9 h 12 : décision de bloquer `/internal/mobile/sync` pour l'agent `4.5.2` au niveau de l'ingress (règle temporaire, 403). Le trafic retombe en 2 minutes. Les chauffeurs en 4.5.2 sont sans synchronisation.

- 9 h 20 : `min-version` monté à 4.5.3 pour Android, la 4.5.3 (correctif) est construite et poussée sur la piste interne à 10 h 15, promue à 100 % directement (exception au processus, deux approbations) à 10 h 40, et la règle ingress retirée à 11 h 30 quand 90 % du parc était migré. Le reste est bloqué par l'écran de mise à jour forcée.

- Incident clos à 11 h 30. Dégradation visible pour les utilisateurs web : 9 h 05 à 9 h 30.

## Cause

Deux bugs et un angle mort.

1. Dans `SyncCoordinator.pull()`, la 4.5.2 avait réordonné le traitement des erreurs pour corriger un autre problème. Le cas 422 `invalid_cursor` effaçait bien le curseur, mais relançait `pull()` **avant** que l'effacement soit commité dans `sync_state` (écriture asynchrone dans drift, `await` manquant). Le second appel repartait avec l'ancien curseur. Boucle infinie, sans délai parce que le 422 n'est pas une erreur réseau et ne passait pas par le backoff.

2. Le mutex de `pull()` protégeait contre deux pulls parallèles, pas contre un pull qui se rappelle lui-même à la fin.

3. L'angle mort : le changement de format de curseur côté API n'avait pas encore été déployé en staging quand la 4.5.2 a été testée. Le cas 422 n'a donc jamais été exercé avec ce build. Le test unitaire du cas existait mais utilisait un faux client qui ne renvoyait le 422 qu'une fois.

## Correctifs

- 4.5.3 : `await` ajouté, et surtout une règle générale dans `SyncCoordinator` : **toute réponse 4xx déclenche le même backoff que les erreurs réseau** (2 s, 4 s, 8 s, ... plafonné à 5 min), et un compteur `consecutive_failures` qui, au-delà de 10, suspend la synchronisation automatique jusqu'au prochain passage au premier plan avec un message au chauffeur. Aucun cas légitime ne demande de réessayer un 4xx immédiatement.

- Le faux client de test renvoie maintenant l'erreur configurée indéfiniment jusqu'à ce que le test dise le contraire, pour que les boucles se voient.

- API : `invalid_cursor` renvoie maintenant aussi `Retry-After: 5`, que l'app respecte comme pour un 429.

- Processus : un changement de contrat entre app et API se teste en staging avec les deux côtés déployés, dans cet ordre, avant le passage en production de l'un ou l'autre. Écrit dans la checklist de [[release-process-stores]].

- Ops : la règle d'ingress par `User-Agent` est maintenant un snippet prêt à l'emploi dans le dépôt de configuration, avec le nom `block-mobile-version`, pour ne pas la réécrire à chaud. Elle a servi une deuxième fois en mars 2026 pour un cas moins grave.

## Pourquoi le déploiement par paliers n'a rien vu

Parce que le déclencheur était le déploiement de l'API, pas celui de l'app. Les 5 %, 20 % et 50 % ont tourné pendant quatre jours contre une API qui acceptait encore l'ancien curseur. Le palier protège contre un bug de l'app seule, pas contre une interaction avec un changement serveur qui arrive après. C'est le point qui a le plus marqué l'équipe.

Voir [[offline-sync-architecture]] pour l'architecture générale, dont la partie outbox n'a pas été touchée par l'incident : les 214 mutations en attente sur l'appareil le plus chargé ont toutes été poussées en 4.5.3.

## Ce qu'on a mesuré dans les semaines suivantes

La règle de suspension après 10 échecs consécutifs a un effet secondaire qu'il fallait chiffrer : un chauffeur dont la synchronisation est suspendue ne le sait que par une bannière. Sur les quatre semaines après la 4.5.3, l'événement de télémétrie `sync.suspended` est remonté 19 fois sur 3 400 appareils, dont 14 pour une vraie coupure réseau de plus de 40 minutes (tunnels, parkings souterrains) et 5 pour un compte désactivé côté transporteur. Aucun cas de boucle. Le support a une macro "Synchronisation suspendue" qui dit d'ouvrir l'app, et ça a suffi.

Le coût de l'incident lui-même : 25 minutes de p99 à 4 s pour le web, 2 h 50 sans synchronisation pour les chauffeurs Android en 4.5.2 (de 9 h 12 à 11 h 30, plus le délai de mise à jour forcée), et 214 mutations en attente sur l'appareil le plus chargé, toutes poussées dans l'ordre à la reprise. Le total de mutations rejouées ce jour-là par le mécanisme d'idempotence côté API : 3 900, dont 0 doublon appliqué, ce qui était la première vraie épreuve de la clé `Idempotency-Key` en conditions réelles.

Sur le trafic : les 110 000 requêtes par minute ont produit environ 2,4 millions de lignes de log d'accès à l'ingress en 25 minutes, soit trois jours de volume normal. C'est ce pic qui a lancé l'examen du volume de logs côté observabilité, parce que Loki a tenu mais les writers étaient à 80 % de leur limite d'ingestion.

Enfin, la règle `block-mobile-version` à l'ingress a été chronométrée : de la décision à l'effet, 4 minutes la première fois (en écrivant le snippet à la main), 40 secondes la seconde fois en mars 2026 avec le snippet prêt. C'est la mesure qui justifie de garder des contournements préparés à l'avance pour les cas qu'on a déjà vus une fois.
