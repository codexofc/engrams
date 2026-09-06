---
name: webhook-mtls-request-declined
description: Décision HF-3099 de ne pas proposer mTLS ni en-tête d'authentification sur les webhooks, et l'alternative : IP de sortie fixes plus signature
type: feedback
status: active
verified: 2026-04-08
---

# Pas de mTLS sur les webhooks sortants (HF-3099)

Un chargeur Enterprise (industrie, équipe sécurité exigeante) a demandé en février 2026 que nos webhooks présentent un certificat client (mTLS) à son endpoint, ou à défaut un en-tête `Authorization` avec un jeton qu'il nous fournirait. Sa politique interne : aucun endpoint exposé ne peut se contenter d'une signature dans le corps. On a passé une semaine dessus et dit non, avec une alternative.

## Ce qui a été examiné

**mTLS.** Techniquement : un certificat client par abonnement ou par organisation, généré par nous ou fourni par le client, présenté par le relais à la connexion. Ce que ça implique :

- Stocker des clés privées de certificats clients, chiffrées, avec rotation, par abonnement. Une nouvelle catégorie de secret, avec les procédures qui vont avec.

- Gérer l'expiration : un certificat expiré côté client, c'est une désactivation silencieuse avec `tls_handshake` en `last_error`, que le client mettra une semaine à comprendre.

- Le relais fait aujourd'hui une connexion par tentative, sans état par abonnement. Le mTLS impose un contexte TLS par abonnement, ce qui casse la mutualisation du client HTTP et son pool.

- Une seule demande en deux ans. Le coût est permanent, le bénéfice concerne un client.

**En-tête personnalisé.** Plus simple : une valeur opaque par abonnement, envoyée dans un en-tête choisi par le client. Mais c'est un secret de plus (le premier, la clé de signature, est déjà à nous ; celui-là serait à eux, stocké chez nous), et ça ne prouve rien que la signature ne prouve pas déjà : la signature HMAC avec un secret partagé **est** une authentification de l'émetteur, plus robuste qu'un jeton statique parce qu'elle couvre le corps et l'horodatage.

## Ce qui a été retenu

- Les deux **adresses IP de sortie fixes** du relais ([[webhook-egress-static-ips]]) : le client les met en liste blanche sur son pare-feu, ce qui satisfait « pas d'endpoint ouvert à tous » au niveau réseau.

- La **signature** ([[webhook-signature-verification-guide]]) pour l'authenticité et l'intégrité, avec la fenêtre de 5 minutes contre le rejeu.

- Une lettre d'architecture d'une page, signée par le responsable sécurité, décrivant les deux mécanismes, que l'équipe sécurité du client a pu classer. C'est ce dont ils avaient besoin en réalité : une justification écrite pour leur audit, pas un certificat.

Le client a accepté. Leur endpoint est derrière une liste blanche et vérifie la signature, comme tous les autres.

## Ce qu'on a écrit dans la doc

Une section « Sécuriser votre endpoint » dans le guide intégrateur : liste blanche des adresses de sortie, vérification de la signature, HTTPS avec un certificat valide, répondre 401 (et non 200) à une signature invalide pour que ça apparaisse dans `last_error`. Et une phrase disant qu'on ne propose ni mTLS ni en-tête d'authentification, avec le lien vers cette justification.

## Si ça revient

Trois demandes Enterprise dans l'année, ou une exigence réglementaire dans un pays où on facture, et on rouvre. Le mTLS resterait par organisation (pas par abonnement), avec des certificats générés par nous, expirant à 13 mois, et une alerte 30 jours avant. La conception est dans le ticket, pas dans le code.
