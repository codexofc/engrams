---
name: restore-2026-02-postgres-pitr-billing
description: Février 2026: migration fausse sur 84 000 lignes de invoice_lines, PITR sur un cluster parallèle à T-3 min, réinjection ciblée, 2 h 50, prod jamais arrêtée
type: project
status: active
verified: 2026-03-04
---

# Restauration PITR du 2026-02-24 : lignes de facture

## Ce qui s'est passé

Migration `Version20260224091500` de la facturation, déployée à 09:22 UTC : elle devait recalculer `invoice_lines.vat_amount_cents` pour les 3 100 lignes d'un expéditeur passé en autoliquidation. La clause `WHERE` avait `shipper_id = :id OR vat_scheme = 'reverse_charge'`, et `vat_scheme` valait `'reverse_charge'` pour 84 000 lignes historiques d'autres expéditeurs, déjà facturées, dont la TVA a été mise à zéro. La migration a été validée en transaction, pas de retour arrière possible. Détecté à 09:31 par la règle de qualité `invoices_amount_eur_consistent` de l'entrepôt, qui a vu 800 factures dont le total ne correspondait plus à la somme des lignes dès la projection CDC suivante.

Aucune facture n'a été émise entre 09:22 et 09:31 (la facturation émet par lots à 12:00), donc aucun document faux n'a quitté la maison. Mais les 84 000 lignes étaient fausses dans la base de production, et une facture consultée dans l'espace client aurait montré une TVA à zéro.

## Pourquoi une restauration et pas une requête

La valeur juste de `vat_amount_cents` n'était pas recalculable sans risque : le taux dépend de la date de la facture, du pays, du régime de l'expéditeur au moment de la facture, et trois de ces règles avaient changé depuis 2024. La seule source certaine de la valeur d'avant 09:22 était la base d'avant 09:22. Le mécanisme PITR ([[backup-inventory-and-retention]], ligne PostgreSQL) existe pour ça, et il avait été exercé sur un cluster jetable mais jamais utilisé en vrai.

## Ce qui a été fait

1. 09:40 : décision de restaurer en parallèle, pas en place. Une restauration en place aurait arrêté l'API et perdu 20 minutes de transactions légitimes (enchères, statuts) pour corriger une table.

2. 09:45 : création d'un cluster PostgreSQL `hf-postgres-restore` par l'opérateur, à partir de la base quotidienne de 01:00 et des WAL de `hf-pg-backups-prod`, `recoveryTarget: 2026-02-24T09:19:00Z` (trois minutes avant la migration, marge sur l'horloge). Le manifeste est celui du runbook, avec la cible changée. Ressources : 8 vCPU, 32 GB, sur `hf-main` dans le namespace `ops-restore`.

3. 09:47 à 10:55 : rejeu des WAL. 8 h 19 de WAL à environ 1,2 GB par 10 minutes, 60 GB, 68 minutes. Plus lent qu'à l'exercice (cluster jetable moins chargé) mais dans l'ordre de grandeur attendu.

4. 10:55 : cluster restauré en lecture seule, `SELECT count(*) FROM invoice_lines WHERE vat_scheme = 'reverse_charge' AND vat_amount_cents > 0` donne 84 112, contre 0 en prod. Bon point de restauration.

5. 11:00 : extraction `COPY (SELECT id, vat_amount_cents, updated_at FROM invoice_lines WHERE id IN (...84 112 ids depuis la prod...)) TO STDOUT` vers un fichier sur `ops-tools`, 4 MB, hash noté dans le ticket.

6. 11:10 : sur la prod, dans une transaction, `CREATE TEMP TABLE fix (...)`, `COPY fix FROM STDIN`, puis `UPDATE invoice_lines il SET vat_amount_cents = f.vat_amount_cents FROM fix f WHERE il.id = f.id AND il.vat_scheme = 'reverse_charge' AND il.vat_amount_cents = 0 AND il.updated_at >= '2026-02-24T09:22:00Z'`. La double condition sur `updated_at` et la valeur zéro garantit qu'on ne touche que ce que la migration a touché. 84 112 lignes mises à jour. Vérification des totaux de 20 factures au hasard contre le PDF déjà émis : identiques. Validation à 11:25.

7. 11:30 : la règle de qualité repasse au vert à la projection CDC suivante. Le CDC a émis 84 112 messages `op: u`, l'entrepôt les a absorbés en 3 minutes.

8. 12:10 : cluster `hf-postgres-restore` supprimé. La migration a été corrigée (`AND` à la place du `OR`, avec un test qui compte les lignes touchées et refuse au-dessus de 5 000 sans drapeau explicite) et rejouée à 12:20 pour les 3 100 lignes prévues.

## Ce que ça a coûté

2 h 50 entre la détection et la validation de la correction, dont 68 minutes de rejeu de WAL. Aucune interruption de l'API. Aucune facture émise fausse. Une personne à plein temps, une seconde pour la relecture des requêtes avant chaque exécution sur la prod (règle posée pendant l'incident : personne n'exécute seul une écriture sur la prod pendant une restauration).

## Ce qui a changé (HF-4610)

- Le runbook PITR a un chapitre « restauration parallèle pour correction ciblée », avec les étapes ci-dessus et la requête type de réinjection à double condition.

- Les migrations qui font un `UPDATE` sans `LIMIT` sur une table de plus d'un million de lignes passent par un `EXPLAIN` et un compte des lignes touchées en staging, collé dans la MR. Le lint des migrations le demande.

- Le rejeu de WAL à 1,2 GB par 10 minutes est le chiffre à retenir pour estimer : une restauration à J-1 en fin de journée prend environ 3 heures. L'exercice de mai ([[restore-drill-2026-05]]) a mesuré la même valeur.

- La base de 01:00 est complétée par une base à 13:00 depuis mars, pour diviser par deux le pire cas de rejeu. Coût : 60 GB de plus par jour dans `hf-pg-backups-prod`, dans les limites du plan de capacité ([[storage-capacity-plan-2026]]).
