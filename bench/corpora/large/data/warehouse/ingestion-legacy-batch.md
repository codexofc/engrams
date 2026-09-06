---
name: ingestion-legacy-batch
description: The hourly Python batch loader used until September 2025, 1 to 2 h latency and duplicates, retired by the Kafka consumer
type: reference
status: archived
superseded_by: [[ingestion-kafka-to-clickhouse]]
verified: 2025-09-20
---

Jusqu'en septembre 2025, l'entrepôt était alimenté par un script Python horaire (`wh_loader.py`) qui interrogeait les bases applicatives (`SELECT ... WHERE updated_at > :last_run`), écrivait des CSV et les insérait dans ClickHouse avec `clickhouse-client --query "INSERT ... FORMAT CSV"`.

Problèmes connus :

- Latence de 1 à 2 heures selon la charge, incompatible avec les tableaux de bord de dispatch.
- Les lignes modifiées deux fois dans l'heure n'apparaissaient qu'une fois (dernier état), donc pas d'historique des changements d'état d'une enchère.
- Les suppressions étaient invisibles : un chargement supprimé restait dans l'entrepôt pour toujours. On avait un script de réconciliation hebdomadaire qui comparait les identifiants et supprimait par `ALTER TABLE DELETE`, lourd et lent.
- `updated_at` non indexé sur deux tables, la requête horaire faisait un balayage complet et l'équipe applicative nous a demandé d'arrêter.
- Reprises manuelles après chaque panne, avec des doublons quand le script était relancé sur une heure déjà chargée (pas de jeton de déduplication).

Le consommateur Kafka décrit dans [[ingestion-kafka-to-clickhouse]] a réglé les cinq points. Les tables `raw.legacy_*` chargées par ce script ont été conservées jusqu'à la fin de la rétention de 400 jours et sont tombées en TTL en octobre 2026 au plus tard ; les modèles `core` ne les lisent plus depuis la bascule.
