---
name: audit-trail-retention-and-export
description: audit_events partitions stay 13 months online, then are exported as gzip NDJSON with a checksum manifest to hf-audit-archive and kept 6 years
type: project
status: active
verified: 2026-06-10
---

# Rétention et export de l'audit (HF-2118)

La table `audit_events` (voir [[audit-trail-schema]]) est partitionnée par mois. La question posée par le DPO fin 2025 : combien de temps garde-t-on ces lignes, et sous quelle forme. La réponse tenue depuis HF-2118 (janvier 2026).

## Règle

- **13 mois en ligne** dans Postgres. Assez pour une revue d'accès annuelle avec un mois de marge. Au-delà, la partition est détachée et supprimée.

- **6 ans en archive froide**. Le mois détaché est exporté avant suppression dans le bucket `hf-audit-archive` (compte de stockage séparé de la prod, accès en écriture uniquement pour le job, lecture pour deux personnes nommées). Six ans parce que c'est la durée de prescription la plus longue qui nous concerne, alignée sur la matrice de rétention du projet conformité.

- Les événements dont `details` contient des données personnelles au sens large (`export.personal_data`, `member.*`) ne sont pas traités différemment : l'archive est chiffrée côté stockage, l'accès est nominatif, et c'est le mécanisme d'effacement du projet conformité qui décide.

## Le job

`audit:archive:month 2025-04`, lancé par un CronJob le 5 de chaque mois pour le mois M-13.

1. `SELECT ... FROM audit_events_2025_04 ORDER BY id` en curseur serveur, 10 000 lignes par lot, écrit en NDJSON gzip dans `audit-events/2025/04/part-0001.ndjson.gz` (un fichier par 500 000 lignes).

2. Un manifeste `audit-events/2025/04/MANIFEST.json` avec le nombre de lignes, le premier et le dernier `id`, le SHA-256 de chaque fichier, et la version du schéma de la table à ce moment-là (le DDL complet, parce que les colonnes bougent).

3. Relecture de vérification : le job relit chaque fichier, recompte, compare au manifeste et au `count(*)` de la partition. Écart non nul : le job s'arrête, la partition n'est pas touchée, alerte sur le canal sécurité.

4. `ALTER TABLE audit_events DETACH PARTITION audit_events_2025_04` puis `DROP TABLE`, seulement si l'étape 3 est passée.

Durée mesurée pour avril 2025 (le premier mois traité, 2,1 millions de lignes) : 11 minutes, 184 Mo compressés.

## Restauration

`audit:archive:restore 2025-04 --into audit_events_restored` recrée une table hors partitionnement avec le DDL du manifeste, recharge, vérifie les checksums. On l'a fait une fois pour de vrai, en mars 2026, pour une demande d'un chargeur qui voulait savoir qui avait modifié un rôle en février 2025. Vingt minutes, dont quinze à retrouver qui avait le droit de lire le bucket.

## Ce qu'on a écarté

- Garder tout en ligne « parce que le disque est pas cher ». Le problème n'est pas le disque, c'est qu'une table de 25 millions de lignes que personne ne consulte au-delà d'un an reste dans les sauvegardes, les réplicas et les exports vers l'entrepôt.

- Exporter en Parquet. Plus compact, mais le manifeste et la restauration deviennent dépendants d'une bibliothèque. NDJSON se relit avec `zcat | jq` dans dix ans.

Le manifeste et les checksums ont été demandés explicitement par la revue d'accès du projet conformité, qui voulait pouvoir prouver qu'une archive n'a pas été modifiée.
