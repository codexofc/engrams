---
name: incident-2026-05-support-account-takeover
description: On 2026-05-06 an attacker with a leaked password had support change a carrier's contact email; the 72 h IBAN cooling period blocked the payout redirect
type: project
status: active
verified: 2026-06-02
---

# Prise de contrôle d'un compte transporteur via le support (mai 2026)

Post-mortem `2026-05-06-support-ato.md`. **SEV2** : contrôle affaibli (un changement d'e-mail obtenu par ingénierie sociale), sans perte (la tentative de détournement de virement a été bloquée). Processus : [[incident-process-severity-levels]].

## Chronologie (UTC)

- **2026-05-05 19:40** `[audit]` : `auth.login_succeeded` sur le compte de l'administrateur d'un transporteur (12 camions) depuis une IP inconnue, pays inhabituel. L'alerte « pays inhabituel » n'existe que pour les comptes staff, pas pour les clients. Le mot de passe venait probablement d'une fuite tierce (il figurait dans la liste des mots de passe compromis, vérification faite après coup ; le compte datait d'avant la vérification à la création).

- **19:45 à 20:10** `[audit]` : lecture des factures, des virements à venir, de la page des paramètres bancaires. Le changement d'IBAN est soumis à une **période de refroidissement de 72 h** avec confirmation par e-mail et SMS (mesure de 2025 après une fraude au faux fournisseur). L'attaquant ne tente pas de changer l'IBAN à ce stade.

- **2026-05-06 07:55** `[ticket]` : un ticket support est ouvert **depuis le compte connecté** : « J'ai changé de téléphone et d'adresse e-mail, je ne reçois plus les codes, pouvez-vous mettre à jour mon e-mail vers <nouvelle adresse> et mon numéro ». Le ticket vient d'un compte authentifié, ce qui à l'époque suffisait au support pour considérer l'identité établie.

- **08:30** `[audit]` : l'agent L1 modifie l'e-mail et le téléphone de contact depuis le back-office (`member.contact_changed`). Le numéro et l'e-mail servent aux confirmations de changement d'IBAN.

- **08:41** `[audit]` : demande de changement d'IBAN depuis le compte. Confirmation par e-mail (nouvelle adresse) et SMS (nouveau numéro) faites en 2 minutes. Période de refroidissement de 72 h enclenchée : l'IBAN ne serait effectif que le 2026-05-09 08:41.

- **2026-05-07 10:15** `[chat]` : le vrai administrateur appelle le support : il ne peut plus se connecter (le mot de passe avait été changé à 08:45 le 6). L'agent L2 regarde `audit_events`, voit la séquence, déclare l'incident à 10:22.

- **10:25** : sessions révoquées, compte gelé (`users.frozen_at`, aucune action possible, connexion refusée avec message d'appeler le support), demande de changement d'IBAN annulée. Le virement hebdomadaire du 9 mai (14 200 EUR) part sur l'IBAN d'origine.

- **10:40 à 12:00** : identité du vrai administrateur vérifiée par appel au numéro de téléphone **historique** (celui d'avant le changement, conservé dans l'audit) et par une facture récente que lui seul pouvait citer. E-mail et téléphone restaurés, mot de passe réinitialisé, MFA non disponible côté client (voir le projet IAM), donc conseil de mot de passe unique et vérification de la liste de compromission imposée.

- **2026-05-07 14:00** : incident clos. Le DPO conclut à un `risk` faible sans notification aux personnes (les données lues étaient celles de l'organisation, pas de personnes physiques hors le nom de l'administrateur lui-même), consigné dans le registre des violations du projet conformité.

## Cause

Le support considérait qu'un ticket ouvert depuis une session authentifiée prouvait l'identité, alors que la session est exactement ce qu'un attaquant possède après un vol de mot de passe. Le changement d'e-mail et de téléphone, qui sont les canaux de confirmation des opérations sensibles, était une action de support ordinaire sans vérification supplémentaire ni délai.

## Ce qui a bien marché

- La période de refroidissement de 72 h sur l'IBAN. Sans elle, l'argent partait le 6 au soir. C'est la mesure de 2025 qui a payé.

- L'historique des contacts dans l'audit a permis de joindre la vraie personne à son ancien numéro.

## Actions

- **Script d'identité pour le support** (HF-2188, livré le 2026-05-20) : un changement d'e-mail, de téléphone ou de mot de passe à la demande du support exige une vérification hors de la session : appel sortant au numéro **enregistré avant la demande**, ou vidéo avec pièce pour un compte sans téléphone. Le back-office refuse la modification tant que la case « vérifié par appel sortant au +XX...XX » n'est pas cochée avec le nom de l'agent, tracée dans l'audit.

- **Délai de 24 h sur les changements de canaux de confirmation** faits par le support, avec e-mail et SMS à l'ancienne adresse et à l'ancien numéro : « vos coordonnées vont changer, si ce n'est pas vous, appelez ».

- **Alerte pays inhabituel étendue aux comptes clients administrateurs** (`shipper_admin`, `carrier_admin`), en information au client par e-mail plutôt qu'en page à la rota : 1 100 comptes concernés, environ 30 e-mails par jour, 2 retours « ce n'était pas moi » en trois semaines, tous deux vrais.

- **Vérification de compromission du mot de passe à la connexion** (et plus seulement à la création), avec réinitialisation forcée si compromis. 4,2 % des comptes clients concernés au premier passage.

- Les modèles de messages au client pour ce type de cas ont été ajoutés à [[incident-comms-templates]].

Ticket : HF-2187.
