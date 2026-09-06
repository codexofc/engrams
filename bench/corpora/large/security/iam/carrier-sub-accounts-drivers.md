---
name: carrier-sub-accounts-drivers
description: Drivers are users with carrier_driver scoped to one carrier, created from a phone number, PIN login, see only their own assignments; agency mode for groups
type: project
status: active
verified: 2026-06-18
---

# Comptes chauffeurs et sous-comptes transporteur (HF-2125)

## Le problème qu'on résolvait

Un transporteur avec trois dépôts partageait un seul compte `carrier_admin` entre ses planificateurs, et ses chauffeurs se connectaient à l'application mobile avec le numéro de téléphone du dépôt. Impossible d'attribuer une action à une personne, impossible de retirer l'accès à un chauffeur parti sans changer le PIN de tout le monde. La revue d'accès du T4 2025 a compté 38 % des comptes transporteur dans ce cas.

## Modèle retenu

Un **chauffeur est un utilisateur** ordinaire (`users`), avec :

- `organization_memberships` sur exactement un transporteur avec le rôle `carrier_driver` (voir [[roles-permissions-model]]). Un chauffeur qui change d'employeur reçoit une nouvelle adhésion et l'ancienne est fermée (`ended_at`), on ne supprime pas l'utilisateur.

- Une identité par **numéro de téléphone** (`users.phone`, unique, format E.164). Pas d'e-mail obligatoire : 40 % des chauffeurs n'en ont pas ou ne le consultent pas.

- Connexion par PIN à 6 chiffres sur l'application mobile, hors ligne possible, tel que décrit par l'équipe mobile. Le PIN est haché côté serveur (Argon2id) et un dérivé est stocké sur l'appareil pour le mode hors ligne.

Ce que voit un chauffeur : `assignment.read` est filtré par `assignments.driver_id = current_user`. Ce n'est pas une ACL par objet mais un filtre de périmètre, appliqué dans `AssignmentRepository::forDriver()` et vérifié par un test qui charge deux chauffeurs du même transporteur et vérifie que chacun ne voit que ses missions. Voir [[tenant-isolation-checks]].

## Création par le transporteur

Un `carrier_planner` ou `carrier_admin` crée un chauffeur depuis `/drivers` avec nom et numéro de téléphone. Le chauffeur reçoit un SMS avec un lien d'activation valable 48 h, choisit son PIN. Pas d'invitation par e-mail. Le transporteur peut « détacher » un chauffeur : l'adhésion est fermée, les sessions sont révoquées ([[session-revocation-on-role-change]]), le chauffeur garde son compte et son historique.

Un numéro déjà connu d'un autre transporteur : on affiche « ce chauffeur a déjà un compte, une demande de rattachement lui a été envoyée ». Le chauffeur accepte depuis l'application, ce qui ferme son adhésion précédente. Cas réel, plus fréquent qu'on ne pensait : 9 % des créations en mai 2026.

## Mode agence

Certains transporteurs sont des groupes avec plusieurs entités juridiques (une organisation chacune chez nous). Un planificateur du groupe veut passer de l'une à l'autre. Depuis HF-2125 un utilisateur peut avoir plusieurs adhésions `carrier_*`, et l'en-tête `X-Organization-Id` sélectionne le contexte (voir [[permission-check-voter-symfony]]). L'interface web propose un sélecteur. Les chauffeurs, eux, restent à une seule adhésion active : un chauffeur ne roule que pour un dépôt à la fois.

## Migration des comptes partagés

On n'a pas forcé. Une bannière dans l'interface transporteur explique et propose de créer les chauffeurs. En juin 2026, 71 % des transporteurs actifs ont au moins deux utilisateurs nommés, contre 62 % en décembre 2025. Le levier qui a le mieux marché : la fonction « signature de POD par le chauffeur » n'est disponible que pour un chauffeur nommé, parce que la signature n'a de valeur que si on sait qui a signé.

## Ce qu'on a écarté

- Un objet `Driver` séparé de `User`. Doublait la gestion des sessions, des révocations, de l'audit. Un chauffeur est un utilisateur avec un rôle.

- Un PIN partagé « de dépôt » comme mode dégradé. C'était exactement ce qu'on voulait faire disparaître.
