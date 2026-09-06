---
name: late-payment-interest-fr
description: French late-payment interest is ECB refinancing rate plus 10 points, plus a fixed 40 EUR recovery indemnity per invoice, computed daily from due date and added as a line on the next invoice
type: reference
status: active
verified: 2026-01-30
---

Pour l'entité FR, les pénalités de retard sont dues de plein droit dès le lendemain de l'échéance (art. L441-10 du Code de commerce), sans mise en demeure. On les applique à partir du niveau 2 de la relance ([[dunning-schedule]]), c'est-à-dire échéance + 10 jours, par choix commercial, mais le calcul remonte à l'échéance.

## Calcul

- Taux : taux de refinancement de la BCE en vigueur au 1er janvier ou au 1er juillet du semestre, majoré de 10 points. Stocké dans `late_interest_rates` (`entity`, `valid_from`, `annual_rate`). Pour le premier semestre 2026 : 2,15 % + 10 = 12,15 %.
- Intérêt = montant TTC impayé × taux annuel × jours de retard / 365. Calculé par `LateInterestCalculator` sur chaque facture ouverte, chaque jour, cumulé dans `invoices.accrued_interest_cents` (non facturé tant que la facture reste ouverte).
- Indemnité forfaitaire de recouvrement : 40 EUR par facture en retard, une seule fois, quel que soit le montant. Elle s'ajoute même si la facture fait 12 EUR, ce qui a surpris un chargeur en décembre 2025 et a été maintenu après avis du cabinet.

Les deux montants sont hors champ de TVA.

## Facturation

Les intérêts et l'indemnité sont ajoutés comme lignes sur la prochaine facture de commission du chargeur, avec la mention « Pénalités de retard sur facture HF-FR-2026-000123 (échéance 2026-01-15, réglée le 2026-02-03) ». On ne fait pas de facture séparée pour les pénalités : les chargeurs les payent plus volontiers noyées dans une facture de commission que sur un document dédié (constat après trois mois de factures séparées début 2025, 61 % de contestation contre 9 %).

Support et finance peuvent annuler les pénalités d'une facture (`POST /internal/invoices/{id}/waive-interest`, avec motif). Fait dans 35 % des cas environ, presque toujours pour un premier retard. Un compte avec plus de deux annulations en 12 mois n'en obtient plus sans validation finance.

## Autres entités

- DE : § 288 BGB, taux de base de la Bundesbank + 9 points pour les transactions B2B, indemnité de 40 EUR aussi. Même calculateur, table de taux différente.
- PL : taux de référence NBP + 10 points, indemnité de 40 EUR convertie en PLN au taux du dernier jour ouvré du mois précédent l'échéance (ce détail vient de la loi polonaise sur les délais de paiement et nous a coûté un aller-retour avec le cabinet).
- NL : taux BCE + 8 points, indemnité 40 EUR.

Le calculateur prend l'entité de la facture, pas le pays du chargeur : un chargeur allemand facturé par l'entité FR se voit appliquer les règles françaises, comme le prévoient nos CGV.
