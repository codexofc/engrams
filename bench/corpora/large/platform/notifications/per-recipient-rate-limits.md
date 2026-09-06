---
name: per-recipient-rate-limits
description: Limites par destinataire dans notify-worker, seaux Redis nrl:<canal>:<destinataire>, 5 SMS/h et 20/j, 30 e-mails/h, 60 push/h, les OTP ont leur propre seau, dépassement = suppressed pas failed
type: reference
status: active
verified: 2026-03-11
---

## Pourquoi une limite par destinataire

Une limite globale protège le fournisseur, pas la personne. Ce qui a fait mal en janvier ([[incident-2026-01-otp-sms-retry-loop]]), c'est 300 chauffeurs qui ont reçu 40 SMS chacun en une heure : le débit global était sous la limite Bipline, et personne n'avait défini ce qu'un destinataire pouvait recevoir au maximum. La limite par destinataire existait sur le papier depuis la refonte (HF-4100) et n'était pas branchée. Elle l'est depuis le 2026-01-22 (HF-4133).

## Implémentation

`RecipientRateLimiter::allow(string $channel, string $recipientKey, string $bucket): Decision` dans `src/Notifications/RateLimit/`. Fenêtre glissante dans Redis, clé `nrl:<canal>:<bucket>:<destinataire>`, un `ZADD` du timestamp puis `ZREMRANGEBYSCORE` et `ZCARD`, dans un script Lua pour l'atomicité. `recipientKey` est `user:<id>` quand on a un utilisateur, sinon `addr:<sha256 de l'adresse normalisée>` pour les envois anonymes (invitations, contacts de facturation non inscrits).

La décision est prise dans `notify-worker` juste avant l'appel au fournisseur, pas au `dispatch()`. Raison : un digest qui attend 50 minutes en file ne doit pas consommer le quota au moment où il est créé mais au moment où il part, sinon la limite ne dit rien du rythme réellement reçu.

### Les seaux

| Canal | Seau | Limite | Fenêtre |
|---|---|---|---|
| sms | `default` | 5 | 1 h |
| sms | `default` | 20 | 24 h |
| sms | `otp` | 3 | 10 min |
| sms | `otp` | 10 | 24 h |
| email | `default` | 30 | 1 h |
| email | `otp` | 5 | 10 min |
| push | `default` | 60 | 1 h |

Les deux limites d'un même seau s'appliquent toutes les deux. Le seau `otp` est séparé pour qu'une rafale d'affectations ne bloque pas la connexion, et l'inverse.

## Ce qui se passe au dépassement

La livraison passe en `suppressed` avec `suppression_reason = 'rate_limited'`, pas en `failed` : elle ne sera pas réessayée, l'intention reste dans `notifications` et l'application l'affiche dans la cloche. Le compteur `notify.rate_limited{channel, bucket}` alerte en `warn` au-dessus de 200 par heure, ce qui est le signal d'un producteur en boucle bien avant la facture.

Sur les OTP, le dépassement remonte jusqu'à l'utilisateur : l'écran de connexion affiche « trop de codes demandés, réessayez dans 10 minutes ». Le service d'authentification lit la décision via le retour de `dispatch()` (`DispatchResult::rateLimited()`), c'est le seul cas où le résultat du rate limit est synchrone, parce que l'OTP est envoyé en priorité par le worker et la décision est prise avant la mise en file.

## Ce que la limite ne fait pas

- Elle ne dédoublonne pas. Deux notifications identiques à 10 secondes d'intervalle passent toutes les deux si le quota le permet. La déduplication est l'affaire de l'`idempotency_key` sur la livraison et du regroupement des digests ([[shipper-bid-digest-batching]]).

- Elle ne s'applique pas aux envois marqués `critical` dans `event_types.yaml` : `load.cancelled` la veille d'un enlèvement et `auth.security_alert`. Trois types en tout, la liste est relue à chaque ajout.

- Elle ne remplace pas la limite fournisseur (sémaphore `bipline:rps`), qui protège le débit global.

## Chiffres

En février 2026, 1 100 livraisons `rate_limited` sur 7,5 millions, dont 900 push vers 6 comptes dispatch d'un même transporteur qui recevaient chaque changement de statut de 400 chargements. La bonne réponse n'était pas d'augmenter la limite mais un digest push pour ce type d'événement, livré en mars.
