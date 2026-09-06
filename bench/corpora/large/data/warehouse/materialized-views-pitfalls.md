---
name: materialized-views-pitfalls
description: Why 14 of 16 materialized views were removed in 2025: they see only the inserted block, do not replay, and multiply insert cost
type: feedback
status: active
verified: 2025-11-28
---

En 2025 on a supprimé 14 des 16 vues matérialisées ClickHouse de l'entrepôt. Les deux qui restent sont justifiées ci-dessous. Ce qu'on a appris, pour ne pas les recréer.

## Ce qu'une vue matérialisée fait vraiment

Une vue matérialisée ClickHouse est un déclencheur d'insertion : elle voit **le bloc inséré**, et seulement lui, et écrit le résultat de sa requête dans la table cible. Conséquences qu'on a toutes payées :

- **Elle ne voit pas le passé.** Créer une vue ne remplit rien ; il faut la peupler à la main (`INSERT INTO cible SELECT ... FROM source`), et si la source reçoit une correction plus tard, la vue a déjà écrit son agrégat. Les doublons de l'incident de janvier ([[duplicate-bids-incident-2026-01]]) auraient été impossibles à corriger dans une vue matérialisée agrégée sans la reconstruire entièrement ; dans un modèle marmot, on recalcule la partition ([[late-arriving-events]]).

- **Une jointure dans une vue matérialisée est jointe au moment de l'insertion**, avec l'état de la table jointe à ce moment-là. Une vue qui enrichissait les enchères avec le cluster de ligne a produit six semaines de `lane_cluster_id` nuls parce que le dictionnaire était rechargé après l'insertion.

- **Le coût d'insertion se multiplie.** Trois vues sur `raw.product_events` triplaient le temps d'insertion et ont contribué à la saturation d'octobre 2025 ([[disk-full-incident-2025-10]]).

- **Le SQL de la vue est invisible dans le DAG.** marmot ne connaissait pas les vues, un analyste a supprimé une colonne source et la vue a cassé silencieusement (les insertions dans la source réussissaient, la vue échouait en arrière-plan).

## Les deux qu'on garde

1. `raw.product_events_by_minute` : compteur par minute et par nom d'événement, `AggregatingMergeTree`. Sert au monitoring de la collecte (un événement qui tombe à zéro). Pas de jointure, pas de correction rétroactive utile, et la latence de 6 s de l'ingestion est ce qu'on veut ici.
2. `raw.driver_positions_latest` : dernière position par conducteur, `ReplacingMergeTree`. Le tableau de bord dispatch la lit toutes les 30 secondes et un modèle marmot à 10 minutes est trop lent.

Dans les deux cas : pas de jointure, une seule source, et une reconstruction complète documentée dans `models/raw/README` qui prend moins d'une heure.

## Comment appliquer

Avant de créer une vue matérialisée, répondre par écrit à trois questions : que se passe-t-il quand la source reçoit une correction, combien de temps prend la reconstruction depuis zéro, et pourquoi un modèle marmot à 10 minutes ne suffit pas. Si la troisième réponse est « ce serait plus élégant », c'est non.
