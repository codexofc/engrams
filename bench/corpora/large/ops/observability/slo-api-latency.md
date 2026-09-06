---
name: slo-api-latency
description: Two SLOs on the public API since Q1 2026, availability 99.9 % (non-5xx over 30 d) and latency 99 % of requests under 800 ms, with multi-window burn-rate alerts (1h/5m at 14.4x pages, 6h/30m at 6x warns) and a monthly error budget review
type: project
status: active
verified: 2026-04-14
---

# SLO de l'API (HF-1530)

Définis en janvier 2026 avec le produit, mesurés depuis février, revus chaque mois.

## Les deux SLO

| SLO | Indicateur | Objectif | Fenêtre |
|---|---|---|---|
| Disponibilité | part des requêtes de l'API publique (`/v2/*`) sans réponse 5xx | 99,9 % | 30 jours glissants |
| Latence | part des requêtes `/v2/*` sous 800 ms | 99 % | 30 jours glissants |

Exclus des deux : `/internal/*` (les clients sont à nous, ils ont leurs propres indicateurs), les exports (`/v2/exports/*`, une latence de 20 s y est normale), et les 429 (c'est le client qui dépasse son quota, voir la note de rate limiting côté API).

Les 800 ms viennent de la distribution mesurée en janvier : p99 à 600 ms en régime normal, et le produit a demandé "à partir de quand un dispatcher trouve que c'est lent", réponse d'après les sessions d'observation : au-delà d'une seconde on voit les gens cliquer deux fois.

## Budget d'erreur

99,9 % sur 30 jours = 43 minutes d'indisponibilité totale, ou l'équivalent en erreurs partielles. En mars 2026 on a consommé 61 % du budget, dont 48 % dans la fenêtre de rollback de l'incident Safari (qui n'a pas produit de 5xx mais des requêtes en moins... en fait non, cet incident n'a pas touché le SLO, c'est l'incident de recherche de février, 48 minutes de timeouts, qui a consommé un budget entier à lui seul, ce qui a été le premier argument concret pour le `statement_timeout`).

## Règles d'enregistrement

```yaml
- record: hf:slo_api_availability_ratio5m
  expr: 1 - (sum(rate(hf_http_requests_total{route=~"api_v2_.*",status=~"5..",route!~"api_v2_exports.*"}[5m])) / sum(rate(hf_http_requests_total{route=~"api_v2_.*",route!~"api_v2_exports.*"}[5m])))
- record: hf:slo_api_latency_ratio5m
  expr: sum(rate(hf_http_request_duration_seconds_bucket{le="0.8",route=~"api_v2_.*"}[5m])) / sum(rate(hf_http_request_duration_seconds_count{route=~"api_v2_.*"}[5m]))
```

Plus les mêmes en `30m`, `1h`, `6h`, `3d` pour les fenêtres de burn rate.

## Alertes par burn rate

Multi-fenêtre, sur les deux SLO :

- **Page** : burn rate > 14,4 sur 1 h **et** sur 5 min. À ce rythme le budget de 30 jours part en 2 jours. Correspond à un ratio d'erreurs > 1,44 % pendant une heure pour la disponibilité.

- **Warn** : burn rate > 6 sur 6 h **et** sur 30 min. Budget épuisé en 5 jours.

- **Ticket** (pas d'alerte, une ligne dans la revue mensuelle) : burn rate > 1 sur 3 jours.

La double fenêtre (longue et courte) évite qu'une alerte reste allumée une heure après la fin du problème. Ces règles ont remplacé `ApiErrorRateHigh` (seuil fixe à 2 %) dans l'intention, mais on a gardé l'ancienne un trimestre en parallèle pour comparer : elle a déclenché 3 fois, la page burn rate 2 fois, sur les mêmes événements, et la burn rate s'est éteinte 40 minutes plus tôt à chaque fois. Voir [[alerting-rules-catalogue]].

## Revue mensuelle

Le premier mardi du mois, 20 minutes, plateforme et produit : budget consommé, par quoi, et une décision. Les décisions prises jusqu'ici :

- Février : le `statement_timeout` à 8 s (fait).

- Mars : rien, budget à 61 % expliqué par un seul incident déjà traité.

- Avril : la latence est à 99,4 % contre 99 % visé, avec la route `api_v2_loads_search` qui représente 70 % des requêtes au-dessus de 800 ms. Décision : la recherche a droit à son propre SLO de latence à 1,5 s à partir de mai, et sort du SLO général. Ça s'appelle ajuster l'objectif à la réalité, et c'est assumé tant que c'est écrit.

Le tableau de bord est `hf-slo`, voir [[dashboards-conventions]].

## Détails de mesure qui changent le résultat

Trois choix de mesure ont été discutés et sont écrits ici parce que chacun déplace le chiffre de quelques dixièmes de point.

Les 429 sont exclus des deux SLO. Un client qui dépasse son quota reçoit une réponse rapide et correcte, la compter comme une erreur ferait dépendre notre SLO du comportement d'un intégrateur. Mais un pic de 429 causé par un bug de notre côté (le rate limiter qui se serait trompé d'organisation, par exemple) échapperait au SLO. Le compromis : les 429 ont leur propre alerte `warn` sur une hausse de 5× en 10 minutes, hors SLO.

Les exports sont exclus de la latence, mais pas de la disponibilité : un export qui répond 500 compte. La route est identifiée par le label `route` du framework (`api_v2_exports_*`), pas par le chemin, ce qui évite qu'un chemin mal routé (un 404 sur `/v2/exportz`) ne soit ni dans l'un ni dans l'autre. Les 404 sur une route inexistante n'ont pas de label `route` et sont hors SLO, ce qui est voulu : les scanners ne comptent pas.

La latence est mesurée par l'API elle-même (histogramme `hf_http_request_duration_seconds`, du début du kernel Symfony à la réponse envoyée), pas par l'ingress. La différence est de 3 à 8 ms de réseau et de sérialisation nginx, négligeable, et l'histogramme applicatif a le label `route` fiable. Le prix : une requête qui n'atteint jamais PHP (503 de l'ingress quand tous les workers sont occupés) n'est pas dans l'histogramme. Ces 503 sont comptés par la métrique de l'ingress et ajoutés au numérateur d'erreurs de la disponibilité par une seconde règle d'enregistrement, `hf:slo_api_ingress_5xx_rate5m`, qui vaut zéro 99 % du temps et qui a compté 14 minutes de 503 pendant l'incident de novembre 2025 (avant que le SLO existe, mais on l'a recalculé pour savoir).

## Ce qu'on a refusé

- Un SLO par organisation cliente. Demandé par le commercial pour un grand compte. Refusé parce que le trafic d'une organisation est trop faible pour qu'un ratio sur 30 jours veuille dire quelque chose (un seul 500 sur 2 000 requêtes, c'est 99,95 %), et parce que ça crée une pression pour prioriser un client sur l'infrastructure commune. Le grand compte a un rapport mensuel de ses propres erreurs, extrait des logs, sans engagement chiffré.

- 99,95 % de disponibilité. Le budget serait 21 minutes par mois, et une seule promotion ratée avec rollback en consomme la moitié. On préfère un objectif qu'on tient à un objectif qui interdit de déployer.

- Un SLO sur la fraîcheur des données du tableau de bord (délai entre commit et affichage via le WebSocket). Mesurable, mais personne n'a su dire quel objectif aurait un sens. Reporté jusqu'à ce qu'un client s'en plaigne, ce qui n'est pas arrivé.
