---
name: access-review-2026-q1-results
description: The Q1 2026 review covered 612 grants, removed 37 (a stale staff_admin, 9 unused service accounts), changed 14, took 9 working days
type: feedback
status: active
verified: 2026-02-10
---

# Revue d'accès T1 2026 : résultats

Ticket `Access review 2026-Q1`, ouverte le 2026-01-06, close le 2026-01-19. Procédure dans [[access-review-quarterly]].

## Chiffres

| Système | Lignes | Gardées | Modifiées | Retirées |
|---|---|---|---|---|
| Groupes Idento (staff) | 84 comptes, 211 appartenances | 196 | 6 | 9 |
| Comptes locaux back-office | 2 | 2 | 0 | 0 |
| Vault | 41 politiques, 18 jetons | 52 | 2 | 5 |
| IAM fournisseur cloud | 27 | 24 | 1 | 2 |
| RBAC Kubernetes (humains) | 38 | 35 | 1 | 2 |
| Rôles Postgres avec login | 19 | 17 | 0 | 2 |
| Consoles fournisseurs (6) | 44 | 37 | 2 | 5 |
| Forge (groupes, jetons de déploiement) | 96 | 90 | 2 | 4 |
| Staff dans des organisations clientes | 23 | 14 | 0 | 9 |
| Comptes de service sans appel 60 j | 9 | 0 | 0 | 9 (désactivés) |
| Autorisations intégrateurs sans appel 90 j | n/a | | | fonctionnalité pas encore livrée |

Total : 612 lignes, 37 retraits, 14 modifications, 9 jours ouvrés.

## Ce qui a été trouvé

**Le `staff_admin` périmé.** Un prestataire parti fin novembre 2025 avait encore une ligne `staff_users` avec ce rôle et une clé API sur une organisation de test. C'est la ligne « comptes Idento inactifs mais présents dans `staff_users` » ajoutée au T4 qui l'a fait remonter. Traité comme incident dans le projet IAM, avec des correctifs structurels.

**Neuf staff dans des organisations clientes réelles.** Six étaient des restes d'onboardings accompagnés en 2025, jamais retirés. Trois étaient des développeurs qui s'étaient ajoutés à un client pour reproduire un bug. Tous retirés, et la règle « organisation `is_probe` ou `is_internal` seulement » écrite noir sur blanc dans la procédure.

**Neuf comptes de service muets.** Dont trois créés pour des expérimentations de la plateforme data en 2025 et deux pour une intégration télématique abandonnée. Désactivés, suppression au T2 si personne ne réclame. Personne n'a réclamé.

**Cinq jetons vault** sans propriétaire identifiable, créés à la main en 2024 avant que la convention de nommage existe. Révoqués un par un avec 24 h d'écart en surveillant les erreurs d'authentification. Un seul a cassé quelque chose (un script de sauvegarde de dashboards Grafana), recréé proprement.

**Consoles fournisseurs.** Cinq retraits, dont deux anciens collègues sur la console du fournisseur d'e-mail transactionnel et un sur celle du KYC. C'est la ligne qui a pris le plus de temps (7 jours) parce que trois consoles n'ont ni export ni SSO et que le titulaire du contrat était en congé.

## Ce qui a changé après cette revue

- La console du fournisseur d'e-mail et celle de l'outil de support sont passées derrière le SSO (février et mars 2026). Deux consoles restent sans SSO, le fournisseur ne le propose pas.

- Le rapport `iam:review:staff-in-orgs` exclut désormais les organisations `is_probe` pour ne pas noyer les vraies lignes.

- Le délai des 10 jours ouvrés a été tenu sans avoir à appliquer la règle de retrait d'office.

## Coût

Environ 22 heures cumulées sur les 11 propriétaires, plus 6 heures pour la rota sécurité. À comparer aux 47 jours du `staff_admin` périmé : c'est le seul chiffre qui compte pour justifier la cadence trimestrielle quand on demande si on ne pourrait pas passer à semestriel.
