---
name: playbook-driver-cannot-log-in
description: Playbook connexion chauffeur : numéro au format E.164, compteur de PIN à 5, compte désactivé, deuxième appareil, KYC transporteur, et la commande pin-link limitée à 3 par jour
type: reference
status: active
verified: 2026-07-15
---

# Le chauffeur ne peut pas se connecter

Catégorie `driver:login`, un ticket sur quatre. Le ticket vient presque toujours du transporteur, rarement du chauffeur lui-même. Il faut le `driver_id` ou le numéro de téléphone.

## Vérifications

1. **Retrouver le chauffeur.** `hfctl driver find --phone +49...`. Le numéro doit être en E.164, avec l'indicatif. Si le transporteur donne `0171...`, convertir. Pas de résultat : soit le chauffeur n'existe pas dans cette organisation (le transporteur doit le créer dans son back-office), soit le numéro enregistré est faux. `hfctl org users` ne liste pas les chauffeurs, seulement les utilisateurs web, ne pas confondre.

2. **L'état du compte.** `hfctl driver get <driver_id>`. Champs à lire : `status` (`ACTIVE` ou `DISABLED`), `pin_attempts`, `pin_set_at`, `devices`.

3. **`status = DISABLED`.** Le transporteur l'a désactivé, ou la désactivation vient du KYC de l'organisation (`disabled_reason = org_kyc`). Dans le premier cas, macro `carrier-reenable-driver`. Dans le second, voir [[playbook-kyc-verifid-stuck]], on ne réactive pas un chauffeur d'un transporteur non vérifié.

4. **`pin_attempts = 5`.** L'app a effacé ses jetons localement et le serveur bloque les nouvelles connexions pour ce chauffeur. `hfctl driver unlock <driver_id> --apply`, puis le chauffeur se reconnecte avec son PIN actuel. S'il ne s'en souvient pas, étape 6.

5. **Deuxième appareil.** `hfctl driver devices` montre deux appareils, le plus récent connecté il y a peu : le chauffeur (ou un collègue avec le même numéro, ça arrive) s'est connecté ailleurs et le premier appareil a été éjecté. C'est normal. Si l'appareil récent n'est pas le sien : `hfctl driver revoke-device <driver_id> <device_id> --apply` et le transporteur vérifie qui a le numéro. Si le même téléphone apparaît deux fois avec deux `device_id` (réinstallation), révoquer l'ancien.

6. **PIN oublié.** `hfctl driver pin-link <driver_id> --apply` envoie un SMS avec un lien à usage unique, valable 15 minutes. Trois par jour et par chauffeur maximum. Le transporteur peut le faire lui-même depuis son back-office depuis HF-3155, le dire dans la réponse pour la prochaine fois.

7. **`pin_set_at` vide.** Le chauffeur a été créé sans PIN et n'a jamais reçu le lien. Même commande qu'en 6.

8. **Tout est bon et ça échoue quand même.** Demander la version de l'app (écran « À propos »). Sous 4.6, la connexion échoue avec `unsupported_client` depuis mars 2026, la mise à jour est obligatoire. Sur un appareil sans services Google, l'installation se fait par le lien direct, pas par le store.

## Messages d'erreur côté app

- « Numéro ou PIN incorrect » : le serveur ne dit pas lequel, volontairement.

- « Trop de tentatives » : étape 4.

- « Compte désactivé » : étape 3.

- « Mise à jour requise » : étape 8.

- « Connexion impossible » sans plus : réseau, ou horloge du téléphone décalée de plus de 5 minutes (le jeton est refusé). Demander de vérifier l'heure automatique.

## Ce qu'on ne fait pas

On ne lit jamais le PIN, il n'est stocké nulle part en clair. On ne le change pas non plus, seul le lien SMS ou le back-office transporteur le font. On ne crée pas de chauffeur à la place du transporteur.

## Escalade

L2 si `unlock` puis reconnexion échoue deux fois avec un PIN dont le transporteur est sûr, ou si `pin-link` est refusé alors que le compteur du jour est à zéro.

L'ancienne version de ce playbook, d'avant le verrouillage à 5 tentatives, est [[playbook-driver-cannot-log-in-old]], gardée pour comprendre les vieux tickets.
