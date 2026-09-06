---
name: incident-2026-03-white-screen-safari
description: March 2026, dispatchers on Safari 17 got a blank page after deploy 2026.11 because a regex lookbehind in a date helper is unsupported there and the error happened before the error boundary mounted
type: project
status: active
verified: 2026-03-26
---

# Incident 2026-03-19: white screen on Safari

## What happened

Deploy 2026.11 at 10:15. By 10:30, four support tickets from dispatchers with a blank white page on load. All on macOS Safari 17.x (a Belgian shipper who standardised on Macs, plus two iPad users). Chrome and Firefox fine. About 6 % of sessions affected, based on the user agent share.

## Root cause

A helper added in `src/utils/dateRange.ts` used a regular expression with a lookbehind assertion (`(?<=T)\d{2}`) to extract the hour from an ISO string. Lookbehind is supported in Safari only from 16.4, and the `regexpu` transform is not part of our build (we target `es2020` in Vite with `browserslist` set to `defaults`, which includes Safari 15 and 16 at the time).

On unsupported engines, a regex literal with lookbehind is a **syntax error at parse time** of the whole module. The module was imported by the app shell, so the entire `app` chunk failed to parse. Nothing rendered, no error boundary could catch it because React never mounted. The console showed `SyntaxError: Invalid regular expression: invalid group specifier name`.

Why CI did not catch it: Playwright runs on Chromium only (see [[e2e-playwright-conventions]]), and the unit tests run in Node, which supports lookbehind.

## Timeline

- 10:15 deploy

- 10:32 first ticket, support suspects a cache issue and asks to reload

- 10:50 a developer reproduces on an iPad in the office

- 11:05 root cause found from the console message

- 11:12 rollback to 2026.10 via ArgoCD (the front is a static image, rollback is instant)

- 11:40 fix merged (`slice(11, 13)` instead of the regex), 12:05 deploy 2026.11.1

50 minutes of blank page for Safari users.

## Fixes beyond the one-liner

1. `eslint-plugin-compat` added with the same `browserslist`, it flags lookbehind, `Array.prototype.at` without polyfill and similar. It found two more pre-existing issues (both `structuredClone` calls, polyfilled since).

2. Playwright now runs the smoke suite (login, board, open a load) on WebKit as well as Chromium. Adds 2 minutes to CI.

3. A tiny inline script in `index.html`, before any module, that catches `window.onerror` during boot and, if the app has not called `window.__hfBooted()` within 8 s, replaces the page with a static message "Une erreur empêche le chargement de l'application, essayez de recharger, sinon contactez le support avec le code SAFARI-BOOT" and the error text. This is the layer under the React error boundary, see [[error-boundary-and-toasts]]. A blank page is the worst possible failure mode because users think their network is broken.

4. Real user monitoring now tags the browser on every error event, so a browser-specific spike is visible on the dashboard in minutes instead of through support.

## What we did not change

The `browserslist` target. Dropping Safari 15 and 16 would have avoided this, but the iPad users are on devices that cannot upgrade. Kept, and revisited each January.
