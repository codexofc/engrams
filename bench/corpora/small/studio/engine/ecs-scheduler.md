---
name: ecs-scheduler
description: The Givre ECS runs systems in four stages with a dependency graph from access declarations, 6 ms CPU budget on the low-end target
type: reference
status: active
verified: 2026-04-08
---

## Étapes

Quatre étapes par image, dans l'ordre : `Input`, `Simulation`, `PhysicsSync`, `Presentation`. Un système déclare son étape et ses accès (`Read<T>`, `Write<T>`, `Res<R>`). Le scheduler construit un graphe de dépendances par étape à partir des accès et exécute en parallèle ce qui ne se chevauche pas. Deux systèmes qui écrivent le même composant sont ordonnés par leur déclaration explicite `after(...)` ; sans déclaration, le scheduler refuse de démarrer (erreur `AmbiguousOrder`) plutôt que de choisir.

## Budget CPU (cible basse, 4 cœurs, scène `port_nuit`)

| Étape | Budget | Mesuré |
|---|---|---|
| Input | 0,2 ms | 0,1 ms |
| Simulation | 3,0 ms | 2,4 ms |
| PhysicsSync | 0,8 ms | 0,6 ms |
| Presentation (extraction rendu) | 2,0 ms | 1,7 ms |
| total | 6,0 ms | 4,8 ms |

La physique elle-même tourne sur son propre pas ([[physics-fixed-step]]) et n'est pas dans ce tableau.

## Ce qui coûte

Dans `Simulation`, les trois systèmes les plus chers : IA de navigation des PNJ (0,9 ms), simulation de l'eau côtière (0,7 ms), animation (0,4 ms). L'IA est le seul qui n'est pas encore parallèle : elle écrit dans un composant partagé `NavGrid`, ticket BR-311.

## Règles

- Un système ne fait pas d'allocation par image. Le mode dev compte les allocations par système ([[memory-arena-allocators]]) et l'overlay les affiche.
- Pas de `Write<T>` sur un composant qu'on lit seulement « pour l'instant ». L'accès en écriture sérialise le graphe.
- Les événements entre systèmes passent par `Events<E>`, vidés à la fin de l'étape ; un événement lu à l'étape suivante est un bug, le scheduler le signale en dev.
