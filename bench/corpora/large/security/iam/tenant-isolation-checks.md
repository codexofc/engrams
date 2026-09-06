---
name: tenant-isolation-checks
description: Every tenant table has organization_id, OrganizationScope filters queries, TenantIsolationTest covers 214 endpoints and a nightly probe checks production
type: reference
status: active
verified: 2026-06-24
---

# Isolation entre organisations

Le périmètre d'un utilisateur est son organisation. C'est la seule frontière de données qu'on garantit aux clients, et la note du modèle de rôles ([[roles-permissions-model]]) explique pourquoi on n'a pas d'ACL par objet. Cette note dit comment on s'assure que la frontière tient.

## Le schéma

Toute table qui contient des données appartenant à un client porte `organization_id uuid NOT NULL` avec une clé étrangère vers `organizations`. En juin 2026 : 47 tables. Les tables qui n'en ont pas et qui n'en auront jamais (référentiels géographiques, tarifs de péage, tables techniques) sont listées dans `config/tenancy/global_tables.yaml`. Une table absente des deux listes fait échouer `TenantSchemaTest`.

Pour les lignes qui appartiennent à deux organisations à la fois (une offre : le transporteur qui la fait, le chargeur qui la reçoit), la table porte les deux colonnes (`bids.carrier_organization_id`, `bids.shipper_organization_id`) et le filtre s'applique sur l'une ou l'autre selon l'audience. Pas de table de liaison « visible par ».

## Le filtre

`OrganizationScope` est un filtre Doctrine SQL activé pour toute requête faite dans un contexte utilisateur ou clé API. Il ajoute `organization_id = :current_org` (ou la variante à deux colonnes) à chaque `FROM` et `JOIN` d'une entité annotée `#[TenantOwned]`. Le contexte vient de `OrganizationContextResolver` (voir [[permission-check-voter-symfony]]).

Les requêtes sans filtre sont possibles, il en faut : facturation, plateforme data, support. Elles passent par `TenantScope::runUnscoped(callable)`, qui journalise la classe appelante. Le nombre de sites d'appel de `runUnscoped` est suivi : 31 en juin 2026, chacun avec un commentaire qui dit pourquoi. Un nouvel appel est refusé en revue s'il n'a pas le commentaire.

Le SQL natif ne passe pas par le filtre Doctrine. Les requêtes natives dans un contexte utilisateur doivent donc porter le filtre à la main, et c'est la classe de bug la plus fréquente qu'on ait eue (voir plus bas).

## Les tests

- **`TenantIsolationTest`** : pour chaque endpoint de l'API `/v2/*` qui retourne une collection ou un objet, le test crée deux organisations avec des données similaires, appelle l'endpoint avec un utilisateur de A, vérifie qu'aucun identifiant de B n'apparaît dans la réponse, puis appelle les endpoints d'objet avec l'identifiant d'une ressource de B et attend 404 (pas 403 : on ne confirme pas l'existence). 214 endpoints couverts, le test échoue si un nouvel endpoint n'est pas dans la liste ou explicitement exclu avec un motif.

- **Sonde nocturne** : `tenancy:probe` tourne à 03:00 sur la production avec deux organisations de sonde (`org-probe-a`, `org-probe-b`, marquées `is_probe = true`, exclues de la facturation et des statistiques), rejoue le même scénario contre l'API réelle. Un échec page la rota sécurité. Depuis sa mise en place en décembre 2025 : un déclenchement, un faux positif dû à une fixture cassée.

## Les bugs trouvés

### IDOR par SQL natif (HF-2021, octobre 2025)

`GET /v2/loads/{id}/documents` utilisait une requête native pour les métadonnées de documents et ne filtrait que sur `load_id`. Un identifiant de chargement d'une autre organisation (UUID v7, donc partiellement devinable dans le temps) donnait la liste des documents, sans le contenu (les URLs présignées, elles, passaient par le filtre). Trouvé par le pentest d'octobre 2025, corrigé en 2 jours, `TenantIsolationTest` créé à cette occasion.

### Recherche plein texte hors périmètre (HF-2067, décembre 2025)

L'index de recherche des chargements était alimenté par un `runUnscoped` légitime, mais la requête de recherche ne réappliquait pas `organization_id` quand le paramètre `q` était vide (chemin de code différent). Résultat : un chargeur qui vidait le champ de recherche voyait les 50 derniers chargements de toute la plateforme, sans détail mais avec les villes et les prix. Signalé par un client, 4 heures d'exposition après un déploiement. C'est l'incident qui a justifié la sonde nocturne.

## Ce qu'on ne fait pas

- Une base par client. Trois mille organisations, et la moitié des requêtes utiles croisent chargeurs et transporteurs.

- Le row-level security de Postgres. On l'a évalué (HF-2030) : il aurait doublé la protection, mais nos connexions passent par un pooler en mode transaction et le `SET LOCAL` par requête coûtait 8 % de latence en test. Décision : le filtre Doctrine plus les tests plus la sonde, et on rouvre la question si on change de pooler.

- Un `organization_id` dans le JWT comme unique source du contexte. Il y est, mais le résolveur revérifie l'adhésion en base à chaque requête, parce qu'un utilisateur retiré d'une organisation garde son jeton jusqu'à 15 minutes (voir [[session-revocation-on-role-change]]).

## Pour ajouter un endpoint sans casser la règle

1. L'entité a `organization_id` et `#[TenantOwned]`, ou elle est dans `global_tables.yaml` avec un motif.

2. Le repository passe par Doctrine (le filtre s'applique) ou, en SQL natif, porte `AND organization_id = :org` avec `:org` venant de `OrganizationContextResolver`, jamais du corps de la requête.

3. L'endpoint est ajouté à `TenantIsolationTest` avec les deux fixtures ; le test échoue tant qu'il n'y est pas.

4. Si l'endpoint retourne une collection, la pagination ne doit pas permettre de compter les lignes des autres (un `total` global serait une fuite ; le `total` est calculé sous le même filtre).

Quatre points, dix minutes, et c'est ce que le référent sécurité vérifie en revue avec la checklist du projet sécurité commun.

Le compteur qui résume l'état : `tenancy_unscoped_callsites` (31 en juin 2026), publié par un test qui compte les appels à `runUnscoped` dans le code, exposé sur le tableau de bord sécurité. Il ne doit monter qu'avec un commentaire justifié dans le diff, et il a baissé deux fois quand une requête de rapport a été réécrite avec le filtre.
