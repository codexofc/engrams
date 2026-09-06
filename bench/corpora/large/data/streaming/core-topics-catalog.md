---
name: core-topics-catalog
description: Catalogue des 30 topics qui comptent sur torrent en juin 2026, par famille, avec partitions, rétention, clé, débit et consommateurs critiques, et les cinq qu'on surveille de près
type: reference
status: active
verified: 2026-06-18
---

# Catalogue des topics principaux

Extrait de `topics.yaml` ([[topic-naming-and-ownership]]) pour les topics qui portent le métier ou le volume. Les 180 autres sont des `ops.*`, des `ml.*` de faible débit et des `cdc.app.*` de tables de référence. Débits mesurés sur la semaine du 2026-06-08, en messages par seconde au pic du matin.

## Capture de changements (`cdc.app.*`)

Produits par le connecteur [[cdc-tap-postgres-connector]], clé = clé primaire, compactés, un message par ligne modifiée avec `before` et `after`.

### Tables métier

| Topic | Partitions | Débit pic | Consommateurs critiques |
|---|---|---|---|
| `cdc.app.loads` | 48 | 900 | `ingest-svc`, `search-indexer`, `eta-projector` |
| `cdc.app.bids` | 48 | 1 400 | `ingest-svc`, `pricing-projector` |
| `cdc.app.load_status_history` | 48 | 1 100 | `ingest-svc`, `tracking-projector` |
| `cdc.app.invoices` | 24 | 90 | `ingest-svc`, `billing-projector` |
| `cdc.app.carriers` | 12 | 15 | `ingest-svc`, `search-indexer` |
| `cdc.app.users` | 12 | 40 | `ingest-svc` (colonnes filtrées) |
| `cdc.app.documents` | 24 | 200 | `ingest-svc`, `ocr-dispatcher` |

`cdc.app.users` est le seul topic dont le connecteur filtre des colonnes (pas de `phone`, pas de `email` en clair, un hash) : le topic est lu par l'entrepôt, qui n'a pas besoin de ces colonnes, et une copie de 380 000 adresses dans un journal compacté est une fuite qui attend.

## Événements métier (`domain.*`)

Produits par l'API ou l'outil dispatch dans la transaction métier (table `outbox` puis relais), clé = identifiant de l'agrégat, 30 jours, `delete`.

### Événements

| Topic | Partitions | Débit pic | Consommateurs critiques |
|---|---|---|---|
| `domain.load.published` | 24 | 60 | `notify-fanout`, `matching-svc` |
| `domain.bid.placed` | 24 | 400 | `notify-fanout`, `pricing-projector`, `ingest-svc` |
| `domain.bid.accepted` | 24 | 50 | `notify-fanout`, `billing-svc`, `dispatch-projector` |
| `domain.load.assigned` | 24 | 70 | `notify-fanout`, `eta-projector`, `driver-sync` |
| `domain.load.status-changed` | 48 | 1 100 | `notify-fanout`, `tracking-projector`, `ingest-svc` |
| `domain.load.cancelled` | 12 | 8 | `notify-fanout`, `billing-svc` |
| `domain.document.available` | 24 | 200 | `notify-fanout`, `ocr-dispatcher` |

La différence entre `cdc.app.load_status_history` et `domain.load.status-changed` : le premier est ce que la base a enregistré (avec les corrections, les rejeux, les modifications par le support), le second est ce que le métier a voulu dire au moment où il l'a dit. L'entrepôt lit les deux et les compare, c'est une des règles de qualité des données.

## Facturation (`billing.*`)

400 jours, clé = identifiant de règlement ou de facture, propriétaire équipe facturation.

| Topic | Partitions | Débit pic | Note |
|---|---|---|---|
| `billing.payla.settlements` | 12 | 5 | webhooks Payla normalisés, source de la réconciliation |
| `billing.invoice.lifecycle` | 24 | 40 | `issued`, `sent`, `paid`, `overdue`, `credited` |
| `billing.payout.requested` | 12 | 20 | un par virement transporteur |

## Chauffeurs (`driver.*`)

7 jours, clé = `driver_id`, le gros du volume.

| Topic | Partitions | Débit pic | Note |
|---|---|---|---|
| `driver.positions` | 96 | 9 000 | une position toutes les 10 s par camion en mouvement |
| `driver.state-reported` | 48 | 600 | tachygraphe, temps de conduite restant, la source de l'incident ETA de février 2026 |
| `driver.app-events` | 24 | 800 | écrans vus, erreurs, versions |

`driver.positions` à 96 partitions est le seul topic dont le nombre a été augmenté deux fois ([[partition-count-decisions]]). Il pèse 1,9 des 4,1 TB du cluster à lui seul.

## ML (`ml.*`)

| Topic | Partitions | Débit pic | Note |
|---|---|---|---|
| `ml.eta.predictions` | 24 | 300 | une prédiction par tick de chargement en cours |
| `ml.features.driver-10min` | 48 | 200 | mises à jour du store en ligne |
| `ml.price.suggestions` | 12 | 30 | |

## Les cinq qu'on regarde tous les matins

1. `cdc.app.loads` : si le connecteur a un problème, c'est là qu'on le voit, et la moitié des projections en dépendent.

2. `domain.load.status-changed` : le plus consommé, 6 groupes, le retard de `notify-fanout` dessus est un retard de notification visible par les clients.

3. `driver.positions` : le volume, le disque ([[incident-2026-05-torrent-3-disk-full]]), le retard d'`eta-projector`.

4. `billing.payla.settlements` : faible débit, mais un message perdu est un règlement non réconcilié, et c'est le seul topic dont le retard a un seuil en messages (10) plutôt qu'en secondes ([[consumer-lag-alerting]]).

5. `dlq.*` dans son ensemble : un message qui arrive dans une file de rebut est un bug quelque part ([[dead-letter-topics-convention]]).

## Ce qui n'est pas dans le catalogue et devrait peut-être l'être

`ops.audit.auth-events` (copie quotidienne des événements d'authentification vers l'entrepôt) passe par un fichier et non par torrent, parce que la table contient des IP et que personne n'a voulu d'un topic avec des IP dessus, même 7 jours. La décision tient, mais elle est notée ici parce qu'on la redemande tous les six mois.
