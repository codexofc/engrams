---
name: error-boundary-and-toasts
description: Three layers of error handling on the front, a route-level error boundary with reload on ChunkLoadError, a toast queue for API errors with the trace id and an undo action, and a pre-React boot guard in index.html
type: reference
status: active
verified: 2026-03-31
---

# Gestion des erreurs côté front

## Couche 0 : garde de démarrage (avant React)

Un script inline dans `index.html`, avant les modules, écoute `window.onerror` et `unhandledrejection` pendant le démarrage. Si `window.__hfBooted()` n'a pas été appelé dans les 8 s, il remplace le corps de la page par un message statique avec un code (`BOOT-TIMEOUT` ou `BOOT-ERROR`) et le texte de l'erreur. Ajouté après [[incident-2026-03-white-screen-safari]]. Aucune dépendance, 40 lignes, testé en injectant une erreur de syntaxe dans un chunk en E2E.

## Couche 1 : error boundaries

- `RouteErrorBoundary` sur chaque route (via `errorElement` du routeur). Affiche "Cette page a rencontré une erreur" avec un bouton "Recharger" et le `trace_id` s'il y en a un. L'erreur part au monitoring avec la route et l'état du store.

- Cas spécial `ChunkLoadError` (un chunk hashé qui n'existe plus après un déploiement) : rechargement automatique de la page une seule fois, avec `sessionStorage.hf_chunk_reload = '1'` pour ne pas boucler. Voir [[build-vite-chunking]].

- `PanelErrorBoundary` autour du panneau latéral et de chaque widget de la page d'accueil, pour qu'un widget cassé ne fasse pas tomber la page.

- Pas d'error boundary global qui avale tout : une erreur non attrapée par une route remonte à la couche 0, ce qui est voulu.

## Couche 2 : erreurs API et toasts

`ApiError` (voir [[api-client-generation-openapi]]) est traitée à trois endroits selon le contexte :

1. Dans un formulaire : les 422 vont sur les champs, voir [[forms-react-hook-form-zod]]. Pas de toast.

2. Dans une mutation hors formulaire (affectation, changement de statut) : `onError` du hook affiche un toast d'erreur avec le message traduit (`errors:<type>`) et "Code support : 0a1f3c9e" avec un bouton copier. Le toast reste jusqu'à fermeture pour les erreurs.

3. Dans une requête de lecture : pas de toast. Le composant affiche un état d'erreur en place avec "Réessayer". Un toast pour chaque liste qui échoue pendant une coupure réseau, c'était insupportable.

Les 401 ne produisent jamais de toast : le client rafraîchit ou déconnecte.

## Toasts

`useToastStore` (Zustand), rendu par `<Toaster>` à la racine, `aria-live="polite"` pour les succès et `assertive` pour les erreurs. File limitée à 3 visibles, les suivants attendent. Durée : 4 s pour un succès, 6 s pour un succès avec action "Annuler", illimitée pour une erreur.

L'action "Annuler" exécute la mutation inverse (désaffecter après affecter, rouvrir après annuler un brouillon) et c'est ce qui a permis de retirer la plupart des boîtes de confirmation, voir [[dispatchers-want-dense-ui]]. Elle n'existe que pour les mutations qui ont une inverse propre, la liste est dans `src/mutations/undoable.ts`.

## Monitoring

Toute erreur des couches 1 et 2 part au collecteur d'erreurs avec : route, navigateur, version du front (`__HF_VERSION__` injecté par Vite), `trace_id` s'il existe, et les 20 dernières entrées du journal de navigation. Jamais le contenu des formulaires. Le tableau de bord des erreurs a une vue par navigateur depuis l'incident Safari.
