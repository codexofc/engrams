---
name: trakko-api-contract-quirks
description: Trakko's API returns a 5 min sliding window, device time not server time, cursors that expire in 60 s, 120 req/min per key and ignition as a string
type: reference
status: active
verified: 2026-05-22
---

# Trakko : ce que leur API fait vraiment

Trakko fournit les boîtiers qu'on installe chez les petits transporteurs. Leur documentation est correcte sur les champs et silencieuse sur le comportement. Cette note est ce qu'on a appris en dix-huit mois, tout est encodé dans `TrakkoAdapter` (dépôt `hf-telematics-gw`).

## Authentification et limites

- Une clé API par **flotte** Trakko, et une flotte Trakko correspond chez nous à un lot de boîtiers, pas à un transporteur : on a 14 flottes pour 780 boîtiers, réparties par date d'achat. La clé vit dans le vault sous `telematics/trakko/fleet-<n>`.

- **120 requêtes par minute par clé.** Au-delà, 429 sans `Retry-After`. Avec un appel toutes les 30 s par flotte on est à 2 par minute, très loin de la limite, mais le rattrapage après une panne (voir plus bas) peut la toucher. `TrakkoAdapter` a un limiteur local à 100 par minute par clé.

- La clé expire au bout d'un an **sans avertissement**. La première fois, en mars 2025, on l'a appris par les 401. Depuis, la date d'expiration est dans l'inventaire des secrets et la rotation est faite à 11 mois.

## L'endpoint de positions

`GET /v3/fleets/{fleet}/positions?since=<iso>&cursor=<c>` retourne les positions de tous les véhicules de la flotte depuis `since`, mais :

- **Fenêtre glissante de 5 minutes** : quelle que soit la valeur de `since`, la réponse contient au plus les 5 dernières minutes. Pour rattraper plus loin, il faut `GET /v3/fleets/{fleet}/vehicles/{v}/history?from=&to=`, un véhicule à la fois, limité à 24 h par appel. Le rattrapage après une heure de panne de notre côté pour 780 boîtiers, c'est 780 appels, donc 7 minutes au rythme du limiteur.

- **Le curseur expire en 60 secondes.** Si on met plus d'une minute entre deux pages (ça arrive sous charge), 400 `cursor_expired` et il faut repartir de `since`. On en déduit que les positions déjà vues vont revenir, d'où la déduplication ([[position-dedup-rules]]).

- **Chaque appel renvoie les mêmes positions que le précédent plus les nouvelles.** La fenêtre glissante fait qu'une position est vue en moyenne 10 fois. Le taux de doublons Trakko est de 90 %, c'est normal, ce n'est pas un bug.

## Les horodatages

`timestamp` est **l'heure du boîtier**, pas celle du serveur Trakko. Un boîtier sans GPS fix au démarrage part de l'epoch ou de sa dernière heure connue ; un boîtier avec une pile faible dérive. L'incident [[incident-2026-02-trakko-timestamp-drift]] raconte ce que ça donne. Depuis, on lit aussi `received_at` (l'heure de réception chez Trakko, présente mais non documentée) et on applique la règle : si `|timestamp - received_at| > 120 s`, on garde `received_at` comme heure de la position et on marque `time_source = 'server'`.

Format : ISO 8601 avec `Z`, sauf le champ `history` qui renvoie des secondes epoch. Les deux sont gérés.

## Les champs

- `lat`, `lon` : décimaux, 6 chiffres. `0.0, 0.0` signifie « pas de fix », pas « au large du Ghana ». Filtré.

- `speed` : km/h, entier. On le lit pour le filtre de plausibilité et on ne le stocke pas (décision conformité).

- `heading` : degrés, `-1` si inconnu.

- `ignition` : la chaîne `"ON"`, `"OFF"` ou `"UNKNOWN"`, jamais un booléen, et `"On"` avec une majuscule sur les boîtiers de la série achetée en 2024. Comparaison insensible à la casse.

- `odometer` : mètres, mais kilomètres sur les boîtiers de 2024 (le firmware a changé). On détecte par plausibilité (un camion ne fait pas 2 millions de kilomètres) et on convertit ; le code a un commentaire résigné.

- `accuracy` : absent. On pose 15 m par défaut, la valeur médiane mesurée en comparant aux positions de l'application sur les mêmes véhicules.

## Événements de flotte

`GET /v3/fleets/{fleet}/events` donne les installations et désinstallations de boîtiers (`device_installed`, `device_removed`) avec l'immatriculation saisie par l'installateur. C'est l'entrée du mapping boîtier vers véhicule ([[tracker-vehicle-mapping]]) et l'immatriculation y est saisie à la main, avec les fautes qu'on imagine.

## Support

Un e-mail, réponse sous 2 jours ouvrés, en anglais. Ils ont corrigé deux choses en dix-huit mois (un `500` sur l'historique d'un véhicule supprimé, la documentation de `received_at`). Le reste est chez nous, dans l'adaptateur.

Coût et clauses contractuelles dans [[telematics-costs-per-provider]].

## Le poller en pratique

Pour retrouver comment on les appelle sans lire le code :

- Un worker par flotte, boucle `sleep(30) ; GET positions?since=<dernier ts vu - 60 s>`. Le `- 60 s` est volontaire : on préfère revoir des positions (la déduplication les jette) que d'en rater une à la frontière de la fenêtre.

- En cas de 429 ou de 5xx, attente exponentielle à partir de 30 s, plafonnée à 5 minutes, et un compteur `trakko_poll_errors_total{fleet, code}`. Trois erreurs de suite sur une flotte déclenchent une alerte informative, pas une page.

- Après une interruption de plus de 5 minutes (la fenêtre glissante), le worker passe en mode rattrapage : pour chaque véhicule de la flotte ayant eu une position dans les 24 h précédentes, `GET history?from=<dernier ts>&to=<maintenant>`, au rythme du limiteur local. Le rattrapage écrit avec `time_source` selon la règle de l'écart et passe par le chemin de plausibilité qui accepte les horodatages anciens.

- La clé de chaque flotte est relue depuis l'environnement à chaque cycle (injection par le vault), donc une rotation ne demande pas de redémarrage.

Ce qu'on a mesuré en mai 2026 : 2,1 millions d'appels par mois, 0,3 % d'erreurs, 6 rattrapages de plus d'une heure (tous des pannes de notre côté ou du réseau, aucune de Trakko).
