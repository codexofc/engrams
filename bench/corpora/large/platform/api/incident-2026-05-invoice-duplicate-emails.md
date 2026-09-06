---
name: incident-2026-05-invoice-duplicate-emails
description: May 2026, 1 340 shippers received their invoice e-mail two to five times after a Messenger retry storm on the notifications transport caused by a mailer timeout, fixed with an idempotency key on notification_deliveries
type: project
status: active
verified: 2026-05-22
---

# Incident 2026-05-12 : e-mails de facture en double

## Ce qui s'est passé

Le 12 mai à 06 h 00, le job de facturation mensuelle finalise 4 100 factures et pousse autant de `SendInvoiceEmail` sur le transport `notifications`. Le relais SMTP de notre fournisseur d'e-mail transactionnel a mis 30 s à répondre pendant 20 minutes (incident de leur côté, confirmé plus tard). Notre `MailerInterface` a un timeout à 10 s.

Le handler `SendInvoiceEmailHandler` faisait : rendre le template, appeler `$mailer->send()`, puis écrire une ligne dans `notification_deliveries`. Le `send()` levait `TransportException` après 10 s. Messenger réessayait (voir [[messenger-transports-and-retries]] : 2 s, 6 s, 18 s). Mais le fournisseur, lui, avait bien reçu et mis en file le message avant de nous répondre lentement. Chaque tentative envoyait un vrai e-mail.

Résultat : 1 340 chargeurs ont reçu entre 2 et 5 fois le même e-mail avec la même facture en pièce jointe. 3 tickets support, 1 chargeur qui a cru à une double facturation et a appelé sa banque.

## Pourquoi on ne l'avait pas vu avant

Les e-mails unitaires (acceptation d'offre, etc.) passent par le même handler et la même stratégie de retry, mais un timeout isolé ne produit qu'un doublon isolé, que personne ne remarque. Il a fallu 4 100 envois en 20 minutes de dégradation pour que ça devienne visible.

## Correctif (HF-1640, déployé le 14 mai)

Clé d'idempotence côté fournisseur et côté nous :

- `notification_deliveries` reçoit une colonne `idempotency_key text UNIQUE` = `sha256(type + recipient + subject_id)`. Pour une facture : `invoice-email:<shipper_uuid>:<invoice_uuid>`.
- Le handler insère la ligne en `SENDING` **avant** d'appeler le mailer, dans une transaction commitée. Si l'`INSERT` viole l'unique, le message a déjà été traité (ou est en cours) : on sort sans rien envoyer.
- L'appel au mailer passe l'en-tête `X-Idempotency-Key` que le fournisseur supporte (on ne le savait pas, c'est dans leur doc depuis 2024). Ils dédupliquent sur 24 h.
- Après envoi, `UPDATE ... SET status = 'SENT'`. Si le processus meurt entre les deux, la ligne reste en `SENDING` et un sweep horaire la passe en `UNKNOWN` sans réessayer : mieux vaut un e-mail manquant qu'un doublon, le chargeur a de toute façon la facture dans l'interface.

Le même schéma a été appliqué aux push et aux SMS dans la foulée, parce qu'un SMS en double coûte de l'argent.

## Ce qu'on n'a pas fait

Baisser le nombre de retries sur `notifications`. Les retries sont utiles quand le fournisseur est vraiment en panne. Le problème n'était pas le retry, c'était que l'envoi n'était pas idempotent.

## Leçon

Un appel externe qui peut réussir sans qu'on le sache (timeout après réception) doit être précédé d'un marqueur en base et accompagné d'une clé d'idempotence si le fournisseur le permet. On avait déjà cette logique pour les webhooks sortants ([[webhook-delivery-outbox]]) et pas pour les e-mails, ce qui n'avait aucune raison d'être.
