---
name: retention-matrix-2025
description: The first retention matrix (2025) kept raw GPS 12 months and contact persons for the life of the load with no enforcement job; replaced in 2026-01
type: reference
status: archived
superseded_by: [[data-retention-matrix]]
verified: 2025-09-15
---

# Matrice de rétention 2025 (remplacée)

Première version, écrite en juin 2025 pour répondre au premier questionnaire de sécurité d'un grand chargeur. Remplacée par [[data-retention-matrix]] en janvier 2026. Conservée parce que les durées ci-dessous expliquent l'état des données qu'on a purgées au premier passage de `compliance:purge`.

## Ce qu'elle disait

- Positions GPS : 12 mois brutes. Justification écrite : « litiges et amélioration des ETA ». Aucune agrégation.

- Personnes de contact sur les chargements : durée de vie du chargement, donc 10 ans.

- Comptes inactifs : jamais pseudonymisés, « pour permettre la réactivation ».

- Messages dispatcher-chauffeur : non listés (la table n'existait pas encore en juin 2025, elle est arrivée en septembre et personne n'a mis la matrice à jour).

- Logs : 90 jours.

- Exports clients : pas de TTL, le bucket `hf-exports` a atteint 900 Go en décembre 2025.

## Ce qui n'allait pas

Trois problèmes soulevés par la revue du DPO en novembre 2025 :

1. Aucune règle n'était appliquée par un mécanisme. La matrice décrivait une intention, la base gardait tout. Au 2025-12-01, la table `positions` contenait 2,9 milliards de lignes, remontant à 2023.

2. Douze mois de positions brutes d'un chauffeur nommé sans fondement autre que « c'est pratique ». Voir [[driver-position-legal-basis]] pour la discussion qui a suivi.

3. Les personnes de contact tierces conservées dix ans, sans qu'elles aient de relation avec nous.

## Le premier passage de purge

Janvier 2026, en trois nuits, avec les règles de la nouvelle matrice : 2,6 milliards de positions supprimées par détachement de partitions (les partitions antérieures à octobre 2025), 1,1 million de chargements dont les contacts ont été mis à null, 780 Go d'exports supprimés. Le détail est dans le ticket HF-2092.
