---
name: partner-load-field-mapping-rules
description: Correspondance des champs vers Cargolink et Fretzone : types de véhicule, ADR, adresses sans rue, prix arrondis, heures locales, champs jamais transmis
type: reference
status: active
verified: 2026-05-15
---

# Correspondance des champs vers les partenaires

Un chargement Halden devient une annonce Cargolink ou Fretzone par `PartnerListingMapper` (une classe par partenaire, une interface commune). Ce qui suit est ce que les tests de contrat (`tests/Partners/Mapping/*Test.php`) figent.

## Types de véhicule

### Correspondance

| Halden | Cargolink | Fretzone |
|---|---|---|
| `TAUTLINER` | `curtainsider` | `TAUT` |
| `BOX` | `box` | `FOURGON` |
| `FRIGO` | `reefer` | `FRIGO` |
| `MEGA` | `mega` | non poussé (`unsupported_vehicle_type`) |
| `FLATBED` | `flatbed` | `PLATEAU` |
| `TANKER` | `tank` | `CITERNE` |
| `VAN` | `van` | `VUL` |

Un chargement acceptant plusieurs types est poussé avec la liste chez Cargolink (qui l'accepte) et avec le **premier** type chez Fretzone (qui n'en prend qu'un), le reste dans le texte libre. C'est perdant mais c'est ce que leur API permet.

## ADR

Halden : `adr_classes[]`. Cargolink : `hazmat: true` plus `adr_classes` tel quel. Fretzone : `adr: true` seulement, sans classe, et la classe 1 (explosifs) n'est pas acceptée : `unsupported_adr_class`, non poussé.

## Adresses

On ne transmet **jamais** l'adresse complète avant attribution : ville, code postal, pays, et pour Cargolink le rayon de 5 km qu'ils affichent. La rue est envoyée à l'attribution seulement, au transporteur retenu, par notre app. C'est une règle commerciale (l'adresse d'un client final ne circule pas sur une bourse) et elle est dans le contrat des deux côtés.

Normalisation avant envoi et avant comparaison (voir [[partner-cargolink-sync-push-feed]]) : repli ASCII des noms de ville (« Straßburg » devient « Strassburg » pour Cargolink, qui le fait de son côté ; Fretzone garde les accents), code postal sans espace, pays en ISO 3166-1 alpha-2 majuscules.

## Prix

Halden : `target_price` en cents dans la devise du chargeur. Cargolink : EUR uniquement, arrondi aux **5 EUR** (leur grille), converti au taux du jour pour PLN et CZK. Fretzone : EUR, au centime, mais ils affichent au transporteur un prix « à partir de » qui est le nôtre moins 3 % (leur marge affichée), ce qui génère des questions de chargeurs ([[partner-fretzone-contract-quirks]]).

Un chargement sans prix indicatif (`target_price = null`, « sur offre ») est poussé chez Cargolink comme `price_on_request` ; Fretzone exige un prix, on pousse alors le prix médian du corridor sur 30 jours calculé par le pricing, avec un marqueur dans le texte. Décision HF-3088, discutée, le chargeur voit le prix que nous avons choisi dans sa fiche.

## Fenêtres horaires

Stockées en UTC. Poussées en **heure locale du lieu** (pickup pour la fenêtre de pickup, livraison pour la fenêtre de livraison), avec le décalage explicite chez Cargolink (`2026-05-05T06:00:00+02:00`) et sans décalage chez Fretzone (`2026-05-05 06:00`, heure du lieu implicite). Le cas Petrov du support (fenêtres saisies dans le mauvais fuseau) venait de la saisie, pas de la correspondance, mais les tests couvrent les deux.

## Référence et identifiants

La référence du chargeur est poussée préfixée : `HF-<org court>-<référence>` pour rester unique sur la bourse et reconnaissable au retour. Notre `load_id` est dans un champ d'identifiant externe chez les deux partenaires et nous sert de clé au retour ([[partner-dedup-external-refs-table]]).

## Texte libre

Marchandise, palettes, instructions : concaténés dans le champ texte du partenaire, tronqués à 500 caractères (Cargolink) ou 300 (Fretzone), avec la mention des types de véhicule perdus. Les numéros de téléphone et e-mails présents dans le texte libre du chargeur sont **supprimés** par une expression régulière avant envoi (règle commerciale encore : le contact passe par la plateforme).

## Ce qu'on ne transmet jamais

Nom du destinataire final, rue, contacts, valeur déclarée de la marchandise (sert à l'assurance chez nous, pas à la bourse), historique des offres, prix plancher fixé par le chargeur (`bid_floor`), identifiants internes autres que `load_id`.

## Retour

Les annonces Fretzone rapatriées chez nous suivent la correspondance inverse, avec les mêmes tables ; ce qui n'a pas d'équivalent (un type de véhicule Fretzone inconnu) donne un chargement avec `vehicle_types = []` et un avertissement dans la recherche.
