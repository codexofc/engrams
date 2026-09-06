---
name: webhook-delivery-metrics-and-slo
description: Métriques du relais webhook, SLO de 99 % des événements tentés sous 60 s, l'âge du plus vieux PENDING comme sonde, et les alertes qui pagent
type: reference
status: active
verified: 2026-07-14
---

# Métriques, SLO et alertes des webhooks

Tout vient du relais (`app:outbox:relay`) qui expose des métriques Prometheus, plus deux requêtes SQL périodiques sur `sys_outbox` pour ce que le relais ne peut pas savoir quand il est en panne.

## Métriques

Exposées par le relais :

- `hf_webhook_attempts_total{result}` : `delivered`, `failed_http`, `failed_connect`, `failed_tls`, `failed_timeout`, `dead`, `skipped_paused_host`.

- `hf_webhook_attempt_duration_seconds` (histogramme) : temps de la requête HTTP vers le client, par hôte tronqué aux 50 plus fréquents pour tenir la cardinalité.

- `hf_webhook_first_attempt_delay_seconds` (histogramme) : délai entre `created_at` de la ligne et sa première tentative. C'est **notre** latence, indépendante du client.

- `hf_webhook_paused_hosts` (jauge) : hôtes en pause ([[webhook-retry-schedule-v2]]).

- `hf_webhook_replays_total` : les rejeux, comptés à part pour ne pas polluer le taux de succès.

Calculées par requête SQL toutes les 30 s (un petit exporter à part, pour survivre à la panne du relais) :

- `hf_outbox_pending_oldest_seconds` : `max(now() - created_at)` des `PENDING`. La métrique qui manquait le 4 mars ([[webhook-incident-2026-03-relay-listen-stall]]).

- `hf_outbox_rows{status}` : nombre de lignes par statut.

## SLO

**99 % des événements ont leur première tentative dans les 60 s après leur création**, mesuré sur 30 jours glissants sur `hf_webhook_first_attempt_delay_seconds`. C'est la seule promesse qu'on peut tenir : le résultat de la tentative dépend du client.

Tenue depuis avril 2026 : 99,7 % en avril, 99,8 % en mai, 99,6 % en juin (une demi-heure de retard le 22 juin pendant une migration, dans le budget). Le budget d'erreur mensuel est d'environ 7 heures de « tout est en retard », ou beaucoup plus de « un peu en retard ».

On publie aux intégrateurs une valeur plus prudente : « en général sous une minute, moins de cinq minutes dans 99,9 % des cas ». La différence est de la marge.

## Alertes

Pagent l'astreinte backend :

- `WebhookRelayStalled` : `hf_outbox_pending_oldest_seconds > 300` pendant 5 minutes. La plus importante.

- `WebhookRelayDown` : aucune métrique du relais depuis 3 minutes alors que `hf_outbox_rows{status="PENDING"} > 0`.

- `WebhookFirstAttemptSlow` : p99 de `first_attempt_delay` au-dessus de 60 s sur 15 minutes. Prévient avant que le SLO du mois soit entamé sérieusement.

Ne pagent pas, notification dans `#webhooks` :

- `WebhookHostPausedLong` : un hôte en pause depuis plus d'une heure. Le support prévient le client si c'est un Enterprise.

- `WebhookDeadSpike` : plus de 500 `DEAD` en une heure. Souvent un seul client dont l'URL est morte ; parfois un problème chez nous (le 17 juin, les 500 des clients qui plantaient sur le `driver` vide).

- `WebhookOutboxGrowth` : plus de 100 000 lignes `PENDING` ou `FAILED`. Jamais déclenchée, le maximum vu est 41 000.

## Tableau Grafana

Dossier « Integrations », tableau « Webhooks ». Rangées : santé du relais (âge des `PENDING`, débit), résultats par statut, latence des clients par hôte (les dix plus lents, utile pour dire à un client que son endpoint met 8 s), hôtes en pause, rejeux, et une rangée « par organisation » qui prend un `org_id` pour le support.

## Ce qu'on ne mesure pas

Le traitement côté client : un 200 en 3 ms ne dit rien. La complétude fonctionnelle (« tous les `load.dispatched` ont-ils bien une ligne d'outbox ») est vérifiée à part par une réconciliation nocturne dans l'entrepôt entre `load_events` et `sys_outbox`, qui n'a pas trouvé d'écart depuis l'outbox transactionnelle.
