---
name: branching-and-pr-flow
description: Trunk-based on main with short-lived branches named type/HF-1234-summary, squash merge, release tags per component, hotfix branches from the tag, no develop branch since 2025
type: reference
status: active
verified: 2026-03-02
---

# Branches et merge requests (plateforme)

Vaut pour `halden-api`, `halden-web`, `halden-driver-app` et les dépôts de configuration. Les équipes ops ont les mêmes règles sur `halden-infra`.

## Branches

- `main` est déployable à tout moment. Protégée : pas de push direct, une approbation minimum, CI verte, à jour avec `main` (rebase ou merge de `main` avant fusion, le bouton de la forge le fait).

- Une branche par ticket : `<type>/HF-<numéro>-<résumé-court>`. Types : `feat`, `fix`, `chore`, `refactor`, `docs`, `perf`. Le hook `pre-receive` refuse une branche sans clé de ticket, sauf `chore/` pour les bumps de dépendances automatiques.

- Durée de vie visée : moins de 3 jours. Une branche de plus d'une semaine est découpée ou passe derrière un flag. Le tableau de bord de l'équipe affiche l'âge des MR ouvertes.

- Pas de branche `develop`, supprimée en mars 2025. Elle servait à accumuler des changements avant une release, et les releases sont continues maintenant.

## Merge requests

- Squash à la fusion. Le message de commit final est le titre de la MR, qui suit [[commit-message-convention]]. Les commits intermédiaires peuvent être sales, personne ne les reverra.

- Le modèle de MR a quatre sections : quoi, pourquoi, comment tester, et pour l'API la ligne d'estimation de migration (voir la note d'incident de novembre 2025 côté API).

- Une MR fait une chose. Une MR qui mélange un renommage et un changement de comportement est renvoyée en deux.

- Revue : voir [[code-review-rules]].

## Releases et tags

- `halden-api` et `halden-web` : déploiement continu en staging à chaque fusion sur `main`, promotion en production par tag `api-2026.19` / `web-2026.19` (année et numéro de semaine, suffixe `.1`, `.2` pour les correctifs de la même semaine). Le tag déclenche la synchronisation ArgoCD de production. Voir [[environments-dev-staging-prod]].

- `halden-driver-app` : branches `release/4.9` coupées de `main`, tag `mobile-v4.9.0`, voir la note de processus de sortie du projet mobile.

## Hotfix

Branche `fix/HF-xxxx-...` créée **depuis le tag de production**, pas depuis `main`, si `main` contient des changements non encore promus. Fusionnée dans `main`, puis un tag `.1` posé sur un cherry-pick de `main` vers... non : le tag est posé sur le commit de fusion dans `main` si `main` est promouvable, sinon sur une branche `hotfix/api-2026.19` qui reçoit le cherry-pick et qui est supprimée après. Ça arrive deux ou trois fois par an, et à chaque fois quelqu'un relit ce paragraphe.

## Dépendances automatiques

Un robot ouvre des MR `chore/deps-<paquet>` chaque lundi. Les mises à jour mineures avec CI verte sont fusionnées par le responsable de la semaine sans lecture approfondie. Les majeures attendent une personne qui lit le changelog.
