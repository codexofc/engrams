---
name: api-keys-lifecycle
description: API keys need scopes and an expiry of at most 365 days, are shown once, rotate with a 7 day overlap and are disabled after 90 days without use
type: reference
status: active
verified: 2026-07-15
---

# Cycle de vie des clés API

Les clés API servent aux intégrations des chargeurs et des transporteurs (TMS, ERP, connecteurs EDI côté client). Elles n'ont rien à voir avec les jetons de session des utilisateurs. Tout ce qui suit est implémenté dans `App\Security\ApiKey\*` et vaut pour les deux audiences.

## Création

Seul un `shipper_admin` ou un `carrier_admin` crée une clé (`apikey.create`), depuis l'interface `/settings/api-keys` ou par `POST /v2/api-keys`. Champs obligatoires :

- `label` : texte libre, unique par organisation. « Clé TMS prod » est un bon label, « test » n'en est pas un, et l'interface refuse les labels de moins de 4 caractères.

- `scopes` : sous-ensemble des permissions de l'audience. La clé ne peut jamais avoir plus que ce que son créateur a au moment de l'appel (voir [[permission-check-voter-symfony]]).

- `expires_at` : obligatoire, maximum 365 jours. Avant HF-2095 les clés étaient sans expiration, on avait 1 400 clés de plus de deux ans dont personne ne savait à quoi elles servaient.

- `allowed_cidrs` : optionnel, liste de CIDR. Recommandé, pas imposé, parce que la moitié des intégrateurs sont derrière des IP dynamiques.

La clé en clair est **affichée une seule fois** dans la réponse de création. Le stockage et le format sont décrits dans [[api-key-hashing-and-prefix]]. Le support ne peut pas la retrouver, seul `apikey.reveal` (staff admin) permet de voir les 8 premiers caractères pour aider un client à identifier laquelle de ses clés est en cause.

## Utilisation

En-tête `Authorization: ApiKey <clé>`. `ApiKeyAuthenticator` retrouve la ligne par préfixe, compare le hash, vérifie `expires_at`, `disabled_at`, les CIDR, puis construit le principal. Chaque appel réussi met à jour `last_used_at`, mais pas à chaque requête : l'écriture est différée par `ApiKeyUsageRecorder` qui agrège en Redis et écrit une fois par minute et par clé. Avant cela, la table `api_keys` prenait 200 écritures par seconde pour rien.

Les échecs d'authentification par clé sont comptés par préfixe dans Redis. Au-delà de 20 échecs en 10 minutes, le préfixe est bloqué 15 minutes et un événement `apikey.bruteforce_suspected` part dans l'audit.

## Rotation

`POST /v2/api-keys/{id}/rotate` crée une nouvelle clé avec les mêmes scopes et CIDR, et donne à l'ancienne un `expires_at` à **7 jours** (ou moins si elle expirait avant). Les deux fonctionnent en parallèle pendant la fenêtre. La nouvelle clé hérite du label avec le suffixe ` (rotated 2026-07-15)`. C'est le seul moyen prévu pour changer de clé sans coupure, et c'est celui qu'on recommande dans la doc intégrateur.

Trente jours avant l'expiration, puis 7 jours avant, un e-mail part à l'administrateur de l'organisation et au contact technique déclaré. Le message contient le label et le préfixe, jamais la clé.

## Désactivation et suppression

- `disabled_at` : posé par l'admin de l'organisation, par le support (`apikey.disable`), ou automatiquement par le job `apikey:disable-unused` quand `last_used_at` est vieux de plus de **90 jours** (ou nul et `created_at` de plus de 90 jours). L'e-mail de désactivation automatique est envoyé la veille. Réactivation possible par l'admin dans les 30 jours, ensuite la clé est supprimée.

- Suppression : `DELETE /v2/api-keys/{id}`. La ligne est conservée avec `deleted_at` pendant 13 mois pour l'audit (qui a appelé quoi avec cette clé), puis purgée.

Tous ces événements (`apikey.created`, `apikey.rotated`, `apikey.disabled`, `apikey.deleted`, `apikey.revealed`) sont dans l'audit, voir [[audit-trail-schema]].

## Chiffres (juillet 2026)

3 812 clés actives, 61 % côté chargeur. 2 190 ont un `allowed_cidrs`. Le job de désactivation en a coupé 318 depuis sa mise en place, 4 ont été réactivées. L'incident qui a précipité une partie de ces règles est décrit dans [[incident-2025-11-api-key-in-support-ticket]].

## Ce que voit l'intégrateur

La documentation publique (`docs.halden.example/api/authentication`) décrit exactement ce qui précède, avec trois exemples que le support renvoie systématiquement :

```
# créer
POST /v2/api-keys
{"label": "TMS prod", "scopes": ["load.read", "bid.create"], "expires_at": "2027-07-01T00:00:00Z", "allowed_cidrs": ["203.0.113.0/24"]}

# faire tourner (7 jours de chevauchement)
POST /v2/api-keys/{id}/rotate

# lister, sans jamais voir les secrets
GET /v2/api-keys
```

La réponse de `GET` contient `prefix`, `label`, `scopes`, `expires_at`, `last_used_at`, `disabled_at`, jamais le secret. C'est ce champ `last_used_at` que les intégrateurs regardent pour savoir laquelle de leurs cinq clés sert encore.

## Erreurs renvoyées

| Code | Enveloppe | Cause |
|---|---|---|
| 401 | `apikey_invalid` | préfixe inconnu, hash faux, ou clé supprimée |
| 401 | `apikey_expired` | `expires_at` passé ; le message contient la date et le lien de rotation |
| 401 | `apikey_disabled` | désactivée par l'admin ou par le job des 90 jours ; le message dit lequel |
| 403 | `apikey_cidr_rejected` | IP hors des CIDR ; l'IP vue est dans le message pour que l'intégrateur corrige sa liste |
| 403 | `apikey_scope_missing` | la permission demandée n'est pas dans les scopes ; le nom de la permission est dans le message |
| 429 | `apikey_prefix_locked` | 20 échecs en 10 minutes sur ce préfixe |

Distinguer `expired` de `disabled` de `invalid` a été discuté (ça renseigne un attaquant sur l'existence d'une clé). Décision : le préfixe est public de toute façon et la valeur de ces messages pour le support dépasse le risque ; un préfixe seul ne donne rien sans les 32 caractères.

## Ce qu'on n'a pas fait

- Des clés à usage unique ou à quota d'appels. Personne ne l'a demandé.

- Des clés par utilisateur (au lieu de par organisation). Une clé appartient à l'organisation et survit au départ de son créateur, à condition que le créateur ait encore ses permissions au moment de l'appel ; sinon elle échoue et l'admin la recrée. C'est le compromis retenu entre « la clé meurt avec la personne » (cassait des intégrations à chaque départ) et « la clé ne dépend de personne » (une clé créée par un stagiaire avec `apikey.*` aurait gardé ses droits pour toujours).
