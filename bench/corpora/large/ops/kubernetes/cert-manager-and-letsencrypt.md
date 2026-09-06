---
name: cert-manager-and-letsencrypt
description: cert-manager 1.16 issues public certificates through the ACME DNS-01 solver against our DNS provider's API, one wildcard per zone, renewal at 30 days before expiry, alert at 21 days, internal certs from a private CA issuer
type: project
status: active
verified: 2026-02-13
---

# Certificats : cert-manager

Mis en place en septembre 2025 (HF-1105) en remplacement du résolveur ACME intégré de Traefik, dont la défaillance silencieuse est la cause de [[incident-2025-10-ingress-cert-expired]].

## Émetteurs

- `letsencrypt-prod` (ClusterIssuer) : ACME, solveur **DNS-01** via l'API de notre hébergeur DNS, identifiants dans un Secret géré par l'opérateur de secrets (voir [[secrets-external-secrets-vault]]). DNS-01 plutôt que HTTP-01 pour pouvoir émettre des wildcards et pour ne pas dépendre de l'ingress au moment du renouvellement, ce qui était précisément le mode de défaillance de l'ancien système.

- `letsencrypt-staging` : même chose contre l'environnement de test de l'autorité, utilisé pour valider un changement de configuration avant de toucher au vrai.

- `hf-internal-ca` (ClusterIssuer) : autorité privée dont la clé est dans un Secret du namespace `cert-manager`, pour les certificats internes (communication entre le collecteur de traces et les agents, l'exporteur PostgreSQL, le registre miroir). La racine est distribuée aux nœuds par un DaemonSet qui l'ajoute au magasin système.

## Certificats publics

Un wildcard par zone, pas un certificat par hôte :

| Certificate | Noms | Namespace | Secret |
|---|---|---|---|
| `wildcard-halden-example` | `*.halden.example`, `halden.example` | `ingress-nginx` | `tls-wildcard-halden` |
| `wildcard-staging-halden-example` | `*.staging.halden.example` | `ingress-nginx` | `tls-wildcard-staging` |
| `wildcard-preview-halden-example` | `*.preview.halden.example` | `ingress-nginx` | `tls-wildcard-preview` |

Le Secret vit dans le namespace du contrôleur d'ingress et les Ingress des autres namespaces le référencent via l'annotation `default-ssl-certificate` pour le premier, et par `tls.secretName` pour les deux autres après copie par un `reflector`. Un certificat par hôte aurait fait 30 renouvellements par trimestre et 30 occasions de rater un enregistrement DNS.

## Renouvellement et alerte

- `duration: 2160h` (90 jours, le maximum de l'autorité), `renewBefore: 720h` (30 jours). Le renouvellement se fait donc à 60 jours d'âge.

- Alerte `CertificateExpiringSoon` (`certmanager_certificate_expiration_timestamp_seconds - time() < 21 jours`) en `page`. Si le renouvellement automatique à 30 jours a échoué, on a 9 jours pour comprendre. Et `CertificateNotReady` (condition `Ready=False` pendant plus d'une heure) en `warn`.

- Une sonde externe (voir les notes d'observabilité, sondes blackbox) vérifie la date d'expiration du certificat réellement servi sur chaque hôte public, indépendamment de cert-manager. C'est la ceinture et les bretelles : cert-manager peut croire qu'il a renouvelé alors que l'ingress sert encore l'ancien Secret.

## Le piège du Secret copié

Le `reflector` copie le Secret vers les namespaces cibles à sa création et à sa mise à jour. Après un renouvellement, la copie a mis jusqu'à 10 minutes une fois, pendant lesquelles les deux versions coexistaient. Sans conséquence, mais la sonde externe l'a vu et a envoyé un `warn`. Documenté pour ne pas chercher.

## Ce qu'on ne fait pas

Pas de certificat public sur les services internes, même ceux joignables par VPN : la CA privée suffit et on n'expose pas les noms internes dans les journaux de transparence des certificats.

Voir [[ingress-nginx-config]] pour la configuration TLS côté serveur.
