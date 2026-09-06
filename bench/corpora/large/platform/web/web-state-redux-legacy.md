---
name: web-state-redux-legacy
description: Historical single Redux store of the dispatch front with server data in slices, removed in HF-1150 in favour of TanStack Query plus Zustand
type: reference
status: archived
superseded_by: [[dispatch-board-state-zustand]]
verified: 2025-09-30
---

# Ancien store Redux (jusqu'en octobre 2025)

Le front dispatch avait un store Redux Toolkit unique avec des slices `loads`, `carriers`, `bids`, `ui`. Les données serveur étaient normalisées avec `createEntityAdapter` et chargées par des thunks.

Ce qui a motivé le remplacement (HF-1150) :

- Chaque nouvel endpoint demandait un slice, un thunk, trois actions et un sélecteur. La moitié des bugs de l'année 2025 sur le front étaient des données périmées dans le store parce qu'une mutation oubliait d'invalider.

- Pas de notion de fraîcheur : le store gardait les chargements de la session entière, et un dispatcher qui laissait l'onglet ouvert la nuit voyait la veille au matin.

- Les mises à jour temps réel écrivaient dans le store en parallèle des thunks, avec des courses entre un `GET` lent et un événement WebSocket plus récent.

- 1 100 lignes de boilerplate supprimées lors de la migration, pour 340 ajoutées.

La migration a duré deux sprints, écran par écran, avec les deux systèmes en parallèle et un pont `useLegacyLoads()` retiré en dernier. Voir [[dispatch-board-state-zustand]] pour l'organisation actuelle.
