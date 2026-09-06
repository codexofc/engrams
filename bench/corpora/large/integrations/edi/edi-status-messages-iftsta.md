---
name: edi-status-messages-iftsta
description: Load events map to IFTSTA codes through a per-partner code set in edi_status_code_sets, sent within 2 min with the geofence timestamp; 6 statuses per load
type: reference
status: active
verified: 2026-05-13
---

# Statuts sortants IFTSTA

Le partenaire a créé un chargement par IFTMIN ; il veut savoir ce qui s'en passe sans ouvrir notre site. Chaque événement du chargement déclenche un IFTSTA. Le producteur est `LoadEventToIftstaHandler` (consommateur Messenger des événements de chargement, filtré sur les chargements dont l'organisation a un partenaire EDI actif).

## Les événements et leurs codes

Le standard donne une liste de codes de statut (élément `4405`), les partenaires en utilisent chacun un sous-ensemble avec leur propre sens. On ne discute plus : `edi_status_code_sets (partner_id, event, code_4405, code_list_1131, include)` dit pour chaque partenaire quel événement produit quel code, et si on l'envoie du tout.

### Jeu par défaut (celui qu'on propose à un nouveau partenaire)

| Événement plateforme | `4405` | Sens |
|---|---|---|
| `load.accepted` (IFTMIN accepté, chargement publié) | `1` | pris en charge dans le système |
| `bid.accepted` (transporteur attribué) | `21` | transporteur affecté, avec `NAD+CA` nom et code du transporteur |
| `stop.arrival_detected` (enlèvement) | `24` | arrivé au lieu de chargement |
| `stop.departed` (enlèvement) | `6` | chargé, parti |
| `stop.arrival_detected` (livraison) | `23` | arrivé au lieu de livraison |
| `pod.uploaded` | `7` | livré, avec `DTM+35` heure de livraison et `RFF+ABO` référence du POD |
| `load.eta_updated` (franchissement de créneau) | `20` | en transit, avec `DTM+132` heure d'arrivée estimée |
| `load.cancelled` | `4` | annulé |
| `assignment.delay_detected` | `17` | retard, avec `FTX+AAI` motif si connu |

Horodatage dans `DTM+334` (date de l'événement), en heure locale du **site** concerné (le partenaire veut lire l'heure du quai, pas UTC), format `203`. Quand l'événement vient du géorepérage (voir les notes télématiques), c'est l'heure détectée ; quand il vient du bouton chauffeur, c'est l'heure du bouton, et `FTX+AAO` dit lequel des deux pour les trois partenaires qui l'ont demandé.

## Les divergences

- **Nordkarton** refuse tout code qu'ils n'ont pas listé : `20` (ETA) et `17` (retard) leur font produire un CONTRL négatif. `include = false` pour ces deux lignes. Leur détail dans [[edi-partner-nordkarton-quirks]].

- **Vestaflor** veut l'ETA à chaque recalcul publié (pas seulement au franchissement de créneau), pour du frais avec des quais à réserver. Leur ligne `20` a `on_every_publication = true`, ce qui fait 25 IFTSTA par chargement au lieu de 6. Ils paient le volume dans leur TMS, pas nous.

- **Bruma Retail** utilise `4405 = 21` non pas pour l'affectation mais pour « en cours d'enlèvement » ; leur jeu décale les codes. C'est la raison d'être de la table : sans elle, on aurait des `if partner` partout.

- **Kalmarine** (D.01B) veut la liste de codes `1131 = 'ZZZ'` avec leurs propres codes maison (`LOADED`, `DELIVERED`). La colonne `code_list_1131` existe pour eux.

## Rythme et fiabilité

- Envoi dans les **2 minutes** de l'événement (consommateur avec un délai de 60 s pour regrouper : un `stop.departed` suivi immédiatement d'un `load.eta_updated` part en un seul IFTSTA avec deux groupes `CNI`). p95 mesuré : 90 s.

- Un IFTSTA par chargement et par événement, jamais de renvoi périodique ; le partenaire qui veut l'état complet demande sur le web ou par l'API JSON ([[edi-api-json-alternative]]).

- Transport et accusés : [[as2-transport-setup]]. Un IFTSTA sans CONTRL sous 24 h remonte dans la réconciliation ([[edi-reconciliation-daily]]).

- Volume : environ 900 IFTSTA par jour, 6 par chargement en moyenne (25 pour Vestaflor).

## Ce qu'on n'envoie pas

- La position du camion. Deux partenaires l'ont demandée dans un `LOC+ZZZ` avec coordonnées ; la réponse est la même que pour les webhooks : la position se regarde pendant la mission sur la page de suivi, elle ne se stocke pas chez le partenaire dans un flux. Le projet conformité a la note.

- Le nom du chauffeur. Le `NAD+CA` porte le transporteur, pas la personne.

- Le prix. Il est dans l'INVOIC ([[edi-invoice-invoic-outbound]]), pas dans les statuts.

Le retour des partenaires sur ces codes, et ce qu'ils ont fait changer, est dans [[edi-partner-feedback-status-codes]].
