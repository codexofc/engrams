---
name: drift-incident-eta-2026-02
description: February 2026: driver app 4.8 sent driving time in seconds, ETA 6 to 9 h early on 30 % of loads for 11 days, range checks added
type: project
status: active
verified: 2026-03-06
---

## Chronologie

- 2026-02-03 : sortie de l'app conducteur 4.8, avec une refonte du module tachygraphe. Le champ `remaining_driving` de l'événement `driver.state_reported` passe de minutes à secondes, sans changement de nom, et sans que l'équipe ML soit dans la revue.

- 2026-02-03 au 2026-02-14 : à mesure que les conducteurs mettent à jour (30 % à J+3, 65 % à J+10), la feature `remaining_driving_minutes` ([[eta-features]]) vaut 60 fois trop pour ces conducteurs. Le modèle voit un conducteur avec 12 000 minutes de conduite restante, valeur jamais vue, et l'arbre tombe dans la feuille « beaucoup de temps » : pas de repos prévu, arrivée optimiste. En réalité l'inverse aurait été anodin ; ici le modèle ne prédit plus jamais le repos obligatoire pour ces conducteurs, et les livraisons avec repos en route (30 % des chargements de plus de 500 km) sont prédites 6 à 9 heures trop tôt.

- Le monitoring de dérive regardait la distribution des prédictions (stable en moyenne, car les autres 70 % compensaient) et la MAE glissante à 7 jours, qui monte doucement de 24 à 39 minutes sans franchir le seuil de 45.

- 2026-02-12 : le support dispatch signale des « ETA absurdes » sur des trajets Pologne-Espagne. L'équipe ML regarde les prédictions par version d'app : MAE 22 minutes pour 4.7, 71 minutes pour 4.8.

- 2026-02-13 : correction côté serveur (`ingest` convertit en minutes quand `app_version >= 4.8.0`), redéploiement du feature store, prédictions correctes en 20 minutes pour les nouveaux ticks. Le 2026-02-14, l'app 4.8.1 renvoie des minutes ; la conversion serveur est conditionnée sur `4.8.0` exactement.

- Impact : 11 jours, environ 19 000 chargements avec une ETA fausse de plus d'une heure, 640 alertes de retard non émises, 90 tickets support. Pas d'impact financier direct.

## Causes

1. Un changement d'unité sans changement de nom dans un événement produit. La convention des événements (registre avec type et unité) existait mais l'unité n'était pas vérifiée par le collecteur.
2. Pas de contrôle de plage sur les features en entrée du modèle : `remaining_driving_minutes` a un maximum physique de 600 (10 heures), et une valeur de 12 000 aurait dû être rejetée.
3. Le monitoring de dérive agrégé masquait un problème sur un sous-ensemble.

## Changements

- **Contrôles de plage** dans le feature store ([[feature-store-design]]) : chaque feature du registre a un `min` et un `max` physiques ; une valeur hors plage est remplacée par la valeur par défaut et comptée dans `feat.out_of_range` par feature. Alerte au-delà de 0,5 % sur une heure. Déployé le 2026-02-20.

- **Feature `driver_app_version_band`** ajoutée au modèle, moins pour prédire que pour que la dérive par version soit visible dans l'importance et dans l'évaluation par segment ([[backtesting-framework]] segmente par version d'app depuis).

- **Dérive par segment** : la MAE glissante est calculée par version d'app, par pays et par bande de distance, avec un seuil relatif de +30 % sur n'importe quel segment de plus de 500 chargements par semaine.

- Le collecteur d'événements vérifie l'unité déclarée dans le registre pour les propriétés numériques avec une plage déclarée ; c'est l'équipe produit commune qui l'a livré.

- L'équipe ML est relectrice obligatoire sur tout changement d'un événement listé dans `feature_registry.yaml` comme source.

## Ce qu'on retient

Un modèle robuste à une valeur absurde n'existe pas ; ce qui existe, c'est un contrôle avant le modèle. Et une métrique agrégée stable n'est pas une preuve d'absence de dérive.

## Chiffres de l'impact par jour

| Jour | Part des conducteurs en 4.8 | MAE livraison à 4 h (toutes versions) | MAE 4.8 seule |
|---|---|---|---|
| 02-03 | 4 % | 24 min | 60 min |
| 02-05 | 18 % | 27 min | 66 min |
| 02-07 | 31 % | 31 min | 70 min |
| 02-10 | 52 % | 36 min | 72 min |
| 02-12 | 65 % | 39 min | 71 min |
| 02-13 (correctif à 14:10) | 68 % | 28 min | 25 min après 14:10 |
| 02-14 | 71 % | 23 min | 22 min |

La MAE globale n'a jamais franchi le seuil d'alerte de 45 minutes, parce que les 30 % de chargements avec repos en route sont ceux où l'erreur se concentre et que les 70 % restants tiraient la moyenne vers le bas. C'est le tableau qui justifie l'alerte par segment.

## Ce qu'on a envisagé et écarté

- **Un modèle « robuste » aux valeurs aberrantes** (winsorisation des features à l'entraînement) : ne règle rien, la valeur en production est hors de tout ce que le modèle a vu, et couper à un quantile d'entraînement revient au contrôle de plage, en moins lisible.
- **Rejeter toute prédiction dont une feature est hors plage** plutôt que remplacer par la valeur par défaut : rejeté parce qu'un chargement sans prédiction n'a pas d'alerte de retard, ce qui est pire qu'une prédiction avec une feature par défaut. Le compromis : remplacement par défaut, `confidence = low`, et un compteur qui alerte.
- **Un test de contrat automatique entre l'app conducteur et le feature store** : c'est ce qu'a livré l'équipe produit commune avec la vérification d'unité dans le collecteur, et c'est la bonne couche ; le feature store n'a pas à connaître l'app.

## Comment le contrôle de plage est calibré

`min` et `max` dans le registre ([[feature-store-design]]) sont des bornes physiques ou réglementaires, pas des quantiles : 600 minutes pour le temps de conduite restant (10 h, le maximum réglementaire avec l'extension bihebdomadaire), 4 500 km pour une distance d'itinéraire (Lisbonne à Helsinki par la route), 24 pour le nombre de frontières. Une valeur hors plage est un bug quelque part, jamais un cas rare, et c'est pourquoi le seuil d'alerte est bas (0,5 % sur une heure).

Depuis février, `feat.out_of_range` a alerté deux fois : une position GPS avec une vitesse de 900 km/h (un téléphone qui a changé de fuseau horaire, `speed_last_15min` calculée sur un intervalle négatif), et une valeur de `stop_count` à 60 sur un chargement de test. Les deux étaient des bugs.

## Ce que la revue d'incident a demandé aux autres équipes

- Produit commun : vérification d'unité dans le collecteur d'événements (livré en mars).
- App conducteur : l'équipe ML relit tout changement de `driver.state_reported` et des trois autres événements sources du groupe `eta`.
- Data : la version d'app est une colonne de `raw.product_events` extraite de l'enveloppe, indexée, pour que « par version d'app » soit une requête de 2 secondes et non de 2 minutes ; livré avec la migration 0161.
