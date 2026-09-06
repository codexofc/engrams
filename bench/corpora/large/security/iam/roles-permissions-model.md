---
name: roles-permissions-model
description: Permissions are strings resource.action checked by voters, roles are named bundles per audience (shipper, carrier, staff), stored in role_permissions, and no code ever checks a role name directly
type: reference
status: active
verified: 2026-06-03
---

# Modèle de rôles et de permissions

Le principe posé en HF-2010 (octobre 2025) et jamais remis en cause : **le code vérifie des permissions, jamais des rôles.** Un rôle est un paquet nommé de permissions, un moyen d'attribution. Si un `if ($user->hasRole('ROLE_DISPATCHER'))` apparaît en revue, il est refusé.

## Vocabulaire

- **Permission** : une chaîne `resource.action`, en minuscules, point comme séparateur. Exemples : `load.create`, `load.cancel`, `bid.accept`, `invoice.read`, `apikey.reveal`, `staff.impersonate`. La liste complète vit dans l'enum PHP `Permission` (backed enum, `App\Security\Permission`), et un test vérifie que chaque valeur de l'enum est présente dans `role_permissions` pour au moins un rôle, sinon la permission est morte.

- **Rôle** : une ligne de `roles` (`id`, `code`, `audience`, `label`, `is_system`). Les rôles système (`is_system = true`) ne sont pas modifiables par les clients.

- **Audience** : `shipper`, `carrier`, `staff`. Un rôle appartient à une audience, un compte a des rôles d'une seule audience. Un utilisateur chargeur ne peut pas recevoir un rôle transporteur, la contrainte est en base (`CHECK` via une fonction, voir [[tenant-isolation-checks]]).

- **Portée** : les rôles chargeur et transporteur sont attribués **par organisation**. La table est `organization_memberships (user_id, organization_id, role_id)`. Un même utilisateur peut être `shipper_dispatcher` chez A et `shipper_viewer` chez B.

## Rôles système

### Chargeur

| code | ce qu'il ajoute |
|---|---|
| `shipper_viewer` | lecture des chargements, offres, factures de son organisation |
| `shipper_dispatcher` | + `load.create`, `load.update`, `bid.accept`, `bid.reject`, `load.cancel` |
| `shipper_finance` | viewer + `invoice.*`, `payment.*`, `dispute.open` |
| `shipper_admin` | dispatcher + finance + `member.invite`, `member.remove`, `apikey.*` |

### Transporteur

| code | ce qu'il ajoute |
|---|---|
| `carrier_driver` | `assignment.read` sur ses propres missions, `pod.upload`, `position.report` |
| `carrier_planner` | `bid.create`, `bid.withdraw`, `assignment.assign_driver`, lecture des documents |
| `carrier_admin` | planner + `member.*`, `vehicle.*`, `apikey.*`, `document.upload` |

### Staff

| code | ce qu'il ajoute |
|---|---|
| `staff_support_l1` | lecture sur tout, `note.create`, pas d'impersonation |
| `staff_support_l2` | + `staff.impersonate` (voir [[impersonation-support-mode]]), `load.cancel` avec motif |
| `staff_finance` | + `invoice.void`, `credit_note.issue`, `payout.hold` |
| `staff_admin` | tout, y compris `role.assign` et `apikey.reveal` |

Les rôles staff viennent des groupes du SSO (voir [[sso-oidc-provider-setup]]), les rôles client sont attribués par l'administrateur de l'organisation ou par le support, avec le flux d'approbation décrit dans [[role-assignment-approval-flow]].

## Rôles personnalisés

Depuis HF-2141 (mars 2026), un `shipper_admin` peut créer des rôles personnalisés pour son organisation : `roles.organization_id` non nul, `is_system = false`, composés uniquement de permissions que l'audience `shipper` autorise (liste blanche `Permission::allowedFor(Audience::SHIPPER)`). Cas d'usage réel qui a déclenché la fonctionnalité : un gros chargeur voulait des utilisateurs qui acceptent des offres mais ne créent pas de chargements. Onze organisations l'utilisent en juin 2026.

## Ce qu'on ne fait pas

- Pas de hiérarchie de rôles (`ROLE_ADMIN` hérite de `ROLE_USER`). L'héritage rend la question « qui a le droit de faire X » impossible à répondre par une requête SQL. Un rôle liste ses permissions à plat, la duplication dans `role_permissions` est le prix accepté.

- Pas de permission négative (« tout sauf »). Une permission est accordée ou absente.

- Pas de permissions au niveau d'un chargement individuel (ACL par objet). Le périmètre d'un utilisateur est son organisation, le filtrage se fait par `organization_id`, voir [[tenant-isolation-checks]]. Les cas où un chargeur veut partager un chargement avec un tiers passent par un lien signé, pas par une ACL.

La vérification concrète côté Symfony est décrite dans [[permission-check-voter-symfony]].
