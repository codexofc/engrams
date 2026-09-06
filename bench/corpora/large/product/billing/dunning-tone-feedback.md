---
name: dunning-tone-feedback
description: A firm reminder that names the failed debit cause and offers a payment date field collects 23 % faster than a generic one
type: feedback
status: active
verified: 2026-01-22
---

En décembre 2025 on a réécrit les relances de niveau 2 et 3 (voir [[dunning-schedule]]) après une série de retours du support et un A/B test de six semaines sur l'entité FR.

## Ce qu'on a appris

Le texte initial était juridique et générique : « Sauf erreur de notre part, la facture ci-dessous reste impayée. » Trois problèmes remontés par le support :

1. Les chargeurs sous mandat SEPA ne comprenaient pas pourquoi on les relançait, puisqu'ils n'ont rien à faire, le prélèvement est de notre côté. Il fallait dire explicitement « le prélèvement du 12/11 a été rejeté par votre banque (provision insuffisante) ».
2. « Sauf erreur de notre part » invitait à contester. 14 % des réponses commençaient par « il y a bien une erreur de votre part ».
3. Aucune action claire. Un lien PDF et un IBAN, mais pas de bouton.

## Ce qu'on a testé

Variante B, envoyée à la moitié des comptes en relance niveau 2 du 2025-12-01 au 2026-01-12 :

- première phrase factuelle avec la cause d'échec quand elle existe ;
- un bouton « Indiquer une date de paiement » qui ouvre un formulaire à un champ (date) et met `dunning_states.paused_until` à cette date, plafonnée à 15 jours ;
- suppression de toute formule d'excuse anticipée.

Résultat mesuré sur 1 830 comptes : délai médian d'encaissement après la relance, 9,1 jours en A contre 7,0 jours en B (23 % plus rapide). Taux de réponse par email, 11 % en A contre 6 % en B. Le champ date a été utilisé par 31 % des comptes B, et 84 % d'entre eux ont payé à la date annoncée ou avant.

## Comment appliquer ailleurs

- Toute communication automatique liée à un échec doit nommer la cause telle qu'on la connaît, dans le vocabulaire du client, pas le code d'erreur Payla. La table de correspondance est `PaymentFailureReason.toCustomerMessage(locale)`.
- Donner une action à un champ vaut mieux qu'un paragraphe d'explication. Le formulaire à un champ a été réutilisé pour les litiges en 2026-03.
- Ne pas s'excuser par anticipation dans un message de relance. On s'excuse quand on a tort, et alors on émet un avoir.

Le texte allemand a été relu par un juriste parce que le § 286 BGB a des exigences de forme sur la mise en demeure ; la variante B y est conforme à condition de garder la mention de la date d'échéance initiale.
