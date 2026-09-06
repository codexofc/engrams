---
name: playbook-invoice-dispute-amount
description: Litige sur un montant de facture : comparer à l'offre acceptée et aux suppléments, TVA, devise, doublon, puis dossier finance pour avoir
type: reference
status: active
verified: 2026-05-20
---

# Litige sur le montant d'une facture

Catégorie `invoice:dispute`. Le chargeur (ou le transporteur en autofacturation) dit que la facture est fausse. Une facture finalisée ne se modifie pas, elle s'annule par avoir. Le support ne fait pas l'avoir, il constitue le dossier pour la finance.

## Vérifications

1. **La facture.** `hfctl invoice get <invoice_id>` : montant HT, TVA, devise, lignes, `finalized_at`, chargements facturés.

2. **Ce qui a été convenu.** Pour chaque chargement de la facture, `hfctl load get <load_id>` donne `accepted_bid_amount` et `accepted_bid_currency`. La ligne de facture doit correspondre à l'offre acceptée, plus les suppléments enregistrés (`extras` : attente, second point de livraison, palettes non restituées).

3. **Les suppléments.** `hfctl load events <load_id>` montre les événements `extra_added` avec l'acteur. Un supplément ajouté par le transporteur après la livraison et non validé par le chargeur ne doit pas être facturé : si c'est le cas, c'est un bug, escalade L2 avec les identifiants.

4. **La TVA.** Trois cas qui reviennent : autoliquidation intra-UE (facture sans TVA avec la mention, le client s'attend à voir de la TVA), TVA du pays du transporteur en autofacturation, et le taux réduit qui n'existe pas pour le transport. La macro `invoice-vat-explain` a les trois textes. Le support ne tranche pas un cas de TVA hors de ces trois, il escalade à la finance.

5. **La devise.** Une offre en PLN ou CZK facturée en EUR : le taux appliqué est celui du jour de la finalisation, indiqué dans le pied de facture. Le client qui compare avec le taux de sa banque aura toujours un écart de quelques centimes ou dizaines d'euros. Macro `invoice-fx-rate`. Ce n'est pas un litige recevable, sauf si le taux affiché est manifestement faux (ordre de grandeur), ce qui n'est jamais arrivé.

6. **Un chargement facturé deux fois.** Chercher les factures de l'organisation avec `hfctl invoice list --org <org_id> --load <load_id>`. Deux factures pour le même chargement : litige recevable, dossier finance, et ticket HF parce que ça ne devrait pas être possible.

## Dossier pour la finance

Si le litige est recevable (étapes 3, 6, ou une erreur avérée), ouvrir un ticket dans la file finance de Deskline avec : `invoice_id`, les `load_id`, le montant contesté, la raison en une phrase, le lien vers la preuve (événement, offre). La finance émet l'avoir et la nouvelle facture. Délai habituel : deux jours ouvrés. Le support répond au client avec la macro `invoice-dispute-accepted` et le délai.

Si le litige n'est pas recevable, macro adaptée (`invoice-vat-explain`, `invoice-fx-rate`, `invoice-extras-explain`) avec la preuve citée. Un client Enterprise qui insiste : escalade au responsable de compte, pas à la finance.

## Ce qu'on ne fait pas

Pas de « on va corriger la facture ». Pas de nouvelle facture sans avoir. Pas de promesse de remboursement, la finance décide.

## Voir aussi

Le cas où la facture est juste mais n'est pas arrivée : [[playbook-invoice-email-not-received]]. Le cas du transporteur qui attend son paiement : [[playbook-carrier-payout-missing]].
