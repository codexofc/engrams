---
name: caught-2026-01-null-carrier-ids
description: Janvier 2026: loads_assigned_have_carrier a appelé sur 38 chargements assignés sans transporteur, événements inversés dans la projection, 50 min, HF-4502
type: project
status: active
verified: 2026-02-03
---

# Attrapé : chargements assignés sans transporteur, 2026-01-18

## Ce que la règle a vu

Le 2026-01-18 à 09:12, `loads_assigned_have_carrier` ([[dq-rule-catalog-core]]) passe de `pass` à `fail` : 38 lignes de `core.loads` avec `status = 'assigned'` et `carrier_id NULL`. Tolérance 0, sévérité `page`, l'astreinte dispatch est appelée. Le message d'alerte contient les 20 premiers `load_id` (`sample_keys`), ce qui a permis de regarder un cas concret en deux minutes plutôt que d'écrire une requête.

## Ce qui s'était passé

À 04:00 le même matin, le nombre de partitions de `driver.positions` avait été porté de 48 à 96 sur le bus (l'opération planifiée du côté streaming). Sans rapport direct avec les chargements. Sauf que le projecteur `loads-projector`, qui construit `core.loads` à partir de `cdc.app.loads` **et** de `domain.load.assigned`, avait été redémarré dans la foulée comme tous les consommateurs, et qu'il a redémarré avec un léger retard sur `cdc.app.loads` (12 s) et aucun sur `domain.load.assigned`.

Pour 38 chargements assignés entre 04:00:10 et 04:00:22, le projecteur a donc vu l'événement `domain.load.assigned` (qui porte `carrier_id`) avant la ligne CDC correspondante (qui porte le nouveau `status`). Sa logique : sur l'événement métier, il pose `carrier_id` ; sur la ligne CDC, il fait un `UPSERT` de toute la ligne, `carrier_id` compris, tel que la source PostgreSQL l'a… et la source à l'instant capturé avait `carrier_id` déjà rempli. Donc ça aurait dû aller. Sauf que le projecteur, pour les colonnes présentes dans les deux flux, appliquait « le dernier message gagne » sans regarder la version de l'agrégat, et l'événement CDC lu en second était en réalité une version **antérieure** (la transition `published → assigned` en deux mises à jour PostgreSQL séparées de quelques millisecondes, la première posant le statut, la seconde le transporteur ; la ligne CDC de la première est arrivée après l'événement métier de la seconde). `status = assigned` et `carrier_id = NULL`, pendant 12 secondes dans la source, figé dans la projection.

Le cas existe en permanence à très petite échelle (deux lignes par jour en moyenne), et le projecteur le rattrapait à la ligne CDC suivante du même chargement. Ces 38 n'ont pas eu de ligne suivante avant l'alerte parce que 04:00 un dimanche est calme.

## Ce qui a été fait

- 09:20 : cause comprise à partir des `sample_keys`, en comparant `core.loads` à `raw.cdc_app_loads` pour un identifiant (la table brute garde toutes les versions).

- 09:40 : correction du projecteur pour utiliser `version` (la version de l'agrégat, présente dans les deux flux) comme garde : un message dont la version est inférieure à celle déjà projetée ne touche pas la ligne. C'était la règle écrite dans la revue des consommateurs côté bus, et ce projecteur datait d'avant.

- 10:00 : rejeu de `cdc.app.loads` pour les 38 identifiants via un topic de rejeu (la procédure du côté streaming), 3 minutes. La règle repasse `pass` à 10:02.

## Ce que ça aurait coûté sans la règle

Les 38 chargements auraient été absents du tableau « chargements assignés par transporteur » du lundi matin et présents dans « chargements assignés sans transporteur » (qui n'existe pas, justement). Le calcul de la ponctualité par transporteur du lundi aurait ignoré 38 chargements sur 4 100 assignés le week-end, invisible. La prochaine ligne CDC de chacun (au chargement du camion, le lundi) aurait tout corrigé sans que personne le sache. Un bug de projection réel, silencieux, corrigé par hasard : c'est exactement la catégorie qu'une règle `not_null` conditionnelle attrape et qu'un compte de lignes ([[legacy-row-count-checks]]) ne voit jamais.

## Ce que ça a changé

- La garde par `version` est passée dans le wrapper commun des projecteurs et les six autres projecteurs qui lisent deux flux pour la même table ont été relus. Deux avaient le même défaut.

- La règle a gagné une sœur en `info`, `loads_assigned_have_carrier_transient`, tolérance 5, sans alerte, pour voir le bruit de fond des deux lignes par jour et vérifier qu'il tombe à zéro après la correction. Il est tombé à zéro.

- La revue mensuelle ([[dq-rules-review-feedback]]) a noté que le `sample_keys` a fait gagner une demi-heure et a demandé qu'il soit systématique sur les règles `page`. Il l'est depuis février.

## Chiffres

| | Valeur |
|---|---|
| lignes fausses | 38 |
| durée dans la projection | 5 h 12 (04:00 à 10:02) |
| délai règle → correction | 50 min |
| tableaux de bord affectés | 0 (aucun n'a été rafraîchi entre 04:00 et 10:02 un dimanche) |
| projecteurs corrigés | 3 |
