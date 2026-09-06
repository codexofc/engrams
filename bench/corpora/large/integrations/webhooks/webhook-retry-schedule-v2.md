---
name: webhook-retry-schedule-v2
description: Relances webhook depuis HF-3120 : 10 tentatives sur 48 h avec gigue, pause par hôte après 20 échecs de connexion, et les mesures qui l'ont décidé
type: reference
status: active
verified: 2026-07-14
---

# Relances des webhooks, version 2

Remplace [[webhook-retry-schedule-v1]] depuis le 2026-05-12 (HF-3120). Le mécanisme (outbox, relais, statuts) n'a pas changé ; le calendrier, la gigue et la pause par hôte, si.

## Le calendrier

Une livraison en échec (autre chose qu'un 2xx, ou timeout à 10 s) passe en `FAILED` avec `next_attempt_at = now() + délai`. Délais par tentative :

### Tentatives

| Tentative | Délai après l'échec | Cumul approximatif |
|---|---|---|
| 1 | immédiat | 0 |
| 2 | 30 s | 30 s |
| 3 | 2 min | 2,5 min |
| 4 | 10 min | 13 min |
| 5 | 30 min | 43 min |
| 6 | 2 h | 2 h 45 |
| 7 | 6 h | 8 h 45 |
| 8 | 12 h | 20 h 45 |
| 9 | 12 h | 32 h 45 |
| 10 | 15 h | environ 48 h |

Après la dixième, `DEAD`, e-mail à l'admin de l'organisation, et comptage pour la désactivation automatique ([[webhook-auto-disable-and-dead-letters]]).

Chaque délai porte une **gigue** uniforme de plus ou moins 20 %. Sans gigue, une panne de 10 minutes chez un client faisait revenir toutes ses livraisons en rafale exactement au même instant ; avec 2 000 livraisons en attente, c'est un pic que leur serveur fraîchement redémarré encaissait mal, et on repartait en échec.

## Pause par hôte

Nouveauté de la v2 : si 20 livraisons consécutives vers le même hôte (nom DNS de l'URL) échouent en moins de 5 minutes avec une erreur de connexion (`connect_timeout`, `connection_refused`, `tls_handshake`), le relais met l'hôte en pause 10 minutes : les livraisons restent `FAILED` avec leur `next_attempt_at`, mais ne sont pas tentées avant la fin de la pause, et le compteur de tentatives n'est pas incrémenté pendant ce temps. Ça évite de consommer les dix tentatives en une heure de panne réseau chez le client.

La pause est visible dans `hfctl webhooks host-status <host>` et dans les métriques ([[webhook-delivery-metrics-and-slo]]). Elle ne concerne pas les erreurs HTTP (un 500 est une réponse, leur serveur est là).

## Pourquoi 10 et 48 h

Mesures de janvier à avril 2026 sur les livraisons `DEAD` de la v1 (7 tentatives, environ 18 h) :

- 61 % des `DEAD` concernaient des pannes client de plus de 18 h, typiquement un week-end (serveur arrêté le vendredi soir, redémarré le lundi). Avec 48 h, ces livraisons auraient abouti.

- 30 % concernaient des URL définitivement mortes (prestataire changé, environnement supprimé). Pour celles-là, dix tentatives ou sept ne changent rien ; la désactivation automatique s'en occupe.

- 9 % des erreurs de signature ou de traitement côté client, répétées à chaque tentative : là non plus le nombre ne compte pas.

Depuis la v2 : livraisons `DEAD` par semaine passées de 1 400 à 520 en moyenne (semaines 20 à 28 de 2026), et les rejeux demandés au support par des intégrateurs après un week-end ont quasiment disparu.

## Coût

Une livraison qui traverse les dix tentatives occupe une ligne `FAILED` 48 h au lieu de 18 h. L'index partiel sur `(next_attempt_at) WHERE status IN ('PENDING','FAILED')` a grossi de 15 %. Sans conséquence mesurable sur le relais.

## Ce qui n'a pas changé

Timeout 10 s par tentative. Un 410 désactive l'abonnement tout de suite. Un 2xx quelconque vaut succès, y compris 204. Les redirections 3xx ne sont **pas** suivies et comptent comme un échec `http_3xx` : on ne veut pas livrer un corps signé vers une URL qu'on n'a pas validée.
