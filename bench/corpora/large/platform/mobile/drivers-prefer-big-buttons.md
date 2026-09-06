---
name: drivers-prefer-big-buttons
description: Driver feedback sessions settled on 56 px minimum tap targets, one primary action per screen, no swipe gestures for anything important, and high contrast for use in sunlight with gloves
type: user
status: active
verified: 2026-01-19
---

# Ce que les chauffeurs demandent de l'interface

Synthèse de trois sessions d'observation en dépôt (2025, deux transporteurs, 11 chauffeurs) et des retours du support. Ces préférences priment sur les recommandations génériques de design.

- **Cibles tactiles de 56 px minimum**, 64 pour l'action principale. Les chauffeurs portent souvent des gants, ont le téléphone sur un support de tableau de bord, et appuient sans regarder. Les 48 px du guide de design Material sont trop petits pour nous.

- **Une seule action principale par écran**, en bas, pleine largeur, couleur pleine. "Arrivé", puis "Photo du bon", puis "Livré". Deux boutons de même poids côte à côte = le mauvais est pressé une fois sur cinq.

- **Pas de geste de balayage pour une action importante.** Le balayage pour marquer livré a été retiré en 4.2 après des livraisons validées par accident dans la poche. Le balayage sert uniquement à revenir en arrière et à parcourir des photos.

- **Confirmation seulement quand c'est irréversible**, et jamais par une boîte modale avec deux boutons de même taille. "Livré" demande une confirmation par appui long de 800 ms sur le bouton lui-même, ce qui a été préféré à une modale par 9 chauffeurs sur 11.

- **Contraste fort, texte grand.** Lecture en plein soleil, sur un écran à 40 % de luminosité pour économiser la batterie. Fond blanc, texte noir, taille minimale 16 sp, 20 sp pour les informations d'arrêt. Le mode sombre est disponible mais peu utilisé.

- **L'adresse et le créneau avant tout le reste.** Sur la carte d'un chargement, l'adresse de l'arrêt courant et l'heure limite sont en haut, en gros. Le numéro de chargement et le nom du chargeur sont en petit en bas : les chauffeurs ne s'en servent qu'au téléphone avec le dispatcher.

- **Le mode paysage ne sert à rien**, l'app est verrouillée en portrait depuis 4.0 et personne ne s'en est plaint.

Ce que les chauffeurs ne demandent pas et qu'on ne fait pas : de la personnalisation, des thèmes, des animations. L'app est un outil, elle doit être la même sur tous les téléphones du dépôt pour que les collègues puissent s'entraider.

Voir [[pod-photo-compression]] pour l'écran de prise de photo, qui suit les mêmes règles (déclencheur plein largeur, prévisualisation immédiate, reprise en un appui).
