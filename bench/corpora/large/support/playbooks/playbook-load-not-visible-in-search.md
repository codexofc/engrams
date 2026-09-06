---
name: playbook-load-not-visible-in-search
description: Chargement absent de la recherche transporteur : état, délai d'indexation haystack, visibilité, filtres, géocodage, reindex manuel
type: reference
status: active
verified: 2026-07-22
---

# Le chargement n'apparaît pas dans la recherche

Catégorie `load:search`. Deux voix : le chargeur (« mon chargement n'est pas visible ») ou le transporteur (« je ne trouve pas le chargement que le chargeur m'a dit d'aller voir »). La recherche transporteur est servie par le service `haystack`, alimenté à partir des événements de chargement, et il y a un délai.

## Vérifications

1. **L'état.** `hfctl load get <load_id>`. Seuls `OPEN` et `BIDDING` sont cherchables. Un `DRAFT` n'est pas publié (le chargeur le voit dans sa liste, pas les transporteurs). Un `DISPATCHED` a disparu de la recherche, c'est normal. Macro `search-load-state`.

2. **Le délai.** `hfctl load get` montre `indexed_at` et `index_version`. `indexed_at` vide ou plus vieux que `updated_at` de plus de 60 s : retard d'indexation. Regarder le tableau Grafana « Haystack indexer lag ». Sous 30 s, on attend et on répond avec la macro `search-index-delay`. Au-dessus de 5 minutes, c'est un incident côté plateforme, vérifier `#incidents` et escalader si rien n'est déclaré.

3. **La visibilité.** `visibility` sur le chargement : `PUBLIC`, `PRIVATE` (réservé aux transporteurs favoris du chargeur), `PARTNER_ONLY` (publié uniquement vers Cargolink ou Fretzone, pas dans notre recherche). Un transporteur qui n'est pas dans les favoris ne verra jamais un `PRIVATE`. Macro `search-visibility` au chargeur, c'est lui qui choisit.

4. **Les filtres du transporteur.** Dans le back-office, impersonner le transporteur en lecture et regarder ses filtres enregistrés : rayon autour du point de départ (par défaut 150 km), types de véhicule, dates. Un chargement à 180 km avec un rayon de 150 km n'apparaît pas et c'est juste. Le rayon se calcule depuis l'adresse de départ du chargement jusqu'au point que le transporteur a choisi, pas depuis sa ville de siège. Macro `search-filters` avec la capture.

5. **La géolocalisation du chargement.** `hfctl load get` champ `pickup_geo`. Vide : l'adresse n'a pas été géocodée (adresse incomplète, code postal inconnu). Le chargement est indexé mais n'apparaît dans aucune recherche par rayon, seulement en recherche par ville exacte. Le chargeur corrige l'adresse, le regéocodage est automatique. Macro `search-address-geocode`.

6. **Réindexation.** Si tout est cohérent et que le chargement manque quand même : `hfctl load reindex <load_id> --apply` (L1 peut le faire). Si après 60 s il n'apparaît toujours pas, escalade L2 avec `load_id` et `index_version`, il y a un document rejeté par l'indexeur et le backend veut le voir.

## Le cas inverse

Un transporteur voit un chargement qui n'existe plus (déjà pris, annulé). Délai de désindexation, même ordre de grandeur. S'il persiste plus de 5 minutes, même chemin que l'étape 6. Le clic mène à `load_not_open`, voir [[playbook-bid-cannot-be-placed]].

## Ce qu'on ne fait pas

On ne modifie pas les filtres du transporteur. On ne change pas la visibilité d'un chargement. On ne réindexe pas toute une organisation, c'est une commande backend.

## Escalade

L2 après une réindexation sans effet. Backend si le retard d'indexation dépasse 5 minutes sans incident déclaré (critère de la page dans le chemin d'escalade).
