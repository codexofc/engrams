---
name: secure-coding-checklist-php
description: The 14-item PHP security review checklist (IsGranted, tenant scope, hash_equals, no SQL concatenation, upload allow-list, audit event) with expected tests
type: reference
status: active
verified: 2026-04-17
---

# Checklist de revue sécurité (PHP, API et back-office)

Utilisée par les référents ([[security-champions-program]]) et par tout relecteur d'une MR étiquetée `security`. Fichier source : `halden-security/checklists/php.md`. Chaque point dit quoi vérifier et quel test on attend. Un point sans test correspondant est un point qu'on vérifie à la main et qu'on écrit dans la revue.

## Accès

1. **Chaque route a `#[IsGranted]` ou un `denyAccessUnlessGranted`** avec une permission de l'enum, pas un rôle. Test : `RouteAuthorizationTest` échoue sur une route sans attribut et hors de la liste d'exclusion (login, santé, webhooks entrants qui ont leur propre vérification).

2. **Le périmètre tenant s'applique.** Nouvelle entité : annotation `#[TenantOwned]` et `organization_id`, ou inscription dans les tables globales. Nouvel endpoint : ajouté à `TenantIsolationTest` (projet IAM). Tout `runUnscoped` porte un commentaire qui dit pourquoi.

3. **Le sujet du voter est le bon objet.** `isGranted('load.update', $load)` et pas `isGranted('load.update')` quand un objet existe. Sans sujet, le contexte est l'organisation de l'en-tête, ce qui est correct pour une création et faux pour une modification.

## Entrées

4. **Pas de SQL natif par concaténation.** Paramètres liés toujours, y compris pour `ORDER BY` (liste blanche de colonnes) et `LIMIT`. Le filtre SCIM du pentest 2026 est l'exemple à citer.

5. **Un parseur maison a un test de rejet** : un jeu d'entrées hostiles qui doivent toutes produire une erreur contrôlée, pas une exception PHP ni un résultat.

6. **Upload : liste blanche de types vérifiée par le contenu**, pas par l'extension ni le `Content-Type` déclaré. Images ré-encodées, PDF passés par le nettoyeur, tout le reste refusé. Taille maximale explicite.

7. **Désérialisation** : jamais `unserialize` sur une entrée externe. JSON avec un schéma ou un DTO validé.

## Secrets et comparaison

8. **Comparaison de secret en temps constant** : `hash_equals`, jamais `==` ni `===` sur un HMAC, un jeton, une clé.

9. **Un secret ne s'affiche qu'à la création**, ne se journalise pas, ne va pas dans une exception. Le schéma JSON de `audit_events.details` refuse les clés `password`, `secret`, `token` ; le test `AuditDetailsSchemaTest` le vérifie. Pour les logs, `SensitiveParameter` sur les paramètres concernés (PHP 8.2+) pour qu'une trace ne les affiche pas.

10. **Nouveau secret émis par nous** : préfixe `hf?_` déclaré dans [[secrets-handling-conventions]] et dans `gitleaks.toml`, stockage haché, et un mécanisme de rotation avec chevauchement avant la mise en production, pas après.

## Sorties et effets

11. **Action sensible = événement d'audit** dans la même transaction, avec `ActorContext` posé dans les handlers Messenger. Test : la matrice `AuditActionCoverageTest` liste les services qui doivent produire un événement.

12. **Erreur = enveloppe standard**, sans trace, sans chemin, sans requête SQL, en production comme en staging. `APP_DEBUG` est forcé à `false` par le déploiement, un test le vérifie au démarrage.

13. **URL présignée : uniquement via `DocumentUrlSigner`**, TTL 15 minutes, jamais dans un e-mail. `DependencyRulesTest` échoue sur toute autre référence à la fabrique de présignature.

## Dépendances

14. **Nouvelle dépendance** : passée par le proxy, âge supérieur à 48 h, éditeur regardé, et une phrase dans la MR qui dit pourquoi une bibliothèque plutôt que trente lignes. Politique dans [[dependency-updates-policy]].

## Comment on l'utilise

Le relecteur copie la liste dans la revue et coche ou écrit « n/a » avec une raison. Pas de cases cochées en bloc : une revue avec quatorze coches et aucun commentaire est renvoyée. Durée constatée : 20 à 40 minutes de plus qu'une revue ordinaire.

Les 14 points viennent des incidents et des pentests de 2025 et 2026 ; chacun a une histoire, la plupart sont racontées dans les projets IAM et incidents. Un point nouveau s'ajoute avec l'incident ou le constat qui le justifie, jamais « par principe ».
