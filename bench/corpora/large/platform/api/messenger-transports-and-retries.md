---
name: messenger-transports-and-retries
description: Symfony Messenger layout on halden-api, four transports on RabbitMQ (async, async_priority, notifications, exports), retry 3x exponential then failed transport, workers under supervisord with --time-limit=3600
type: reference
status: active
verified: 2026-04-15
---

# Messenger: transports, retries, workers

## Transports (`config/packages/messenger.yaml`)

| Transport | Broker | Usage | Consumers prod |
|---|---|---|---|
| `async` | RabbitMQ, queue `hf.async` | tout ce qui n'a pas besoin de latence | 6 pods |
| `async_priority` | RabbitMQ, `hf.async_priority` | acceptation d'offres, changements de statut, tout ce que le mobile attend | 4 pods |
| `notifications` | RabbitMQ, `hf.notifications` | push, e-mail, SMS | 3 pods |
| `exports` | RabbitMQ, `hf.exports` | PDF, exports comptables, rapports | 2 pods, 1 Go de RAM chacun |
| `failed` | Doctrine, table `messenger_messages` | messages en échec définitif | aucun |

Le routage est par classe de message. Un message sans route explicite lève une exception au boot (`MessengerRoutingCompletenessTest`) plutôt que de tomber silencieusement en synchrone, ce qui nous est arrivé une fois avec `GenerateInvoicePdf` qui a bloqué des requêtes HTTP pendant 6 s.

## Retries

```yaml
retry_strategy:
  max_retries: 3
  delay: 2000
  multiplier: 3
  max_delay: 60000
```

Soit 2 s, 6 s, 18 s puis `failed`. Les exceptions qui implémentent `UnrecoverableExceptionInterface` sautent directement en `failed` : c'est le cas de `LoadNotFoundException` (le chargement a été supprimé entre temps, inutile de réessayer) et de `InvalidPayloadException`.

Un handler qui doit réessayer plus tard sans compter comme un échec lève `RecoverableMessageHandlingException` avec `getRetryDelay()`. Utilisé par l'envoi de push quand le fournisseur répond 429.

La table `failed` se surveille : l'alerte `MessengerFailedQueueGrowing` sur la métrique `messenger_failed_messages_total` (exporter maison, commande `app:metrics:messenger` scrappée toutes les 60 s). Rejouer : `bin/console messenger:failed:retry --force`, après avoir compris pourquoi.

## Workers

Chaque pod worker fait tourner `supervisord` avec un `messenger:consume <transport> --time-limit=3600 --memory-limit=256M --limit=500`. Le `--time-limit` est là parce que le processus PHP accumule des choses (cache Doctrine, closures), pas à cause d'une fuite identifiée. Le `--memory-limit` est à 256 Mo sauf pour `exports` (768 Mo).

Le `terminationGracePeriodSeconds` du Deployment est à 90 s, et `messenger:consume` reçoit `SIGTERM` proprement grâce à `pcntl`. Un message qui prend plus de 90 s (certains exports) est redélivré par RabbitMQ à un autre consumer, donc les handlers d'export doivent être idempotents : ils vérifient `exports.status` avant de commencer.

## Pièges connus

- Un handler qui fait `$em->flush()` puis publie un message : si le message est consommé avant que la transaction soit commitée (oui, ça arrive, RabbitMQ est rapide), le consumer ne voit pas la ligne. Solution : `DoctrineTransactionMiddleware` est activé sur les bus et le `DoctrineTransportMiddleware` reporte l'envoi après commit. Voir [[webhook-delivery-outbox]] pour la version robuste.
- `messenger:consume` avec `--time-limit` et un `sleep` long dans un handler : la limite ne s'applique qu'entre deux messages.
- Les messages sérialisés contiennent les UUIDs, jamais les entités. Sérialiser une entité Doctrine dans un message, ça marche en dev et ça casse au premier changement de classe.
