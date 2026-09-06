---
name: editor-hot-reload
description: The editor hot-reloads textures and meshes in under 2 s by watching assets/src and running the pipeline, shaders and levels differ
type: project
status: active
verified: 2026-04-14
---

## Ce qui est rechargé à chaud

- **Textures et meshes** : le watcher voit le fichier source changer, lance `brumepipe` pour cet asset seul ([[asset-pipeline-overview]]), et le moteur remplace la ressource dans son pool à la prochaine image. Délai médian 1,8 s pour une texture 2048, 0,9 s pour un mesh. Le moteur garde l'ancien tant que le nouveau n'est pas prêt, donc jamais de trou.
- **Audio** : rechargement des banques par le même chemin, avec un arrêt des voix qui y jouent (le mixeur ne relit pas une banque en cours de lecture).
- **Shaders** : recompilation de la variante en cours et remplacement au vol ; les autres variantes du même fichier sont marquées obsolètes et recompilées en tâche de fond. 3 à 20 s selon le nombre d'includes.
- **Niveaux** : pas de rechargement à chaud du niveau entier. L'éditeur applique les modifications par commandes (déplacer, ajouter, supprimer) directement dans le monde en cours ; le fichier de niveau est réécrit à la sauvegarde. Le système d'annulation qui va avec est dans [[level-editor-undo]].
- **Code** : non. Le rechargement à chaud de code Rust a été essayé en 2025 et abandonné ; la liaison dynamique en dev garde le cycle à 4 s, ce qui suffit.

## Comment ça marche

L'éditeur et le jeu sont le même binaire (`maree --editor`). Le watcher est un thread qui pousse des événements `AssetChanged(path)` dans la file de commandes du moteur ; le moteur les traite dans l'étape `Input` pour ne pas concurrencer le rendu.

## Pièges

- Un enregistrement depuis un outil d'art écrit souvent le fichier en deux fois (fichier temporaire puis renommage). Le watcher attend 200 ms de silence sur un chemin avant de lancer le pipeline, sinon on importait un fichier à moitié écrit.
- Le rechargement d'une texture référencée par 300 matériaux prenait 4 s parce que chaque matériau était rebâti ; depuis mars, les matériaux référencent la texture par identifiant et rien n'est rebâti.
