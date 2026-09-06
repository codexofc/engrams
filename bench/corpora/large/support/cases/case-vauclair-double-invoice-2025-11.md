---
name: case-vauclair-double-invoice-2025-11
description: Novembre 2025, Transports Vauclair facturé deux fois pour un chargement après une remise en transit manuelle, origine du contrôle facture d'undeliver
type: project
status: active
verified: 2026-03-12
---

# Cas : Transports Vauclair, deux factures pour un chargement

Transporteur fictif, PME de 30 camions, autofacturation activée. Ticket du 2025-11-19, catégorie `invoice:dispute`.

## Ce qui s'est passé

Le chauffeur avait déclaré la livraison le 13 novembre sur le mauvais chargement (deux livraisons dans la même zone industrielle). Le chargeur l'a signalé le 14. À l'époque, la remise en `IN_TRANSIT` se faisait par le backend avec un `UPDATE` et une correction de `load_events`, sur ticket. Le backend l'a faite le 15 au matin.

Entre-temps, la clôture de facturation du 14 au soir avait pris le chargement en `DELIVERED` et généré la facture d'autofacturation F-2025-11-0412. Personne n'a regardé. Le chauffeur a livré pour de vrai le 17, a déclaré la livraison, et la clôture du 17 a généré F-2025-11-0503 pour le même chargement. Vauclair a reçu deux factures, s'est fait payer deux fois par le chargeur (qui paie par prélèvement automatique sur facture), et c'est le chargeur qui a ouvert le ticket.

## Ce qu'on a fait

- Finance : avoir sur F-2025-11-0412 le 20, remboursement du chargeur le 22.

- Support : réponse aux deux parties, macro écrite pour l'occasion, devenue `invoice-duplicate-load`.

- Ticket HF-3040 pour comprendre comment une remise en transit pouvait laisser une facture derrière elle. Réponse : parce que rien ne le vérifiait, la remise en transit n'était pas une commande, c'était une requête SQL.

## Ce que ça a changé

C'est ce cas qui a fixé la première précondition de `hfctl load undeliver` livré en HF-3105 : refus si une facture, brouillon ou finalisée, référence le chargement. Voir [[case-lessons-recurring-themes-2026-h1]] pour la place de ce cas dans l'année. La finance a aussi ajouté à la clôture un contrôle « chargement déjà facturé » qui bloque la génération d'une seconde facture pour un même `load_id`, quel que soit son état ; il s'est déclenché deux fois depuis, les deux fois à raison.

## Ce qu'on a appris

- Une correction manuelle en base n'a pas de préconditions. Une commande en a. Tout ce que le support demande plus de deux fois par mois doit devenir une commande.

- La clôture de facturation tourne tous les soirs et ne sait pas qu'un ticket est ouvert. Une correction qui traîne une nuit a des effets comptables.

- Le chargeur payait par prélèvement sur facture, donc l'erreur a coûté de l'argent réel pendant huit jours. Les tickets `invoice:dispute` d'un client en prélèvement passent en priorité depuis.

## Le transporteur

Vauclair a été correct, a rendu le trop-perçu sans discussion, et a demandé une chose : être prévenu quand une facture le concernant est annulée. L'e-mail d'avoir existe, il partait à l'adresse de facturation du chargeur seulement. Depuis HF-3048 il part aussi au transporteur en autofacturation.
