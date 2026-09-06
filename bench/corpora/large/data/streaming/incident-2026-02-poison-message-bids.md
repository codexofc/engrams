---
name: incident-2026-02-poison-message-bids
description: Février 2026: une enchère à montant négatif a fait planter pricing-projector en boucle sur la partition 17 pendant 3 h 20, pas de file de rebut, HF-4420
type: project
status: active
verified: 2026-03-05
---

# Incident 2026-02-11 : message empoisonné sur `domain.bid.placed`

## Impact

De 13:40 à 17:00 UTC, `pricing-projector` (le consommateur qui alimente les suggestions de prix et le tableau des enchères en direct de l'outil dispatch) est resté bloqué sur la partition 17 de `domain.bid.placed`. Les 23 autres partitions avançaient normalement. Résultat : environ 1/24 des chargements (ceux dont le `bid_id` tombe sur la partition 17, soit environ 900 chargements actifs) ont eu un tableau d'enchères figé pendant 3 h 20 dans l'outil dispatch, et les suggestions de prix pour ces chargements étaient calculées sur des enchères en retard. Aucune enchère perdue : le topic les avait, le consommateur ne les lisait pas.

Le support a reçu 6 tickets « je ne vois pas la nouvelle enchère », tous sur des chargements de la partition 17, ce qui a été la clé.

## Le message

Une enchère avec `amount_cents = -1` et `currency = "EUR"`. Produite par un client API d'un transporteur qui avait un bug de conversion (un « prix à négocier » codé en -1 dans son système). L'API avait accepté l'enchère parce que la validation de `POST /v1/bids` vérifiait `amount_cents != 0` et pas `> 0`. Le schéma du topic ([[schema-registry-compatibility-rules]]) disait `"type": "integer"` sans `minimum`. Deux gardes absents, un message valide au sens du schéma et absurde au sens du métier.

`pricing-projector` (Rust) faisait `u64::try_from(amount_cents).expect("amount is positive")`. Panique, redémarrage du pod, relecture du même offset, panique. 340 redémarrages en 3 h 20, `CrashLoopBackOff` avec des délais croissants, ce qui a rallongé l'affaire.

## Pourquoi si long

- L'alerte `torrent_partition_lag_stuck` ([[consumer-lag-alerting]]) a bien déclenché à 13:46 (`page`, `pricing-projector`, partition 17). L'astreinte produit a vu un pod en `CrashLoopBackOff`, a supposé un problème d'infrastructure, et a escaladé à la plateforme, qui a vu des redémarrages et a supposé un problème de code, et a escaladé au produit. Une heure de ping-pong.

- La trace de panique disait `amount is positive` et le fichier, pas l'offset ni la clé du message. Il a fallu ajouter des journaux et redéployer pour savoir quel message.

- La file de rebut ([[dead-letter-topics-convention]]) existait pour `ingest-svc` et `notify-fanout`, pas pour `pricing-projector`, qui avait été écrit avant la convention et jamais mis à jour. Un consommateur avec une file de rebut aurait poussé le message dans `dlq.pricing-projector` et continué en 30 secondes.

## Résolution

- 16:30 : le message identifié (offset 88 412 336, partition 17, `bid_id` 99 340 122).

- 16:45 : l'enchère retirée dans l'API par le support avec le transporteur au téléphone ; ça ne retire pas le message du topic.

- 16:55 : l'offset du groupe `pricing-projector` sur la partition 17 avancé de un (`torrent-consumer-groups --reset-offsets --group pricing-projector --topic domain.bid.placed:17 --shift-by 1 --execute`, le consommateur arrêté pendant l'opération). Redémarrage, rattrapage en 4 minutes.

## Ce qui a changé (HF-4420)

1. `POST /v1/bids` valide `amount_cents > 0` et `<= 50 000 000` (500 000 EUR, aucune enchère réelle n'a dépassé 180 000). Le schéma `domain.bid.placed` a `"minimum": 1` sur `amount_cents`, et la revue des schémas demande une borne sur tout entier métier.

2. Tous les consommateurs critiques ont la file de rebut, vérifié par un test d'intégration commun : un message qui fait échouer la désérialisation ou la logique métier finit dans `dlq.<groupe>` avec l'offset, la partition et l'erreur en en-têtes, et le consommateur continue. Le wrapper Rust `torrent_client::Consumer` le fait par défaut ; le désactiver demande un argument nommé `without_dead_letter_because: &str`.

3. Le wrapper ajoute l'offset, la partition et la clé au contexte de toute erreur ou panique. Plus jamais « amount is positive » sans savoir lequel.

4. La fiche d'alerte `torrent_partition_lag_stuck` dit en première ligne : « c'est presque toujours un message que le consommateur ne sait pas traiter ; trouvez l'offset dans les journaux du consommateur, puis lisez le message avec `torrent-console-consumer --partition N --offset M --max-messages 1` ». L'astreinte fait ça avant d'escalader.

## Ce qu'on retient

Un message valide pour le schéma peut être invalide pour le métier, et le consommateur qui fait confiance au schéma pour le métier plante. Une file de rebut n'est pas une option d'un consommateur, c'est ce qui fait qu'une partition bloquée est un incident de 30 secondes et pas de 3 heures. Le message du 11 février est toujours dans le topic, il sortira par la rétention de 30 jours, et n'importe quel rejeu ([[replay-procedure-runbook]]) le relira : depuis, il finit en file de rebut.
