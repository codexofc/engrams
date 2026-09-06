---
name: entity-naming-and-table-prefixes
description: Doctrine entities map to snake_case plural tables without prefix, UUID v7 for new tables since HF-1420, enums stored as text with a CHECK constraint, timestamps are timestamptz
type: reference
status: active
verified: 2026-05-02
---

# Conventions de nommage des entités et des tables

## Tables

- Nom de table = pluriel snake_case du nom de l'entité : `Load` → `loads`, `LoadEvent` → `load_events`, `CarrierDocument` → `carrier_documents`. Pas de préfixe `hf_`, on en avait un sur les 6 premières tables et on l'a retiré en 2024.

- Tables de jointure ManyToMany : les deux noms au singulier reliés par `_`, dans l'ordre alphabétique : `carrier_lane` et pas `lane_carrier`. En pratique on évite les ManyToMany Doctrine et on crée l'entité de jointure explicitement dès qu'elle a un attribut.

- Tables techniques (pas d'entité) : préfixe `sys_` : `sys_feature_flags`, `sys_outbox`. Elles n'apparaissent pas dans `doctrine:schema:validate` parce qu'elles sont déclarées dans `config/doctrine/schema_filter.yaml`.

## Clés primaires

- UUID partout. Type Doctrine `uuid`, colonne `uuid`.

- Depuis HF-1420 (février 2026), les nouvelles tables utilisent **UUID v7** (`Symfony\Component\Uid\UuidV7`) pour que l'ordre d'insertion se retrouve dans l'index. Les tables existantes restent en v4, on ne migre pas.

- Le générateur est `App\Doctrine\UuidV7Generator`, déclaré par `#[ORM\CustomIdGenerator]`. Ne pas utiliser `#[ORM\GeneratedValue(strategy: 'AUTO')]`.

## Colonnes

- snake_case, jamais de camelCase en base. La `NamingStrategy` est `UnderscoreNamingStrategy(CASE_LOWER, true)`.

- Timestamps : `created_at timestamptz NOT NULL`, `updated_at timestamptz NOT NULL`, gérés par `TimestampableListener`. Jamais `timestamp without time zone`, voir [[timezone-handling-utc-rule]].

- Booléens : préfixe `is_` ou `has_` en base (`is_hazardous`), propriété PHP sans préfixe (`hazardous`) avec un getter `isHazardous()`.

- Montants : `numeric(12,2)` en base, `Money` (classe maison `App\Money\Money`, entier de centimes + devise) en PHP. Jamais de `float`.

- Enums : colonne `text` + `CHECK (status IN (...))` généré par `App\Doctrine\EnumCheckConstraintListener` à partir du `BackedEnum`. Pas de type `ENUM` PostgreSQL : ajouter une valeur à un type enum PG demande un `ALTER TYPE` qui ne peut pas tourner dans une transaction sur les vieilles versions et ça nous a coûté une soirée.

- Clés étrangères : `<entity>_id`, index systématique (`idx_<table>_<column>`), `ON DELETE` explicite (`RESTRICT` par défaut, `CASCADE` uniquement sur les tables enfants pures comme `load_events`).

## Index

- Nom : `idx_<table>_<colonnes>` pour les index simples, `uniq_<table>_<colonnes>` pour les uniques, suffixe `_trgm` ou `_gin` pour les index spécialisés.

- Les index partiels portent le nom de leur condition résumée : `uniq_bids_accepted_per_load`, voir [[bid-acceptance-race-condition]].

- Index de pagination `idx_<table>_cursor`, voir [[api-pagination-cursor-convention]].

## Entités

- Une entité = un fichier sous `src/Entity/`, pas de sous-dossier par contexte. On a essayé, les `use` deviennent illisibles.

- Pas de setters génériques. Une entité expose des méthodes métier (`$load->dispatchTo($carrier)`), et la validation des transitions est dans l'entité, voir [[load-status-state-machine]].

- Les repositories sont explicites, voir [[team-prefers-explicit-repositories]].
