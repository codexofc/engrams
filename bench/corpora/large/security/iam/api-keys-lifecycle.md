---
name: api-keys-lifecycle
description: API keys are created by an org admin with explicit scopes and an expiry of at most 365 days, shown once, rotated with a 7 day overlap, and every use updates last_used_at; unused keys are disabled after 90 days
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
