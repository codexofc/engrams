---
name: backlog-drain-2026-06-courrix-outage
description: Juin 2026: panne Courrix de 3 h 10, file email à 410 000, digests en pause, drain en 52 min à 130 envois/s, 18 000 messages périmés abandonnés, HF-4260
type: project
status: active
verified: 2026-07-01
---

# Drain de la file après la panne Courrix du 2026-06-17

## Ce qui s'est passé chez le fournisseur

Courrix a été indisponible sur son API d'envoi de 08:50 à 12:00 UTC le 2026-06-17 (incident de leur côté, une migration de base de données qui a mal tourné, d'après leur rapport). Les webhooks ont continué de fonctionner pour les messages déjà acceptés, le suivi de l'existant n'a donc pas souffert. Bipline et le push n'étaient pas touchés.

## Ce qu'on a fait pendant

- 08:56 : `NotifyBacklog` page à 20 000. `messenger:stats` : tout dans `email`, `sms` normale. Panneau fournisseur : 100 % de timeouts vers `api.courrix.example`. Diagnostic en trois minutes grâce au [[notifications-oncall-runbook]], qui a été suivi à la lettre pour la première fois en conditions réelles.

- 09:05 : pause des digests (`notifications:pause-event bid.digest`). Les enchères continuaient d'être regroupées dans `notification_batches`, seule la livraison du digest attendait. Sans cette pause, à la reprise on aurait envoyé un digest de 9 h puis un de 10 h puis un de 11 h à chaque expéditeur, avec des enchères périmées.

- 09:10 : les OTP e-mail (repli de l'OTP SMS en Italie et pour les pays hors table) basculés sur SMS partout par un drapeau temporaire `OTP_FORCE_SMS=1` sur les workers. 1 200 OTP de plus par SMS sur la matinée, 70 EUR.

- 10:30 : la file dépasse 300 000. Aucune action : le runbook dit d'attendre et c'est ce qu'on a fait. Les factures du matin (`invoice.issued`, 4 100 messages) attendaient dans la file comme le reste, ce que la facturation a accepté après un message dans leur canal.

- 12:00 : Courrix répond. File à 410 000.

## Le drain

Six workers, 130 envois par seconde en régime établi (Courrix accepte 300 par seconde sur notre offre, mais le rendu et la base font le reste), 52 minutes pour vider la file. On n'a pas ajouté de workers : à 300 par seconde on aurait touché le seuil du fournisseur pendant une reprise où lui aussi était fragile, et la moitié des messages dans la file n'avaient plus de raison d'être envoyés.

Ce qui a été abandonné volontairement avant la reprise, par `notifications:purge-queue` :

| Type | Abandonnés | Raison |
|---|---|---|
| `auth.otp` (e-mail) | 2 300 | TTL 5 min, tous périmés |
| `bid.received` immédiats | 9 800 | l'enchère est visible dans l'outil, le digest de reprise la contient |
| `load.eta_alert` | 4 100 | une alerte d'ETA de 09:00 à 12:30 n'informe plus |
| `document.available` | 1 800 | le document est dans l'espace client, notification redondante à 3 h |

18 000 au total, tous en `suppressed` avec `suppression_reason = 'expired'` ou `'purged_by_oncall'`, visibles dans `notifications:trace`. Les 392 000 restants ont été envoyés. À 12:55 la file était vide, à 13:05 le digest a été repris avec un seul envoi par expéditeur couvrant 08:50 à 13:00.

## Conséquences

Aucun ticket support sur les e-mails eux-mêmes. Trois tickets d'expéditeurs disant « je n'ai pas reçu d'enchères ce matin » alors qu'ils en avaient dans l'outil, ce qui confirme que l'e-mail est le canal de réveil, pas la source.

Le temps médian de `dispatch()` à `sent` sur la journée est de 47 minutes contre 1,8 s en temps normal, chiffre qui sera dans le rapport mensuel comme la seule anomalie de juin.

## Ce qui a changé après (HF-4260)

- Le drapeau `OTP_FORCE_SMS` est devenu une règle : quand Courrix est déclaré indisponible par la sonde (3 échecs consécutifs sur `GET /v1/ping`), `SmsRouter` inverse la préférence e-mail-first pour l'OTP. Automatique, plus de drapeau à poser à la main.

- La pause des digests est automatique aussi : `bid.digest` a `pause_when_provider_down: true` dans `event_types.yaml`, et `notify-worker` reporte la livraison de 15 minutes en boucle tant que la sonde est rouge. Idem pour `load.eta_alert`, qui est en plus purgé automatiquement par son `ttl` de 30 minutes.

- On a écrit la liste des types « à abandonner sans réfléchir en cas de reprise » dans le yaml (`drop_on_recovery: true`) plutôt que de la laisser à la mémoire de l'astreinte. Le `purge-queue` du runbook a un mode `--drop-on-recovery` qui l'applique.

## Ce qu'on retient

Le pipeline a fait exactement ce pour quoi il a été refait ([[notifications-pipeline-overview]]) : rien n'a été perdu, rien n'a été envoyé deux fois, et ce qui n'avait plus de sens a été jeté en connaissance de cause. La seule décision humaine qu'on veut garder humaine, c'est le choix des workers au moment de la reprise.
