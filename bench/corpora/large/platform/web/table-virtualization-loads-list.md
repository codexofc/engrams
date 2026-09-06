---
name: table-virtualization-loads-list
description: The loads table renders only visible rows with TanStack Virtual (row height fixed at 44 px, overscan 8), which took a 2 000 row list from 1.8 s to 60 ms to render, and requires stable row keys and no auto-height rows
type: project
status: active
verified: 2025-12-11
---

# Virtualisation de la table des chargements (HF-1265)

## Constat

Un chargeur avec 2 000 chargements ouverts (un distributeur, pic de novembre). La table `LoadsTable` rendait 2 000 lignes de 9 cellules : 1,8 s de rendu au premier affichage, 400 ms à chaque changement de filtre, et le défilement à 15 images par seconde sur un portable de bureau standard. Le profiler React montrait 18 000 composants montés.

## Ce qui a été fait

`@tanstack/react-virtual` avec `useVirtualizer` sur le conteneur de la table :

- Hauteur de ligne fixe : 44 px en mode normal, 32 px en mode dense (voir [[dispatchers-want-dense-ui]]). Pas de `measureElement`, la mesure dynamique coûte un reflow par ligne et on n'a pas de lignes à hauteur variable : le contenu qui dépasse est tronqué avec une infobulle.

- `overscan: 8`. En dessous, des lignes blanches apparaissaient au défilement rapide à la molette.

- Les en-têtes sont hors du conteneur virtualisé, en `position: sticky`.

- Les lignes sont `position: absolute` avec `transform: translateY()`, ce qui est le mode par défaut de la bibliothèque, et le conteneur a une hauteur totale calculée.

Résultat : 60 ms de rendu initial quel que soit le nombre de lignes, 40 lignes montées, défilement à 60 images par seconde.

## Contraintes que ça impose

- **Clés de ligne stables** : la clé est `load.id`, jamais l'index. Avec l'index, une insertion en tête faisait glisser l'état local de toutes les lignes (case cochée, ligne dépliée).

- **Pas de ligne dépliable en place.** Le détail d'un chargement s'ouvre dans un panneau latéral, pas sous la ligne. C'était déjà le cas, la virtualisation l'a rendu obligatoire.

- **Sélection multiple par `Shift+clic`** : l'ancre et la cible peuvent ne pas être montées. La sélection se calcule sur le tableau de données (`rows` triées), pas sur le DOM.

- **Recherche navigateur (`Ctrl+F`)** ne trouve pas les lignes non montées. Les dispatchers s'en servaient. Compensé par le champ de filtre rapide qui a le focus avec `/`, et documenté dans l'aide.

- **Accessibilité** : `role="grid"` avec `aria-rowcount` sur le conteneur et `aria-rowindex` sur chaque ligne, pour que le lecteur d'écran annonce "ligne 412 sur 2 000". La navigation clavier entre lignes fait défiler la ligne cible dans la vue avec `scrollToIndex`. Voir [[a11y-keyboard-drag-drop-dispatch]].

- **Restauration du focus** après un changement de données : voir la même note, le `requestAnimationFrame` avec 5 tentatives vient de là.

## Le compteur

La table affiche "2 000+" et pas le total exact, parce que l'API ne renvoie pas de `total` avec la pagination par curseur. La table charge les pages suivantes quand le défilement atteint 80 % de la hauteur (`fetchNextPage` de TanStack Query). Pour un chargeur avec plus de 2 000 chargements ouverts, le tri côté client est désactivé et remplacé par le tri serveur, sinon on trierait une liste incomplète.

## Colonnes

Largeurs fixes par colonne en `px`, redimensionnables à la souris, sauvegardées dans `useBoardStore` (voir [[dispatch-board-state-zustand]]). Pas de largeur automatique au contenu, incompatible avec la virtualisation.
