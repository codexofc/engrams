---
name: password-policy-and-argon2
description: Argon2id 64 MiB, 3 itérations, parallélisme 1, 140 ms par vérification, mot de passe de 12 caractères minimum sans règle de composition, liste de compromission locale de 600 M de hachés, rehash transparent à la connexion
type: reference
status: active
verified: 2026-03-24
---

## Hachage

Argon2id via `sodium_crypto_pwhash_str` avec `AUTH_ARGON2_MEMORY_KIB=65536` (64 MiB), 3 itérations, parallélisme 1. Mesuré sur les pods de `auth-svc` (2 vCPU, limite mémoire 1 GiB) : 140 ms par vérification, ce qui fait la majeure partie des 180 ms du p50 de `POST /v1/auth/login`. À 27 000 connexions par mot de passe par jour ce n'est rien, mais pendant la vague de décembre 2025 ([[incident-2025-12-credential-stuffing]]) c'est devenu 2,1 millions de vérifications en trois jours et le CPU des pods a plafonné : un hachage coûteux est aussi une surface de déni de service, et c'est pourquoi la limitation ([[login-rate-limiting-rules]]) s'applique avant le hachage, jamais après.

Chaque ligne de `credentials` porte `algo` et `params` (JSONB : mémoire, itérations). À la connexion réussie, si les paramètres stockés diffèrent de la cible courante, le mot de passe est rehaché et la ligne mise à jour dans la même transaction (`RehashOnLogin`). C'est ainsi que les 40 000 comptes encore en bcrypt coût 10 de l'ancien monolithe sont passés en Argon2id sans rien demander à personne : 71 % en un mois, 96 % en trois. Les 4 % restants sont des comptes inactifs, qui seront désactivés à 90 jours de toute façon.

## Politique

- 12 caractères minimum, 128 maximum (au-delà on tronque, pas Argon2 mais l'interface, avec un message). Pas de règle de composition : pas de majuscule, chiffre ou symbole obligatoire. La composition forcée produit `Halden2026!` et le fichier de compromission le sait.

- Refus si le mot de passe est dans la liste de compromission (ci-dessous), s'il contient l'adresse e-mail ou le nom de l'entreprise, ou s'il est dans les 10 000 mots de passe les plus courants (fichier `common_passwords.txt` du dépôt, vérifié avant la liste).

- Pas d'expiration périodique. Un mot de passe change quand il est compromis ou quand l'utilisateur le veut. La révision de l'extraction (HF-4300) a supprimé la règle des 90 jours du monolithe, qui produisait `motdepasse1`, `motdepasse2`.

- Un indicateur de force côté client, calculé localement (un estimateur d'entropie par zxcvbn embarqué), qui refuse sous le seuil « moyen ». Le serveur ne fait pas confiance au client et réapplique ses propres règles.

## Liste de compromission

`AUTH_BREACH_LIST_PATH` pointe vers un fichier de 600 millions de préfixes SHA-1 (les 5 premiers octets), trié, 3 GB, monté depuis un volume en lecture seule et mis à jour trimestriellement depuis les corpus publics de fuites. La vérification est une recherche binaire dans le fichier mappé en mémoire : 0,1 ms. On ne consulte pas de service externe avec le haché, même partiel : la dépendance réseau au moment où quelqu'un change son mot de passe, et le fait d'envoyer quoi que ce soit dérivé du mot de passe à un tiers, ont suffi à trancher.

Un mot de passe existant trouvé dans une mise à jour de la liste n'est pas révoqué : à la connexion suivante, `BreachCheckOnLogin` vérifie le mot de passe en clair reçu contre la liste, et s'il y est, pose `credentials.breached_at`, laisse la session s'ouvrir, et l'outil affiche un bandeau non fermable « changez votre mot de passe ». Après 7 jours, la connexion suivante impose le changement. Mesuré à la mise à jour de la liste de janvier 2026 : 3 100 comptes marqués, 2 400 changés dans la semaine, 700 forcés.

## Ce qu'on a écarté

- Le hachage côté client avant envoi : ne protège de rien tant que TLS tient, et complique le rehash.

- Un « poivre » (secret serveur ajouté au hachage) : le gain contre une fuite de la table est réel, mais la rotation du poivre casse tous les hachés et le vault a déjà les secrets TOTP. Décision reportée, pas refusée, ticket HF-4336 en attente.

- Un nombre d'itérations plus élevé : à 140 ms on est au coût que le déni de service par hachage rend risqué ; la sécurité supplémentaire viendrait de l'entropie des mots de passe, que la politique adresse, pas du hachage.
