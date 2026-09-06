---
name: performance-budget-web
description: Front performance budget is 350 KB initial JS gzipped, LCP under 2.5 s at p75 on the board, INP under 200 ms, measured by real user monitoring, with the CI size check and the profiler habit that go with it
type: feedback
status: active
verified: 2026-05-15
---

# Budget de performance du front

## Les chiffres à tenir

| Métrique | Budget | Mesuré (avril 2026, p75) |
|---|---|---|
| JS initial (gzip) | 350 Ko | 318 Ko |
| LCP sur le tableau | 2,5 s | 1,9 s |
| INP | 200 ms | 140 ms |
| Temps de réponse au drag (premier mouvement) | 50 ms | 30 ms |

Mesuré par le RUM (`web-vitals` envoyé vers le collecteur, échantillon 20 % des sessions, ventilé par route et par type d'appareil). Pas par Lighthouse sur un poste de développeur, qui donne des chiffres flatteurs.

## Comment on le tient

- **Taille** : le CI compare le poids des chunks avec `main` et échoue au-delà de +15 Ko gzip sur le JS initial sans label `size-ok`. Voir [[build-vite-chunking]] pour le découpage. Le budget de 350 Ko a été fixé après être passé de 890 à 310 : la marge est là pour ne pas revenir en arrière par petites touches.

- **LCP** : sur le tableau, l'élément LCP est la première carte de chargement. Il dépend de l'appel API de la liste, pas du JS. Le gain est venu de la pagination par curseur côté API et de la virtualisation ([[table-virtualization-loads-list]]). Le squelette de chargement est affiché immédiatement pour que le LCP ne soit pas le spinner.

- **INP** : les deux coupables trouvés au profiler étaient la re-rendu du tableau entier pendant le drag (corrigé par les sélecteurs étroits de Zustand, voir [[dispatch-board-state-zustand]]) et le formatage des dates de 2 000 lignes à chaque changement de filtre (corrigé par `useMemo` par ligne et par la virtualisation).

## Habitudes qui marchent

- Ouvrir le profiler React sur toute PR qui touche le tableau ou le formulaire de chargement. Chercher les composants qui se rendent plus de 5 fois pour une interaction.

- Tester sur le profil "tablette dépôt" : CPU ralenti 4x et réseau 3G rapide dans les outils de développement. C'est là que vivent 30 % des sessions.

- Ne pas ajouter de bibliothèque pour ce qu'`Intl`, `fetch`, `AbortController` et `structuredClone` font. `dayjs` et `lodash` sont partis en 2025, personne ne les regrette.

- Une image, c'est du WebP ou du SVG, jamais de PNG au-dessus de 50 Ko. Il n'y a de toute façon presque pas d'images.

## Ce qu'on ne mesure pas

Le temps de build (2 min, personne ne s'en plaint) et la taille totale de tous les chunks (les chunks différés se chargent à l'usage). Le budget porte sur ce que l'utilisateur attend.

## Quand le budget est dépassé

On ne le dépasse pas "temporairement". Soit la PR trouve où reprendre les kilo-octets, soit elle attend. La dernière fois (février 2026, éditeur de texte riche pour les messages), la solution a été de le charger à la demande, ce qui était de toute façon mieux.
