---
name: retention-by-data-class
description: Cinq classes de données (documents légaux, données métier, télémétrie, sauvegardes, caches) avec une durée de conservation, un mécanisme d'effacement et un propriétaire chacune, 10 ans pour les CMR et POD, 5 ans pour les factures, 400 jours pour le brut de l'entrepôt, et la conservation légale qui bloque l'effacement
type: reference
status: active
verified: 2026-04-15
---

## Les classes

Toute donnée stockée appartient à une classe, et la classe fixe la durée, le mécanisme d'effacement et qui en répond. Le tableau est celui de `halden-infra/storage/RETENTION.md`, relu avec le juridique une fois par an (dernière relecture janvier 2026).

### Tableau des classes

| Classe | Exemples | Durée | Mécanisme | Propriétaire |
|---|---|---|---|---|
| documents à valeur légale | CMR, POD, lettres de voiture, factures PDF | 10 ans après la date du document (5 pour les factures, 10 retenu pour tout par simplicité) | `retain_until` sur `documents`, règle de cycle de vie du bucket, verrou de conservation | juridique |
| données métier | chargements, enchères, transporteurs, utilisateurs | durée de la relation + 5 ans, effacement individuel sur demande | purge applicative, pipeline d'effacement RGPD de l'entrepôt | produit |
| télémétrie | positions, événements d'app, traces, journaux | 7 jours (traces), 30 jours (journaux), 90 jours (positions dans `raw`), 400 jours (`raw` agrégé) | TTL ClickHouse, rétention Tempo et Loki, rétention des topics | données |
| sauvegardes | WAL, bases, instantanés, Velero | 35 jours (PITR), 30 jours (volumes), 14 jours (objets), 90 jours (vault), 52 semaines (hors site) | rétention de l'outil de sauvegarde, cycle de vie du bucket | ops |
| caches et dérivés | tuiles, exports clients, index de recherche | 7 jours (exports), reconstruit (tuiles, index) | cycle de vie, reconstruction | plateforme |

## Les documents

`documents.retain_until` est posé à la création à partir du type de document et de sa date (`document_type_retention.yaml` : `cmr: 10y`, `pod: 10y`, `invoice_pdf: 10y`, `carrier_license: 2y après expiration`, `driver_id_scan: 30 jours après vérification Verifid`, et ainsi de suite). La règle de cycle de vie du bucket ([[bucket-versioning-and-lifecycle-rules]]) ne connaît pas `retain_until` ; c'est la commande nocturne `documents:purge` qui lit les lignes dont `retain_until < now()` et sans verrou, supprime l'objet (avec la seule clé qui a le droit de supprimer), puis la ligne. Une trentaine de documents par nuit, surtout des scans d'identité.

Le verrou de conservation (`documents.legal_hold = true`) bloque la purge quel que soit `retain_until`. Il est posé par le juridique pour un litige ou une demande d'une autorité, sur un chargement ou un transporteur entier, et levé par lui. 1 400 documents sous verrou en avril 2026, pour 6 dossiers. Le verrou est aussi posé sur l'objet côté appliance (mode conformité), ce qui fait que même la clé de suppression ne peut pas le retirer ; c'est la ceinture, `legal_hold` est les bretelles.

## Les demandes d'effacement

Un utilisateur qui demande l'effacement de ses données obtient : la pseudonymisation dans PostgreSQL (le pipeline de la plateforme), l'effacement dans l'entrepôt (le pipeline RGPD du côté données), la suppression des scans d'identité, la suppression dans les notifications. Il n'obtient pas l'effacement des CMR et des factures où son nom apparaît comme conducteur ou signataire : la classe légale prime, et c'est écrit dans la réponse type. Les sauvegardes ne sont pas retouchées ; leur rétention de 35 jours fait que la donnée en disparaît d'elle-même, et la restauration d'une sauvegarde de moins de 35 jours rejoue la liste des effacements en attente (`privacy_erasure_requests`, elle-même conservée 3 ans comme preuve) avant d'ouvrir l'accès. C'est une étape du runbook de restauration, vérifiée à l'exercice de mai ([[restore-drill-2026-05]]).

## La télémétrie

`raw.driver_positions` à 90 jours dans l'entrepôt est le compromis entre l'équipe ML (qui voulait 180 pour les backtests) et la classe télémétrie (qui dit que la position d'un conducteur nommé est une donnée personnelle). 90 jours en brut, puis un agrégat par lane et par heure sans identifiant de conducteur pour 400 jours. Le topic sur le bus est à 7 jours.

## Les sauvegardes

35 jours de PITR parce qu'une erreur découverte à la clôture mensuelle (J+30 au pire) doit encore être corrigeable, et [[restore-2026-02-postgres-pitr-billing]] a validé le raisonnement à J+0. 52 semaines hors site parce que le contrat ([[offsite-weekly-copy-contract]]) est annuel et que l'assurance demandait « au moins un an ».

## Ce qui n'a pas de classe

`hf-ops-misc` ([[object-store-buckets-and-layout]]) est par définition le bucket de ce qui n'a pas trouvé sa classe. Sa revue trimestrielle consiste à classer ou supprimer chaque préfixe. En avril 2026 : 9 préfixes, 4 supprimés, 3 déplacés vers un bucket de classe, 2 gardés avec une date de revue.
