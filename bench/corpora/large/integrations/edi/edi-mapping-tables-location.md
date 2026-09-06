---
name: edi-mapping-tables-location
description: Partner-specific EDI data lives in 5 tables edited from the back-office with audit and exported nightly to halden-edi-config as YAML for review; never in code
type: reference
status: active
verified: 2026-03-31
---

# Où vivent les tables de correspondance EDI

La règle depuis 2025 : **rien de spécifique à un partenaire dans le code.** Le code connaît le standard et les mécanismes ; le partenaire est de la donnée. Quand une bizarrerie ne rentre pas dans la donnée, on ajoute une clé de surcharge générique, pas un `if`.

## Les cinq tables

**`edi_partners`** : le profil (identifiants EDIFACT et AS2, version de syntaxe, fuseau, contacts, transport, options de facturation). Une ligne par partenaire, décrite dans [[edi-partners-overview]].

**`edi_partner_locations (partner_id, code, site_id, timezone, valid_from, valid_to)`** : les codes de lieu du partenaire (`LOC` avec liste `92`) vers nos sites. C'est la table la plus vivante : 1 900 lignes en mars 2026, une vingtaine d'ajouts par semaine, presque tous depuis la file de rejets ([[edi-rejects-handling]]). Un code peut changer de site (fermeture d'un dépôt, réutilisation du code) : `valid_to` sur l'ancienne ligne, nouvelle ligne, le mapper cherche à la date du message.

**`edi_partner_overrides (partner_id, key, value, ticket, created_by, created_at)`** : les surcharges du mapper ([[edifact-iftmin-mapping]]), du transport ([[as2-transport-setup]]) et de la facturation. Clés connues listées dans l'enum `OverrideKey` ; une clé inconnue est refusée à l'écriture. Quarante-trois lignes en mars 2026 pour cinq partenaires. Le champ `ticket` est obligatoire : une surcharge sans histoire est une surcharge qu'on n'osera jamais retirer.

**`edi_status_code_sets (partner_id, event, code_4405, code_list_1131, include, on_every_publication)`** : le vocabulaire de statut sortant par partenaire ([[edi-status-messages-iftsta]]).

**`edi_package_type_map (partner_id, code_7065, package_type)`** : leurs codes de colis vers les nôtres. Le standard en a 200, chaque partenaire en utilise 5, jamais les mêmes.

## Édition

`/backoffice/edi/partners/{code}` : cinq onglets, un par table, édition en ligne, permission `edi.partner_config` (rôle `staff_support_l2` et intégrations). Chaque modification écrit une ligne d'audit avec l'ancienne et la nouvelle valeur (le projet IAM a le mécanisme), et pour `edi_partner_locations` l'écran de la file de rejets écrit dans la même table par le même chemin.

Pas d'édition en SQL direct. Deux raisons : l'audit, et le cache. Le mapper met les cinq tables en cache 60 s ; l'écran invalide le cache à l'écriture, une requête SQL ne le fait pas, et on a passé une heure en 2025 à comprendre pourquoi un code ajouté à la main « ne marchait pas ».

## Export vers Git

Chaque nuit, `edi:config:export` écrit les cinq tables en YAML dans le dépôt `halden-edi-config`, un fichier par partenaire, et ouvre une MR si quelque chose a changé. Personne ne fusionne à la main : la MR est fusionnée automatiquement le matin, elle existe pour que le **diff soit lu**. La personne d'astreinte intégrations la regarde avec le rapport de réconciliation. Trois fois en 2026, ce diff a montré une surcharge posée par le support pour « débloquer » un partenaire qu'il fallait discuter (une fois, une désactivation de code de statut qui cassait la facturation des attentes).

Le sens est bien base vers Git, pas Git vers base. On a envisagé l'inverse (la config en Git, chargée au déploiement) et on l'a écarté : la file de rejets ajoute vingt codes de site par semaine, et attendre un déploiement pour qu'un chargement passe n'est pas acceptable. Git est la trace, pas la source.

## Restauration

`edi:config:import --partner nordkarton --from halden-edi-config@<sha>` recharge une table depuis un export, en écrasant. Utilisé une fois, en janvier 2026, quand un agent a supprimé par erreur 40 codes de site en croyant filtrer. Dix minutes.

## Ce qu'on ne met pas dans ces tables

- Des secrets. Les clés AS2 sont dans le vault ; `edi_partners` a le chemin, pas la valeur.

- Des règles métier de la plateforme (tarification, machine à états). Le gateway traduit, il ne décide pas.

- Des données de chargement. Les tables décrivent le partenaire, pas ses envois ; ceux-là sont dans `edi_messages` et `loads`.
