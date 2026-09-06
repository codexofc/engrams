---
name: driver-otp-login-server-side
description: Côté serveur de la connexion chauffeur: code à 6 chiffres HMAC dans otp_codes, 5 min, 5 essais, 3 demandes/10 min, liaison device_id, 68 000 connexions/jour
type: project
status: active
verified: 2026-04-28
---

# Connexion chauffeur par OTP : ce que fait le serveur

L'app conducteur n'a pas de mot de passe. Le chauffeur saisit son numéro, reçoit un code par SMS, le saisit, et l'appareil est lié à son compte pour 30 jours. Le côté app (écran, mode hors ligne avec le PIN local) est documenté par l'équipe mobile ; ici c'est ce que `auth-svc` fait, tel que livré par HF-4305 en novembre 2025 quand ce chemin a quitté le backend mobile.

## Demande de code

`POST /v1/auth/otp/request {"phone": "+48...", "device_id": "<uuid généré à l'installation>"}`.

1. Normalisation E.164, `phone_hash = sha256(numéro)`. Limites avant tout le reste : 3 demandes par numéro par 10 minutes, 10 par jour, 30 par IP par heure. Au-delà, `202` quand même et rien n'est envoyé (la même réponse, le même temps, voir [[incident-2026-03-reset-timing-enumeration]] pour la correction de mars sur ce point).

2. Le numéro doit appartenir à un chauffeur actif (`users.phone_verified` et un rattachement transporteur actif). Sinon `202` et rien. Un chauffeur qui vient d'être créé par son transporteur a `phone_verified = false` jusqu'à son premier OTP réussi, c'est le premier OTP qui vérifie le numéro.

3. Code à 6 chiffres aléatoire, `otp_codes(id, phone_hash, code_hash, device_id, expires_at = now + 5 min, attempts = 0, used_at NULL)`. `code_hash` est un HMAC-SHA256 du code avec une clé du vault (`auth/otp_hmac`) : Argon2id serait absurde pour un secret de 20 bits qui vit 5 minutes et dont la protection réelle est la limite d'essais, et 68 000 hachages coûteux par jour pour rien.

4. Le SMS part par le pipeline de notifications (`auth.otp`, TTL 5 min, seau `otp`), jamais par un appel direct au fournisseur. Le texte : « Halden : votre code est 123456. Valable 5 min. Ne le partagez pas. » plus la ligne de liaison automatique du système d'exploitation du téléphone (`@app.halden.example #123456`) qui permet le remplissage automatique.

Un nouveau code invalide les précédents du même numéro (`used_at = now()` avec `invalidated = true`).

## Vérification

`POST /v1/auth/otp/verify {"phone", "code", "device_id", "device_label"}`.

1. `SELECT ... FOR UPDATE` sur le code actif du numéro. `attempts += 1` avant la comparaison. Au 5e essai raté, le code est invalidé, et le compteur de demandes du numéro est saturé pour 10 minutes (le chauffeur devra attendre pour redemander).

2. Comparaison HMAC en temps constant. Si bon : `used_at`, création de l'utilisateur-appareil dans `driver_devices(user_id, device_id, label, first_seen_at, last_seen_at, revoked_at)`, session de 30 jours avec `device_id` ([[session-model-and-revocation]]), `auth_events` `otp.verified`.

3. Un `device_id` déjà lié à un autre chauffeur est accepté (un téléphone de société qui change de main) et l'ancien lien est révoqué avec `revoke_reason = 'device_reassigned'`, ce qui déconnecte l'ancien chauffeur de cet appareil. Un chauffeur peut avoir 2 appareils liés (le téléphone et la tablette du camion) ; un troisième révoque le plus ancien.

## Le rafraîchissement

Le refresh token est lié au `device_id` : un refresh présenté avec un autre `device_id` que celui de la session est refusé et révoque la famille (`reuse_detected`). Le `device_id` n'est pas un secret (il est dans le stockage de l'app), mais il faut l'avoir en plus du refresh token, et surtout il permet au transporteur de révoquer « le téléphone du camion 14 » depuis son outil sans savoir qui l'utilise : `DELETE /v1/carrier/devices/{device_id}`, 900 fois par mois, surtout des fins de contrat.

## Chiffres (avril 2026)

| | Valeur |
|---|---|
| demandes de code par jour | 74 000 |
| codes vérifiés par jour | 68 000 (92 %) |
| codes expirés sans essai | 4 200 (6 %) |
| codes invalidés au 5e essai | 180 |
| numéros ayant atteint 3 demandes en 10 min | 1 100 par jour, presque tous des SMS lents à l'étranger |
| appareils liés actifs | 291 000 |

Les 6 % d'expirés sans essai sont le coût d'un SMS lent : le chauffeur a redemandé, ou a abandonné. Le repli e-mail-first pour l'Italie et les pays hors table vient de ce chiffre, ventilé par pays.

## Ce qu'on n'a pas fait

- Un OTP par appel vocal pour les numéros fixes : il n'y a pas de chauffeur sur un numéro fixe.

- La vérification du numéro par « SMS silencieux » ou par l'opérateur : dépendance à des accords par pays, hors de portée.

- Une authentification par mot de passe pour les chauffeurs qui le demandent : aucun ne l'a demandé, les transporteurs non plus ; c'est le PIN hors ligne de l'app qui répond au besoin « je suis dans un tunnel ».
