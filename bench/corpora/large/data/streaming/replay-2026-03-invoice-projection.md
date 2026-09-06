---
name: replay-2026-03-invoice-projection
description: Mars 2026, billing-projector a écrit un statut overdue à tort sur 2 140 factures pendant 14 jours à cause d'une comparaison de dates en UTC contre une échéance en heure locale, corrigé par un rejeu filtré sur un topic de rejeu de 61 000 messages en 40 minutes, HF-4432
type: project
status: active
verified: 2026-04-07
---

# Rejeu de mars 2026 : projection des factures

## Le bug

`billing-projector` lit `billing.invoice.lifecycle` et maintient `billing.invoice_state` (la table que l'espace client et le tableau de bord finance lisent pour afficher l'état d'une facture et calculer les relances). La version 3.7.0 déployée le 2026-02-16 a introduit une comparaison `now() > due_date` où `now()` était en UTC et `due_date` une date sans heure interprétée à minuit dans le fuseau du client. Pour une échéance au 28 février, une facture d'un client à Varsovie passait `overdue` à 23:00 UTC le 27, une heure trop tôt. Une heure, ça n'aurait pas été grave. Mais le même code servait à la relance `invoice.overdue`, et le job de relance tourne à 23:30 UTC : les factures à échéance du lendemain ont reçu une relance « facture en retard » la veille de leur échéance.

2 140 factures, 14 jours, 380 clients, 96 tickets support, une conversation désagréable avec un gros expéditeur. La règle de qualité des données qui compare le nombre de `overdue` par jour à la moyenne glissante a levé un avertissement le 2026-02-20 (+18 %) qui a été classé « fin de mois ». Le 2026-03-01, le support a fait le lien.

## La correction du code

Version 3.7.2 le 2026-03-02 : `due_date` interprété à 23:59:59 dans le fuseau du client, comparé à `now()` dans le même fuseau ; un test avec trois fuseaux et une facture à échéance le jour même. Ce n'est pas le sujet de la note.

## Pourquoi un rejeu

Corriger le code ne corrige pas les 2 140 lignes de `billing.invoice_state` déjà fausses (un `overdue` qui aurait dû être `sent`, et pour 600 d'entre elles un `overdue_since` faux alors qu'elles sont maintenant vraiment en retard). Les options :

- Un script SQL qui recalcule les lignes : c'est réécrire la logique du projecteur en SQL, une deuxième version de la même règle, avec ses propres bugs.

- Rejouer tout `billing.invoice.lifecycle` depuis le 16 février : 2,1 millions de messages, le projecteur en absorbe 900 par seconde, 40 minutes, mais ça réécrit aussi 60 000 factures qui étaient justes, ce qui est sans effet (le projecteur fait des `UPSERT`) mais bruyant, et surtout ça réémet les événements `billing.invoice.state-changed` que le projecteur produit en aval pour la relance, donc 2,1 millions de messages vers `notify-fanout`.

- Un topic de rejeu filtré ([[replay-procedure-runbook]], mode « topic de rejeu ») : seulement les messages des 2 140 factures.

La troisième.

## Ce qui a été fait (HF-4432, 2026-03-03)

1. Liste des factures touchées : requête sur `billing.invoice_state` (`status = 'overdue' AND overdue_since < due_date_local_end`) croisée avec `auth`, 2 140 identifiants dans le ticket.

2. `torrent-replay-extract --source billing.invoice.lifecycle --from 2026-02-16T00:00 --to 2026-03-02T12:00 --key-in @invoice_ids.txt --target replay.billing-projector.hf4432` : 61 000 messages retenus sur 2,1 millions (toutes les transitions de ces factures sur la période, pas seulement le passage `overdue`, parce que le projecteur reconstruit l'état par transition), 6 minutes d'extraction.

3. Liste de contrôle : idempotent oui (`UPSERT` par `invoice_id`, et l'état final se recalcule depuis la première transition de la période, qui est dans le topic de rejeu par construction) ; effets de bord, oui, la production de `billing.invoice.state-changed`, coupée par `BILLING_PROJECTOR_EMIT=0` pour cette instance ; alertes silencées sur le groupe temporaire ; support et finance prévenus.

4. Une instance de `billing-projector` 3.7.2 lancée avec `--topics replay.billing-projector.hf4432 --group billing-projector-replay-hf4432 --from-beginning`, `EMIT=0`. 40 minutes (le projecteur fait une lecture de l'état courant par message, il est plus lent que sur le flux normal). Vérification : 2 140 lignes changées, 0 autre.

5. Topic de rejeu supprimé, groupe temporaire supprimé.

6. Les 96 tickets ont reçu une réponse, les 380 clients un e-mail d'excuse (envoyé par le produit, à la main, pas par le pipeline : un `invoice.correction` automatique vers des gens qu'on venait d'agacer aurait eu l'air d'un autre bug).

## Ce que la règle de qualité a changé

L'avertissement du 20 février à +18 % a été classé sans regarder. La règle est passée d'un seuil sur le total à un seuil par pays (le fuseau étant la variable), et « fin de mois » n'est plus une raison de classement acceptée sans une requête qui le montre. Le détail est du côté de la qualité des données ; ici, l'enseignement est que le consommateur avait raison de façon uniforme et tort de façon corrélée au fuseau, ce qu'aucun compteur de retard ([[consumer-lag-alerting]]) ne montre.

## Ce qu'on retient

Le mode topic de rejeu a fait de la correction une opération de 40 minutes sur 3 % des messages, sans un seul événement en aval. Il aurait fallu 4 heures et un silence général en rejouant le groupe entier. C'est la raison pour laquelle `torrent-replay-extract` a été écrit en 2025 après le rejeu de `search-indexer`, et c'était la première fois qu'il servait en vrai.
