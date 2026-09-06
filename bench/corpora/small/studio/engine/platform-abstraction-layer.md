---
name: platform-abstraction-layer
description: givre-platform hides OS and GPU behind traits, two GPU backends, no other crate names a platform, low-end port status
type: project
status: active
verified: 2026-06-10
---

## Principe

`givre-platform` est la seule crate qui connaît un système d'exploitation ou une API graphique. Elle expose des traits (`Window`, `Gpu`, `Fs`, `Input`, `Clock`) et une fonction `platform::init()` qui renvoie les implémentations du binaire courant. Aucune autre crate ne contient de `cfg(target_os)` ; `cargo deny`-like lint maison `no-platform-cfg` le vérifie en CI.

## Backends GPU

Deux aujourd'hui : le backend bureau (API graphique moderne du bureau) et le backend de la cible basse (console portable, API propriétaire). Le trait `Gpu` est bas niveau : buffers, textures, pipelines, passes, barrières. Le frame graph ([[renderer-frame-graph]]) est écrit une fois contre ce trait.

Ce qui diffère entre backends et qui a demandé du travail :

- Les formats de texture : BC7 partout sur bureau, mais la cible basse préfère un format propriétaire pour les normal maps. Le pipeline d'assets produit les deux ; le moteur ne convertit jamais au runtime.
- La mémoire : la cible basse a une mémoire unifiée ; le backend expose `Gpu::memory_kind()` et le streamer ([[asset-streaming-budget]]) adapte ses budgets.
- Les shaders : compilés hors ligne par backend ([[shader-compile-cache]]).

## État du port cible basse (juin 2026)

- Fonctionnel : rendu, audio, entrée, sauvegarde, streaming.
- 60 images par seconde tenues sur 5 des 7 scènes de référence ; `port_nuit` et `tempete` tombent à 48 à 52. Le coût est GPU (volumétrique et ombres), pas CPU.
- Manque : la certification (contraintes de suspension et de reprise), en cours avec le fournisseur, ticket BR-402.

## Décisions

- Pas de troisième backend tant que Marée n'est pas sorti. Une demande mobile a été refusée en mars.
- Le trait `Gpu` n'a pas d'API « facile » au-dessus ; le confort est dans le frame graph, pas dans l'abstraction. Deux couches d'abstraction graphiques étaient le problème de 2024.
