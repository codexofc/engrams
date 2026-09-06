---
name: alerting-rules-v1
description: Historical alert rules set (2024 to autumn 2025) with four severities and no runbook requirement, replaced by the two-severity catalogue with tested rules
type: reference
status: archived
superseded_by: [[alerting-rules-catalogue]]
verified: 2025-10-01
---

# Règles d'alerte v1 (jusqu'en octobre 2025)

Un seul fichier `alerts.yaml` de 900 lignes, quatre sévérités (`info`, `warning`, `critical`, `page`), pas d'annotation `runbook` obligatoire, pas de tests.

Ce qui n'allait pas :

- 140 règles dont une trentaine n'avaient jamais déclenché, et une dizaine déclenchaient tous les jours sans que personne n'agisse (le bruit qui fait ignorer le reste).

- `critical` et `page` étaient utilisés de façon interchangeable selon l'auteur. L'astreinte recevait des `critical` la nuit pour des choses qui pouvaient attendre le matin.

- Les seuils avaient été copiés de règles publiques génériques sans être adaptés (par exemple une alerte sur 80 % de CPU nœud, qui est notre régime normal le lundi matin).

- Aucun test : trois règles avaient une expression qui ne retournait jamais rien à cause d'un label renommé.

La refonte (HF-INFRA-320, octobre 2025) a produit [[alerting-rules-catalogue]] : deux sévérités, un fichier par équipe, runbook obligatoire pour `page`, tests `promtool`. Le nombre de règles est passé de 140 à 74, et le nombre de pages par mois de 22 à 4.
