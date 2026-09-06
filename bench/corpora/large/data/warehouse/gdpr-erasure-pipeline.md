---
name: gdpr-erasure-pipeline
description: Erasure requests processed weekly by the Erasure job with lightweight deletes in raw and mutations in core, proof row per request
type: project
status: active
verified: 2026-05-13
---

## Entrée

Le service de protection des données crée une ligne dans `privacy.erasure_requests` (base applicative, répliquée par CDC dans `raw.cdc_privacy_erasure_requests`) : `request_id`, `subject_type` (`carrier_contact`, `driver`, `shipper_contact`, `recipient`), `subject_ids` (les identifiants concernés dans chaque système), `received_at`, `deadline_at` (30 jours), `status`.

L'entrepôt n'est pas le premier système à traiter la demande : l'app efface d'abord, puis le CDC apporte les mises à jour (les colonnes deviennent nulles ou pseudonymes dans `core` au prochain run). Ce qui reste à traiter dans l'entrepôt, c'est l'historique : les lignes `raw` antérieures à l'effacement, et les partitions `core` déjà pseudonymisées ou non.

## Le job

`Erasure` tourne le samedi 02:00, une fois par semaine, sur toutes les demandes `status = 'pending_warehouse'` :

1. Pour chaque table listée dans `privacy/erasure_map.yaml` (table, colonnes personnelles, colonne d'identifiant du sujet), construire le prédicat.

2. `raw.*` : `DELETE FROM raw.x WHERE subject_col IN (...)` (suppression légère, disponible depuis la 23.3, marquée puis appliquée à la fusion). Les lignes CDC brutes d'un sujet effacé n'ont plus de raison d'exister ; on ne les pseudonymise pas, on les supprime.

3. `core.*` : `ALTER TABLE core.x UPDATE name = '', phone = '', email = '' WHERE ...` (mutation), parce que la ligne elle-même garde un sens analytique (un chargement a bien eu lieu) et que seules les colonnes personnelles partent. Une position GPS de conducteur est supprimée, pas mise à jour : elle n'a pas de sens sans conducteur.

4. Attendre la fin des mutations (`system.mutations`, `is_done`), en général 1 à 4 heures par shard, en parallèle sur les deux shards.

5. Vérifier par une requête de contrôle que plus aucune ligne ne contient les identifiants, écrire une ligne dans `privacy.erasure_proofs` (`request_id`, `tables`, `rows_affected`, `completed_at`) et passer la demande à `done_warehouse`.

Le délai réel : demande reçue un mardi, traitée le samedi, preuve écrite le dimanche. Toujours sous 30 jours, et l'alerte `erasure.deadline_close` se déclenche à J-7 si une demande est encore `pending_warehouse`.

## Ce qui ne peut pas être effacé

- Les sauvegardes de l'entrepôt : 35 jours de rétention, l'effacement se propage par expiration. C'est écrit dans la réponse au demandeur.
- `marts.*` agrégés : pas de donnée personnelle, rien à faire ; un test dans marmot (`no_pii_columns`) vérifie qu'aucun modèle `marts` n'a de colonne listée dans `erasure_map.yaml`.
- Les données déjà pseudonymisées à 24 mois ([[retention-rules]]) : le hachage salé n'est pas réversible et le sel tourne chaque année, on considère la pseudonymisation comme un effacement, avis du délégué de novembre 2025.

## Volumes

2026 à ce jour : 41 demandes, presque toutes des conducteurs qui ont quitté un transporteur, 3 destinataires de livraison. Aucune demande d'un chargeur. Le job a pris au maximum 5 h 20 (une demande portant sur 4 identifiants conducteur avec 2 ans de positions GPS).

## Mutations et fusions

Les mutations du samedi sont la raison pour laquelle le passage hebdomadaire de [[late-arriving-events]] est le dimanche et non le samedi : deux charges lourdes le même jour dépassaient la fenêtre de faible activité. Voir [[partitioning-and-ttl]] pour le coût des mutations.
