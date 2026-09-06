---
name: legacy-php-session-cookies
description: Jusqu'en octobre 2025 les sessions web étaient des sessions PHP natives dans Redis, cookie PHPSESSID de 24 h glissantes, sans liste de sessions, sans révocation ciblée, sans liaison appareil, remplacées par le modèle de auth-svc
type: reference
status: archived
superseded_by: [[session-model-and-revocation]]
verified: 2025-10-10
---

# Les sessions du monolithe (archivé)

Description de ce qui existait avant l'extraction de `auth-svc`, gardée pour comprendre les tickets d'avant octobre 2025. Le modèle actuel est dans [[session-model-and-revocation]].

## Ce que c'était

Le composant de sécurité Symfony avec le gestionnaire de sessions natif de PHP, stockage dans le Redis de l'API (`session:<id>`, sérialisation PHP), cookie `PHPSESSID`, durée de vie 24 h glissante (`gc_maxlifetime = 86400`, remise à zéro à chaque requête). L'app conducteur et l'API publique utilisaient déjà des JWT et des refresh tokens ; seuls l'outil dispatch et l'espace expéditeur étaient en session PHP.

## Ce qui manquait

- **Aucune liste des sessions d'un utilisateur.** La clé Redis était l'identifiant de session ; retrouver toutes les sessions d'un `user_id` demandait de parcourir toutes les clés (`SCAN` sur 900 000 clés, 40 secondes) et de désérialiser chacune. On ne le faisait pas.

- **Pas de révocation ciblée.** « Déconnecter cet utilisateur » n'existait pas ; la seule action possible était le changement de mot de passe, qui ne fermait pas les sessions existantes (elles restaient valides 24 h). Un compte compromis en 2024 est resté utilisable une journée après le changement de mot de passe par son propriétaire.

- **Pas de fin de session absolue.** Une session glissante de 24 h utilisée tous les jours ne se terminait jamais. Des sessions de 2023 étaient encore actives en 2025 sur des postes de dispatch.

- **Pas de liaison à l'appareil, pas de MFA possible sans refaire le tout.** Le composant de sécurité pouvait le faire, mais l'état « MFA validé » aurait vécu dans la même session PHP sans possibilité de le vérifier côté API.

- **Le `PHPSESSID` en clair dans les journaux de l'ingress** pendant six mois en 2024, parce que le cookie était recopié dans un paramètre d'URL par un vieux morceau de code de partage de lien. Corrigé à l'époque, mais c'est le genre de chose qui arrive quand le jeton de session est aussi la clé de stockage.

## La migration

Le 2025-10-14, les utilisateurs web ont tous été déconnectés une fois (bandeau annoncé une semaine avant), et se sont reconnectés sur le nouveau flux. Les clés `session:*` de Redis ont été laissées expirer (24 h) puis le préfixe a été vidé. Aucun ticket support lié à la migration elle-même, une quinzaine à propos du bandeau.

Ce qui a été gardé : la durée de 12 h pour le web (moins que les 24 h d'avant, sans plainte), le stockage dans le même Redis pour la révocation, l'idée qu'un utilisateur ne doit jamais avoir à se reconnecter dans la journée.
