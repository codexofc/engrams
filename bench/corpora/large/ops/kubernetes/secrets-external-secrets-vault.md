---
name: secrets-external-secrets-vault
description: Secrets live in the ops vault on ops-tools and are projected into namespaces by External Secrets Operator ExternalSecret objects, refresh 1 h, no Secret manifests in Git, rotation by writing a new version in the vault, Kubernetes auth per namespace
type: project
status: active
verified: 2026-01-26
---

# Secrets : coffre et External Secrets Operator

## Principe

Aucun `Secret` Kubernetes n'est dans `halden-infra`. Git contient des `ExternalSecret` qui disent quelle clé du coffre va dans quel Secret, et l'opérateur (ESO) crée et maintient le Secret dans le namespace. Le coffre (serveur de secrets avec moteur clé-valeur versionné) tourne sur `ops-tools`, avec son stockage sur Longhorn sauvegardé toutes les heures.

## Organisation du coffre

Chemins `kv/<cluster>/<namespace>/<component>/<name>` :

- `kv/hf-main/platform-prod/api/database` (les champs `username`, `password`, `host`, `port`, `dbname`)

- `kv/hf-main/platform-prod/api/jwt-signing` (`private_key_pem`, `kid`)

- `kv/hf-main/platform-prod/api/object-storage`

- `kv/hf-main/platform-prod/api/webhook-secret-key`

- `kv/hf-main/ops/cert-manager/dns-provider`

- ...

Le même arbre existe pour `platform-staging` avec des valeurs différentes. Une clé de staging n'ouvre rien en prod.

## Un `ExternalSecret` typique

```yaml
apiVersion: external-secrets.io/v1
kind: ExternalSecret
metadata: { name: api-database }
spec:
  refreshInterval: 1h
  secretStoreRef: { name: vault-platform-prod, kind: SecretStore }
  target:
    name: api-database
    template:
      data:
        DATABASE_URL: "postgresql://{{ .username }}:{{ .password }}@{{ .host }}:{{ .port }}/{{ .dbname }}?serverVersion=16"
  dataFrom:
    - extract: { key: kv/hf-main/platform-prod/api/database }
```

Le `template` compose la variable que l'application attend. Le mot de passe de la base est écrit dans le coffre par l'opérateur PostgreSQL (voir [[postgres-operator-cloudnative]]) à la création du rôle, via un petit Job de synchronisation, pour qu'il n'y ait qu'une source.

## Authentification

Le `SecretStore` de chaque namespace utilise l'auth Kubernetes du coffre avec le compte de service `eso-reader` du namespace, lié à une politique qui n'autorise que `kv/hf-main/<ce namespace>/*`. ESO dans `platform-staging` ne peut pas lire une clé de `platform-prod` même si quelqu'un écrit le mauvais chemin dans un manifeste.

La politique réseau ([[network-policies-baseline]]) n'autorise l'agent ESO à joindre le coffre que depuis le namespace `external-secrets`, sur le port 8200, vers l'adresse du service sur `ops-tools`.

## Rotation

Écrire une nouvelle version dans le coffre. ESO la projette au prochain `refreshInterval` (1 h) ou immédiatement avec une annotation `force-sync`. Le Deployment ne redémarre pas seul : un contrôleur (`reloader`) surveille les Secrets référencés et fait un rollout. Pour la clé de signature JWT, la rotation suit le protocole à deux clés décrit côté API (nouvelle clé publiée dans le JWKS avant de signer avec).

Ce qui a été fait et daté : rotation du mot de passe de la base en février 2026 (10 minutes, zéro erreur grâce au rollout progressif), rotation des identifiants du fournisseur DNS en mars 2026.

## Ce qu'on ne fait pas

- Pas de `kubectl create secret` à la main, même en urgence. Le self-heal d'ArgoCD ne le supprimerait pas (le Secret n'est pas dans Git), mais personne ne saurait qu'il existe. Un secret d'urgence se met dans le coffre.

- Pas de secrets dans les variables d'environnement des pipelines CI au-delà du token de lecture du coffre pour le job qui en a besoin.

- Pas de chiffrement de secrets dans Git (`sops` ou équivalent). Discuté en 2025, rejeté : deux endroits où un secret peut vivre, c'est un de trop.

## Accès humain

Interface web du coffre par SSO, groupes `ops-admin` (tout) et `platform-oncall` (lecture des chemins `platform-prod` pendant l'astreinte, synchronisé depuis le calendrier comme les rôles Kubernetes). Toute lecture est journalisée avec l'identité.
