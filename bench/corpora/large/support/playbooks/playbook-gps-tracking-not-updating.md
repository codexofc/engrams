---
name: playbook-gps-tracking-not-updating
description: Position GPS figée : fréquence normale (2 min en transit, 15 min à l'arrêt), permission arrière-plan, batterie Samsung, positions aberrantes
type: reference
status: active
verified: 2026-05-28
---

# La position du camion ne bouge pas

Catégorie `driver:sync` (sous-tag `gps`). Le chargeur regarde la carte de suivi et le point ne bouge plus. Avant de chercher un bug, vérifier que le suivi est censé être actif.

## Quand le suivi est actif

L'app n'envoie des positions que pour un chargement `IN_TRANSIT` assigné au chauffeur connecté. Avant le pickup, rien. Après la livraison, rien. Fréquence : une position toutes les 2 minutes en mouvement, toutes les 15 minutes à l'arrêt (détection par l'accéléromètre), pour tenir le budget batterie. La carte affiche « il y a X min » sous le point, et c'est normal qu'il y ait 2 minutes.

1. `hfctl load get <load_id>` : `status`. Pas `IN_TRANSIT` : pas de suivi, et le ticket est en fait un [[playbook-load-stuck-dispatched-no-pickup]]. Macro `gps-not-in-transit`.

2. `hfctl load positions <load_id> --last 20` : les dernières positions reçues avec `recorded_at` (heure du téléphone) et `received_at` (heure serveur). Un écart entre les deux qui grandit : le téléphone enregistre mais n'envoie pas (pas de réseau), les positions arriveront en rafale. Macro `gps-buffered`.

3. Aucune position depuis plus de 30 minutes en journée :

- `hfctl driver sync-status <driver_id>` : `last_seen_at` ancien aussi. Téléphone éteint, mode avion, ou app tuée. Voir [[playbook-driver-app-not-syncing]].

- `last_seen_at` récent mais pas de positions : l'app tourne mais n'a pas la permission de localisation en arrière-plan. Sur Android 12 et plus, la permission « Toujours autoriser » doit être donnée explicitement, et le système la retire après quelques mois sans usage. L'app affiche un bandeau orange quand elle manque. Macro `gps-permission-background` avec les captures par version d'Android.

- Sur un Samsung, économie de batterie « optimisée » qui tue le service en arrière-plan : macro `gps-samsung-battery`. Sur un Huawei sans services Google, le service de localisation alternatif est moins précis mais fonctionne, l'écart de 100 à 300 m est connu.

4. Positions présentes mais aberrantes (le camion « saute » de 50 km) : précision GPS en zone couverte (tunnel, hangar) ou position réseau. On filtre les points avec `accuracy > 500 m` depuis 4.8, mais les anciennes versions les envoient. Vérifier `app_version`.

## Le chauffeur refuse le suivi

Ça arrive. Le suivi est une condition du service pour le transporteur, pas pour le chauffeur individuellement, et il ne peut pas le désactiver dans l'app pendant un transit. S'il a désactivé la localisation au niveau du téléphone, c'est entre lui et son employeur. Le support ne rentre pas dans cette discussion, macro `gps-carrier-policy` au transporteur.

## Ce qu'on ne fait pas

On n'invente pas de position, on n'appelle pas le chauffeur. On ne partage pas l'historique de positions d'un chargement avec quelqu'un d'autre que le chargeur et le transporteur de ce chargement.

## Escalade

L2 si les positions arrivent (`received_at` récent) mais que la carte du chargeur ne les montre pas : c'est le front ou le websocket, pas l'app. Backend si plus de 10 chargements en transit sont sans position depuis 30 minutes : c'est l'ingestion.
