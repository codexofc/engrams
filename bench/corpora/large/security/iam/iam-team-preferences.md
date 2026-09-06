---
name: iam-team-preferences
description: The IAM group wants permission strings over role checks, deny by default with logged denials, role changes by migration, and no per-object ACLs
type: user
status: active
verified: 2026-04-14
---

# Préférences de l'équipe IAM

Trois personnes, partagées avec l'équipe API. Ce qui revient en revue de code et en discussion d'architecture, pour que les nouveaux arrivants ne le redécouvrent pas à leurs dépens.

- **On vérifie une permission, pas un rôle.** Tout `hasRole` en dehors de `StaffGroupToRoleMapper` est refusé en revue, sans exception. Voir [[roles-permissions-model]] pour le pourquoi.

- **Refus par défaut, et le refus se voit.** Un voter qui s'abstient sur une permission connue, un `catch` qui avale une `AccessDeniedException`, un endpoint sans `#[IsGranted]` : refusés. Un refus est journalisé, on préfère un client qui se plaint d'un 403 qu'un accès silencieux.

- **Les changements de rôles système passent par une migration**, jamais par l'interface d'administration. La ligne `role_permissions` modifiée est dans le diff de la MR, avec le ticket qui l'explique. L'interface ne touche que les rôles personnalisés des organisations.

- **Pas d'ACL par objet.** Chaque fois qu'une demande produit « seulement certains utilisateurs doivent voir ce chargement », on répond par un lien signé ou par une organisation distincte, jamais par une table d'ACL. Voir [[tenant-isolation-checks]] pour la raison de fond : on veut pouvoir prouver l'isolation par une requête, pas par la lecture du code.

- **Une clé, un usage.** Une clé API partagée entre deux systèmes est un ticket de rotation à venir. On le dit aux clients dans la doc, on le fait chez nous avec les comptes de service ([[service-accounts-convention]]).

- **Les secrets ne s'affichent qu'une fois**, à la création, et jamais dans le back-office. Point acquis depuis [[incident-2025-11-api-key-in-support-ticket]], plus discuté.

- **L'audit dans la même transaction.** Pas d'outbox pour `audit_events`, une ligne d'audit qui ne correspond pas à un changement réel est pire que rien.

- **On écrit les tickets IAM en anglais**, contrairement au reste de la plateforme (qui les écrit en français), parce que les questionnaires de sécurité des clients citent nos tickets et que la moitié de ces clients ne lisent pas le français. Le code et les identifiants sont en anglais partout de toute façon.

- **On dit non aux délais de grâce « pour ne pas gêner »** au-delà de ce qui est mesuré et écrit : 15 minutes pour un jeton d'accès, 7 jours pour une rotation de clé, 30 minutes pour une impersonation. Quand quelqu'un demande plus, la réponse est « montre-moi le cas », et le cas justifie en général un autre mécanisme.

Ce qui n'est pas une préférence mais une fatigue connue : on refait la même explication sur « pourquoi mon rôle ne change pas tout de suite » à chaque nouveau membre du support. La réponse tient dans [[session-revocation-on-role-change]], la lire avant de poser la question.
