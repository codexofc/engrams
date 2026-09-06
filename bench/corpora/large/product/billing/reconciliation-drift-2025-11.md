---
name: reconciliation-drift-2025-11
description: November 2025 incident: Payla swapped gross and net columns, 6 days of false mismatches, header-name reading and sum checks added
type: project
status: active
verified: 2025-12-15
---

## Ce qui s'est passé

Entre le 2025-11-04 et le 2025-11-10, le rapport de règlement quotidien de Payla est arrivé avec les colonnes `gross_amount` et `net_amount` inversées par rapport à la documentation. L'importateur `PaylaSettlementImporter` lisait les colonnes par position (indice 7 pour le brut, 8 pour le net) depuis la première version, parce que le premier fichier reçu en mars 2025 n'avait pas d'en-tête.

Résultat : pendant six jours, le job de réconciliation ([[reconciliation-nightly-job]]) comparait nos paiements bruts au montant net de Payla. Chaque prélèvement SEPA sortait avec un écart de 0,35 EUR, chaque paiement carte avec un écart de 1,4 % + 0,25 EUR. La file `reconciliation_unmatched` est passée de 30 lignes à 2 900 lignes le premier matin, avec la raison `amount_mismatch`.

La finance a d'abord pensé à une hausse de frais non annoncée et a ouvert un ticket chez Payla. Le vrai diagnostic est venu le 2025-11-06 en comparant un fichier d'octobre et un fichier de novembre ligne à ligne : l'en-tête avait changé d'ordre, et il y avait désormais un en-tête.

Montant total des faux écarts sur la période : 41 230 EUR, aucun impact réel sur la trésorerie, mais trois jours de travail de finance pour vérifier que les prélèvements étaient bien complets.

## Correction (HF-2140)

- L'importateur lit désormais les colonnes par nom d'en-tête, et refuse le fichier (`SettlementFormatException`) si une colonne attendue manque ou si l'en-tête est absent. Le job envoie alors l'alerte `payla_settlement_format` et ne touche pas à la réconciliation du jour.
- Un contrôle de cohérence : sur un fichier, `sum(gross) - sum(net)` doit être égal à `sum(fee)` à 1 cent près par ligne. Un écart supérieur fait échouer l'import.
- Un test à partir des deux fichiers réels anonymisés (`fixtures/payla/settlement-2025-10-28.csv` et `settlement-2025-11-04.csv`) qui doit donner les mêmes montants bruts.

## Re-réconciliation

Les six jours ont été rejoués avec `billing:reconcile --date <jour> --source payla --force` après la correction, dans l'ordre chronologique. Les 2 900 lignes ont été appariées au tiers 1 sauf 12, qui étaient de vrais écarts (deux remboursements partiels et dix prélèvements rejetés tardivement).

## Ce qu'on retient

Payla ne prévient pas des changements de format de fichier ; leur changelog API ne couvre pas les exports. On a ajouté une lecture hebdomadaire de leur page « exports » dans la routine du lundi de l'équipe billing, et le contrat de support Payla a été renégocié pour inclure un préavis de 30 jours sur les formats de fichiers, effectif depuis janvier 2026.
