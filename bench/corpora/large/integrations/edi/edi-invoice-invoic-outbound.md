---
name: edi-invoice-invoic-outbound
description: Three partners get the commission invoice as an EDIFACT INVOIC within 48 h of issue, with their load reference in RFF+ON and VAT per line, checked to the cent
type: project
status: active
verified: 2026-04-24
---

# INVOIC sortant (HF-2068)

Trois partenaires ([[edi-partners-overview]]) ingèrent leurs factures fournisseurs en EDIFACT et ne veulent pas de PDF à ressaisir. L'INVOIC est produit par `InvoiceToInvoicHandler` dans `hf-edi-gateway`, à partir de la facture **émise** par la plateforme de facturation ; le gateway ne calcule jamais un montant, il traduit une facture qui existe.

## Déclenchement

Événement `invoice.issued` de la facturation, filtré sur les organisations dont le partenaire a `invoice_enabled = true`. Le PDF et l'e-mail partent comme pour tout le monde ; l'INVOIC est en plus. Délai : dans les 48 h de l'émission, engagement pris avec Nordkarton ([[edi-partner-nordkarton-quirks]]) ; p95 mesuré 25 minutes, le reste du délai est de la marge pour les incidents.

Pour Nordkarton, une facture par chargement (leur mode `per_load_48h`). Pour Bruma Retail et Kalmarine, la facture peut regrouper plusieurs chargements (facturation à l'échéance de la plateforme) ; l'INVOIC porte alors un groupe `LIN` par chargement.

## Structure

- `BGM+380` (facture commerciale) ou `BGM+381` (avoir), `1004` = notre numéro de facture (séquence sans trou de la facturation), `1225 = 9` original.

- `DTM+137` date d'émission, `DTM+13` échéance (les conditions du contrat commercial, pas celles d'un message entrant).

- `RFF+ON` : **la référence du partenaire** (`loads.partner_reference`, celle de leur IFTMIN). C'est le champ qui fait que leur comptabilité rapproche automatiquement ; sans lui, la facture tombe en file manuelle chez eux. Pour une facture multi-chargements, un `RFF+ON` par groupe `LIN`.

- `NAD+SE` nous (avec numéro de TVA de l'entité émettrice, qui dépend du pays de la facture), `NAD+BY` eux (leur code EDI et leur numéro de TVA).

- `CUX` devise de la facture.

- `LIN` + `IMD` (description « Commission de mise en relation, chargement X ») + `QTY+47` (1) + `MOA+203` (montant ligne HT) + `PRI+AAA` + `TAX+7+VAT` avec `5278` taux et `5305` catégorie (`S` standard, `AE` autoliquidation intracommunautaire, `E` exonéré) + `MOA+124` (montant de TVA de la ligne).

- `UNS+S`, puis `MOA+79` (total lignes HT), `MOA+176` (total TVA), `MOA+77` (total TTC), et un `TAX` récapitulatif par taux.

Les codes de catégorie de TVA suivent les règles de la facturation (autoliquidation, exonérations) ; le gateway lit `invoice_lines.vat_category` et ne décide rien. Un `vat_category` inconnu du mapping (arrivé une fois avec une nouvelle règle) fait échouer la production de l'INVOIC et ouvre un ticket, plutôt que d'envoyer un code faux.

## Contrôle avant envoi

`InvoicConsistencyCheck` recompose les totaux à partir des lignes du message et les compare à la facture au centime ; un écart bloque l'envoi. Il a bloqué deux fois : un arrondi de conversion PLN sur une facture multi-lignes (corrigé côté gateway en prenant les montants stockés au lieu de recalculer) et un avoir dont le signe était inversé dans une première version du handler.

## Transport et accusés

AS2 comme le reste ([[as2-transport-setup]]). Un INVOIC sans CONTRL sous 24 h ouvre un ticket **facturation** (pas intégrations) parce que la conséquence est un paiement en retard ; la réconciliation mensuelle ([[edi-reconciliation-daily]]) vérifie que chaque facture émise pour un partenaire `invoice_enabled` a un INVOIC accusé.

## Avoirs

`BGM+381`, `RFF+IV` avec le numéro de la facture d'origine, montants positifs (le type de document porte le sens, exigence de deux partenaires sur trois ; le troisième accepte les deux). Le signe négatif de la première version est l'un des deux blocages du contrôle de cohérence.

## Ce qu'on ne fait pas

- Envoyer l'INVOIC avant que la facture soit émise (« pour qu'ils provisionnent »). Une facture qui n'existe pas n'a pas de numéro.

- Un INVOIC pour les pénalités de retard ou les intérêts : ils sont sur une facture séparée avec PDF, ces partenaires les traitent à la main et l'ont demandé ainsi.

- Le format XML de facturation électronique obligatoire dans certains pays : chantier de la facturation, pas du gateway ; le jour où il existe, l'INVOIC sera peut-être redondant pour ces partenaires.

## Chiffres (mars 2026)

1 620 INVOIC envoyés, 100 % accusés sous 24 h, 0 rejet applicatif chez les partenaires, 2 tickets facturation ouverts par la règle des 24 h (les deux : endpoint AS2 du partenaire en maintenance, résolus par le renvoi).
