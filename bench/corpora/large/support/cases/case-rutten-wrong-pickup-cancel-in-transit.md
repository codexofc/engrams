---
name: case-rutten-wrong-pickup-cancel-in-transit
description: Janvier 2026, un chargement de Rutten & Zonen annulé en transit sur l'e-mail d'un chargeur homonyme, origine de la règle des deux personnes
type: project
status: active
verified: 2026-03-20
---

# Cas : Rutten & Zonen, l'annulation en transit qu'il ne fallait pas faire

Transporteur fictif néerlandais, 25 camions, Business. Ticket du 2026-01-14, `load:cancel`, puis un second ticket du chargeur le 16.

## Ce qui s'est passé

Le 14 janvier à 15 h, un e-mail arrive au support depuis une adresse au nom d'une société de négoce qui est un de nos chargeurs, demandant l'annulation « urgente » du chargement portant la référence chargeur `PO-88214`, camion déjà parti, « marchandise refusée par le client final ». L2 cherche la référence, trouve un chargement `IN_TRANSIT` de Rutten, motif `shipper_vanished` accepté faute de mieux, annule.

Le 16, le chargeur réel du chargement ouvre un ticket : le camion de Rutten a livré le 15, tout s'est bien passé, mais le chargement est `CANCELLED` et la facture ne peut pas être émise. L'e-mail du 14 venait d'un **autre** chargeur, au nom presque identique (deux sociétés d'un même groupe, deux organisations chez nous), qui parlait de son propre chargement `PO-88214` chez un autre transporteur, en `DISPATCHED`, et qui aurait pu l'annuler lui-même. Les références chargeur ne sont uniques que par organisation ; L2 avait cherché sans filtrer sur l'organisation.

## Ce que ça a coûté

Un chargement annulé en transit ne revient pas. Il n'y a pas de transition depuis `CANCELLED`. La livraison de Rutten a été payée par une facture manuelle de la finance, hors plateforme, trois semaines de va-et-vient pour les pièces, la TVA et le rapprochement. Rutten a été patient ; le chargeur moins.

## Ce qu'on a changé

- HF-3075 : `cancel-in-transit` demande un jeton de confirmation saisi par une seconde personne dans les 10 minutes. La seconde personne relit le ticket, pas seulement la commande. Voir la procédure de support « annulation à deux ».

- Le motif `shipper_vanished` exige trois tentatives de contact tracées dans le ticket. Un e-mail entrant n'est pas une tentative de contact sortante.

- `hfctl load list --reference` exige `--org` depuis 1.12. Sans, il refuse, avec un message qui cite ce cas.

- La demande d'annulation en transit doit venir de l'organisation propriétaire du chargement, vérifiée par l'`org_id` du compte qui écrit, pas par le nom dans la signature.

## Ce qu'on a appris

- L'urgence dans un e-mail n'est pas une raison d'aller plus vite ; c'est une raison de relire.

- Une référence sans son organisation n'identifie rien.

- Une action irréversible mérite une seconde paire d'yeux, même quand l'équipe est petite et que ça ralentit. Depuis février, six annulations en transit ont été faites avec le jeton, aucune contestée.

Ce cas ouvre la liste des leçons de [[case-lessons-recurring-themes-2026-h1]].
