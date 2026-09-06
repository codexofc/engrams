---
name: edifact-iftmin-mapping
description: IftminToLoadMapper turns BGM.1004 into the idempotency key, LOC and NAD into sites, DTM in partner timezone into slots, GID and MEA into goods; 14 overrides
type: reference
status: active
verified: 2026-05-19
---

# Mapping IFTMIN vers chargement

Un message IFTMIN (instruction de transport) entrant devient un chargement (`Load`) sur la plateforme, dans l'organisation du partenaire ([[edi-partners-overview]]). Le travail est fait par `IftminToLoadMapper` dans `hf-edi-gateway`, avec les tables de correspondance par partenaire décrites dans [[edi-mapping-tables-location]]. Cette note dit quel segment devient quel champ, et où les partenaires divergent.

## Enveloppe et idempotence

- `UNB` : identifiants émetteur et récepteur, comparés à `edi_partners.sender_id` et à notre identifiant. Un émetteur inconnu est rejeté au niveau CONTRL, avant tout mapping.

- `UNH` : type de message et version (`IFTMIN:D:96A:UN` ou `D:01B`). La version choisit le jeu de qualifiants.

- `BGM` : `1001` code de fonction (`335` instruction, `9` original, `5` remplacement, `1` annulation). **`1004` est la référence du partenaire**, et c'est la clé d'idempotence : `loads.partner_reference` unique par organisation. Un IFTMIN original avec une référence déjà connue est un doublon (voir [[incident-2026-03-edi-duplicate-loads]] pour ce qui arrive quand cette règle a une faille). Un remplacement met à jour le chargement s'il n'est pas encore attribué, sinon il est rejeté avec APERAK `AP` motif `load_already_dispatched`. Une annulation suit la machine à états des chargements de la plateforme.

## Dates et heures

`DTM` avec qualifiant `2` (livraison demandée) et `10` (enlèvement) et parfois `64`/`63` (au plus tôt / au plus tard). Format `203` (`CCYYMMDDHHMM`) chez quatre partenaires, `102` (date seule) chez Vestaflor pour la livraison, complété par une heure par défaut de leur table.

**Les heures sont locales au partenaire**, sans fuseau dans le message. `edi_partners.timezone` donne le fuseau, et le mapper convertit en UTC. Bruma Retail a des sites en deux fuseaux ; leur table de sites (`edi_partner_locations`) porte le fuseau par site et il prime sur celui du partenaire. Erreur classique corrigée en 2025 : un enlèvement à 06:00 heure de Varsovie enregistré comme 06:00 UTC.

## Lieux

Groupe `TDT` (mode et moyen de transport : on lit `8067 = 3` route, on ignore le reste) puis groupes `LOC` :

- `LOC+9` lieu de chargement, `LOC+11` lieu de déchargement. Le lieu est soit un **code partenaire** (`3225` avec liste de codes `92` = attribué par l'émetteur), résolu par `edi_partner_locations (partner_id, code, site_id)`, soit une adresse complète dans le groupe `NAD` associé (`NAD+CZ` expéditeur, `NAD+CN` destinataire).

- Code inconnu : le mapper tente une correspondance par l'adresse `NAD` si présente (géocodage puis rapprochement à 200 m d'un site existant du partenaire), sinon rejet `unknown_location` avec le code en clair dans l'APERAK. C'est la première cause de rejets ([[edi-rejects-handling]]).

- La personne de contact (`CTA+IC` et `COM`) va dans `loads.delivery_contact_*`, soumise à la rétention de 13 mois du projet conformité.

## Marchandise

- `GID` : nombre et type de colis (`7224` quantité, `7065` type : `PX` palette, `CT` carton...). Additionné en `loads.pallet_count` quand le type est palette, sinon en `package_count` avec le type.

- `MEA+WT` poids brut en `KGM` (Kalmarine envoie des `TNE`, converti), `MEA+VOL` volume en `MTQ`.

- `FTX+AAA` texte libre vers `loads.instructions`, tronqué à 2 000 caractères. `FTX+HAN` instructions de manutention vers `loads.handling_notes`.

- `TCC` ou `FTX+ADR` (selon partenaire) pour les matières dangereuses : classe et numéro UN vers `loads.adr_class`, `loads.un_number`, ce qui déclenche la majoration ADR de la tarification.

- Température dirigée : `TMP+2` avec `6246` valeur et `6411` unité. Vestaflor uniquement.

## Prix et conditions

`PRI` ou `MOA+203` : le prix cible du partenaire s'il en envoie un, vers `loads.target_price`. Nordkarton et Bruma envoient, les autres laissent notre tarification proposer. `PAT` conditions de paiement : ignoré, les conditions sont celles du contrat commercial, pas du message, décision de 2024 quand un partenaire a envoyé 90 jours par erreur.

## Ce que le mapper refuse

- IFTMIN sans `BGM.1004` : rejet `missing_reference`.

- Enlèvement après livraison : rejet `slot_order`.

- Plus d'un `LOC+9` ou `LOC+11` : rejet `multi_stop_unsupported`. Les tournées multi-arrêts ne sont pas prises en EDI ; deux partenaires les envoient en plusieurs IFTMIN liés par `RFF+ACD`, qu'on relie par `loads.group_reference`.

## Les 14 surcharges partenaire

Dans `edi_partner_overrides (partner_id, key, value)`, lues par le mapper : format de date de la livraison, heure par défaut, unité de poids, segment ADR, liste de codes de colis, fuseau par site, tolérance de géocodage, et la règle « un remplacement écrase-t-il les instructions libres » (Nordkarton : non, elles envoient des remplacements sans `FTX` qui auraient vidé le champ). Quatorze clés en mai 2026, chacune avec le ticket qui l'a créée en commentaire de ligne. Une surcharge nouvelle exige un cas réel, pas une lecture du standard.

Le test `IftminMappingTest` passe 61 messages réels anonymisés (un par cas de mapping et par partenaire) et compare le `Load` obtenu à un instantané JSON.

## Un message minimal qui passe

Pour tester une nouvelle surcharge ou un nouveau partenaire sans attendre un vrai message :

```
UNB+UNOC:3+PARTNER01+HALDEN+260519:0800+00001'
UNH+1+IFTMIN:D:96A:UN'
BGM+335+REF-2026-000123+9'
DTM+10:202605200600:203'
DTM+2:202605201400:203'
TDT+20++3'
LOC+9+SITE-A::92'
LOC+11+SITE-B::92'
GID+1+12:PX'
MEA+WT+G+KGM:8400'
UNT+10+1'
UNZ+1+00001'
```

Avec `SITE-A` et `SITE-B` dans `edi_partner_locations` du partenaire, ce message devient un chargement de 12 palettes, 8 400 kg, enlèvement le 20 mai à 06:00 heure du partenaire, livraison à 14:00. `bin/console edi:map-dry-run <fichier> --partner <code>` affiche le `Load` obtenu en JSON sans rien écrire, et c'est ce qu'on envoie au partenaire quand il demande « qu'est-ce que vous comprenez de mon message ».

Le dry-run sert aussi en revue : une MR qui touche le mapper joint la sortie de `edi:map-dry-run` sur les 61 fixtures, et le relecteur lit le diff des instantanés plutôt que le code.
