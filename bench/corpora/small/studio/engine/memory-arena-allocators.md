---
name: memory-arena-allocators
description: Per-frame bump arena of 16 MB, pools per asset type, and a dev allocator that counts allocations per system with zero tolerance
type: reference
status: active
verified: 2026-02-24
---

## Trois allocateurs

1. **Arène par image** (`FrameArena`) : bump allocator de 16 Mo, remis à zéro au début de chaque image. Tout ce qui vit une image (listes de visibilité, commandes de rendu, événements) y passe via `frame.alloc::<T>()` ou `FrameVec<T>`. Pic mesuré sur `port_nuit` : 9 Mo. Un dépassement fait basculer sur l'allocateur global avec un avertissement, jamais un crash.
2. **Pools par type** pour les ressources chargées (textures, meshes, banques audio) : un pool par type avec des blocs de taille fixe, libérés par le streamer ([[asset-streaming-budget]]). Pas de fragmentation, et le budget par type est exactement la taille du pool.
3. **Allocateur global** pour le reste (initialisation, éditeur, outils). En release c'est l'allocateur système ; en dev il est enveloppé dans `CountingAlloc`, qui attribue chaque allocation au système ECS en cours d'exécution.

## Compteur par système

`CountingAlloc` lit un thread-local posé par le scheduler ([[ecs-scheduler]]) et incrémente un compteur par système. L'overlay `F3` affiche les systèmes qui ont alloué pendant l'image. Les systèmes listés dans `hot_systems.toml` (rendu, physique, animation, streaming) ont une tolérance de 0 : une allocation dans l'un d'eux fait échouer le test `no_alloc_in_hot_systems` en CI, qui joue 300 images de `port_nuit`.

Ce test a attrapé, entre autres, un `format!` dans un chemin de log de la physique (appelé même quand le log est désactivé) et un `Vec::push` dans l'extraction de rendu qui grossissait d'une image sur l'autre.

## Règles

- `FrameVec` plutôt que `Vec` dans un système, sauf raison écrite.
- Une `String` par image est presque toujours un bug ; les noms de debug sont des `&'static str` ou des identifiants.
- Les threads de travail ont chacun une sous-arène de 2 Mo prise sur l'arène de l'image, pour ne pas se disputer le pointeur de bump.
