---
name: reverse-charge-intra-eu
description: Reverse charge mentions per entity, the VIES cache TTL of 72 h, and the rule that a VAT number invalidated after issue never retroactively changes an invoice
type: reference
status: active
verified: 2026-02-09
---

Quand un chargeur est établi dans un autre État membre que l'entité facturante et dispose d'un numéro de TVA valide dans VIES, la commission Halden Freight est facturée sans TVA avec la mention d'autoliquidation. Le contexte général est dans [[vat-rules-by-country]].

## Mentions obligatoires par entité

Chaque entité a sa mention légale dans `invoice_templates.reverse_charge_mention` :

- FR : « Autoliquidation, article 283-2 du CGI et article 196 de la directive 2006/112/CE »
- DE : « Steuerschuldnerschaft des Leistungsempfängers, § 13b UStG »
- PL : « Odwrotne obciążenie, art. 28b ustawy o VAT »
- NL : « BTW verlegd, art. 196 Richtlijn 2006/112/EG »

La mention est rendue dans la langue de l'entité, pas dans celle du client. Un client italien facturé par l'entité FR reçoit la mention française plus la ligne anglaise « Reverse charge » ajoutée systématiquement. Décision HF-2034, validée avec le cabinet comptable.

## Cache VIES

L'appel VIES est fait par `ViesClient` avec un cache de 72 heures dans `shipper_accounts.vies_checked_at`. Au moment de l'émission d'une facture :

1. si `vies_checked_at` a moins de 72 h, on utilise `vies_status` tel quel ;
2. sinon on refait l'appel, avec un timeout de 4 s ;
3. si VIES ne répond pas, on garde l'ancien statut mais on marque la facture `vies_stale = true` pour le rapport de clôture.

Le numéro de TVA et le résultat de la vérification sont copiés sur la facture (`invoices.customer_vat_number`, `invoices.vat_check_reference`). Le `vat_check_reference` est l'identifiant de consultation renvoyé par VIES, c'est ce que demande l'administration en cas de contrôle.

## Un numéro invalidé après coup ne change rien

Cas rencontré en janvier 2026 (HF-2402) : un chargeur autrichien radié fin décembre, 14 factures émises en autoliquidation entre le 3 et le 28 décembre. Le contrôleur interne voulait les réémettre avec TVA. Réponse du cabinet : la vérification faite de bonne foi à la date d'émission suffit, on conserve les factures et on archive la preuve de consultation. Ce qui compte est `vat_check_reference` sur chaque facture, pas le statut actuel du compte.

Conséquence pour le code : `VatResolver` ne doit jamais relire `shipper_accounts.vies_status` pour une facture déjà émise. Le test `issuedInvoiceKeepsItsVatRule()` le garantit.

## Ce qu'on ne fait pas

Pas de guichet unique OSS : il concerne les ventes B2C, et Halden Freight n'a aucun client particulier. Voir [[vat-oss-not-applicable]] pour l'explication complète, la question revient à chaque nouveau comptable.
