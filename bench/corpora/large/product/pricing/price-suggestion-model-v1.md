---
name: price-suggestion-model-v1
description: The v1 price suggestion was a linear model per distance band on 9 features with an absolute EUR target, MAPE 13.1 %, retired in January 2026 for the gradient boosted v2
type: project
status: archived
superseded_by: [[price-suggestion-model-v2]]
verified: 2025-11-12
---

Première version du modèle de suggestion de prix, en production de mars 2025 à janvier 2026.

## Modèle

Une régression linéaire par bande de distance (quatre modèles), neuf features : distance, type de véhicule (one-hot), jour de semaine, heures avant chargement, indice carburant, médiane du cluster, pays d'origine, pays de destination, nombre d'arrêts. Cible : le prix attribué en euros, hors majorations.

MAPE de 13,1 % sur les deux dernières semaines de novembre 2025. Les résidus croissaient avec la distance : sur les chargements de plus de 800 km, l'erreur médiane était de 190 EUR, contre 35 EUR sous 150 km. C'est ce qui a motivé le passage à une cible en prix au kilomètre en logarithme dans [[price-suggestion-model-v2]].

## Pourquoi on l'a gardé si longtemps

Il était lisible : un chargeur qui demandait pourquoi la suggestion était à 1 480 EUR obtenait une réponse avec les coefficients. La v2 a perdu cette lisibilité et on a compensé par la décomposition en majorations, qui reste lisible, et par une explication en trois facteurs (« distance, ligne peu desservie, chargement le lundi ») calculée par contribution des features.

## Ce qui a été réutilisé

- La construction du jeu d'entraînement depuis l'entrepôt (pas depuis la base de production), avec exclusion des enchères dont la décomposition est incohérente.
- Le fichier de 100 chargements de référence pour la validation au chargement du modèle.
- La règle du repli sur la médiane du cluster quand le modèle est absent.

Le code est dans l'historique de `pricing-svc` sous le tag `price-model-v1-final`. Il ne tourne plus nulle part.
