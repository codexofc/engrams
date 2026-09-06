---
name: playbook-webhook-not-received
description: Intégrateur sans webhooks : abonnement désactivé, livraisons DEAD et last_error, pare-feu et IP de sortie, signature, rejeu par L2
type: reference
status: active
verified: 2026-07-22
---

# L'intégrateur ne reçoit pas ses webhooks

Catégorie `integration:webhook`. Un chargeur ou un transporteur qui a branché son système sur nos webhooks dit qu'il ne reçoit plus rien, ou plus certains événements. Le diagnostic est presque toujours dans les livraisons.

## Vérifications

1. **L'abonnement.** `hfctl webhooks subs <org_id>`. Lire `status`, `disabled_reason`, `events[]`, `url`, `payload_version`.

- `disabled_reason = too_many_failures` : 50 livraisons mortes sur 7 jours, on a coupé. L'e-mail est parti à l'admin de l'org, souvent une personne partie depuis. Corriger le point d'entrée côté client, puis `hfctl webhooks enable <sub_id> --apply` et rejouer (étape 5). Macro `webhook-reenabled`.

- `disabled_reason = gone` : leur serveur a répondu 410, on coupe immédiatement, c'est documenté. Même sortie.

- `status = ACTIVE` mais l'événement attendu n'est pas dans `events[]` : ils ne sont pas abonnés à ce qu'ils attendent. Macro `webhook-events-list`, ils modifient l'abonnement eux-mêmes.

2. **Les livraisons.** `hfctl webhooks deliveries <sub_id> --status DEAD --since 7d`, puis `--status FAILED`. Lire `last_error` :

- `connect_timeout`, `connection_refused` : leur serveur ne répond pas depuis nos IP de sortie. Leur pare-feu doit autoriser nos deux adresses publiques, listées dans la doc intégrateur. Macro `webhook-firewall-ips`.

- `tls_handshake` : certificat expiré ou chaîne incomplète chez eux. Macro `webhook-tls`.

- `http_401`, `http_403` : leur point d'entrée exige une authentification qu'on n'envoie pas. On signe (`X-Halden-Signature`), on n'authentifie pas. Macro `webhook-signature-doc`.

- `http_500`, `http_502` : leur bug. On leur donne un `X-Halden-Delivery` et l'heure, ils cherchent dans leurs logs.

- `http_200` mais ils disent ne rien recevoir : un proxy ou un CDN devant eux répond à leur place. Ça arrive plus souvent qu'on croit. Macro `webhook-200-but-nothing`.

3. **La signature refusée côté client.** Trois causes : ils vérifient le corps après l'avoir reparsé (il faut le corps brut), ils ont copié le secret d'un autre abonnement, ou l'horloge de leur serveur dérive de plus de 5 minutes et ils rejettent le `X-Halden-Timestamp`. La doc intégrateur a un exemple par langage. On ne renvoie jamais un secret : s'ils l'ont perdu, ils font tourner le secret depuis leur écran, l'ancien reste valable 24 h.

4. **Test.** `hfctl webhooks test <sub_id> --apply` envoie un événement `ping`. S'il arrive, le problème est dans le filtre d'événements ou dans leur traitement. S'il n'arrive pas, c'est réseau ou TLS.

5. **Rejeu.** Une fois le point d'entrée réparé : `hfctl webhooks replay <delivery_id> --apply` pour une livraison, ou `hfctl webhooks replay-range <sub_id> --from <ts> --to <ts> --apply` (L2, 24 h maximum par appel) pour tout rattraper. Les livraisons rejouées portent le même `X-Halden-Delivery`, ils déduplique là-dessus, le leur rappeler.

## Ce qu'on ne fait pas

Pas de modification de l'URL de l'abonnement à leur place. Pas d'envoi du secret. Pas de rejeu au-delà de 30 jours, les lignes sont purgées.

## Escalade

L2 pour tout rejeu de plage, et pour un `last_error` qui n'est pas dans la liste. Backend si plus de 10 organisations ont des livraisons `FAILED` en même temps : c'est notre relais, pas leur serveur, voir [[playbook-integrator-rate-limited-429]] pour l'autre problème d'intégrateur fréquent.
