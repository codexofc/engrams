---
name: asset-streaming-budget
description: Streaming budget for Marée: 1.2 GB textures and 300 MB meshes on the low-end target, 64 m chunks, 2 rings ahead, eviction order
type: reference
status: active
verified: 2026-04-20
---

## Budgets (cible basse)

| Ressource | Budget résident | Mesuré scène `port_nuit` |
|---|---|---|
| textures | 1 200 Mo | 1 080 Mo |
| meshes | 300 Mo | 240 Mo |
| audio (banques chargées) | 120 Mo | 95 Mo |
| animations | 80 Mo | 60 Mo |

Les budgets sont des constantes dans `givre-core/src/budget.rs` et l'overlay `F3` les affiche en pourcentage. Un dépassement en dev déclenche un avertissement rouge ; en release, l'éviction.

## Découpage et anticipation

Le monde est découpé en chunks de 64 m. Le streamer charge les chunks à 2 anneaux du chunk du joueur (25 chunks), en priorisant ceux dans la direction du déplacement. À 12 m/s (vitesse max du bateau), un chunk a 5 s pour arriver, et le p99 de chargement d'un chunk depuis le disque cible bas est 1,4 s.

## Éviction

Ordre : mips hautes des textures non visibles depuis 3 s, puis meshes des chunks à plus de 3 anneaux, puis banques audio non référencées, puis animations. Jamais les ressources marquées `pinned` (UI, joueur, bateau).

## Format sur disque

Les paquets sont produits par le pipeline d'assets de l'équipe outils, un fichier par chunk, textures en BC7 ou BC5 selon la règle de compression de l'outil. Le moteur ne lit que ce format ; pas de chargement de PNG au runtime, même en dev (le hot reload passe par le pipeline).

## Piège rencontré

Le streamer priorisait par distance euclidienne ; sur la carte en anneau de Marée, un chunk de l'autre côté du bras de mer était « proche » mais inaccessible avant 40 s. Depuis mars 2026 la priorité utilise la distance de navigation précalculée par l'éditeur.
