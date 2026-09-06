---
name: eta-feed-publication
description: ETAs are recomputed per kept position but published to shippers only on a 5 min change, a slot boundary crossing or 30 min silence; webhooks down 96 %
type: project
status: active
verified: 2026-04-28
---

# Publication des ETA aux chargeurs

Le calcul du modèle d'ETA appartient à la plateforme data (leurs notes décrivent le modèle). Ici : quand on recalcule, où on écrit, et surtout quand on prévient le chargeur, parce que prévenir trop souvent a été le problème.

## Déclenchement

`eta-consumer` lit le topic `positions` ([[position-ingestion-pipeline]]), un message par position conservée, clé `vehicle_id`. Pour chaque message :

1. Trouver la mission active du véhicule (cache des fenêtres, même index que l'étape de filtrage).

2. Si la dernière ETA calculée pour cette mission date de moins de **60 s**, ignorer. Un camion qui envoie une position toutes les 10 s n'a pas besoin de six ETA par minute.

3. Sinon, appeler le service de routage (`routing.hf.internal`, temps de parcours routier camion vers le prochain point de la mission), appliquer la correction du modèle (service d'inférence de la plateforme data, timeout 300 ms, repli sur le routage seul si dépassé), écrire dans `assignment_eta (assignment_id, stop_id, computed_at, eta, confidence_min, source_position_id)`.

Volume : 9,2 millions de positions par jour donnent environ 900 000 calculs d'ETA, soit une dizaine par seconde en journée.

## Quand on publie

C'est la partie qui a demandé trois itérations. Le chargeur voit l'ETA sur sa page de suivi et peut recevoir un webhook `load.eta_updated`. Au début (HF-2050, novembre 2025) on publiait chaque recalcul : 900 000 webhooks par jour, des ETA qui oscillaient de 3 minutes toutes les minutes, et deux chargeurs qui ont désactivé le webhook parce qu'il saturait leur TMS.

Règle actuelle (HF-2131, mars 2026), dans `EtaPublicationPolicy` :

- On publie si `|nouvelle ETA - dernière ETA publiée| >= 5 min`, **ou**

- si l'ETA franchit une borne du créneau promis (passe de « dans le créneau » à « en retard », ou l'inverse), quelle que soit l'amplitude, **ou**

- si la dernière publication date de plus de 30 minutes et que la mission est en cours (pour que la page de suivi ne montre pas une ETA de deux heures sans confirmation).

Résultat : 38 000 webhooks `load.eta_updated` par jour en avril 2026, 96 % de moins, et les deux chargeurs ont réactivé le webhook.

La page de suivi web, elle, reçoit toutes les ETA calculées par WebSocket (pas de coût pour le chargeur, et le dispatcher qui regarde veut la valeur fraîche), mais affiche l'heure arrondie à 5 minutes et un indicateur de confiance plutôt que « 14:37 » puis « 14:34 ». Retour des dispatchers là-dessus dans [[dispatchers-feedback-position-age]].

## Ce que contient la publication

```json
{
  "event": "load.eta_updated",
  "load_id": "…",
  "stop_id": "…",
  "eta": "2026-04-28T14:35:00Z",
  "eta_window": {"from": "2026-04-28T14:20:00Z", "to": "2026-04-28T14:50:00Z"},
  "status": "late",
  "position_age_s": 42,
  "computed_at": "2026-04-28T13:10:12Z"
}
```

Pas de position dans le webhook : le chargeur qui veut la position l'a sur la page de suivi pendant la mission, pas dans un flux qu'il stockerait. `position_age_s` dit depuis combien de temps on n'a pas de nouvelle du camion ; au-delà de 15 minutes, `status` passe à `stale` et on le publie une fois.

## Sans position

Pas de position depuis 15 minutes (tunnel, zone blanche, boîtier éteint) : l'ETA n'est pas recalculée, `status = 'stale'`, et le dispatcher du transporteur reçoit une notification au bout de 30 minutes. Chauffeur en opposition au suivi (voir le projet conformité) : pas de position du tout, l'ETA est celle du créneau promis, `status = 'manual'`, et ce sont les boutons de statut du chauffeur qui font avancer la mission.

## Ce qu'on a écarté

- Publier l'ETA à un rythme fixe (toutes les 10 minutes). Simple, mais un retard de 40 minutes détecté à 14:01 aurait attendu 14:10 ; la règle du franchissement de créneau publie immédiatement.

- Laisser le chargeur configurer son seuil. Deux l'ont demandé, on a préféré une règle unique et lisible ; on rouvrira si dix le demandent.
