---
name: timezone-convention
description: All timestamps stored as DateTime64 UTC, local time derived from entity or location columns, daily marts in Europe/Paris
type: reference
status: active
verified: 2026-01-08
---

## Stockage

Toute colonne d'horodatage est `DateTime64(3, 'UTC')`. Pas de `DateTime` sans fuseau (ClickHouse l'interpréterait dans le fuseau du serveur, qui est UTC chez nous, mais un jour quelqu'un lance un client avec un autre fuseau), pas de chaîne. L'ingestion convertit ce qui arrive : les événements produit sont déjà en UTC par convention, le CDC porte des `timestamptz` que Postgres émet en UTC, et les fichiers Payla ont des dates locales Europe/Paris que l'importateur convertit avant de publier.

Les colonnes de date pure (`invoice_date`, `delivery_date`) sont des `Date` et représentent la date légale ou métier dans le fuseau de l'entité ou du lieu, calculée par l'app, pas par l'entrepôt.

## Fuseau local à la requête

Les faits portent le fuseau qui compte pour eux :

- `core.loads` : `origin_tz` et `destination_tz` (IANA, depuis les coordonnées via le dictionnaire `tz_by_nuts3`), pour les créneaux de chargement et de livraison.
- `core.invoices` : `entity_tz`, celui de l'entité facturante, qui est la référence légale (l'équipe billing a documenté l'incident de la nuit du nouvel an polonais de son côté).
- `core.driver_positions` : `position_tz` par point, calculé à l'ingestion.

Conversion à la lecture : `toDateTime(created_at, origin_tz)` ou `toStartOfDay(created_at, entity_tz)`. Le fuseau doit être une constante dans certaines fonctions ; quand il vient d'une colonne, utiliser `toTimeZone(created_at, origin_tz)`, qui accepte une colonne depuis la 23.x.

## Marts quotidiens

Deux familles, et le nom le dit :

- `marts.*_daily` : jour calendaire **Europe/Paris**, la vue « entreprise » qu'utilisent les tableaux de bord de direction. Un chargement posté à 00:30 à Varsovie le 2 est compté le 1 (23:30 à Paris). C'est un choix assumé : une seule journée pour toute l'entreprise.
- `marts.*_by_entity_day` : jour calendaire du fuseau de l'entité, pour la finance et le rapprochement avec les exports comptables ([[invoice-mart]]). Les totaux d'un mois y correspondent au cent près à ce que billing exporte ; ceux de `_daily` ne le peuvent pas.

Un analyste qui compare les deux et trouve un écart de quelques chargements en début et fin de mois n'a pas trouvé un bug.

## Pièges rencontrés

- `toStartOfWeek` commence le dimanche par défaut ; on veut le lundi : `toStartOfWeek(ts, 1)` ou `toMonday`. Un rapport hebdomadaire a été faux d'un jour pendant un trimestre en 2025.
- Le passage à l'heure d'été : un jour de 23 heures dans `_daily`. Les métriques « par heure » de ce jour-là ont un trou à 02:00 et c'est normal. Ne pas « corriger ».
- Les événements de l'app conducteur portent `occurred_at` pris sur l'horloge du téléphone. 0,2 % ont une dérive de plus de 5 minutes, et quelques-uns arrivent avec des dates de 1970 ou 2038. L'ingestion les garde tels quels dans `raw`, et [[loads-fact-model]] borne les transitions à ±48 h de `received_at` avec un drapeau `clock_suspect`.
