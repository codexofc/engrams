---
name: self-billing-carriers
description: Self-billing lets us invoice on the carrier's behalf with a signed mandate, Wednesday payouts, 2.5 % early-payout option, 48 h contest
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

## Contestation et acceptation tacite

Le chargeur a 48 h après la livraison déclarée pour contester (POD manquant, marchandise endommagée, retard facturable). Sans contestation, la facture d'autofacturation est émise et le transporteur est payé au prochain reversement. Une contestation ouvre un litige dans le back-office (`/backoffice/disputes`) et suspend l'émission ; 2,3 % des chargements en autofacturation sont contestés, et 70 % des litiges se règlent en moins de 5 jours par une réduction convenue, qui devient le montant facturé.

Le transporteur, lui, accepte tacitement la facture émise en son nom sous 48 h après réception de l'email (`selfbilling.invoice_issued`), sauf refus explicite par le lien « contester cette facture ». Le refus est rare (0,4 %) et porte presque toujours sur un montant de péage ou d'attente non prévu dans l'enchère. Le mandat PL exige la trace de cette acceptation : `self_billing_invoices.accepted_at` et `acceptance_mode` (`tacit`, `explicit`) sont remplis pour toutes les entités, pas seulement PL.

## Frais d'attente et suppléments après enchère

Un transporteur peut demander un supplément après livraison (attente au chargement de plus de 2 h, palettes supplémentaires, second passage). Il le déclare dans l'app dans les 24 h, le chargeur a 48 h pour accepter ou refuser, et le supplément accepté est une ligne de plus sur la facture d'autofacturation, avec sa propre TVA. Les suppléments représentent 3,1 % du montant facturé en autofacturation en juin 2026, dont 60 % d'attente. Un supplément refusé peut être escaladé au support, qui tranche avec les positions GPS (l'équipe data expose la durée d'arrêt sur site).

## Ce que l'autofacturation change dans la comptabilité

- Le chiffre d'affaires de Halden Freight ne comprend pas le montant du transport ; on encaisse pour le compte du transporteur. La facture 1 (transport) est comptabilisée en compte de tiers, seules les factures 2 et 3 sont notre chiffre d'affaires. Le mart facture de l'entrepôt porte `is_self_billing` pour cette raison.
- Les reversements sont des paiements de dettes fournisseurs, pas des charges. L'export comptable allemand les code en compte 1600, l'export FEC en 401.
- La TVA de la facture 1 est celle du transporteur : elle apparaît dans nos exports comme TVA collectée pour compte de tiers, ligne à part, que les cabinets des trois pays ont validée en 2025.

## Chiffres de délai de paiement transporteur

Avant l'autofacturation (2024) : délai médian entre livraison et paiement du transporteur de 41 jours. Avec : 6 jours (livraison le lundi, validation mercredi, reversement le mercredi suivant), et 0 jour pour les 18 % en paiement anticipé. C'est l'argument commercial principal auprès des petites flottes, et la raison pour laquelle le risque de crédit chargeur est assumé.
