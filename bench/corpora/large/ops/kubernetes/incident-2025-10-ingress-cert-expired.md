---
name: incident-2025-10-ingress-cert-expired
description: 2025-10-02, the api.halden.example certificate expired at 08:14 because Traefik's ACME store had been locked for 3 weeks after a Longhorn rebuild, 41 minutes of TLS errors for every client, led to cert-manager and the external expiry probe
type: project
status: active
verified: 2025-10-10
---

# Incident 2025-10-02 : certificat expiré sur `api.halden.example`

## Impact

De 08 h 14 à 08 h 55 (heure de Paris), tous les clients ont reçu une erreur TLS sur `api.halden.example` et `app.halden.example`. L'app chauffeur a affiché "Connexion impossible", le front une page blanche du navigateur, les intégrateurs des erreurs de certificat. Environ 1 100 chauffeurs en tournée, tous les dispatchers. Les données en attente sur les téléphones ont été poussées après rétablissement, rien de perdu. 41 minutes, en pleine heure de pointe du matin.

## Chronologie

- 08 h 14 : expiration. Le certificat avait été émis le 4 juillet pour 90 jours.

- 08 h 16 : premiers tickets support, puis le canal d'incident s'emballe. Aucune alerte : la sonde HTTP externe de l'époque ne vérifiait pas la date d'expiration, et l'alerte de Traefik sur les échecs ACME n'existait pas.

- 08 h 25 : l'astreinte ops constate que le certificat servi est expiré et que `acme.json` de Traefik n'a pas été modifié depuis le 9 septembre.

- 08 h 30 : dans les logs Traefik depuis le 11 septembre, toutes les 12 heures : `unable to obtain ACME certificate: unable to lock acme.json`. Le fichier était sur un volume Longhorn, et la reconstruction du réplica pendant le remplacement d'un nœud le 9 septembre avait laissé un verrou de fichier orphelin.

- 08 h 38 : suppression du fichier de verrou, redémarrage de Traefik. Traefik relance l'émission. Le défi HTTP-01 échoue une première fois parce que le certificat expiré fait échouer la redirection interne.

- 08 h 50 : émission réussie après avoir désactivé temporairement la redirection HTTPS.

- 08 h 55 : certificat valide servi, vérifié depuis l'extérieur. Fin de l'impact.

## Cause

Le système de renouvellement avait un mode de défaillance silencieux : un échec de renouvellement ne produisait qu'une ligne de log, à un endroit que personne ne lisait, et rien ne mesurait la date d'expiration du certificat réellement servi. Trois semaines d'échecs toutes les 12 heures sans que personne ne le voie.

Le verrou orphelin est l'élément déclencheur, pas la cause. La cause est l'absence de mesure indépendante.

## Ce qui a changé

1. Migration vers cert-manager avec le solveur DNS-01, décrite dans [[cert-manager-and-letsencrypt]]. Un renouvellement qui ne dépend ni de l'ingress ni d'un fichier sur un volume.

2. Sonde blackbox externe qui vérifie chaque hôte public toutes les 5 minutes et exporte `probe_ssl_earliest_cert_expiry`. Alerte `page` à 21 jours de l'expiration, indépendamment de ce que cert-manager croit. C'est la mesure qui manquait.

3. Alerte `page` sur toute condition `Ready=False` d'un `Certificate` pendant plus d'une heure.

4. Migration de l'ingress vers ingress-nginx dans la foulée, voir [[ingress-traefik-legacy]] pour les autres raisons.

5. La checklist de remplacement de nœud ([[runbook-node-drain-replace]]) a une ligne "vérifier les volumes Longhorn des pods à état après reconstruction".

## Ce qu'on retient

Un certificat, c'est une date de péremption avec une sonde dessus, ou ce n'est pas géré. Et un système qui ne peut pas remonter son propre échec doit être mesuré de l'extérieur.
