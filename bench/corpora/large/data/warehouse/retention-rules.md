---
name: retention-rules
description: Retention per layer (raw 400 days, core 5 years, scratch 30 days), pseudonymisation at 24 months, legal holds and their basis
type: reference
status: active
verified: 2026-02-20
---

## Par couche

| Couche | Rétention | Mécanisme | Base |
|---|---|---|---|
| `raw.*` | 400 jours | TTL ClickHouse sur `received_at` ([[partitioning-and-ttl]]) | assez pour rejouer un an de modèles plus une marge ; au-delà, `core` fait foi |
| `core.*` faits | 5 ans | TTL sur la colonne de partition | délai de prescription commerciale et contrôle fiscal (les factures elles-mêmes sont conservées 10 ans côté billing, l'entrepôt n'est pas la copie légale) |
| `core.*` dimensions | illimitée | aucune | petites tables |
| `marts.*` agrégés par jour | illimitée | aucune | pas de donnée personnelle, volume négligeable (8 GB au total) |
| `marts.*` par ligne (`invoice_mart`, `pricing_experiments`) | 5 ans | TTL | comme `core` |
| `scratch.*` | 30 jours sans requête | job `DropStaleScratch` du dimanche, qui lit `system.query_log` | éviter l'accumulation, 1,2 TB de tables oubliées en 2025 |
| `ml.predictions.*` dans `raw` | 180 jours | TTL | évaluation glissante des modèles, l'équipe ML garde ses propres archives |

## Données personnelles

L'entrepôt contient des données personnelles dans trois endroits : `core.load_addresses` (adresses de livraison, dont des artisans et particuliers destinataires), `core.driver_positions` (positions GPS des conducteurs pendant un transport), `core.carrier_contacts` (noms et téléphones des contacts transporteurs).

- **Pseudonymisation à 24 mois** : le job `PseudonymiseOld` remplace, dans les partitions de plus de 24 mois, les noms par un hachage salé et supprime les téléphones et emails ; les adresses sont réduites au code postal. Les positions GPS de plus de 24 mois sont agrégées à la maille de 1 km et l'identifiant conducteur supprimé. Base : finalité analytique qui ne nécessite plus l'identification au-delà de deux ans, avis du délégué à la protection des données de novembre 2025.
- **Effacement sur demande** : [[gdpr-erasure-pipeline]].
- **Accès** : `core.load_addresses`, `core.driver_positions` et `core.carrier_contacts` sont hors du rôle `analyst` ; le rôle `analyst_pii` est attribué nominativement, revu chaque trimestre, 6 personnes en février 2026.

## Ce qui n'est pas dans l'entrepôt du tout

- Les documents des transporteurs (licences, assurances, pièces d'identité) : jamais, ni en contenu ni en OCR. Seuls le type, le statut et les dates de vérification sont chargés.
- Les IBAN : le CDC de `bank_accounts` n'est pas consommé. `core.carriers` a `has_bank_account` et `bank_country`, rien d'autre.
- Les mots de passe, jetons, clés API : les tables correspondantes ne sont pas dans la liste CDC.

## Modifier une règle

Une règle de rétention se change par une migration ([[schema-migration-process]]) et une ligne dans cette note, avec la base. Raccourcir une rétention est immédiat (le TTL fait le travail à la prochaine fusion) ; l'allonger ne récupère rien de ce qui est déjà parti, il faut le savoir avant de raccourcir.

## Sauvegardes et rétention

Les sauvegardes nocturnes ([[clickhouse-cluster-layout]]) sont conservées 35 jours. Une donnée supprimée par TTL ou par effacement reste donc récupérable 35 jours dans la sauvegarde ; c'est écrit dans la réponse aux demandes d'effacement et dans le registre des traitements. Il n'y a pas de sauvegarde annuelle ni d'archive longue durée de l'entrepôt : ce qui doit être conservé 10 ans (les factures) l'est dans le système de facturation, pas ici.

## Tables techniques

| Table | Rétention | Note |
|---|---|---|
| `raw._offsets` | illimitée | 12 partitions × 26 topics, quelques milliers de lignes |
| `raw._dead_letters` | 30 jours | quelques milliers par jour |
| `raw._skipped` | illimitée | rare, valeur de preuve |
| `marmot._runs` | 1 an | |
| `marmot._tests` | 90 jours | |
| `admin.freshness_probes` | 90 jours | une ligne par table et par minute |
| `system.query_log` | 30 jours | sert au nettoyage de `scratch` et à la refacturation |

## Ce que « 5 ans » veut dire pour une partition mensuelle

Le TTL `DELETE` sur `core.loads` est `posted_at + INTERVAL 5 YEAR`. Une partition mensuelle contient des lignes dont le TTL est atteint sur tout le mois ; ClickHouse supprime les lignes à la fusion, pas la partition d'un coup, et une partition dont toutes les lignes ont expiré est retirée par `ttl_only_drop_parts = 1`, réglé sur les tables de faits pour éviter une fusion de 60 Go qui n'écrit rien. Les premières partitions à expirer datent de 2021 et arriveront en 2026 ; on regardera si le réglage se comporte comme prévu, c'est la première fois.

## Demandes de conservation étendue

Trois cas où une donnée est conservée au-delà de la règle :

1. Chargements liés à un dossier de fraude : positions et événements conservés 5 ans sans pseudonymisation, marqués par `legal_hold = 'fraud_case'` dans `core.loads`, colonne lue par `PseudonymiseOld` pour sauter la ligne. 1 chargement en 2025.
2. Litige commercial en cours : le service juridique pose un `legal_hold = 'dispute'` par une ligne dans `privacy.legal_holds`, avec une date de revue ; 4 en cours.
3. Demande d'une autorité : jamais arrivé ; la procédure est la même que le litige.

Un `legal_hold` est levé par la même personne qui l'a posé ou par le délégué à la protection des données, et la levée est journalisée. Les lignes sous hold sont exclues de l'effacement ([[gdpr-erasure-pipeline]]), qui renvoie alors `partially_done_warehouse` avec la raison.

## Revue

Cette note est revue chaque année en novembre avec le délégué à la protection des données, et à chaque nouvelle source. La dernière revue a ajouté les positions GPS agrégées à 1 km et la règle des holds ; la prochaine devra traiter les prédictions ML, qui contiennent des identifiants conducteur pendant 180 jours et devraient peut-être être pseudonymisées plus tôt.
