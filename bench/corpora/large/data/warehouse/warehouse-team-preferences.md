---
name: warehouse-team-preferences
description: Warehouse team preferences: plain SQL models, one model per table, every column in the registry, recompute not patch, LTS only
type: user
status: active
verified: 2026-02-25
---

Préférences de l'équipe entrepôt (trois personnes), issues d'un an de marmot et d'un incident.

- **SQL plat, sans templating.** Un modèle est un fichier SQL lisible tel quel, avec les deux seuls remplacements `{{ partition }}` et `{{ last_run }}` ([[marmot-model-runner]]). Une logique répétée entre modèles devient une fonction SQL dans `dict` ou une colonne dans un modèle amont, pas une macro.

- **Une table, un modèle, un fichier**, nommé comme la table. Pas de modèle qui écrit deux tables.

- **Chaque colonne a une ligne dans `models/registry.yaml`** avec une description en une phrase et son unité (cents, secondes, kilomètres). Une colonne sans description est refusée en revue. Le catalogue des analystes est généré de là.

- **On recalcule une partition, on ne corrige pas des lignes.** Un `ALTER TABLE UPDATE` sur `core` ou `marts` est réservé à l'effacement ([[gdpr-erasure-pipeline]]). Tout le reste passe par [[backfill-runbook]].

- **Version LTS de ClickHouse uniquement**, montée une réplique à la fois, shard 2 d'abord, jamais le vendredi ni pendant une clôture mensuelle.

- **Les analystes ont `scratch`, pas `core`.** Une table utile dans `scratch` devient un modèle avec une revue ; elle ne reste pas six mois dans `scratch` à alimenter un tableau de bord.

- **Les chiffres publiés portent leur `computed_at`.** Un tableau de bord sans « données au » n'est pas mis en production.

- **Un test qui échoue alerte, sauf pour ce que la finance lit.** Pour `marts.invoice_mart`, un test qui échoue bloque ([[invoice-mart]]). On accepte que ce soit asymétrique.

- **Langue** : les notes et les descriptions de colonnes en français ou en anglais selon qui écrit, les noms de colonnes en anglais, toujours.

- **Astreinte data** : une semaine à tour de rôle, page seulement sur le lag d'ingestion, le disque et les écarts de réconciliation ; tout le reste attend le matin.

