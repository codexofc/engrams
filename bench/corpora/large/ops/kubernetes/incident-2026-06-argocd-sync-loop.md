---
name: incident-2026-06-argocd-sync-loop
description: June 2026, platform-prod-web synced 1 400 times in 2 hours because a webhook-injected annotation and self-heal fought each other, no user impact but ArgoCD was unusable, fixed with ignoreDifferences and a sync rate alert
type: project
status: active
verified: 2026-06-12
---

# Incident 2026-06-03 : boucle de synchronisation ArgoCD

## Ce qui s'est passé

Le 3 juin entre 14 h 10 et 16 h 05, l'Application `platform-prod-web` s'est synchronisée 1 400 fois. Pas d'impact utilisateur : le Deployment du front n'a pas redémarré (rien ne changeait dans le pod spec), mais l'interface ArgoCD était inutilisable (l'historique des syncs mangeait toute la page, le contrôleur à 100 % CPU) et une promotion prévue de l'API à 15 h a été repoussée parce que personne ne voulait cliquer sur "Sync" dans cet état.

## Cause

Un contrôleur d'admission de politique (Kyverno) avait reçu le 3 juin à 14 h 05 une nouvelle règle `mutate` qui ajoute une annotation `hf.example/policy-version: "12"` sur tout Deployment. ArgoCD, en self-heal, a vu une différence entre Git (pas d'annotation) et le cluster (annotation), a resynchronisé, Kyverno a remis l'annotation, et ainsi de suite. Une boucle de 5 secondes.

Ça n'a touché que `platform-prod-web` et `platform-prod-live-gw`, les deux Applications de prod avec self-heal, et pas l'API (sync manuelle). Le staging a bouclé aussi, mais personne ne regarde le staging à 14 h un mercredi.

La règle Kyverno avait été testée en staging en mode `audit`, qui ne mute rien. Le passage en `enforce` s'est fait directement en prod. C'est la faute de procédure.

## Détection

Aucune alerte. L'ops qui voulait promouvoir l'API à 15 h a ouvert l'interface. Une heure sans que personne ne voie un contrôleur qui tourne en boucle, parce que le CPU d'`ops-tools` n'a pas d'alerte fine et que le nombre de syncs n'était pas une métrique surveillée.

## Correctifs

1. Immédiat (14 h 55) : self-heal désactivé sur les deux Applications. La boucle s'arrête. Règle Kyverno passée en `audit`. Self-heal réactivé à 16 h 05 après le point 2.

2. `ignoreDifferences` sur les Applications concernées pour l'annotation en question, et plus généralement pour `metadata.annotations` avec le préfixe des annotations posées par les contrôleurs d'admission :
   ```yaml
   ignoreDifferences:
     - group: apps
       kind: Deployment
       jqPathExpressions:
         - '.metadata.annotations | with_entries(select(.key | startswith("hf.example/policy-")))'
   ```
   Appliqué à toutes les Applications via le template de l'app-of-apps, voir [[argocd-app-of-apps]].

3. Alerte `ArgoCDSyncRateHigh` : `increase(argocd_app_sync_total[10m]) > 20` par Application, en `page`. Vingt syncs en dix minutes n'est jamais normal.

4. Procédure : une règle `mutate` de Kyverno passe par `audit` en staging, puis `enforce` en staging pendant au moins un jour avec ArgoCD en self-heal, puis prod. Écrit dans le README de `components/kyverno`.

5. La règle elle-même : l'annotation de version de politique est maintenant posée par Kustomize dans les manifestes (donc présente dans Git) et Kyverno ne fait que la vérifier. Muter ce qu'ArgoCD gère est une mauvaise idée en général.

## Ce qu'on a vérifié après

- Que l'historique de 1 400 syncs ne remplissait pas Redis d'ArgoCD au-delà de sa mémoire : 60 Mo, acceptable, purgé par la rétention par défaut de l'historique (10 par Application).

- Que l'API n'avait rien reçu d'inhabituel : les syncs du front ne touchent pas l'API.

## Leçon

Deux contrôleurs qui écrivent le même champ d'un même objet et qui ont chacun raison, c'est une boucle. La question à poser avant d'ajouter une mutation : est-ce qu'ArgoCD gère cet objet, et si oui, est-ce que Git pourrait porter cette valeur à la place ?
