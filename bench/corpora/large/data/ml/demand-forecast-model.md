---
name: demand-forecast-model
description: Demand forecast per lane cluster and day for 14 days, global gradient boosted model on lags and calendar, WAPE 18 % at day 7
type: project
status: active
verified: 2026-04-27
---

## Objet

Prévoir le nombre de chargements publiés par cluster de ligne et par jour sur 14 jours. Deux consommateurs : l'équipe commerciale transporteurs, qui cible ses relances sur les lignes où la demande va dépasser l'offre, et le moteur de prix, qui utilise le ratio prévu offre/demande comme feature (`imbalance_forecast_7d`) dans son modèle de suggestion.

## Modèle

Un modèle global (un seul pour tous les clusters), gradient boosting sur :

- comptes retardés : chargements publiés à J-1, J-7, J-14, J-28, moyenne mobile 7 et 28 jours, même jour de semaine sur les 4 dernières semaines ;
- calendrier : jour de semaine, jour férié au départ et à l'arrivée ([[holiday-calendar-feature]]), semaine de l'année, veille et lendemain de férié ;
- cluster : pays, bande de distance, type de véhicule, volume moyen sur 90 jours (encodage du niveau) ;
- exogènes : indice carburant du mois, nombre de transporteurs actifs sur le cluster à J-7.

Cible : `log1p(loads_posted)`. Horizon traité par un modèle par horizon (14 modèles), ce qui est plus simple qu'une récursion et ne coûte rien à entraîner. Réentraînement hebdomadaire le dimanche, sur 2 ans d'historique par cluster (les clusters de moins de 6 mois sont exclus et reçoivent la moyenne de leur groupe).

## Résultats (mars 2026, voir [[demand-forecast-eval-metrics]] pour les définitions)

| Horizon | WAPE | Biais |
|---|---|---|
| J+1 | 12 % | +1 % |
| J+7 | 18 % | +2 % |
| J+14 | 24 % | +3 % |

Référence naïve (même jour de semaine, semaine précédente) : 27 % à J+7. Sur les 200 plus gros clusters (58 % du volume), WAPE à J+7 de 11 %.

Le biais positif de 2 à 3 % vient des semaines de jours fériés : le modèle sous-estime la chute (Noël, Pâques, 1er mai) puis surestime le rebond. La feature « lendemain de férié » a réduit le biais de 5 % à 2 % en février.

## Ce qui a raté

- Une version par cluster (2 140 modèles) en 2025 : meilleure sur les 50 gros clusters, catastrophique ailleurs, et 2 heures d'entraînement. Abandonnée.
- Le prix comme feature exogène : une hausse de prix suit la demande plus qu'elle ne la précède, la feature ajoutait du bruit.
- Les données macro (production industrielle, prix du gazole hors indice) : aucun gain mesurable à 14 jours, elles jouent à 3 mois, ce qu'on ne prévoit pas.

## Fuite de données évitée

Le nombre de transporteurs actifs à J-7 est calculé avec la date de calcul comme référence, pas la date de prévision ; la première version utilisait les transporteurs actifs jusqu'à la date prévue, ce qui est l'histoire racontée dans [[training-data-leakage-lesson]].

## Prochaines étapes

Une sortie par intervalle (p10, p90) demandée par le commercial pour ne pas relancer sur une prévision fragile. Quantile loss comme pour l'ETA, prévu T3 2026.
