---
name: api-keys-legacy-plaintext
description: Until December 2025 API keys were 40 hex chars stored in clear in api_keys.secret, looked up by equality, never expiring, and visible to any staff user in the back-office
type: reference
status: archived
superseded_by: [[api-key-hashing-and-prefix]]
verified: 2025-11-10
---

# Anciennes clés API (avant HF-2087)

Note conservée pour comprendre les vieilles clés encore en circulation et les tickets de support qui les mentionnent. Le format et le stockage actuels sont dans [[api-key-hashing-and-prefix]].

## Ce que c'était

- Une clé était `bin2hex(random_bytes(20))`, soit 40 caractères hexadécimaux, sans préfixe. Impossible à distinguer d'un hash SHA-1 dans un log ou un dépôt, ce qui rendait la détection de fuite quasi impossible.

- Stockée en clair dans `api_keys.secret`, index unique sur la colonne. Le lookup était `WHERE secret = :secret`. Toute personne avec un accès lecture à la base de production voyait toutes les clés de tous les clients.

- Pas de date d'expiration. `expires_at` existait dans la table depuis 2023 mais l'interface ne la remplissait jamais.

- Visible dans le back-office : la page d'une organisation listait ses clés en clair, avec un bouton copier. Le support l'utilisait pour « aider » les clients qui avaient perdu leur clé, en la leur renvoyant par e-mail ou dans le ticket.

## Pourquoi c'est devenu intenable

L'incident [[incident-2025-11-api-key-in-support-ticket]] a été le déclencheur, mais le constat était antérieur : le pentest d'octobre 2025 avait classé le stockage en clair en sévérité haute, et l'audit d'accès du T3 2025 avait compté 23 comptes staff capables de lire toutes les clés.

## Ce qu'il faut savoir pour le support

- Une clé de 40 hexadécimaux sans `hfk_` est une clé ancienne. Elle fonctionne encore si elle n'a pas expiré (une date d'expiration a été posée rétroactivement à 365 jours lors de la migration HF-2095).

- Son préfixe affiché dans l'interface est constitué de ses 6 premiers caractères hexadécimaux, calculé lors de la migration.

- On ne peut plus la lire. Un client qui l'a perdue doit en créer une nouvelle, avec la rotation décrite dans [[api-keys-lifecycle]].
