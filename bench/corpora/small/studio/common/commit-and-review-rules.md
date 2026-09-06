---
name: commit-and-review-rules
description: Commits as type(scope): subject with BR-nnn, one reviewer normally, two for unsafe, allocators, GPU and formats, 3-day MR limit
type: reference
status: active
verified: 2026-02-05
---

## Messages de commit

`<type>(<portée>): <sujet>` sur la première ligne, le sujet à l'infinitif, sans point final, 72 caractères maximum. Types : `feat`, `fix`, `perf`, `refactor`, `tools`, `assets`, `ci`, `docs`. Portée : la crate ou l'outil (`render`, `ecs`, `brumepipe`, `editor`).

Le corps explique le pourquoi, pas le quoi (le diff dit le quoi), et cite le ticket `BR-nnn`. Le nom de branche porte la clé du ticket : `feat/BR-388-undo-global-stack`.

Exemple :

```
fix(editor): garder une seule pile d'annulation entre chunks

Deux piles par chunk laissaient un objet dupliqué quand on annulait
un déplacement de A vers B. BR-388.
```

## Revue

- Une personne pour la plupart des changements. Deux pour tout ce qui touche `unsafe`, les allocateurs, le backend GPU, ou un format de fichier lu par le moteur (paquets, tables de chaînes, caches de shaders), parce qu'un format cassé invalide tous les assets.
- Une revue est demandée avec la lane code verte, jamais avant.
- Le relecteur lit le ticket d'abord, puis le diff. Un commentaire commence par ce qu'il demande (« bloquant », « suggestion », « question »).
- Une merge request de plus de 3 jours est fermée ou découpée. Les longues branches ont coûté deux semaines de conflits sur `givre-render` en 2025.

## Ce qu'on ne fait pas

- Pas de squash automatique ; l'auteur nettoie son historique avant de demander la revue, et la fusion garde les commits.
- Pas de revue par message instantané. Si la discussion sort de la merge request, elle y est résumée après.

Les conventions de code sont dans [[coding-conventions-rust]] ; la sévérité des bugs, qui décide de ce qui passe avant quoi, dans [[bug-severity-scale]].
