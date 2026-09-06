---
name: mfa-rollout-backoffice
description: MFA (TOTP or WebAuthn) became mandatory for all staff in Idento on 2026-01-15 after a 6 week opt-in; hardware keys required for staff_admin and finance
type: project
status: active
verified: 2026-03-18
---

# Déploiement du second facteur pour le staff (HF-2099)

## Point de départ

Fin 2025, l'authentification staff via Idento (voir [[sso-oidc-provider-setup]]) reposait sur mot de passe seul pour 61 comptes sur 84. Les 23 autres avaient activé le TOTP de leur propre initiative. Le pentest d'octobre 2025 et l'incident de phishing du même mois (projet `security/incidents`) ont fait passer le sujet de « souhaitable » à « avant fin janvier ».

## Options retenues

Idento propose TOTP, WebAuthn (clé matérielle ou authentificateur de plateforme) et codes de secours. On a activé les trois avec les règles suivantes :

- **TOTP ou WebAuthn au choix** pour tout le monde. Pas de SMS, on ne l'a jamais eu et on ne l'ajoute pas.

- **Codes de secours** : dix codes à usage unique générés à l'inscription, à conserver hors du poste. Utilisés 4 fois depuis janvier.

- **WebAuthn obligatoire** pour `staff_admin` et `staff_finance` depuis le 2026-03-02, avec une clé matérielle fournie par l'entreprise. Onze personnes concernées. Le TOTP reste possible en second facteur de secours pour ces comptes, mais le premier enregistrement doit être une clé.

## Calendrier

- 2025-12-01 : opt-in ouvert, bannière dans le back-office, message sur le canal général. Aide à l'inscription en visio deux fois par semaine.

- 2025-12-15 : 58 % inscrits.

- 2026-01-08 : 89 %. Relance nominative des 9 restants par leur responsable.

- 2026-01-15 : bascule `otp_required = true` sur le realm. Les comptes non inscrits sont forcés à s'inscrire à la prochaine connexion, pas bloqués.

- Jour J : 3 personnes bloquées. Deux avaient changé de téléphone entre l'inscription et janvier sans migrer l'application TOTP, une avait perdu ses codes de secours. Réinitialisation par un admin Idento après vérification d'identité en visio (règle : on voit la personne, on lui demande une info que seul son manager connaît, et le manager confirme par écrit). Quinze minutes chacune.

## Ce qu'on a appris

- L'opt-in de six semaines a fait 89 % sans contrainte. La contrainte n'a servi qu'aux 11 % restants, et c'est très bien comme ça.

- La réinitialisation MFA est le point faible du dispositif. Elle est maintenant réservée à deux personnes, tracée comme `auth.mfa_reset` dans l'audit ([[audit-trail-schema]]) avec le nom de la personne qui a vérifié l'identité, et elle déclenche un e-mail au compte concerné et à son manager.

- Le support console (impersonation, voir [[impersonation-support-mode]]) exige une ré-authentification avec second facteur au démarrage de chaque session d'impersonation, pas seulement à la connexion. Ajouté après la bascule, quand on a réalisé qu'une session de 4 heures ouverte le matin suffisait pour l'après-midi.

## Chiffres de mars 2026

84 comptes, 100 % inscrits, 39 en WebAuthn (dont 11 obligatoires), 45 en TOTP. Zéro connexion réussie sans second facteur depuis le 15 janvier, vérifié par la requête sur `auth.login_succeeded` avec `details.mfa = false`.

Les clients (chargeurs, transporteurs) ne sont pas concernés par cette note. Le second facteur côté client est un autre chantier, non commencé, dont le blocage principal est que la moitié des utilisateurs transporteurs partagent un compte par agence.
