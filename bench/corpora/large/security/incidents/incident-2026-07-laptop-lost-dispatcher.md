---
name: incident-2026-07-laptop-lost-dispatcher
description: On 2026-07-03 a shipper's dispatcher lost a logged-in laptop; their admin revoked sessions in 20 min; set the pattern for customer-side device loss
type: feedback
status: active
verified: 2026-07-24
---

# Ordinateur perdu chez un chargeur (juillet 2026)

Post-mortem court `2026-07-03-customer-laptop.md`. **SEV3** : pas d'exposition constatée, les contrôles côté client ont fonctionné. Consigné parce que c'est le premier incident déclaré chez nous pour un appareil qui n'est pas le nôtre, et qu'il a fallu décider ce qu'on fait dans ce cas.

## Ce qui s'est passé

- **2026-07-03 16:50 UTC** : une dispatcheuse d'un chargeur (organisation de 60 utilisateurs) laisse son ordinateur portable dans un train. Session web `app.halden.example` ouverte, rôle `shipper_dispatcher`, plus la messagerie de l'entreprise.

- **17:05** : elle prévient son administrateur chez le chargeur.

- **17:10** `[audit]` : l'administrateur utilise « Déconnecter partout » sur son compte depuis `/settings/members` (fonction du projet IAM, rotation du `security_stamp`), puis désactive temporairement le compte. `auth.session_revoked` avec `details.reason = 'admin_logout_everywhere'`.

- **17:30** `[ticket]` : l'administrateur ouvre un ticket support « ordinateur perdu, j'ai déconnecté le compte, pouvez-vous vérifier s'il y a eu de l'activité ». Le support L2 déclare l'incident à 17:38 pour avoir une trace, SEV3.

- **17:45** `[audit]` : requête sur `audit_events` et sur les journaux d'accès API pour ce compte entre 16:50 et 17:10 : dernière requête à 16:41 (avant la perte), rien ensuite. Le jeton d'accès de 15 minutes a expiré à 16:56 au plus tard ; le rafraîchissement aurait été refusé à partir de 17:10.

- **18:00** : réponse au client avec les faits, le modèle « réponse au client » de [[incident-comms-templates]] adapté. Incident clos.

- **2026-07-07** : l'ordinateur est retrouvé par la compagnie ferroviaire, disque chiffré, remis à l'entreprise.

## Ce qu'on a décidé pour ces cas

La question était : un appareil client perdu est-il notre incident ? Réponse tenue : **c'est l'incident du client, on l'aide et on garde une trace**.

- On déclare un SEV3 chez nous quand un client nous signale une perte ou un vol d'appareil avec session ouverte, pour que la vérification d'activité soit faite par quelqu'un avec la permission de la faire, tracée, et que la réponse au client soit écrite depuis un modèle.

- La vérification, c'est : `audit_events` et journaux d'accès pour le compte, de l'heure de la perte à la révocation, plus les 24 heures suivantes pour les tentatives de rafraîchissement refusées (qui indiqueraient que quelqu'un a essayé). Une commande `iam:account-activity <user_id> --from <ts> --to <ts>` a été ajoutée pour que le support L2 n'écrive pas la requête à la main (HF-2201).

- On ne notifie rien à personne au-delà du client : ses données sont les siennes, il décide.

- Si l'activité montre des actions **après** la perte, ça devient un SEV2 chez nous (compte client utilisé par un tiers, comme dans [[incident-2026-05-support-account-takeover]]) et on suit ce chemin.

## Ce qui a bien marché

- Le client savait qu'il pouvait révoquer lui-même, en 20 minutes, sans nous appeler. C'est l'argument pour continuer à mettre ces boutons dans l'interface client plutôt que de tout passer par le support.

- La durée de vie de 15 minutes des jetons d'accès a fait son travail : même sans révocation, la session aurait été morte à 16:56.

## Ce qu'on a ajouté

- Une page d'aide `docs.halden.example/security/lost-device` : quoi faire dans l'ordre (déconnecter partout, désactiver, nous prévenir si vous voulez une vérification), en français et en anglais. Liée depuis la page des membres.

- Le rapport d'activité en libre-service pour les administrateurs clients (`GET /v2/members/{id}/activity`, 7 derniers jours) est passé en priorité produit : le client aurait pu vérifier lui-même. Prévu T4 2026.

Ticket : HF-2200.
