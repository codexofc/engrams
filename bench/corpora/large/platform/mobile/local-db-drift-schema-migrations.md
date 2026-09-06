---
name: local-db-drift-schema-migrations
description: Local SQLite schema is managed by drift with a schemaVersion integer, step-by-step migrations in lib/db/migrations.dart, exported schema JSON per version tested by the drift migration test, destructive resets are never allowed
type: reference
status: active
verified: 2026-04-14
---

# Schéma local (drift) et migrations

Base : `drift` avec `sqlite3_flutter_libs`, fichier `halden_driver.db` dans le répertoire de documents de l'app. Schéma à la version 19 (app 4.9).

## Règles

- **Chaque changement de schéma incrémente `schemaVersion`** dans `AppDatabase` et ajoute un bloc `from(n, n+1)` dans `lib/db/migrations.dart` avec `Migrator.stepByStep`. On ne saute jamais une étape, même triviale.

- **Jamais de reset destructif.** Un chauffeur peut avoir des mutations en attente dans l'outbox (voir [[offline-sync-architecture]]) au moment de la mise à jour. Supprimer la base = perdre des livraisons confirmées. La seule exception tolérée est si la migration échoue avec une exception : on copie la base dans `halden_driver.broken.db`, on remonte un événement `db.migration_failed` avec la version, et on repart d'une base vide. C'est arrivé 3 fois sur 3 400 appareils depuis 2025, à chaque fois sur un appareil avec le stockage plein.

- **Pas de `ALTER TABLE ... DROP COLUMN`** dans les migrations : SQLite le supporte depuis 3.35 mais les versions embarquées sur Android 10 ne l'ont pas toutes. On recrée la table (`createTable` + `INSERT INTO ... SELECT` + `DROP` + `RENAME`) avec `TableMigration` de drift.

- **Les index se créent dans la migration**, pas seulement dans la définition de table, sinon les installations existantes n'en ont pas. Erreur classique, on l'a faite pour `idx_pending_mutations_seq` en version 12.

## Test de migration

`drift_dev` exporte le schéma de chaque version dans `drift_schemas/drift_schema_v<n>.json` (`dart run drift_dev schema dump`). Le test `test/db/migration_test.dart` généré par `drift_dev schema generate` fait, pour chaque version de 1 à 19 : créer une base à cette version, insérer des données de test, migrer jusqu'à la version courante, vérifier que les données sont toujours lisibles. Le test tourne dans le CI et prend 4 s.

La PR qui oublie l'export de schéma échoue au CI (`drift_schemas` est comparé à la sortie de `schema dump`).

## Taille et entretien

- `VACUUM` n'est jamais lancé automatiquement, il bloque et peut prendre plusieurs secondes sur une base de 40 Mo. Il est déclenché uniquement par la purge des chargements de plus de 30 jours, la nuit, si l'app est ouverte et branchée.

- `PRAGMA journal_mode = WAL` activé, `synchronous = NORMAL`. Le mode WAL a supprimé les blocages de lecture pendant la synchronisation.

- Pas de chiffrement au repos de la base (pas de SQLCipher). Le contenu est des chargements et des adresses, le téléphone est chiffré par l'OS, et le jeton d'accès est dans le keychain, pas dans SQLite. Décision HF-1050, revue en 2025 par la sécurité, acceptée.

## Historique utile

- v7 → v8 (2025-06) : ajout de `stops.arrival_at_local` et `stops.tz`, voir la règle de fuseaux côté API.

- v12 → v13 (2025-11) : `pending_mutations.seq` autoincrement, la colonne qui rend l'ordre du push fiable.

- v16 → v17 (2026-01) : table `document_uploads` pour reprendre un envoi à l'étape où il s'est arrêté, suite à [[incident-2026-01-duplicate-pod-uploads]].
