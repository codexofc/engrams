---
name: mfa-recovery-codes
description: Dix codes de secours de 10 caractères générés à l'activation TOTP, stockés hachés dans mfa_methods, usage unique, régénération invalide le lot, la réinitialisation MFA par le support exige deux preuves et révoque toutes les sessions
type: reference
status: active
verified: 2026-04-20
---

## Les codes

À la confirmation d'une méthode TOTP ([[mfa-totp-rollout-dispatchers]]), `RecoveryCodeIssuer::issue(User $u)` génère dix codes de 10 caractères dans l'alphabet `abcdefghjkmnpqrstuvwxyz23456789` (sans `i`, `l`, `o`, `0`, `1`, qui se confondent une fois écrits à la main), affichés une seule fois sous la forme `xxxxx-xxxxx`, avec un bouton « télécharger en texte » et un bouton « imprimer ». Ils sont stockés dans `mfa_methods` avec `type = 'recovery'`, un enregistrement par code, `secret_ref` contenant le hash Argon2id du code (paramètres allégés : 16 MiB, 2 itérations, ça reste un secret de 50 bits d'entropie qu'on vérifie rarement).

Un code est valable tant qu'il n'a pas été utilisé (`last_used_at` nul) et que le lot n'a pas été régénéré. Utiliser un code à la place du TOTP crée la session normalement, écrit un événement `mfa.recovery_used` ([[auth-events-audit-log]]) et envoie un e-mail `auth.security_alert` (critique, non désactivable) au compte : « un code de secours a été utilisé, il vous en reste N ». À deux codes restants, l'outil dispatch affiche un bandeau pour régénérer.

La vérification passe par `RecoveryCodeVerifier::consume(User $u, string $code)`, qui essaie le code contre chaque hash non utilisé (au plus dix Argon2id à 16 MiB, environ 40 ms au total) dans une transaction avec `SELECT ... FOR UPDATE` sur les lignes, pour qu'un même code présenté deux fois en parallèle ne passe qu'une fois. Les échecs comptent dans le même seau que les échecs TOTP ([[login-rate-limiting-rules]]) : 5 par 15 minutes.

## Régénérer

`POST /v1/me/mfa/recovery/regenerate` demande un code TOTP frais (pas un code de secours, sinon un attaquant avec un seul code obtiendrait dix codes), marque les dix lignes actuelles `revoked_at = now()` et en crée dix nouvelles. Environ 300 régénérations par mois, en majorité des gens qui n'ont pas gardé le premier lot et s'en rendent compte au bandeau.

## Réinitialisation par le support

Le cas réel : le téléphone est perdu, les codes n'ont jamais été enregistrés. En mars 2026, au jour du mur, c'était 40 % des tickets MFA.

Le support ne peut pas désactiver le MFA. Il peut lancer une réinitialisation (`bin/console auth:mfa reset --user <id> --ticket <n>`, ou le bouton équivalent du back-office, même code derrière) qui :

1. exige que deux preuves d'identité soient cochées dans le ticket : un appel au numéro de téléphone enregistré sur le compte depuis plus de 30 jours, et la confirmation par l'administrateur du transporteur ou de l'expéditeur (par e-mail depuis son compte, ou par le back-office s'il est lui-même connecté avec MFA). Pour un compte administrateur, la seconde preuve est un second administrateur ou, s'il est seul, un document Verifid déjà en dossier pour l'entreprise ;

2. supprime la méthode TOTP et les codes, révoque toutes les sessions et tous les appareils de confiance ([[session-model-and-revocation]]), envoie un `auth.security_alert` à l'adresse du compte et une copie à l'administrateur ;

3. force la ré-inscription TOTP à la connexion suivante (le compte est dans l'état « MFA requis, non inscrit », comme au jour du mur).

Le tout est journalisé avec l'identifiant du ticket et celui de l'agent. 410 réinitialisations entre mars et juin 2026, aucune contestée. Un audit trimestriel prend 20 tickets au hasard et vérifie que les deux preuves y sont ; le premier audit (avril) en a trouvé 2 sur 20 où la seconde preuve était une réponse e-mail d'un dispatcher, pas d'un administrateur. Le back-office refuse depuis de valider une preuve dont l'auteur n'a pas le rôle.

## Ce qu'on a écarté

- Les codes de secours envoyés par e-mail à la demande : c'est ramener la sécurité du compte à celle de la boîte mail, ce qu'on essayait justement de dépasser.

- Un code de secours unique et long à la place de dix : l'utilisateur qui l'a utilisé une fois n'a plus rien jusqu'à régénération, et il ne régénère pas. Dix codes donnent neuf occasions de voir le bandeau.
