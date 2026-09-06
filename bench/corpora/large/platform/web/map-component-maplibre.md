---
name: map-component-maplibre
description: The dispatch map uses MapLibre GL with self-hosted vector tiles (tiles.halden.example, OpenMapTiles schema), truck markers as a GeoJSON source updated in place, clustering above 200 trucks, chosen over a commercial map SDK in HF-1140
type: project
status: active
verified: 2026-03-09
---

# Map component (HF-1140)

## Choice

MapLibre GL JS with our own vector tile server (`tiles.halden.example`, serving an OpenMapTiles-schema planet extract for Europe, refreshed quarterly by the ops team). Decision against the commercial SDK we used in 2024: cost was growing with map loads (the board reloads the map on every tab open), and the driver app needed the same tiles for the no-GMS devices. One tile server, two clients.

Tiles are cached at the ingress with a 30 day TTL. Tile requests per day: about 900 000, 97 % served from cache.

## Truck markers

Not DOM markers. A GeoJSON source `trucks` with one feature per truck, rendered by a `symbol` layer with an icon per status and a text label with the truck's plate below zoom 10. Positions arrive from the live socket (see [[websocket-live-updates]]) and the source is updated with `source.setData()` at most every 2 s, batched. 700 DOM markers made the map stutter at 20 fps; 700 features in a symbol layer render at 60.

Clustering (`cluster: true, clusterRadius: 40`) is enabled above 200 trucks in view. Under that, individual icons. The threshold comes from a test with a dispatcher managing 350 trucks who could not read anything at the country zoom without clusters.

## Interaction

- Click on a truck opens the load panel (same panel as the board, same route param).

- The followed truck (`useMapStore.followedId`) keeps the map centred on it with `easeTo` and no animation longer than 150 ms, see [[dispatchers-want-dense-ui]].

- Pickup and delivery points of the selected load are drawn as a `line` layer for the straight line and a `circle` layer for the stops. The road route from the routing service is drawn only when the panel's "Itinéraire" toggle is on, because it is a second request per load.

## Accessibility

The map itself is not keyboard operable beyond zoom. A "Liste des camions" panel lists the same trucks with the same statuses and the same actions, and is the documented alternative. See [[a11y-keyboard-drag-drop-dispatch]].

## Bundle

`maplibre-gl` is 220 KB gzipped and lives in its own chunk, loaded when the `/map` route or the board's mini-map is first shown. See [[build-vite-chunking]].

## Known issues

- WebGL context loss when a laptop's GPU sleeps: the map goes black. Handled by listening to `webglcontextlost` and recreating the map instance, with the viewport restored from `useMapStore`. Took a while to reproduce; closing the lid for 10 minutes on a Windows laptop with an integrated GPU does it reliably.

- Labels in the local language of each country (Polish names in Poland) confuse French dispatchers. `name:latin` is used, with `name:fr` as a fallback only for country and large city names. Product accepted the compromise.

- Tile style is `hf-light` and `hf-dark`, generated from the same source as the design tokens so the map background matches the theme.
