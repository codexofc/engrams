---
name: missed-2026-04-sek-invoices-summed-as-eur
description: Avril 2026: 1 900 factures SEK additionnées comme des EUR dans le mart de chiffre d'affaires pendant six semaines, aucune règle sur la devise, HF-4530
type: project
status: active
verified: 2026-05-14
---

# Manqué : factures SEK additionnées comme des EUR, 2026-03-02 au 2026-04-14

## Ce qui s'est passé

Le 2026-03-02, l'équipe facturation a activé la facturation en couronnes suédoises pour les expéditeurs suédois (jusque-là facturés en EUR). `core.invoices` a bien reçu `currency = 'SEK'` et `amount_cents` en öre. Le mart `mart.revenue_monthly` faisait `sum(amount_cents) / 100` avec un filtre `currency = 'EUR'`… écrit en 2024, quand il n'y avait que des EUR, et **retiré en 2025** par quelqu'un qui l'avait trouvé redondant (« on n'a que des EUR »). Le mart a donc additionné 1 900 factures en öre comme si c'étaient des centimes d'euro : une facture de 45 000 SEK (4 100 EUR) comptait 45 000 EUR. Environ +11 fois sur ces factures, soit +5,2 M EUR sur un chiffre d'affaires mensuel de 38 M, +14 %.

## Pourquoi personne n'a rien vu pendant six semaines

- **Aucune règle sur la devise.** `core.invoices.currency` n'avait pas de `accepted_values` et le mart ne déclarait pas d'hypothèse sur la devise. Le framework ([[dq-framework-overview]]) était en place depuis octobre, les règles de facturation étaient des règles de clé et de fraîcheur.

- **Le mart était plausible.** +14 % sur le chiffre d'affaires de mars dans un mois où la Suède venait d'ouvrir et où le nombre de chargements montait de 6 %. La règle d'anomalie de volume ([[volume-anomaly-seasonal-thresholds]]) regardait le nombre de factures, pas les montants, et le nombre était normal.

- **La finance ne regarde pas ce mart.** Sa comptabilité vient de l'outil de facturation, pas de l'entrepôt. Le mart sert au produit et au commercial. La réconciliation Payla ([[reconciliation-payla-settlements]]) compare des règlements en devise d'origine et n'a rien vu non plus : elle était juste.

- **Personne n'avait relu le mart** quand la Suède a ouvert. Le ticket de facturation SEK listait les impacts sur l'API, la facturation, les notifications, pas sur l'entrepôt.

## Qui a vu

Le 2026-04-14, un commercial prépare une revue avec un expéditeur suédois à partir du tableau « chiffre d'affaires par client » et voit un client à 520 000 EUR sur mars, alors qu'il sait que le contrat est à 45 000. Il écrit au produit, le produit à l'équipe données, cause trouvée en 40 minutes.

## Ce qui a été fait (HF-4530)

- Le mart calcule `amount_eur_cents` à partir de `currency` et du taux du jour de `dim.fx_rates` (déjà utilisé par la facturation pour l'affichage), et `core.invoices` a maintenant une colonne `amount_eur_cents` calculée à la projection, pour que le mart n'ait plus à convertir.

- Trois règles :

  `invoices_currency_not_null` (`not_null`, `page`) ;

  `bids_currency_in_dim` et `invoices_currency_in_dim` (`accepted_values` contre `dim.currencies`, `page`) ;

  `invoices_amount_eur_consistent` (requête : `amount_eur_cents` égal à `amount_cents × taux` à 1 centime près, `page`), qui aurait fait échouer le mart dès la première facture SEK si la colonne avait existé.

- Une règle d'anomalie sur les **montants**, pas seulement sur les nombres : `revenue_daily_by_currency` (`anomaly`, `warn`), somme par jour et par devise. Un +14 % sur EUR aurait été dans la bande ; une devise nouvelle avec un montant non nul aurait été « série sans historique », ce que la règle signale explicitement depuis.

- Mars et début avril recalculés, tableaux de bord réémis, un mail du produit aux commerciaux.

## Ce qu'on retient

Une hypothèse implicite retirée du code parce qu'elle semblait redondante devient une hypothèse invisible. La règle `accepted_values currency = ['EUR']` aurait fait échouer la première facture SEK avec un message clair, et sa modification aurait été la relecture qui a manqué. Depuis, la revue des règles ([[dq-rules-review-feedback]]) demande pour chaque mart la liste de ses hypothèses (devise, fuseau, unité) et une règle par hypothèse, même celle qui « ne changera jamais ».

## Chiffres

| | Valeur |
|---|---|
| factures concernées | 1 900 |
| surévaluation | +5,2 M EUR sur 38 M |
| durée | 43 jours |
| tableaux affectés | 4 (chiffre d'affaires mensuel, par client, par pays, par commercial) |
| règles ajoutées | 5 |
| délai signalement → correction | 40 min pour la cause, 1 jour pour le mart |
