---
name: abuse-shared-dispatcher-accounts
description: Février 2026, 1 800 comptes dispatch partagés par plusieurs personnes détectés par sessions simultanées et appareils de confiance saturés, décision de ne pas bloquer mais de rendre les comptes nominatifs gratuits et de forcer le MFA par personne, HF-4330
type: project
status: active
verified: 2026-05-20
---

# Comptes dispatch partagés

## Le constat

En préparant le mur MFA ([[mfa-totp-rollout-dispatchers]]), on a regardé les comptes qui allaient poser problème : un compte dont le TOTP est sur le téléphone d'une personne et qui est utilisé par cinq personnes ne survit pas au mur. Requête sur `auth.sessions` et `trusted_devices` : comptes avec au moins 3 sessions web actives simultanées depuis des IP distinctes sur 7 jours, ou 5 appareils de confiance actifs, ou plus de 2 `device.mismatch` par semaine. Résultat, février 2026 : 1 800 comptes dispatch sur 41 000, chez 600 transporteurs, avec une estimation de 6 000 personnes derrière.

Les raisons, d'après 40 appels passés par le support : la licence. Beaucoup de transporteurs pensaient que chaque compte dispatch était payant (ce n'est pas le cas depuis 2024, mais l'ancienne grille tarifaire le disait). Ensuite la commodité : « le compte du bureau », avec le mot de passe sur un post-it. Enfin, dans 30 cas, une volonté de ne pas savoir qui a accepté quel chargement, ce qui est le cas qu'on voulait vraiment trouver.

## Pourquoi c'est un problème pour nous

Un compte partagé, c'est un mot de passe qui ne change jamais, connu de gens qui ont quitté l'entreprise, sans MFA possible, et une trace d'audit qui dit « le compte a accepté » sans dire qui. Quand un transporteur conteste une acceptation d'enchère (« ce n'est pas nous »), on ne peut rien lui répondre. Deux litiges de ce type en 2025 ont fini par un geste commercial, faute de preuve.

## La décision (HF-4330)

On n'a pas bloqué. Bloquer 1 800 comptes, c'est bloquer 600 transporteurs en pleine préparation du MFA, et le résultat aurait été le contournement (un compte par personne… avec le même mot de passe partout). À la place :

1. **Message clair sur la gratuité.** Bandeau dans l'outil dispatch, pour les comptes détectés : « un compte par personne, c'est gratuit et c'est requis pour le MFA, créez-les ici ». Lien direct vers la création d'utilisateur par l'administrateur du transporteur, avec un import CSV pour les gros.

2. **Détection continue, pas ponctuelle.** `SharedAccountDetector` tourne chaque nuit sur les critères ci-dessus et pose `users.shared_suspected_at`. Un compte suspecté voit le bandeau ; l'administrateur du transporteur voit la liste des comptes suspectés de sa flotte.

3. **Le MFA fait le tri.** Au mur du 9 mars, un compte partagé sans MFA s'est retrouvé bloqué comme tout le monde, et la personne qui a le téléphone est devenue la seule à pouvoir entrer. Les autres ont dû demander un compte. C'est le mur qui a fait le travail, le bandeau l'a juste annoncé.

4. **Sessions web simultanées limitées à 3 par compte** depuis avril (la 4e révoque la plus ancienne), ce qui rend le partage pénible sans le rendre impossible. Un dispatcher légitime avec deux navigateurs et une tablette passe.

## Résultat

| | Février 2026 | Mai 2026 |
|---|---|---|
| comptes dispatch | 41 000 | 46 800 |
| comptes suspectés partagés | 1 800 | 240 |
| transporteurs concernés | 600 | 110 |
| `device.mismatch` par semaine | 1 100 | 400 |

5 800 comptes créés en trois mois, ce qui correspond à peu près aux 6 000 personnes estimées. Les 240 restants sont surtout des postes partagés de nuit dans des entrepôts, où trois personnes se relaient sur un écran ; pour ceux-là, le compte nominatif plus la déconnexion en fin de poste est la bonne réponse, et le support fait de la pédagogie, pas de la police.

Les 30 cas de « ne pas savoir qui » : 22 ont créé des comptes nominatifs après le mur, 8 ont un compte unique avec le TOTP sur le téléphone du gérant, ce qui est leur droit et ce qui rend le gérant responsable de chaque acceptation. Aucun litige d'acceptation contestée depuis mars.

## Ce qu'on retient

Le partage de compte est un problème de tarif et de commodité avant d'être un problème de sécurité, et un mécanisme de sécurité (le MFA obligatoire) l'a réglé mieux qu'une interdiction. La limite de sessions simultanées ([[session-model-and-revocation]]) est là pour que ça ne revienne pas.
