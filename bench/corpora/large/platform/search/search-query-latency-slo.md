---
name: search-query-latency-slo
description: SLO de la recherche : p95 sous 150 ms, p99 sous 300 ms, disponibilité 99,9 % avec repli compté dégradé, chiffres mensuels, ventilation des 35 ms
type: reference
status: active
verified: 2026-08-05
---

# Latence et disponibilité de la recherche

Mesuré côté API (`GET /v2/loads/search`, variante place de marché), du début de la requête HTTP à la fin de la réponse, hors réseau du client. C'est ce que le transporteur perçoit, moins son réseau mobile.

## Le SLO

- **p95 < 150 ms** et **p99 < 300 ms** sur 30 jours glissants, aux heures ouvrées (6 h à 20 h CET). La nuit compte pour la disponibilité, pas pour la latence : trop peu de requêtes pour un percentile stable.

- **Disponibilité 99,9 %** : une requête est « disponible » si elle renvoie des résultats classés par haystack. Une réponse servie par le repli PostgreSQL (sans classement, bannière « recherche dégradée ») compte comme indisponible pour le SLO même si l'utilisateur a eu une liste. On s'impose ça pour ne pas se raconter d'histoires.

Budget d'erreur mensuel : environ 43 minutes de repli, ou un p99 au-dessus de 300 ms pendant l'équivalent.

## Chiffres mensuels

### Depuis la mise en production

| Mois | p50 | p95 | p99 | Disponibilité | Remarque |
|---|---|---|---|---|---|
| 2026-03 | 38 ms | 120 ms | 290 ms | 99,84 % | lancement, retard d'indexation du 24 ([[search-incident-2026-03-index-lag]]) compté hors SLO de latence |
| 2026-04 | 35 ms | 105 ms | 250 ms | 99,97 % | |
| 2026-05 | 36 ms | 130 ms | 410 ms | 99,95 % | shard chaud trois jours ([[search-incident-2026-05-shard-hotspot]]), SLO p99 manqué |
| 2026-06 | 34 ms | 108 ms | 265 ms | 99,96 % | mise à niveau du cluster, 41 s de retard d'index |
| 2026-07 | 35 ms | 110 ms | 270 ms | 99,98 % | classement v2 déployé le 8, pas d'effet mesurable |

Le mois de mai est le seul manqué. Le compte rendu mensuel va au produit et à l'équipe plateforme ; le manquement de mai a donné les correctifs de tableau de bord décrits dans l'incident.

## Où passent les 35 ms

Requête typique (rayon 150 km, dates sur 7 jours, deux types de véhicule, pas de texte), mesurée par traçage sur une semaine d'avril :

- 3 ms : authentification, chargement du profil transporteur (flotte, favoris, recherches enregistrées) depuis le cache Redis

- 2 ms : construction de la requête, dont le calcul du `fit` ([[search-relevance-rules-v2]])

- 22 ms : haystack, dont 3 ms de filtre géographique, 12 ms de `function_score` sur environ 1 800 candidats, 5 ms de collecte et de tri sur six shards, 2 ms de sérialisation

- 5 ms : enrichissement des 20 résultats (distance routière depuis la table du pricing, nom d'affichage du chargeur) depuis Redis

- 3 ms : sérialisation JSON et réponse

Le `function_score` domine et il est proportionnel au nombre de candidats après filtre. Un transporteur avec un rayon de 800 km et « tous véhicules » a 15 000 candidats et une requête à 140 ms ; c'est la queue du p99. On a choisi de ne pas limiter le nombre de candidats (un `rescore` sur les 500 meilleurs par distance changerait le classement), et d'accepter que les recherches larges soient lentes.

## Alertes

- `SearchLatencyP99High` : p99 sur 5 min > 280 ms pendant 10 min, notification. Page si > 1 s pendant 5 min.

- `SearchFallbackActive` : plus de 1 % des requêtes servies par le repli sur 5 min, page. Ça veut dire haystack ne répond pas.

- `SearchErrorRate` : plus de 0,5 % de 5xx sur 5 min, page.

- Les alertes du cluster lui-même sont dans [[search-haystack-cluster-layout]].

## Ce qu'on ne mesure pas dans le SLO

La pertinence (c'est [[search-ab-testing-ranking]]), la fraîcheur de l'index (c'est le retard d'indexation, avec ses propres alertes), le temps de rendu dans l'app. Le premier affichage des résultats dans l'app est mesuré à part par la mobile : 600 ms en médiane sur 4G, dont nos 35 ms.
