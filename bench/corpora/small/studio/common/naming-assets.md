---
name: naming-assets
description: Asset file names as category_name_variant_suffix in lowercase ASCII, the usage suffix drives the pipeline, wrong names fail import
type: reference
status: active
verified: 2026-01-15
---

Le nom d'un asset source décide de ce que le pipeline en fait ; ce n'est pas de la décoration. Règle appliquée par `brumepipe` à l'import, en erreur, pas en avertissement.

## Forme

`<catégorie>_<nom>_<variante>_<suffixe d'usage>.<ext>`, en minuscules ASCII, chiffres et soulignés, sans espace ni accent.

- Catégorie : `env` (décor), `prop`, `chr` (personnage), `veh` (bateaux), `fx`, `ui`, `sfx`, `mus`, `anim`, `lvl`.
- Nom : un ou deux mots, `harbour_crane`, `buoy_red`.
- Variante : optionnelle, `a`, `b`, `broken`, `night`.
- Suffixe d'usage : obligatoire pour les textures (`_alb`, `_nrm`, `_msk`, `_orm`, `_hdr`, `_ui`, `_raw`, voir la note de compression de l'équipe outils), pour les meshes (`_lod0` à `_lod3`, ou rien pour un mesh sans LOD manuel), pour l'audio (`_loop`, `_oneshot`).

Exemples : `prop_buoy_red_alb.png`, `prop_buoy_red_nrm.png`, `veh_cutter_hull_lod0.fbx`, `sfx_rope_creak_loop.wav`, `lvl_port_nuit.lvl`.

## Pourquoi c'est strict

- Le suffixe choisit la compression et les mips ; un `_alb` enregistré en `_msk` sort en un canal, ce qui est arrivé 30 fois avant que la règle soit une erreur.
- La catégorie choisit le dossier de sortie et le budget de streaming.
- Les noms sont des identifiants dans les fichiers de niveau ; renommer un asset renomme ses références par `brumepipe rename <old> <new>`, jamais à la main.

## Ce qui n'est pas dans le nom

La résolution, la plateforme, la date, le nom de l'artiste. Tout cela est dans les métadonnées du dépôt d'assets ou décidé par le pipeline.

Le glossaire ([[studio-glossary]]) donne les mots à utiliser pour les noms (« cutter » et non « voilier », « harbour » et non « port » dans les identifiants).
