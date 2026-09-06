---
name: case-nordwerk-sso-idp-migration-lockout
description: Juillet 2026, Nordwerk Stahl change de fournisseur d'identité un lundi matin, 60 dispatcheurs bloqués trois heures, d'où le repli e-mail de 7 jours
type: project
status: active
verified: 2026-08-12
---

# Cas : Nordwerk Stahl, migration SSO un lundi matin

Chargeur fictif, sidérurgie, Enterprise, 60 utilisateurs web avec SSO imposé (`enforced = true`). Ticket du 2026-07-06 à 07:12, premier ticket Enterprise, puis 41 autres tickets en une heure, tous des dispatcheurs différents.

## Ce qui s'est passé

Leur DSI a basculé leur fournisseur d'identité pendant le week-end. Le nouveau fournisseur émet des `sub` différents pour les mêmes personnes. Leur configuration chez nous avait `email_fallback = false`, choisi par eux à la mise en place pour éviter la liaison par e-mail. Lundi 7 h, chaque connexion échoue avec « Aucun compte ne correspond à votre identité ». SSO imposé, donc pas de mot de passe de secours, sauf l'admin de dernier recours, qui était en congé.

## Ce qu'on a fait

- 07:20 : L1 reconnaît le motif (même org, même message) et escalade en L3 selon le critère « plus de 10 organisations » lu à l'envers : c'était une org et 40 personnes, le critère ne collait pas, L2 a pagé quand même, à raison.

- 07:45 : L2 identifie le changement d'`issuer` dans l'historique de configuration. Pour relier 60 comptes, la seule commande était `hfctl user sso-reset` un par un, avec confirmation e-mail de chaque utilisateur. Impraticable en urgence.

- 08:10 : le backend passe `email_fallback = true` sur l'org par ticket HF, ce qui relie chaque utilisateur à sa première connexion. 09:50 : les 60 sont reconnectés.

- Trois heures d'arrêt du dispatch pour un client Enterprise. Le SLA « bloquant 4 h » a tenu, de peu.

## Ce qu'on a changé

- HF-3180 : l'admin de l'org peut activer lui-même le repli e-mail pour sept jours depuis ses réglages SSO, avec un avertissement. Le backend n'est plus dans la boucle. Livré fin juillet.

- La mise en place SSO Enterprise inclut désormais une consigne écrite : prévenir le support une semaine avant tout changement de fournisseur d'identité. Trois clients l'ont depuis fait ; deux ont activé le repli avant la bascule, zéro ticket.

- Le critère de page « plus de 10 organisations » est complété par « ou une organisation Enterprise avec plus de 10 utilisateurs bloqués ». C'est ce que L2 avait décidé seul, autant l'écrire.

- L'admin de dernier recours d'une org Enterprise doit être deux personnes, pas une. La configuration l'exige depuis août.

## Ce qu'on a appris

- Le choix sécurisé du client (`email_fallback = false`) était le bon, et il faut pouvoir le suspendre vite sans le retirer. Sept jours, activé par eux, tracé.

- Un mécanisme de dernier recours qui repose sur une personne n'est pas un dernier recours.

- Quarante tickets identiques en une heure doivent être regroupés au premier, pas au dixième. Deskline a maintenant une règle qui suggère la fusion quand trois tickets d'une même org arrivent en quinze minutes.

Compté dans [[case-lessons-recurring-themes-2026-h1]].
