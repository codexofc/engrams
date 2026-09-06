---
name: payment-terms-negotiation-preferences
description: The billing team's standing preferences, 30 days default and never beyond 45, no custom invoice layouts per shipper, and one finance approver per entity for anything off the standard
type: user
status: active
verified: 2026-04-28
---

Préférences de l'équipe billing et de la finance, à connaître avant d'accepter une demande commerciale.

- **Délai de paiement** : 30 jours date de facture par défaut, 45 jours maximum, jamais 60 même pour un grand compte. La loi française plafonne à 60 jours mais on porte le risque de crédit des transporteurs (voir [[self-billing-carriers]]), et chaque jour de délai en plus coûte de la trésorerie. Un délai à 45 jours se négocie contre un mandat SEPA B2B obligatoire.
- **Pas de mise en page de facture spécifique par chargeur.** Trois grands comptes l'ont demandé (leur numéro de commande en gros, leur logo, un ordre de colonnes). On met leur référence de commande dans `invoices.customer_reference`, rendue dans l'en-tête, et c'est tout. Un gabarit par client devient impossible à maintenir avec les changements légaux, voir [[invoice-pdf-rendering]].
- **Un approbateur finance par entité**, pas un comité. Pour les avoirs au-dessus du seuil, les pauses de relance de plus de 15 jours et les délais dérogatoires. Les noms sont dans `finance_approvers`, on ne les met pas ici.
- **Les commerciaux ne modifient pas une facture émise.** Ils demandent un avoir avec un motif. L'équipe préfère cent avoirs tracés à une facture modifiée.
- **Facturation mensuelle groupée** : refusée. Une facture par chargement livré, parce que le litige se fait par chargement et qu'une facture mensuelle de 200 lignes bloquée par un seul litige bloque tout. Un relevé mensuel récapitulatif existe (`GET /shipper/statements/{month}`) pour ceux qui veulent une vue consolidée.
- **Langue** : la facture est dans la langue de l'entité, avec les libellés de lignes en anglais si le chargeur est d'un autre pays. Pas de facture entièrement en anglais, le cabinet ne le souhaite pas.
- **Réunions** : la revue billing du lundi 10:00 traite la file de réconciliation de la semaine précédente et les avoirs en attente, rien d'autre. Les demandes produit passent par le ticket, pas par la réunion.
