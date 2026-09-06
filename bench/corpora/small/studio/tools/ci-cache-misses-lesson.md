---
name: ci-cache-misses-lesson
description: April 2026: CI lanes went from 11 to 28 minutes because the asset cache evicted the compilation cache, separate quotas since
type: feedback
status: active
verified: 2026-05-08
---

## Ce qui s'est passé

Semaine du 2026-04-13 : la lane code ([[ci-pipeline-layout]]) passe de 11 à 28 minutes sans changement de code. Le cache de compilation, sur le disque local de chaque runner, affichait 30 % de hits au lieu de 95 %.

Cause : le cache d'assets, sur le même disque, avait grossi de 60 Go après le passage des textures en BC7 haute qualité, et le nettoyage par taille du disque (LRU au niveau du disque entier) évinçait les artefacts de compilation, plus anciens que les artefacts d'assets de la nuit.

Personne ne l'a vu pendant trois jours parce que la lane restait verte, juste lente, et que la métrique de durée n'avait pas d'alerte.

## Ce qu'on a changé

- Les deux caches sont sur le stockage réseau du studio, chacun dans son volume avec son quota (compilation 200 Go, assets 400 Go) et son propre nettoyage LRU.
- Une alerte sur la durée de la lane code : au-dessus de 15 minutes sur trois exécutions consécutives, message dans le canal outils.
- Le taux de hits du cache de compilation est affiché dans le résumé de chaque exécution.

## Comment appliquer

- Deux caches sur un même disque avec un nettoyage global finissent toujours par se manger ; un quota par cache ou rien.
- Une CI verte et lente est une CI cassée. Mettre une alerte sur la durée, pas seulement sur le résultat.
- Quand une lane ralentit sans changement de code, regarder les caches avant les tests.
