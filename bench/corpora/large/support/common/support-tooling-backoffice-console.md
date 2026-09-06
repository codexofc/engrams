---
name: support-tooling-backoffice-console
description: Outillage support : Deskline pour les tickets, le back-office bo.halden.example avec impersonation lecture seule, hfctl pour les actions, la réplique pg-ro.hf.internal pour L2, Grafana pour les logs
type: reference
status: active
verified: 2026-06-18
---

# Outils de l'équipe support

Cinq outils, pas plus. Tout ce qui suit est accessible depuis le VPN, rien n'est exposé sans SSO.

## Deskline

Le ticketing. Hébergé chez l'éditeur, connecté en SSO. Une vue par plan ([[support-sla-tiers]]), une vue « escaladés », une vue « incident ». Les macros sont dans un dépôt Git (`support-macros`, un fichier YAML par macro) et synchronisées chaque nuit vers Deskline : on ne modifie pas une macro dans l'interface, la synchro écraserait la modification.

Les champs personnalisés qui comptent : `org_id`, `load_id`, `invoice_id`, `driver_id`, `category` (voir [[support-ticket-tagging-taxonomy]]). Un ticket escaladé sans `org_id` est renvoyé en L1.

## Back-office

`bo.halden.example`, une application Symfony à part qui parle à l'API interne. Ce qu'on y fait :

- chercher une organisation, un chargement, une facture, un chauffeur, un abonnement webhook

- voir la chronologie d'un chargement (`load_events`) avec les acteurs

- voir les livraisons webhook d'un abonnement, avec `last_error`

- impersonner un utilisateur chargeur ou transporteur **en lecture seule** : on voit son écran, on ne peut rien cliquer qui écrive. Une bannière rouge, une entrée dans `sys_audit_log` avec le login support.

L'impersonation en écriture a existé jusqu'en septembre 2025 et a été retirée (HF-3001) après qu'un agent a accepté une offre à la place d'un chargeur en voulant lui montrer le bouton. Depuis, toute action passe par `hfctl`, qui laisse une trace nominative.

## hfctl

CLI interne (Go, binaire signé, distribué par `registry.hf.internal`). Rôles : `support-read` pour L1, `support-write` pour L2. Chaque commande est journalisée dans `sys_audit_log` avec l'utilisateur SSO, la commande et les arguments. Les commandes les plus utilisées sont dans [[support-hfctl-cli-cheatsheet]].

`hfctl` parle à l'API admin (`admin-api.hf.internal`), jamais à la base. Ce que l'API admin ne permet pas, `hfctl` ne le permet pas non plus, et c'est voulu.

## Réplique SQL

`pg-ro.hf.internal`, PostgreSQL en lecture seule, retard habituel sous 2 s. Accès L2 uniquement, avec un rôle `support_ro` qui ne voit pas les colonnes chiffrées ni les tables `sys_secret_*`. On s'en sert pour les questions que le back-office ne pose pas : « combien de chargements de cette org sont bloqués en `DISPATCHED` depuis plus de 48 h », « quels abonnements webhook ont plus de 20 `DEAD` cette semaine ».

Règle : une requête utile trois fois devient une commande `hfctl` ou un écran du back-office. On a un ticket ouvert pour ça (HF-3140) avec la liste.

## Grafana

`grafana.hf.internal`, dossier « Support ». Le tableau « Org overview » prend un `org_id` et montre les requêtes API, les erreurs, les livraisons webhook et les synchros app chauffeur sur 24 h. C'est la première chose à ouvrir quand un client dit « rien ne marche ». Les logs sont interrogeables par `org_id` et `load_id`, la rétention est de 30 jours.

## Ce qu'on n'a pas

Pas d'accès au cluster Kubernetes, pas d'accès à Payla ni à Verifid en direct (le back-office montre l'état, l'action est chez la finance ou le backend), pas d'accès au Kafka. Quand un playbook demande quelque chose qu'on n'a pas, il dit vers qui escalader.
