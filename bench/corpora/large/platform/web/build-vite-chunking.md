---
name: build-vite-chunking
description: Vite build with manual chunks (vendor-react, vendor-map, vendor-charts, app), map and charts lazy-loaded by route, initial JS 310 KB gzipped down from 890 KB after HF-1290
type: project
status: active
verified: 2026-01-13
---

# Découpage du bundle (HF-1290)

## Avant

Un seul `index.js` de 890 Ko gzippé. La carte (MapLibre, 220 Ko gzippé), les graphiques (150 Ko) et l'éditeur de texte riche des messages (90 Ko) étaient chargés sur la page de connexion. Sur la connexion de bureau des dispatchers c'est invisible, mais 30 % des sessions viennent de tablettes en dépôt sur du Wi-Fi partagé, et là le premier affichage prenait 6 s.

## Configuration (`vite.config.ts`)

```ts
build: {
  rollupOptions: {
    output: {
      manualChunks(id) {
        if (id.includes('node_modules/react') || id.includes('node_modules/react-dom')) return 'vendor-react';
        if (id.includes('maplibre-gl')) return 'vendor-map';
        if (id.includes('node_modules/recharts') || id.includes('node_modules/d3-')) return 'vendor-charts';
        if (id.includes('node_modules')) return 'vendor';
      },
    },
  },
  chunkSizeWarningLimit: 400,
}
```

Et surtout, les routes lourdes en `React.lazy` : `/map`, `/reports/*`, `/messages` (pour l'éditeur). Le préchargement se fait au survol du lien de navigation (`onMouseEnter` → `import()`), ce qui masque le délai dans la plupart des cas.

## Résultats

| | Avant | Après |
|---|---|---|
| JS initial (gzip) | 890 Ko | 310 Ko |
| `vendor-react` | | 48 Ko |
| `vendor-map` (différé) | | 220 Ko |
| `vendor-charts` (différé) | | 150 Ko |
| Premier affichage utile, tablette dépôt | 6,1 s | 2,3 s |

Mesuré avec le profil réseau "3G rapide" des outils de développement et confirmé par les métriques réelles du mois suivant (voir [[performance-budget-web]]).

## Pièges rencontrés

- `manualChunks` avec `react` attrapait aussi `react-hook-form` et `@tanstack/react-query`. Le test `id.includes('node_modules/react')` a été restreint à `react/` et `react-dom/` avec le slash.

- Deux chunks qui importent le même module de `d3-` le dupliquaient avant la règle `vendor-charts`. Vérifier avec `npx vite-bundle-visualizer` après tout changement de règle.

- Le cache long (`Cache-Control: max-age=31536000, immutable`) sur les fichiers hachés est posé par l'ingress, pas par Vite. `index.html` est en `no-cache`. Un déploiement qui change `vendor-react` invalide les chunks qui l'importent, c'est le comportement attendu du hachage de Rollup.

- Après un déploiement, un onglet ouvert depuis la veille demande un chunk qui n'existe plus (404). `RouteErrorBoundary` détecte `ChunkLoadError` et recharge la page une fois, avec un drapeau en `sessionStorage` pour ne pas boucler. Voir [[error-boundary-and-toasts]].

## Ce qu'on n'a pas fait

Pas de rendu côté serveur. L'app est derrière une authentification, il n'y a pas de page publique à indexer, et le premier affichage est dominé par l'appel API du tableau de bord, pas par le JS.
