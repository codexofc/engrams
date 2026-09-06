---
name: reserved-capacity-decision-2026-01
description: Janvier 2026: engagement annuel Skyvale à 80 % de l'usage (−900 EUR/mois) et tarif interne engagé pour 4 nœuds du pool, 0 nœud commandé en 2026, HF-4700
type: project
status: active
verified: 2026-02-12
---

# Engagements de capacité, janvier 2026

## Le contexte

Presque tout est en propre dans les racks ([[infra-cost-overview-2026]]) : le matériel est acheté, amorti sur quatre ans, et il n'y a rien à « réserver » au sens d'un fournisseur cloud. Deux endroits font exception et se facturent à l'usage : Skyvale (CDN, egress, DNS, page de statut, VM de burst pour staging et les runners CI) et, en interne, la refacturation des nœuds du pool partagé aux équipes, qui distinguait depuis 2025 un tarif « à la demande » (des nœuds pris et rendus au mois) et un tarif « engagé » (des nœuds gardés douze mois, 27 % moins chers dans la formule, parce qu'ils permettent de planifier les achats).

HF-4700 a posé la question pour les deux en janvier 2026, avec douze mois de données d'usage.

## Skyvale

L'usage à la demande de 2025 : 5 500 EUR par mois en moyenne pour le CDN et l'egress après la correction des tuiles ([[egress-finding-map-tiles-2025-11]]), 4 200 pour le DNS, la page de statut et les VM. Skyvale propose un plan annuel avec un volume d'egress et un nombre d'heures de VM inclus, à −22 % sur le volume engagé, et l'excédent au tarif normal.

Décision : engager 80 % de l'usage moyen de 2025 après correction, pas 100 %. L'egress du CDN varie de 30 % entre un mois calme et une fin d'année, et un engagement sur le pic paie pour de la capacité vide neuf mois sur douze. À 80 %, l'économie mesurée sur janvier à juin 2026 est de 900 EUR par mois ([[per-service-cost-table-q2-2026]] la ventile), avec un dépassement facturé au tarif normal deux mois sur six. Un engagement à 100 % aurait économisé 1 050 les mois pleins et perdu 400 les mois creux ; le calcul sur l'année est à l'avantage des 80 %.

Ce qu'on n'a pas engagé : les VM de burst des runners CI. Leur usage a été divisé par trois par la réduction de staging ([[staging-environment-cost-cut]]) pendant qu'on négociait, et engager sur un usage qu'on est en train de faire baisser aurait figé le mauvais chiffre. Revu en juillet, engagé à ce moment sur le nouvel usage.

## Le pool partagé interne

Le tarif interne n'est pas de l'argent qui sort, c'est une répartition. Mais il change les décisions : une équipe qui prend des nœuds « à la demande » pour un pic et les rend est plus chère pour tout le monde parce que le pool doit garder la marge. Fin 2025, 6 des 22 nœuds du pool étaient encore au tarif à la demande, dont 4 pris par l'équipe données pour l'entrepôt depuis dix mois sans jamais les rendre.

Décision : les 4 nœuds de l'entrepôt passent en engagé (l'équipe données l'a acté dans son propre suivi de coûts, c'est leur ligne « réservation annuelle » à −3 300 par mois dans leur formule), les 2 autres restent à la demande parce que ce sont réellement des nœuds de pic (fin de mois pour la facturation, deux semaines par trimestre pour un entraînement ML). Le pool garde 3 nœuds de marge au lieu de 5, ce qui a permis de ne pas commander les deux nœuds prévus au budget 2026 : 1 100 EUR par mois d'amortissement évités pendant quatre ans, l'effet réel de la décision.

## Ce qui a été refusé

- Un engagement Skyvale sur trois ans à −35 % : le CDN pourrait ne plus être Skyvale en 2028, ou ne plus être nécessaire si les tuiles sont servies depuis les racks, et trois ans d'engagement pour 13 % de mieux est une option qu'on ne veut pas vendre.

- Un tarif interne « engagé » obligatoire pour tout nœud gardé plus de trois mois : la règle automatique aurait engagé les deux nœuds de pic à leur troisième mois d'usage cumulé. Le tarif reste un choix par équipe, revu à la revue mensuelle ([[finops-monthly-review-feedback]]).

## Résultat sur six mois

| | Avant (T4 2025) | Après (T1 et T2 2026) |
|---|---|---|
| Skyvale, tout compris | 9 700 EUR / mois | 4 600 EUR / mois (dont −900 d'engagement, le reste vient des tuiles et de staging) |
| nœuds du pool à la demande | 6 | 2 |
| marge du pool | 5 nœuds | 3 nœuds |
| nœuds commandés en 2026 | 2 prévus | 0 |

L'engagement en lui-même vaut 900 par mois. Les décisions qu'il a forcées (rendre les nœuds, arrêter de garder de la marge pour de l'usage qui n'est pas du pic) valent le reste.
