---
name: e2e-playwright-conventions
description: Playwright E2E runs against a seeded staging-like stack in CI (Chromium and WebKit for smoke, Chromium only for the rest), selectors by role and data-testid, API calls mocked only for third parties, 9 min budget
type: reference
status: active
verified: 2026-04-03
---

# Tests de bout en bout (Playwright)

## Contre quoi ça tourne

Une pile complète lancée par le CI : l'image de l'API du dernier `main`, une base PostgreSQL restaurée depuis `fixtures/e2e-seed.sql` (40 chargements, 5 transporteurs, 3 chargeurs, tous les statuts représentés), Redis, et le front servi par `vite preview`. Pas de mock de l'API : un test qui passe contre un mock et échoue contre l'API réelle ne sert à rien, et c'est arrivé.

Les seuls appels mockés (`page.route`) sont ceux vers des tiers : tuiles de carte, service de routage. Ils répondent depuis des fichiers de `e2e/fixtures/`.

## Navigateurs

- Suite complète (58 scénarios) : Chromium.

- Suite `smoke` (connexion, tableau de bord, ouverture d'un chargement, affectation) : Chromium **et** WebKit depuis [[incident-2026-03-white-screen-safari]]. Firefox a été retiré en 2025, aucun bug spécifique trouvé en un an et 3 minutes de CI.

## Sélecteurs

Dans l'ordre de préférence :

1. `getByRole('button', { name: 'Affecter' })`. Le nom est le texte traduit en `fr`, langue forcée dans les tests. Ça teste aussi l'accessibilité au passage.

2. `getByLabel`, `getByPlaceholder` pour les champs.

3. `getByTestId('load-card-L-2026-004512')` uniquement quand le rôle n'est pas discriminant (les cartes du tableau). Les `data-testid` sont conservés en production, ils pèsent rien.

Interdit : sélecteurs CSS de structure (`.board > div:nth-child(2)`), et XPath. ESLint `playwright/no-raw-locators`.

## Structure

`e2e/<zone>/<scenario>.spec.ts`. Chaque spec commence par `loginAs('dispatcher@shipper-a')` (un helper qui poste sur l'API de connexion et pose le cookie de session de test, pas via le formulaire, pour gagner 3 s par test). Les données sont créées via l'API dans `beforeEach` avec `apiFixture` et supprimées après, jamais partagées entre specs. Un test qui dépend de l'ordre est un bug.

Le formulaire de connexion lui-même a son propre spec, le seul qui passe par l'interface.

## Attentes

`expect(locator).toBeVisible()` et `toHaveText()` avec le timeout par défaut (5 s). Jamais de `page.waitForTimeout()`. Pour les mises à jour temps réel (voir [[websocket-live-updates]]) : déclencher le changement via l'API, puis `expect(...).toHaveText(..., { timeout: 10_000 })`.

## Accessibilité automatisée

`@axe-core/playwright` lancé sur les 12 routes principales dans le spec `a11y.spec.ts`. Échec sur `serious` et `critical`. Voir [[a11y-keyboard-drag-drop-dispatch]].

## Budget et instabilité

9 minutes en CI, 4 workers. Un test instable est mis en quarantaine (`test.fixme` avec le ticket) et compté dans le résumé de PR. Plus de 3 en quarantaine = on arrête d'en ajouter jusqu'à ce qu'on en répare. Les causes d'instabilité rencontrées, dans l'ordre : animation de panneau latéral pas finie (corrigé par `prefers-reduced-motion` forcé dans les tests), tri instable de la liste sur des dates identiques (corrigé côté API), et le `seq` de WebSocket non remis à zéro entre deux tests (corrigé par un rechargement de page dans `afterEach`).

## Traces

`trace: 'retain-on-failure'`, la trace est jointe au job CI. C'est le premier réflexe quand un test échoue, avant de relancer.
