---
name: renderer-frame-graph
description: Givre builds a frame graph each frame from declared passes, culls and aliases, 11 passes at 2.9 ms GPU on the reference card
type: reference
status: active
verified: 2026-05-12
---

Le rendu de Givre (notre moteur Rust) passe par un frame graph reconstruit à chaque image. Chaque passe est un type qui implémente `RenderPass` avec `declare(&mut GraphBuilder)` (ressources lues et écrites) et `execute(&PassContext)`. Le graphe est compilé en début d'image : passes non atteignables depuis la sortie écran retirées, ressources transitoires aliasées dans un même heap quand leurs durées de vie ne se chevauchent pas, barrières insérées entre passes.

L'ancien renderer à passes câblées est décrit dans [[engine-old-renderer]].

## Passes du build Marée (mai 2026)

Dans l'ordre : depth prepass, shadow cascades (4), GBuffer, SSAO, lumière directe, lumière indirecte (probes), transparents, volumétrique, post (bloom, tonemap), UI, debug (retirée en release). 11 passes en release, 12 en dev.

Coût GPU sur la carte de référence bureau (celle des postes de dev) en 1440p : 2,9 ms hors scène, 8,1 ms sur la scène de test `port_nuit`. L'aliasing des transitoires économise 210 Mo de mémoire vidéo sur cette scène par rapport à l'allocation naïve.

## Règles

- Une passe ne garde pas d'état entre images. Ce qui doit persister (historique TAA, probes) est une ressource importée déclarée `persistent`, jamais un champ de la passe.
- Déclarer une lecture qu'on ne fait pas coûte une barrière inutile ; le validateur en mode dev compte les ressources déclarées et non touchées et les affiche dans l'overlay (`F3`).
- Ajouter une passe : un type, une ligne dans `passes.rs`, et un test dans `frame_graph_tests.rs` qui vérifie qu'elle est culled quand sa sortie n'est pas consommée.

## Ce qui reste à faire

Le graphe est recompilé à chaque image (0,08 ms CPU, mesuré). Un cache par empreinte des déclarations est prévu mais pas prioritaire : le CPU n'est pas le goulot, voir [[ecs-scheduler]] pour ce qui l'est.

## Détail des passes qui ont demandé un choix

**Cascades d'ombres.** Quatre cascades à 0 à 12 m, 12 à 40, 40 à 140, 140 à 500, résolution 2048 pour les deux premières et 1024 pour les deux autres sur la cible basse (2048 partout sur bureau). Les cascades lointaines ne sont rendues qu'une image sur deux et interpolées ; personne ne l'a vu en playtest et ça économise 0,6 ms sur la cible basse.

**SSAO.** Demi-résolution, 8 échantillons, flou bilatéral. Un essai en pleine résolution à 16 échantillons coûtait 1,1 ms de plus pour une différence que seul l'artiste lumière voyait. Gardé en demi.

**Volumétrique.** C'est la passe la plus chère sur `tempete` (2,3 ms sur la cible basse). Grille de 160 × 90 × 64 froxels, réprojection temporelle, 1 échantillon par froxel par image. Sur la cible basse la grille passe à 120 × 68 × 48 automatiquement quand le budget GPU de l'image précédente dépasse 15 ms (`GpuBudgetGovernor`), et remonte quand trois images consécutives sont sous 13 ms. Le gouverneur est la seule chose du renderer qui décide seule d'une qualité ; les autres réglages sont fixes par plateforme.

**Transparents.** Triés par distance, rendus après la lumière, sans réfraction (la mer utilise un rendu à part dans la passe de lumière directe, pas la passe transparente, parce que c'est 60 % de l'écran et qu'un tri par distance n'a pas de sens pour elle).

## Ressources importées et persistantes

Les ressources qui traversent les images sont déclarées `persistent` avec un nom stable : `taa_history`, `probe_atlas`, `shadow_cascade_{0..3}` (persistantes parce que les cascades lointaines sont réutilisées une image sur deux), `sea_displacement` (calculée en compute au début de l'image, lue par trois passes). Une ressource persistante est allouée une fois, jamais aliasée, et redimensionnée seulement au changement de résolution, avec une image de transition où l'historique TAA est ignoré.

Le validateur en mode dev vérifie qu'une ressource déclarée `persistent` est bien lue à l'image suivante ; une ressource persistante jamais relue est signalée, c'est arrivé avec un ancien historique de bloom qui a survécu six mois pour rien, 24 Mo.

## Capture et comparaison

Les 40 captures de référence (10 scènes × 4 points de vue) sont rendues par la lane code de la CI sur la machine de référence bureau, en 1440p, comparées à la référence stockée avec une tolérance par pixel de 2/255 sur 0,1 % des pixels au plus. Une passe modifiée met à jour la référence par `maree --capture-reference --scene port_nuit` et le diff d'images est joint à la merge request ; le relecteur regarde les images, pas les chiffres.

Depuis le passage au frame graph, 11 merge requests ont changé une référence ; 3 étaient des régressions rattrapées grâce au diff (un bloom doublé, une cascade décalée d'un texel, une passe de debug laissée active).

## Chiffres au 2026-05-12, cible basse, `port_nuit`, 1080p

| Passe | ms GPU |
|---|---|
| depth prepass | 0,4 |
| cascades d'ombres (2 rendues par image) | 1,3 |
| GBuffer | 2,1 |
| SSAO | 0,7 |
| lumière directe (mer comprise) | 3,2 |
| lumière indirecte | 0,9 |
| transparents | 0,5 |
| volumétrique | 1,8 |
| post | 1,1 |
| UI | 0,2 |
| total | 12,2 |

Objectif 16,6 ms, tenu avec 4 ms de marge ; `tempete` est à 15,9 ms avec le gouverneur en grille réduite, sans marge, et c'est la scène qu'on optimise en ce moment.
