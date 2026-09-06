---
name: billing-runbook-month-close
description: Month-close runbook, the order of the six steps from freezing loads at 00:00 entity time to the JPK and DATEV exports on business day 3, and the checks that block each step
type: reference
status: active
verified: 2026-07-02
---

La clôture mensuelle produit les factures de commission non encore émises, les exports comptables par entité et le rapport de réconciliation du mois. Elle est pilotée par `MonthCloseOrchestrator`, lancée le 1er du mois à 02:00 dans le fuseau de chaque entité (voir [[billing-cutoff-timezone]]), et se termine par des exports manuels au jour ouvré 3.

## Étapes

1. **Gel du périmètre** (automatique, 1er à 00:00 heure de l'entité). Tout chargement dont `delivered_at` est dans le mois clos et dont la contestation est expirée entre dans le périmètre. Le job écrit `month_close_runs` avec `entity`, `month`, `loads_in_scope`. Un chargement livré le dernier jour à 23:50 est dedans ; contesté après, il donne un avoir le mois suivant.
2. **Émission des factures restantes** (automatique, 02:00). Les factures de commission sont normalement émises au fil de l'eau, mais les chargeurs en facturation à échéance (`billing_mode = 'end_of_month'`, environ 8 % des comptes) reçoivent une facture par chargement, toutes émises cette nuit-là. Volume janvier 2026 : 9 212 factures en 17 min 40 s. Blocage si `billing.invoice_sequence.gaps > 0`.
3. **Rendu PDF et envoi** (automatique, à la suite). Les envois par email sont étalés jusqu'à 08:00 pour ne pas dépasser le quota du fournisseur d'envoi (12 000 par heure). Voir [[invoice-pdf-rendering]].
4. **Réconciliation du mois** (jour ouvré 1, 07:45). Le job quotidien ([[reconciliation-nightly-job]]) tourne normalement ; la clôture ajoute le rapport `reconciliation_monthly` qui liste les paiements non appariés de plus de 15 jours. Blocage de l'étape 6 si le total non apparié dépasse 0,5 % des encaissements du mois.
5. **Revue finance** (jour ouvré 2). Avoirs en attente d'approbation, pénalités à annuler, intérêts courus (voir [[late-payment-interest-fr]]). Rien d'automatique, une checklist dans le back-office `/backoffice/month-close/{entity}/{month}`.
6. **Exports** (jour ouvré 3). `billing:export:datev --entity DE --month 2026-06` pour l'Allemagne, `billing:export:jpk --entity PL --month 2026-06` pour la Pologne (fichier JPK_V7M), `billing:export:fec --entity FR --month 2026-06` pour la France (FEC mensuel, le FEC annuel étant reconstruit à partir des mensuels). Chaque export vérifie que la somme des factures émises égale la somme des lignes exportées, au cent près, sinon il refuse d'écrire le fichier.

## Vérifications avant l'étape 6

- `select count(*) from invoices where status = 'issued' and pdf_key is null and issued_at >= <mois>` doit être 0.
- `select count(*) from invoices where vies_stale and issued_at >= <mois>` : à revoir une par une, généralement moins de 10.
- Le nombre de factures du mois dans le rapport doit correspondre au nombre attendu par `month_close_runs.loads_in_scope` plus les factures au fil de l'eau ; l'écart est journalisé et expliqué dans le ticket de clôture.

## Réouverture

Une clôture n'est jamais rouverte. Une correction après export passe par un avoir daté du mois courant, et l'export du mois suivant porte la correction. Le cabinet PL l'exige, les autres l'acceptent.

Ticket de clôture : un ticket `HF-` par mois et par entité, avec les chiffres des étapes 1, 2 et 4 en commentaire. Le dernier en date sert de gabarit.
