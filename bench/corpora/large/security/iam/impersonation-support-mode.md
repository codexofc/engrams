---
name: impersonation-support-mode
description: staff_support_l2 can act as a customer user for 30 min with a stated reason, MFA re-auth, read-only plus a write allow-list, banner shown to the customer
type: project
status: active
verified: 2026-05-28
---

# Mode « voir comme le client » (impersonation)

## Pourquoi il existe

Le support passait des heures à demander des captures d'écran. « Je ne vois pas le bouton », « le chargement n'apparaît pas », « la facture est fausse » : trois quarts des tickets L2 se résolvaient en regardant l'interface telle que le client la voit. Le mode impersonation a été construit en HF-2070 (novembre 2025) et durci trois fois depuis.

## Ce que c'est

Un `staff_support_l2` (permission `staff.impersonate`, voir [[roles-permissions-model]]) ouvre la console support, cherche un utilisateur client, clique « Voir comme », saisit un motif obligatoire (texte libre, minimum 15 caractères, généralement la référence du ticket), se ré-authentifie avec second facteur (voir [[mfa-rollout-backoffice]]) et obtient une session de **30 minutes** sur l'application client dans l'identité de cet utilisateur.

Techniquement : `ImpersonationTokenIssuer` émet un JWT d'accès client normal avec deux revendications en plus, `act` (l'identifiant staff) et `imp` (l'identifiant de session d'impersonation). Pas de refresh token : au bout de 30 minutes la session tombe, il faut recommencer avec un nouveau motif. Le JWT a le même `sst` que l'utilisateur (voir [[session-revocation-on-role-change]]), donc une révocation du client coupe aussi l'impersonation.

## Lecture seule par défaut

La session impersonnée a les permissions de l'utilisateur **filtrées par une liste blanche d'actions d'écriture**. En mai 2026 la liste blanche contient :

- `load.update` limité aux champs de contact et aux instructions de livraison

- `bid.reject` (jamais `bid.accept`)

- `document.upload`

- `member.invite` (renvoi d'une invitation expirée)

Tout le reste des permissions d'écriture est retiré par `ImpersonationPermissionFilter` avant que le voter ne réponde. Concrètement : un agent peut corriger un numéro de téléphone de livraison, pas accepter une offre ni émettre une facture. Pour ces actions, l'agent demande au client de le faire ou passe par le back-office avec sa propre identité et sa propre trace.

Étendre la liste blanche est une décision de l'équipe sécurité, pas du support, et chaque ajout est passé en revue avec la question « que se passe-t-il si un compte support est compromis ».

## Ce que le client voit

- Un bandeau rouge permanent en haut de l'interface web : « Session support en cours (motif) », visible par l'agent, évidemment, mais aussi par le client si l'utilisateur impersonné est connecté en même temps : le bandeau apparaît sur **ses** onglets aussi, poussé par WebSocket, avec « Un agent Halden Freight consulte votre compte ». Choisi délibérément. Le client a le droit de savoir.

- Un e-mail à l'utilisateur impersonné et à l'administrateur de l'organisation à la fin de chaque session, avec la durée, le motif et la liste des actions d'écriture effectuées. Le premier mois, deux clients ont écrit pour demander pourquoi ; depuis on l'annonce dans les conditions d'utilisation et il n'y a plus de question.

## Audit

Chaque requête faite pendant la session porte `acting_as_id` dans `audit_events` (voir [[audit-trail-schema]]) et `impersonation_id` dans les logs applicatifs. Les événements `staff.impersonation_started` et `staff.impersonation_ended` encadrent la session avec le motif. Le rapport hebdomadaire au responsable support liste les sessions par agent, la durée médiane et les actions d'écriture.

Chiffres de mai 2026 : 1 860 sessions, durée médiane 6 minutes, 7 % avec au moins une action d'écriture, 91 % des motifs sont une référence de ticket.

## Durcissements successifs

- HF-2070 (novembre 2025) : version initiale, 60 minutes, permissions complètes de l'utilisateur. Deux semaines.

- HF-2088 (décembre 2025) : passage en lecture seule avec liste blanche, après qu'un agent a accepté une offre « pour débloquer le client » et que le client a contesté.

- HF-2112 (janvier 2026) : 30 minutes, ré-authentification MFA au démarrage, bandeau poussé au client.

- HF-2155 (avril 2026) : e-mail de fin de session au client, rapport hebdomadaire.

## Ce qu'on refuse

- L'impersonation des comptes `staff`. Un staff qui a besoin de voir comme un autre staff a un problème de rôle, pas un problème de support.

- L'impersonation sans utilisateur cible (« voir l'organisation en général »). Il faut choisir un utilisateur, parce que les permissions et les rôles personnalisés dépendent de l'utilisateur, et parce que ça oblige à documenter qui exactement on a regardé.

- Une durée plus longue pour les « cas compliqués ». Trente minutes, on recommence, chaque reprise a son motif.

## Pour le support : la procédure en six lignes

1. Ouvrir la console support, chercher l'utilisateur (pas l'organisation).

2. « Voir comme », saisir la référence du ticket comme motif.

3. Second facteur.

4. Regarder, corriger ce que la liste blanche permet, noter le reste dans le ticket pour que le client le fasse.

5. Fermer la session avec le bouton, ne pas attendre les 30 minutes : le client reçoit l'e-mail de fin plus tôt et la durée dans le rapport est la vraie.

6. Si une action hors liste blanche est nécessaire (annuler un chargement, émettre un avoir), la faire depuis le back-office avec sa propre identité, jamais en demandant une extension de la liste « pour ce cas ».

## Ce que disent les chiffres sur les agents

Le rapport hebdomadaire a montré en avril 2026 qu'un agent faisait trois fois plus de sessions que la médiane, presque toutes de moins d'une minute. Ce n'était pas un abus : il ouvrait une session pour lire un champ que la console support n'affichait pas (le mode de facturation de l'organisation). Le champ a été ajouté à la console, ses sessions sont revenues à la médiane. Le rapport sert à ça : trouver ce qui manque dans les outils, pas à surveiller les gens, et la note qui l'accompagne le dit.
