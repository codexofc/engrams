---
name: rest-stop-detection
description: rest-detect-v1 classifies stops as rest, loading, border queue or traffic from duration, distance and driver state, 91 % accuracy
type: project
status: active
verified: 2026-04-15
---

## Pourquoi

Le modèle d'ETA a besoin de savoir si un arrêt en cours est un repos réglementaire (9 ou 11 heures) ou une attente sur site (1 à 3 heures) ou un bouchon (minutes). Le tachygraphe n'est pas connecté à l'app ; l'app remonte un état de conduite déclaré par le conducteur, souvent en retard sur la réalité. `rest-detect-v1` complète cela à partir des positions.

Il sert aussi à calculer `shipper_site_avg_loading_minutes` ([[eta-features]]) : le temps passé sur un site de chargeur est la durée des arrêts classés `loading` ou `unloading` à moins de 300 m du site.

## Entrée

Un arrêt : séquence de positions à moins de 150 m les unes des autres pendant au moins 8 minutes. Détecté en flux par `ml-infer` sur les positions reçues toutes les 2 à 5 minutes de l'app conducteur, avec un délai de fin d'arrêt de 10 minutes (une position qui repart clôt l'arrêt).

Features : durée courante et durée finale (à la clôture), distance au site de chargement et de livraison déclarés, distance au poste frontière le plus proche, type de route (parking poids lourds connu, aire d'autoroute, zone industrielle, route ouverte), heure locale de début, temps de conduite restant au début de l'arrêt (déclaré), heures depuis le dernier repos, et si l'arrêt est le premier ou le dernier du trajet.

## Classes et résultats

Contre 1 200 arrêts étiquetés à la main par l'équipe dispatch (deux personnes, désaccord sur 4 %, arbitré) :

| Classe | Part | Rappel | Précision |
|---|---|---|---|
| `rest` (repos de 45 min, 9 h ou 11 h) | 31 % | 94 % | 92 % |
| `loading` / `unloading` | 38 % | 93 % | 95 % |
| `border_or_queue` | 9 % | 78 % | 81 % |
| `traffic` | 14 % | 86 % | 79 % |
| `other` (panne, pause repas hors aire) | 8 % | 61 % | 70 % |

Exactitude globale 91 %. Les confusions : `border_or_queue` contre `traffic` près des frontières encombrées (Calais, la frontière polono-ukrainienne), et `other` contre `rest` sur des arrêts de 45 à 60 minutes.

La classification en flux (avant la fin de l'arrêt) est moins bonne : à 20 minutes d'arrêt, on ne sait pas encore si c'est un repos de 45 minutes ou un repos de 9 heures, et l'ETA utilise alors les deux hypothèses pondérées par la probabilité, ce qui élargit l'intervalle p10 à p90 pendant l'arrêt, ce qui est le comportement voulu.

## Ce qui a été décidé

- La classe `rest` alimente une feature `rest_taken_minutes_today` qui corrige le temps de conduite restant déclaré quand le conducteur a oublié de le mettre à jour (12 % des trajets). Gain sur l'ETA : 3 minutes de MAE, 6 sur les trajets de plus de 800 km.
- Pas de retour au conducteur (« vous devriez faire une pause ») : ce n'est pas notre rôle et le tachygraphe le fait déjà.
- Le jeu étiqueté est ré-enrichi de 100 arrêts par trimestre par dispatch, en priorité sur les classes faibles.

## Registre

Enregistré comme `rest-detect-v1` ([[model-registry-conventions]]), promotion manuelle uniquement : la métrique de promotion (exactitude sur le jeu étiqueté) dépend d'un jeu qui grandit, et on préfère regarder.
