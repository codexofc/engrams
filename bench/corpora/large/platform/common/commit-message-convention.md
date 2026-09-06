---
name: commit-message-convention
description: Squash commit subject is type(scope): summary with the ticket key at the end, body in first person explaining why, footers for breaking changes and migration notes, enforced by a commit-msg hook
type: reference
status: active
verified: 2025-12-01
---

# Convention de message de commit

S'applique au commit de squash de chaque MR (voir [[branching-and-pr-flow]]). Les commits intermédiaires d'une branche sont libres.

## Sujet

```
<type>(<scope>): <résumé à l'infinitif> (HF-1234)
```

- `type` : `feat`, `fix`, `perf`, `refactor`, `chore`, `docs`, `test`, `build`.

- `scope` : le module ou l'écran, en un mot : `loads`, `bids`, `sync`, `board`, `invoicing`, `ingress`. Facultatif pour `chore`.

- Résumé en français ou en anglais selon le dépôt (l'API et l'infra sont en anglais, le front et le mobile en français, c'est historique et personne ne veut migrer). Verbe à l'infinitif, pas de majuscule initiale, pas de point final, 72 caractères maximum tout compris.

- La clé de ticket entre parenthèses à la fin. Le hook `commit-msg` la vérifie sur `main`.

Exemples :

```
fix(sync): ajouter un backoff sur toute réponse 4xx (HF-1462)
perf(loads): use fetch joins on the list query (HF-1275)
chore(deps): bump doctrine/orm to 3.3.2 (HF-1470)
```

## Corps

Le pourquoi, pas le quoi (le diff dit le quoi). À la première personne quand c'est une décision : "J'ai choisi le verrou `NOWAIT` plutôt que bloquant parce que...". Lignes de 72 caractères. Le corps est ce que `git log` montrera dans deux ans à quelqu'un qui cherche pourquoi cette ligne existe, il vaut mieux qu'il dise quelque chose.

## Pieds de page

- `BREAKING CHANGE: <description>` pour un changement de contrat API ou de format de données. Il apparaît en gras dans le changelog généré.

- `Migration: <table>, <lignes en prod>, <durée estimée>` pour l'API, copie de la ligne du modèle de MR.

- `Flag: <nom du flag>` quand le changement est derrière un flag.

- `Refs: HF-1233, HF-1201` pour les tickets liés qui ne sont pas celui du sujet.

## Changelog

`CHANGELOG.md` de chaque dépôt est généré à chaque tag depuis les sujets des commits de squash depuis le tag précédent, groupés par type. Un sujet mal formé donne une ligne de changelog moche, ce qui est la sanction la plus efficace qu'on ait trouvée.

## Ce qui est refusé

Un sujet vague ("fix stuff", "corrections"), un sujet sans ticket sur `main`, un sujet de plus de 72 caractères, un message qui décrit le diff ligne par ligne. Et tout ce qui est décrit dans les règles de rédaction de l'entreprise pour les textes publiés sous notre nom : pas de tiret cadratin, pas de tournures impersonnelles dans le corps.
