---
name: position-source-priority
description: With several sources per vehicle, SourceSelector shows the freshest if within 60 s, else ranked app > Geolyx > Trakko > shadow; 2 km disagreement flags mapping
type: project
status: active
verified: 2026-03-13
---

# Priorité entre sources de position (HF-2122)

Un véhicule peut avoir un boîtier Trakko, être dans une flotte Geolyx, **et** avoir un chauffeur avec l'application. Les trois flux sont conservés dans `position_events` (la déduplication ne fusionne pas entre fournisseurs, voir [[position-dedup-rules]]). Mais la carte, l'ETA et le géorepérage ont besoin d'**une** position courante par mission. `SourceSelector` la choisit.

## Combien de véhicules sont concernés

Mars 2026 : 2 900 véhicules suivis, 310 avec deux sources actives en même temps, 12 avec trois. Le cas le plus fréquent : un transporteur équipé de Trakko dont les chauffeurs ont adopté l'application parce qu'ils voulaient signer les POD.

## La règle

Pour une mission donnée, à chaque nouvelle position conservée d'une source quelconque :

1. Prendre la dernière position de chaque source active pour ce véhicule (cache `lastpos:{vehicle}:{provider}`).

2. Si la plus fraîche a moins de **60 s** d'écart avec les autres, **la plus fraîche gagne**. En régime normal, c'est presque toujours l'application ou Geolyx (8 s de latence) plutôt que Trakko (40 s).

3. Sinon (une source est en retard de plus de 60 s, ou muette), la source la mieux classée parmi celles qui ont une position de moins de 15 minutes gagne. Classement : **application > Geolyx > Trakko > fournisseurs en shadow run**.

4. Aucune source depuis 15 minutes : `stale`, dernière position connue toutes sources confondues, avec son âge.

La position choisie est publiée sur le topic `positions_selected` (une par véhicule et par changement de source ou de position) ; c'est ce topic que consomment l'ETA ([[eta-feed-publication]]), le géorepérage ([[geofence-arrival-detection]]) et la carte. Le topic `positions` brut reste consommé par la plateforme data.

## Pourquoi ce classement

- **Application d'abord** : c'est la source qui sait dans quelle mission on est (le chauffeur a appuyé sur « démarrer » sur ce véhicule), donc elle a la meilleure chance d'être le bon camion. Un boîtier est mappé à un véhicule, l'application est mappée à un chauffeur qui a déclaré un véhicule ; en cas de désaccord, c'est le chauffeur qui a raison neuf fois sur dix (le boîtier a été déplacé sans qu'on le sache, voir [[tracker-vehicle-mapping]]).

- **Geolyx avant Trakko** : latence plus faible, heure serveur fiable, précision fournie (HDOP) plutôt qu'estimée.

- **Shadow en dernier** : un fournisseur en évaluation ([[provider-onboarding-runbook]]) ne doit jamais être affiché.

## Désaccord entre sources

Quand deux sources fraîches (moins de 60 s toutes deux) sont à plus de **2 km** l'une de l'autre, ce n'est pas un problème de priorité, c'est un problème de mapping : l'une des deux n'est pas ce camion. `SourceSelector` émet `telematics.source_conflict` avec les deux positions, le véhicule et la mission ; le rapport quotidien des conflits va au support, qui regarde le mapping. 14 conflits en mars 2026, 11 étaient des boîtiers déplacés, 2 des chauffeurs qui avaient choisi le mauvais camion au démarrage, 1 jamais expliqué.

Pendant le conflit, la règle de classement s'applique (l'application gagne), et l'ETA est marquée `confidence = 'low'`.

## Ce que voit l'utilisateur

- Le dispatcher voit un seul marqueur par véhicule, avec la source dans le tooltip (« via application », « via Geolyx »). Personne ne l'a demandé, mais ça a réduit les tickets « pourquoi mon boîtier ne sert à rien » : la réponse est visible.

- Le chargeur ne voit rien de tout ça, un marqueur et son âge.

## Ce qu'on a écarté

- **Fusionner** les sources (moyenne pondérée, filtre de Kalman). Deux GPS à 15 m l'un de l'autre n'ont pas besoin d'être fusionnés, et deux GPS à 2 km sont un problème de mapping, pas de fusion. Le gain de précision théorique ne valait pas la complexité.

- **Laisser le transporteur choisir** sa source préférée par véhicule. Personne ne le ferait, et la règle de fraîcheur fait mieux.

- **Éteindre le boîtier quand l'application est active.** Le boîtier ne s'éteint pas à distance, et la redondance a sauvé des missions quand un téléphone est tombé en panne.
