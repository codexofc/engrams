---
name: team-prefers-explicit-repositories
description: The backend team wants one explicit repository class per aggregate with named query methods, no generic findBy() calls from services, no magic __call finders
type: user
status: active
verified: 2026-02-10
---

# Préférence : repositories explicites

Décision d'équipe (rétro de janvier 2026, 5 personnes pour, 0 contre, 1 absent) : les services ne parlent à Doctrine qu'à travers des méthodes nommées sur un repository dédié.

Concrètement :

- `LoadRepository::findOpenNear(Point $p, int $radiusM): iterable<Load>`, pas `$repo->findBy(['status' => 'OPEN'])` depuis un service. `findBy`, `findOneBy` et les `findByXxx` magiques sont autorisés dans le repository lui-même, pas ailleurs. PHPStan a la règle `NoGenericFinderOutsideRepositoryRule`.

- Un repository par racine d'agrégat : `LoadRepository`, `BidRepository`, `CarrierRepository`, `InvoiceRepository`. Pas de repository pour `LoadEvent` ou `Address`, on y accède par leur parent.

- Les méthodes disent ce qu'elles retournent et pourquoi : `findForUpdate()` (verrou, voir [[bid-acceptance-race-condition]]), `findForListing()` (fetch joins, voir [[n-plus-one-loads-list-fix]]), `findForInvoicing()`.

- Un repository ne retourne jamais un `QueryBuilder` à un service. Il retourne des entités, des DTO ou un `iterable`.

Pourquoi l'équipe y tient : chercher "qui lit les chargements ouverts" se fait en une recherche sur `findOpen`, au lieu de chercher tous les `findBy` avec un tableau qui contient `status`. Et quand une requête devient lente, il y a un seul endroit à modifier.

Ce qu'on tolère : les commandes de maintenance sous `src/Command/Maintenance/` peuvent utiliser DBAL directement, voir [[doctrine-dql-vs-native-sql-advice]].
