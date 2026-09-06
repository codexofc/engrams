---
name: asset-pipeline-overview
description: brumepipe turns sources into per-chunk packs per platform, incremental by content hash with a shared cache, 41 min full rebuild
type: reference
status: active
verified: 2026-05-19
---

`brumepipe` (Rust, 12 000 lignes) transforme les sources (`assets/src/`, versionnées dans le dépôt d'assets avec un stockage de gros fichiers) en paquets par chunk et par plateforme (`assets/out/<platform>/<chunk>.pak`) que le moteur charge tel quel.

## Étapes

1. **Import** : lecture des formats sources (FBX et glTF pour les meshes, PNG et EXR pour les textures, WAV pour l'audio, le format de l'éditeur pour les niveaux). Chaque importeur produit un artefact intermédiaire dans `assets/cache/` keyé par le hash du contenu source plus la version de l'importeur.
2. **Cuisson** : compression des textures ([[texture-compression-settings]]), génération des LOD de meshes, encodage audio, compilation des variantes de shaders, calcul des distances de navigation par chunk.
3. **Empaquetage** : assemblage des artefacts par chunk avec une table des matières, alignement 4 Ko.

Chaque étape est incrémentale : un artefact dont le hash d'entrée et la version d'étape n'ont pas changé n'est pas refait. Le cache est partagé sur le réseau du studio, donc un artiste qui importe une texture déjà cuite par la CI la récupère.

## Chiffres (Marée, mai 2026)

- 14 200 assets sources, 9,8 Go de sources, 3,1 Go de paquets bureau, 2,2 Go cible basse.
- Rebuild complet sur la CI : 41 minutes (textures 24, shaders 9, meshes 5, reste 3).
- Journée typique incrémentale : 6 minutes sur la CI, 40 secondes sur un poste pour un seul asset modifié.

## Commandes

- `brumepipe build --platform desktop` (ou `lowend`), `--chunk port_nuit` pour un seul chunk.
- `brumepipe why <asset>` explique pourquoi un asset serait rebâti (hash changé, version d'importeur, dépendance).
- `brumepipe verify` recalcule les hashs des paquets et compare à la table, utilisé avant une livraison.

Les échecs d'import et leur traitement sont dans [[asset-import-failures-2026]]. La CI qui lance tout ça est décrite dans [[ci-pipeline-layout]].

## Le cache partagé, en détail

Le cache est un répertoire sur le stockage réseau du studio (`\\brume-nas\pipecache`, monté en `/mnt/pipecache` sur les postes), organisé par hash d'entrée : `cache/<2 premiers caractères>/<hash>/artefact`. Un poste qui a besoin d'un artefact regarde d'abord son cache local (`~/.brumepipe/cache`, 50 Go), puis le partagé, puis calcule et écrit dans les deux. L'écriture dans le partagé est atomique (fichier temporaire puis renommage), sinon deux postes qui cuisent la même texture au même moment se corrompent mutuellement, ce qui est arrivé une fois en janvier 2026.

Quota du cache partagé : 400 Go, nettoyage LRU par un job du dimanche (voir [[ci-cache-misses-lesson]] pour la raison du quota séparé). Taux de hits sur les postes des artistes en mai 2026 : 88 % ; sur la CI : 96 %.

## Versions d'importeur et d'étape

Chaque importeur et chaque étape de cuisson porte un numéro de version dans son code (`const VERSION: u32`). Incrémenter la version invalide tous les artefacts de cette étape, ce qui déclenche un rebuild partiel de la taille correspondante : la texture est la plus chère (24 minutes sur la CI). La règle : on incrémente la version quand la sortie change pour une même entrée, pas quand le code change sans effet sur la sortie ; un test compare la sortie de l'importeur sur 30 assets de référence avant et après pour le vérifier, et refuse un changement de sortie sans incrément de version.

Historique des incréments en 2026 : textures 3 fois (passage en BC7 haute qualité, suppression de BC1, correction du gamma sur les `_msk`), meshes 1 fois (nouveau calcul des tangentes), shaders 6 fois (le plus fréquent, chaque changement de l'ABI des shaders), audio 0.

## Dépendances entre assets

Un matériau référence des textures, un mesh référence des matériaux, un niveau référence des meshes. `brumepipe` construit le graphe à l'import (chaque importeur déclare les chemins qu'il lit) et le stocke dans `assets/cache/deps.db`. Une texture modifiée rebâtit le matériau, puis le paquet du chunk, mais pas le mesh (qui référence le matériau par identifiant, pas par contenu). `brumepipe why prop_buoy_red.mat` affiche la chaîne.

Le graphe sert aussi à `brumepipe unused`, qui liste les sources qu'aucun niveau ne référence : 1 240 assets en avril 2026, 900 Mo de sources, dont 300 supprimés après revue par les artistes. Le reste est du travail en cours ou des variantes gardées volontairement.

## Ce que le pipeline garantit au moteur

- Un paquet est complet ou absent, jamais partiel (écriture atomique).
- Les identifiants d'assets dans un paquet sont stables entre builds tant que le nom ne change pas ; le moteur peut les mettre en dur dans une sauvegarde.
- La table des matières d'un paquet a un hash du contenu, vérifié au chargement en dev, pas en release (0,3 s par chunk sur la cible basse).
- Un paquet cible basse ne contient rien au-dessus des tailles maximales de la plateforme ; c'est le pipeline qui coupe, le moteur ne vérifie pas.

Les erreurs et leurs messages sont dans [[asset-import-failures-2026]] ; les règles de nom que le pipeline applique sont chez les conventions communes.
