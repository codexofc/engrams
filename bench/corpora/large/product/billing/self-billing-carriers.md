---
name: self-billing-carriers
description: Self-billing (autofacturation) lets Halden Freight issue the carrier's invoice on its behalf, requires a signed mandate per carrier, and pays out weekly on Wednesdays with a 2.5 % early-payment option
type: project
status: active
verified: 2026-07-08
---

## Principe

Avec l'autofacturation, c'est Halden Freight qui émet la facture de transport au nom du transporteur, à destination du chargeur, et qui reverse le montant au transporteur. Le transporteur n'a rien à saisir. Cela suppose un mandat d'autofacturation signé (art. 289-I-2 du CGI en France, § 14 Abs. 2 UStG en Allemagne), stocké dans `carrier_accounts.self_billing_mandate_id` avec sa date de signature.

Les factures d'autofacturation portent le numéro de TVA et l'adresse du transporteur, la mention « Autofacturation » / « Gutschrift », et sont numérotées dans une séquence propre au transporteur : `SB-<carrier_id>-<année>-<compteur>`, parce que le transporteur doit pouvoir justifier une suite continue de ses propres factures. C'est le seul cas où la séquence n'est pas par entité (voir [[invoice-numbering-sequence]]).

## Ce qu'on facture et à qui

Pour un chargement livré et validé (POD accepté par le chargeur, ou délai de contestation de 48 h écoulé) :

1. facture du transporteur au chargeur, montant du bid gagnant, TVA selon les règles du pays du transporteur et du chargeur ([[vat-rules-by-country]] s'applique, mais avec le transporteur comme prestataire) ;
2. facture de commission Halden Freight au chargeur ;
3. facture de frais de service Halden Freight au transporteur (1,5 % du bid), déduite du reversement.

Le chargeur voit une seule échéance pour 1 et 2, ce qui est la raison d'être du dispositif : avant l'autofacturation, 30 % des transporteurs facturaient avec plus de 15 jours de retard et les chargeurs payaient en désordre.

## Reversement

Payout Payla (voir [[payla-integration-overview]]) chaque mercredi à 11:00 pour tout ce qui est validé avant le mardi 23:59. Le reversement est fait que le chargeur ait payé ou non : Halden Freight porte le risque de crédit, c'est pour cela que la relance ([[dunning-schedule]]) est stricte.

Option de paiement anticipé : le transporteur peut demander le reversement le jour même de la validation contre une décote de 2,5 %. Activation par transporteur (`carrier_accounts.early_payout_enabled`), 18 % des transporteurs actifs l'ont activée en juin 2026, et ils sont surreprésentés parmi les petites flottes (1 à 3 camions).

## Chiffres

Juin 2026 : 71 % des chargements livrés sont en autofacturation, 4 130 transporteurs sous mandat. Le reste : transporteurs qui refusent le mandat (grandes flottes avec leur propre ERP) et pays où on n'a pas encore validé la base légale (Espagne, Italie, prévu T4 2026).

## Points ouverts

- La Pologne exige que la procédure d'acceptation de chaque facture d'autofacturation soit décrite dans le mandat. Notre mandat PL a été réécrit en mars 2026 avec une acceptation tacite sous 48 h, alignée sur le délai de contestation.
- Un transporteur qui perd son numéro de TVA en cours de mois : les factures déjà émises restent valides (même logique que [[reverse-charge-intra-eu]]), les suivantes sont bloquées et le compte passe en revue.
