---
name: role-assignment-approval-flow
description: Assigning shipper_admin, carrier_admin or any staff role needs a second approver within 48 h via role_assignment_requests; waived only for a first admin
type: project
status: active
verified: 2026-05-06
---

# Flux d'approbation pour les rôles sensibles (HF-2137)

## Règle

Attribuer un rôle **sensible** ne se fait pas en un clic. Sont sensibles : `shipper_admin`, `carrier_admin`, tout rôle `staff_*`, et tout rôle personnalisé qui contient `member.remove`, `apikey.create` ou `invoice.void`. La liste est calculée par `Role::isSensitive()`, pas maintenue à la main.

Pour ces rôles, `POST /v2/organizations/{id}/members/{user}/roles` ne modifie rien : il crée une ligne dans `role_assignment_requests (id, organization_id, user_id, role_id, requested_by, requested_at, approved_by, approved_at, rejected_at, expires_at)` et répond 202 avec l'identifiant de la demande. Un second administrateur de la même organisation (ou un `staff_admin` pour les rôles staff) approuve depuis l'interface ou par `POST /v2/role-requests/{id}/approve`. L'approbateur ne peut pas être le demandeur, c'est vérifié par une contrainte `CHECK (approved_by <> requested_by)` en plus du code.

Délai : 48 h, ensuite la demande expire (`expires_at`), il faut la refaire. Le job `iam:expire-role-requests` tourne toutes les heures.

Les rôles non sensibles (`shipper_dispatcher`, `carrier_driver`, etc.) sont attribués immédiatement comme avant.

## L'exception : le premier administrateur

Une organisation qui vient d'être créée n'a personne pour approuver. Le premier `shipper_admin` ou `carrier_admin` est posé par le flux d'onboarding sans approbation, avec `role_assignment_requests.auto_approved_reason = 'first_admin'`. Une organisation qui n'a plus qu'un seul admin et veut en nommer un second est dans la même situation : l'unique admin demande, et c'est le support (`staff_support_l2`) qui approuve après vérification par téléphone. Cas rare, 14 fois depuis mars 2026.

## Pourquoi

Deux raisons dans le ticket :

1. Le questionnaire de sécurité de deux chargeurs demandait « existe-t-il une séparation des tâches pour l'attribution de droits d'administration ». La réponse était non.

2. L'incident [[incident-2026-01-stale-admin-role]] côté staff : un rôle `staff_admin` posé par une seule personne, sans que personne d'autre ne le sache.

## Ce que ça a changé

- Les demandes de rôle sensible : 320 en avril 2026, 91 % approuvées en moins de 4 h, 6 % expirées (le demandeur avait oublié de prévenir le collègue), 3 % rejetées.

- Le support reçoit des tickets « j'ai donné le rôle admin et rien ne se passe ». L'interface affiche pourtant « En attente d'approbation par un autre administrateur » ; la réponse tient dans la fiche support, avec le lien vers la liste des demandes en attente.

- Côté staff, une demande de rôle `staff_*` est approuvée par un `staff_admin` différent du demandeur ; comme les rôles staff viennent des groupes Idento (voir [[sso-oidc-provider-setup]]), c'est en fait l'ajout au groupe qui est soumis au flux, via le webhook d'événements d'administration d'Idento qui crée la demande et un script qui retire du groupe si elle n'est pas approuvée sous 48 h.

## Audit

`member.role_requested`, `member.role_request_approved`, `member.role_request_rejected`, `member.role_request_expired` dans [[audit-trail-schema]], et `member.role_changed` au moment de l'application effective avec `details.request_id`.

## Écarté

- Une approbation par e-mail (lien à cliquer). Trop facile à hameçonner. L'approbateur doit être connecté, et pour les rôles staff, ré-authentifié.

- Un délai plus long que 48 h. Une demande qui traîne une semaine est une demande dont personne ne se souvient de la raison.
