---
name: sync-conflict-resolution-rules
description: Server wins on load status, device wins on stop-level facts (arrival time, POD, notes), a 409 with conflict_kind decides which, and the driver is only asked in the reassigned case
type: reference
status: active
verified: 2026-03-02
---

# Règles de résolution de conflits de synchronisation

Un conflit, c'est une mutation du téléphone que le serveur refuse avec un 409 parce que l'état a changé entre temps. Le corps du 409 contient `conflict_kind` et l'état serveur courant. `ConflictResolver` (dans `lib/sync/conflict_resolver.dart`) applique une règle par `conflict_kind`, et il n'y en a que cinq.

## Les cinq cas

**`load_reassigned`** : le chargement a été retiré au chauffeur (par le dispatcher ou par un retrait transporteur) pendant qu'il était hors ligne. C'est le seul cas où on demande au chauffeur. L'app affiche "Ce chargement ne vous est plus attribué. Vos actions n'ont pas été envoyées" avec le détail des mutations en attente, et un bouton pour les exporter en texte (le chauffeur les envoie au dispatcher par message). Les mutations sont supprimées de l'outbox. On a essayé de les garder "au cas où" et ça bloquait la file pour toujours.

**`status_regression`** : le téléphone envoie `pickup` mais le serveur est déjà en `IN_TRANSIT` (typiquement : le dispatcher a fait la transition à la main). Le serveur gagne, la mutation est jetée en silence, la prochaine pull met le replica à jour. Le chauffeur ne voit rien, parce que le résultat est celui qu'il voulait.

**`stop_fact_diverged`** : le serveur a une heure d'arrivée ou un commentaire différent sur un arrêt. Le téléphone gagne. Le serveur accepte la mutation avec `force: true` que l'app renvoie immédiatement. La raison : le chauffeur était sur place, le dispatcher non. C'est la règle qui a le plus été discutée avec le produit et elle tient depuis 2025.

**`document_already_attached`** : un POD existe déjà pour cet arrêt (autre appareil, ou renvoi). Les deux sont gardés, le serveur répond 200 avec `duplicate: true`, pas un vrai conflit depuis [[incident-2026-01-duplicate-pod-uploads]]. Listé ici parce que l'app le traitait comme tel avant 4.7.

**`load_cancelled`** : comme `load_reassigned` mais sans export, le chargement n'existe plus pour personne. Message court, mutations supprimées.

## Ce qui n'est pas un conflit

Un 409 `load_locked` (verrou `NOWAIT` côté API) n'est pas un conflit, c'est une collision temporaire : on réessaie 3 fois avec 200 ms de jitter avant de le traiter comme une erreur réseau.

## Ordre

Le résolveur ne traite qu'une mutation à la fois, celle en tête de file. Après résolution, il relance le push. Si la résolution supprime des mutations (cas reassigned et cancelled), elle supprime toutes celles du même chargement, pas seulement celle en tête, sinon la suivante produit le même 409 et on tourne en rond. C'est un bug qu'on a eu en 4.5.

## Tests

`test/sync/conflict_resolver_test.dart` a un cas par `conflict_kind` avec un faux serveur en mémoire. Le test qui compte : `reassigned drops all mutations of the load, keeps others`. Voir aussi [[offline-sync-architecture]].
