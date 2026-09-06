---
name: driver-login-pin-offline
description: Drivers log in with phone number plus 6-digit PIN, the app caches an argon2id hash of the PIN in the keychain so unlock works offline for 30 days, and the refresh token is the actual credential
type: reference
status: active
verified: 2026-04-24
---

# Connexion chauffeur et déverrouillage hors ligne

## Première connexion (en ligne obligatoire)

`POST /v2/auth/driver-login` avec `phone` (E.164) et `pin` (6 chiffres). Le serveur vérifie, renvoie un jeton d'accès (15 min) et un jeton de rafraîchissement (30 jours pour le rôle `driver`). Les deux vont dans le keychain (`flutter_secure_storage`, `accessibility: first_unlock_this_device` sur iOS, `EncryptedSharedPreferences` sur Android).

Le PIN est créé par le transporteur dans le back-office, ou par le chauffeur lui-même via un lien SMS à usage unique valable 15 minutes. Les PIN triviaux (`000000`, `123456`, `111111` et les 20 les plus courants) sont refusés.

## Déverrouillage (hors ligne possible)

À chaque retour au premier plan après plus de 10 minutes, l'app demande le PIN (ou la biométrie si activée). La vérification est **locale** : au moment de la connexion réussie, l'app a stocké dans le keychain un hash argon2id du PIN avec un sel aléatoire (paramètres : 64 Mo, 3 itérations, parallélisme 2, environ 300 ms sur un Galaxy A25). Le PIN lui-même n'est jamais stocké. Le déverrouillage ne fait aucun appel réseau.

Ce qui rend ça acceptable : le PIN local ne protège que l'accès à l'interface. La vraie autorisation, c'est le jeton de rafraîchissement. Si le compte est désactivé côté serveur, le prochain rafraîchissement échoue (401 `refresh_revoked`), l'app efface tout et revient à l'écran de connexion. Délai maximal entre désactivation et éjection effective : 15 minutes en ligne, ou la fin de la période hors ligne.

Après 5 PIN faux d'affilée, l'app efface ses jetons et exige une reconnexion en ligne. Le compteur est dans le keychain, pas en mémoire, pour survivre à un redémarrage de l'app.

## Biométrie

Optionnelle, activée depuis l'écran des réglages. Utilise `local_auth`. Elle remplace la saisie du PIN au déverrouillage, pas à la connexion. En cas d'échec biométrique, repli sur le PIN. Sur les appareils sans biométrie ou sans capteur enregistré, l'option n'apparaît pas.

## Changement de PIN

En ligne uniquement (`POST /v2/auth/driver-pin`, ancien et nouveau PIN). Le hash local est remplacé après succès. Un PIN changé depuis le back-office pendant que le chauffeur est hors ligne : l'ancien PIN continue à déverrouiller localement jusqu'au prochain passage en ligne, où le rafraîchissement du jeton renvoie `pin_changed` et force une reconnexion. Cas rare, documenté pour le support.

## Un seul appareil

La connexion sur un second appareil révoque la famille de jetons du premier. Le premier appareil s'en rend compte à son prochain rafraîchissement. Voir [[offline-sync-architecture]] pour la conséquence sur l'outbox.

## Ce qu'on ne fait pas

Pas de mot de passe, pas d'e-mail pour les chauffeurs. Beaucoup n'ont pas d'adresse professionnelle et le numéro de téléphone est ce que le transporteur connaît. Pas de SSO transporteur non plus, demandé une fois par un grand compte, pas assez de demande.
