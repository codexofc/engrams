---
name: invoice-numbering-sequence
description: Invoice numbers per legal entity and year from invoice_sequences with SELECT FOR UPDATE, no gaps, format HF-FR-2026-000123
type: reference
status: active
verified: 2026-03-14
---

Le numéro de facture est alloué par la table `invoice_sequences` (colonnes `legal_entity_id`, `fiscal_year`, `last_value`), une ligne par entité juridique et par année. L'allocation se fait dans la même transaction que l'insertion de la facture, avec `SELECT ... FOR UPDATE` sur la ligne de séquence. Pas de séquence Postgres native : une séquence Postgres n'est pas transactionnelle et laisse des trous au moindre rollback, ce qui est interdit en France (CGI art. 242 nonies A) et en Allemagne (GoBD).

Format : `HF-<pays de l'entité>-<année>-<compteur sur 6 chiffres>`. Exemples : `HF-FR-2026-000123`, `HF-DE-2026-004410`, `HF-PL-2026-000007`. Le compteur repart à 1 chaque 1er janvier à 00:00 dans le fuseau de l'entité (Europe/Paris pour FR, Europe/Warsaw pour PL). Voir [[billing-cutoff-timezone]] pour l'incident qui a fixé cette règle.

## Ce qui a remplacé l'ancien schéma

L'ancien schéma (un seul compteur global, préfixe `INV-`) est décrit dans [[invoice-numbering-legacy]]. La migration HF-2211 a rejoué les 41 806 factures existantes en leur attribuant un numéro dans le nouveau format, en conservant l'ancien dans `invoices.legacy_number` pour les rapprochements avec les paiements Payla antérieurs à novembre 2025.

## Règles pratiques

- Une facture annulée n'est jamais supprimée : on émet un avoir (voir [[credit-notes-flow]]) qui consomme son propre numéro dans la séquence `CN` (`HF-FR-CN-2026-000045`).
- Les brouillons (`invoices.status = 'draft'`) n'ont pas de numéro, la colonne `number` est NULL et la contrainte `invoices_number_unique` est partielle (`WHERE number IS NOT NULL`).
- L'allocation en lot (clôture mensuelle, jusqu'à 9 000 factures en 20 minutes) prend le verrou par lots de 200 pour ne pas tenir la ligne de séquence pendant toute la clôture. Mesuré le 2026-02-01 : 9 212 factures en 17 min 40 s, contention nulle sur `invoice_sequences`.
- Le test `InvoiceSequenceGapTest` compte les trous chaque nuit sur la veille et pousse la métrique `billing.invoice_sequence.gaps` : elle doit rester à 0. Une valeur non nulle réveille l'astreinte billing.

## Pièges

Les tests d'intégration qui créent des factures en parallèle sur la même entité doivent s'attendre à une sérialisation : un test qui suppose deux numéros consécutifs pour deux threads est faux par construction. Utiliser `InvoiceFactory.withAllocatedNumber()` qui isole une entité de test par cas.
