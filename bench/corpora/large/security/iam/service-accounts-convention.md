---
name: service-accounts-convention
description: Internal calls use service accounts svc-<consumer>-<provider> with one fixed role, mTLS inside the mesh and 5 min ES256 JWTs outside it, never human creds
type: reference
status: active
verified: 2026-03-30
---

# Comptes de service

Un compte de service est une identité non humaine pour un appel entre nos propres composants (worker vers API, plateforme data vers API, passerelle télématique vers dispatch). Ce n'est ni une clé API client, ni un utilisateur staff.

## Nommage et création

`svc-<consommateur>-<fournisseur>`, tout en minuscules : `svc-datawh-api`, `svc-telematics-dispatch`, `svc-billing-payla-webhook`. Le nom dit qui appelle qui. Une ligne dans `service_accounts (code, role_code, owner_team, created_at, disabled_at)`.

Création par migration Doctrine, jamais par l'interface : un compte de service est du code, il est relu en MR. La migration insère la ligne et le rôle, le secret est posé dans le vault par la personne qui déploie (voir la note du projet ops sur External Secrets).

## Rôles

Un compte de service a **un** rôle de l'audience `service`, composé de permissions au plus juste. Les rôles existants en mars 2026 :

- `svc_read_loads` : `load.read`, `assignment.read`. Utilisé par la plateforme data.

- `svc_positions_writer` : `position.report`, `eta.publish`. Utilisé par la passerelle télématique.

- `svc_billing_events` : `payment.record`, `invoice.read`. Utilisé par le consommateur de webhooks Payla.

- `svc_edi_gateway` : `load.create`, `load.update`, `status.publish`. Utilisé par la passerelle EDI.

Pas de rôle `svc_admin`. Si un composant a besoin d'écrire partout, la conversation à avoir est « pourquoi », pas « quel rôle ».

## Authentification

Deux cas :

1. **Dans le mesh** (même cluster) : mTLS par le maillage réseau, l'identité est le SPIFFE ID du pod, mappé sur `service_accounts.code` par `ServiceAccountFromMtlsAuthenticator`. Aucun secret applicatif.

2. **Hors mesh** (un appel depuis `ops-tools` vers l'API, ou depuis un fournisseur externe qu'on héberge) : JWT signé ES256 par une clé privée propre au compte, `iss = svc-<...>`, `aud = api.halden.example`, durée 5 minutes, `jti` unique vérifié en Redis pour éviter le rejeu. La clé publique est dans `service_accounts.public_key_pem`. Rotation : on ajoute une deuxième clé publique (`public_key_pem_next`), on déploie le consommateur avec la nouvelle privée, on supprime l'ancienne. Deux colonnes plutôt qu'une table, parce qu'on n'a jamais eu besoin de plus de deux clés valides à la fois.

Ce qu'on refuse : un compte humain utilisé par un cron (retrouvé deux fois en 2025, dans les deux cas la personne avait quitté l'équipe et le cron s'est arrêté avec son compte), et une clé API client utilisée par un composant interne (retrouvé une fois, la clé d'un chargeur de test dans un script de la plateforme data).

## Audit

Les appels d'un compte de service sont dans l'audit avec `actor_type = 'service'` (voir [[audit-trail-schema]]). La revue trimestrielle du projet conformité liste les comptes sans appel depuis 60 jours, qui sont désactivés à moins qu'une équipe les réclame.

## Cas limite : les intégrateurs externes

Un partenaire qui appelle notre API pour plusieurs clients à la fois (un éditeur de TMS) n'est ni un client ni un service interne. Il passe par le flux décrit dans [[oauth-client-credentials-for-integrators]].
