---
name: sms-provider-bipline-integration
description: Bipline est le fournisseur SMS depuis octobre 2025, API REST v2, expéditeur alphanumérique HALDEN sauf pays interdits, accusés par webhook, 3 segments max, contrat 300 000 SMS par mois
type: reference
status: active
verified: 2026-06-02
---

## Le fournisseur

Bipline (`sms.bipline.example`), choisi en septembre 2025 (HF-4071) contre deux autres sur trois critères : couverture directe des opérateurs polonais et roumains sans passer par un agrégateur gris, accusés de réception réels (pas « remis à l'opérateur »), et une API qui accepte un identifiant client d'idempotence. Le contrat est de 300 000 SMS par mois, dépassement facturé à l'unité, révisé en janvier ([[sms-country-routing-and-unit-costs]] pour les tarifs par pays).

## L'adaptateur

`BiplineSmsClient` dans `src/Notifications/Sms/`. Un appel `POST /v2/messages` par SMS avec :

- `to` en E.164, normalisé par `PhoneNumberNormalizer` à partir de `users.phone` et du pays de l'entreprise quand l'indicatif manque (11 % des numéros carriers étaient saisis sans indicatif en 2025) ;

- `from` : l'expéditeur alphanumérique `HALDEN` partout où c'est autorisé, un numéro long local dans les pays qui l'interdisent ou le filtrent (voir la table de routage) ;

- `text`, au plus 3 segments GSM-7 (459 caractères) ; l'adaptateur compte les segments avec la table GSM-7 et refuse au-delà (`SmsTooLong`, pas de retry). Un caractère hors GSM-7 (un `ő` hongrois, un guillemet typographique collé depuis un traitement de texte) fait passer le message en UCS-2 et divise la capacité par deux ; le lint des templates le signale ;

- `client_ref` : l'`idempotency_key` de la livraison, que Bipline garde 48 h et sur lequel il déduplique ;

- `callback_url` : `https://api.halden.example/webhooks/bipline`.

La réponse porte un `message_id` stocké dans `notification_deliveries.provider_ref`.

## Accusés de réception

Le webhook reçoit `queued`, `sent`, `delivered`, `undelivered`, `rejected`, avec un `reason` (`absent_subscriber`, `unknown_number`, `blocked`, `expired`). `undelivered` avec `unknown_number` trois fois de suite sur 30 jours met le numéro dans la liste de suppression SMS ([[bounce-handling-and-suppression]]). Le webhook est authentifié par une signature HMAC dans l'en-tête `X-Bipline-Signature`, clé dans le vault, vérifiée avant toute lecture du corps.

Le taux de `delivered` final est de 97,8 % sur mai 2026. Les 2,2 % restants sont surtout des `absent_subscriber` sur des chauffeurs à l'étranger sans itinérance data, qui reçoivent le SMS quand ils rentrent, souvent après l'expiration de 6 h que nous demandons (`validity_period: 21600`).

## Limites et quotas

Bipline limite à 50 requêtes par seconde par compte. `notify-worker` respecte cette limite avec un sémaphore Redis (`bipline:rps`), et de toute façon le pic observé est de 18 par seconde à 7 h, quand les OTP des connexions du matin et les affectations de la nuit se cumulent. Le pic accidentel de janvier ([[incident-2026-01-otp-sms-retry-loop]]) a atteint la limite et Bipline a répondu 429, ce qui a été notre premier signal.

## Environnements

En staging l'adaptateur est configuré sur le compte de test Bipline, qui n'envoie qu'aux numéros d'une liste blanche (les téléphones de l'équipe). En dev, `NOTIFY_SMS_PROVIDER=sink` écrit les SMS dans le journal. Un déploiement de prod avec `NOTIFY_SMS_PROVIDER=sink` est refusé au démarrage par une vérification dans `Kernel::boot()`, ajoutée après qu'une release de staging soit restée deux jours à envoyer dans le vide.

## Coupure du fournisseur

Pas de second fournisseur SMS. La décision a été prise en janvier : un second fournisseur coûte un contrat minimum et une seconde intégration à maintenir, pour un canal dont 70 % des messages sont des OTP qui ont un repli (l'OTP e-mail, plus lent mais fonctionnel). Si Bipline est indisponible, les OTP basculent sur e-mail après 90 secondes et les affectations restent en file. La panne la plus longue à ce jour est de 40 minutes (mars 2026), sans ticket support.
