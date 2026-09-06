---
name: playbook-load-stuck-dispatched-no-pickup
description: Chargement bloqué en DISPATCHED sans pickup : chauffeur assigné, appareil, synchro, verrouillage PIN, visibilité, jamais de transition par le support
type: reference
status: active
verified: 2026-06-30
---

# Chargement en `DISPATCHED`, le pickup n'arrive pas

Catégorie `load:stuck`. Le cas le plus fréquent du support (un ticket sur cinq). Le chargeur voit « transporteur assigné » depuis des heures, le camion est peut-être déjà parti, mais l'état ne bouge pas parce que personne n'a appuyé sur « Chargé » dans l'app.

## Vérifications, dans l'ordre

1. **L'état réel.** `hfctl load get <load_id>`. Confirmer `status = DISPATCHED`, noter `carrier_id`, `driver_id`, `pickup_window_start`. Si `driver_id` est vide : le transporteur a accepté sans assigner de chauffeur. Sortie : macro `carrier-assign-driver` au transporteur, pas au chargeur. Résolu côté support.

2. **Le chauffeur a-t-il l'app ?** `hfctl driver devices <driver_id>`. Aucun appareil : le chauffeur n'a jamais installé l'app ou ne s'est jamais connecté. Sortie : macro `driver-install-app` au transporteur avec le lien d'installation. C'est le cas dans environ 40 % des tickets de cette catégorie.

3. **L'app se synchronise-t-elle ?** `hfctl driver sync-status <driver_id>`. Regarder `last_push_at` et `outbox_depth`. Un `last_push_at` de plus de 2 h avec `outbox_depth > 0` : le chauffeur a peut-être appuyé, l'événement est dans sa file locale et n'est pas remonté. Passer au playbook [[playbook-driver-app-not-syncing]] sans fermer celui-ci.

4. **Le chauffeur est-il verrouillé ?** `hfctl driver get <driver_id>`, champ `pin_attempts`. À 5, l'app a effacé ses jetons et le chauffeur doit se reconnecter en ligne. Passer à [[playbook-driver-cannot-log-in]].

5. **Le chargement est-il visible pour lui ?** Dans `hfctl load get`, `driver_visible_at` doit être renseigné. S'il est vide alors que `driver_id` est posé, c'est un bug connu (HF-3162, corrigé en 4.9) quand l'assignation s'est faite pendant que l'app était ouverte sur l'écran de la liste. Sortie : demander au chauffeur de fermer et rouvrir l'app, sinon escalade L2 avec le `load_id`.

## Ce qu'on ne fait pas

Le support ne déclenche **jamais** la transition `pickup`. Elle appartient au chauffeur et elle horodate le début de la responsabilité du transporteur sur la marchandise. Une transition faite par nous fausserait la chronologie et le calcul de ponctualité. Si le chargement est déjà parti physiquement, c'est au transporteur de faire faire le pickup par le chauffeur, en retard, l'horodatage sera en retard, c'est la réalité.

## Si le transporteur ne répond pas

Fenêtre de pickup dépassée de plus de 4 h, transporteur injoignable : le chargeur peut demander le retrait. Deux issues :

- Le chargeur retire lui-même le transporteur depuis son écran (bouton « Retirer le transporteur », disponible tant que le chargement est `DISPATCHED`). Le chargement revient en `OPEN` ou `BIDDING`. Macro `shipper-withdraw-carrier`.

- Le chargeur veut annuler : il a le bouton aussi. Voir [[playbook-shipper-cannot-cancel-load]] si le bouton refuse.

Dans les deux cas, le retrait pénalise la note du transporteur. Si le transporteur conteste ensuite, voir [[playbook-carrier-rating-complaint]].

## Escalade

L2 si l'étape 5 échoue ou si le chargeur est Enterprise et que la fenêtre de pickup est dans moins d'une heure. Le message d'escalade contient `load_id`, `driver_id`, la sortie de `sync-status`.

## Pourquoi c'est si fréquent

Le pickup est le premier geste demandé au chauffeur dans l'app, souvent le jour même de son premier chargement chez nous. Le rappel SMS J-1 (HF-3160) devrait faire baisser le volume, on regardera au tri.
