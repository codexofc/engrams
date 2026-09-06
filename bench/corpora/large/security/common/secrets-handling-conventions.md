---
name: secrets-handling-conventions
description: App secrets come only from the vault at runtime, never Git; our own secrets carry prefixes hfk_, hfs_, hfc_, hfw_; gitleaks runs pre-commit, in CI, on CI logs
type: reference
status: active
verified: 2026-05-19
---

# Conventions de gestion des secrets

La note du projet ops explique comment les secrets arrivent dans les pods (vault, External Secrets). Celle-ci dit ce qu'on attend des développeurs et du code, tous dépôts.

## Où vit un secret

- **Production et staging** : dans le vault, projeté en variables d'environnement. Le code lit `$_ENV['PAYLA_API_SECRET']` via le composant de configuration, jamais un fichier, jamais une valeur codée.

- **Développement local** : `.env.local`, ignoré par Git (`.gitignore` racine de chaque dépôt, vérifié par un test de la CI qui échoue si le motif manque). Les valeurs de dev sont des valeurs de dev : comptes sandbox, mots de passe `dev`, jamais une copie d'une valeur de staging. `make secrets:dev` remplit `.env.local` depuis le vault de dev avec des valeurs jetables.

- **CI** : variables masquées injectées depuis le vault par job, au plus juste (un job de build n'a pas les secrets du job de test, décision issue de l'incident du log de build de décembre 2025 dans le projet incidents).

- **Nulle part ailleurs.** Pas dans un ticket, un message de chat, une capture d'écran, un commentaire de code, un fichier de fixture, une documentation. Un secret qui a été à l'un de ces endroits est considéré fuité et se fait tourner.

## Nos propres formats

Tout secret que **nous** émettons porte un préfixe qui le rend reconnaissable par un scanner :

| Préfixe | Ce que c'est | Émis par |
|---|---|---|
| `hfk_` | clé API client | projet IAM |
| `hfs_` | secret SCIM d'un tenant | projet IAM |
| `hfc_` | secret de client OAuth intégrateur | projet IAM |
| `hfw_` | secret de signature de webhook sortant | plateforme |
| `hft_` | jeton de lien de suivi public | plateforme |

Un nouveau type de secret prend un préfixe `hf?_`, l'ajoute à ce tableau et au fichier `gitleaks.toml` partagé le même jour. Les jetons de session (JWT) n'ont pas de préfixe mais le motif JWT est dans le scanner.

## Détection

`gitleaks.toml` vit dans le dépôt `halden-security` et est tiré par chaque dépôt via un sous-module léger (un fichier, mis à jour par un job hebdomadaire qui ouvre une MR). Il contient les règles par défaut plus nos préfixes, les formats de nos fournisseurs (Payla, Verifid, les deux fournisseurs télématiques) et deux règles maison sur les IBAN dans les fixtures.

Trois points d'exécution :

1. **Pre-commit** sur le poste (voir [[gitleaks-precommit-feedback]] pour ce que ça a donné).

2. **CI** sur chaque MR, sur le diff et sur l'historique de la branche.

3. **Sortie de CI** : le log de chaque job est scanné après exécution, un hit fait échouer le job et page la rota ([[security-oncall-rota]]).

Une fausse alerte se supprime avec un commentaire `# gitleaks:allow` sur la ligne, qui doit être justifié dans la MR. On en compte 41 dans tous les dépôts en mai 2026, tous relus au dernier audit, aucun retiré.

## Rotation

- Secrets qu'on émet aux clients : rotation à leur initiative, avec chevauchement (7 jours), voir le projet IAM.

- Secrets de fournisseurs qu'on détient : rotation annuelle au minimum, à la date anniversaire notée dans `vault:secret-inventory` (un fichier YAML dans `halden-security` avec chemin vault, propriétaire, date de dernière rotation, procédure). Le job hebdomadaire signale les secrets de plus de 330 jours. Vingt-six secrets dans l'inventaire.

- Secrets de signature internes (JWT, comptes de service) : deux clés valides à la fois, on ajoute la nouvelle, on bascule, on retire l'ancienne. Jamais de rotation « d'un coup » sans chevauchement, l'incident de rotation des webhooks de juin 2026 a coûté assez cher pour qu'on l'écrive ici.

## En cas de fuite

Un secret aperçu là où il ne devrait pas être : on le fait tourner d'abord, on déclare l'incident (SEV2 si interne, SEV1 si externe) ensuite, on cherche la cause après. L'ordre n'est pas négociable. Le canal est `#signalement-securite`, la phrase attendue est « j'ai vu un secret dans X ».

## Ce qu'on ne fait pas

- Chiffrer des secrets dans Git (fichiers `.enc`). On l'a fait en 2023, la clé de déchiffrement s'est retrouvée dans trois postes et une CI. Le vault existe pour ça.

- Des secrets « partagés d'équipe » dans un gestionnaire de mots de passe grand public. Ce qui est partagé va dans le vault avec une politique nominative.
