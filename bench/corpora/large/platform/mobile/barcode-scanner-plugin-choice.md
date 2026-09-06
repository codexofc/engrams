---
name: barcode-scanner-plugin-choice
description: Pallet label scanning uses the on-device ML barcode plugin (mobile_scanner style) rather than a pure-Dart decoder, chosen in HF-1345 after 0.4 s vs 2.8 s median decode on Galaxy A25, Code 128 and QR only
type: project
status: active
verified: 2026-02-09
---

# Scanner de codes-barres : choix du plugin (HF-1345)

Depuis l'app 4.7, un chauffeur peut scanner l'étiquette d'une palette (Code 128 ou QR, selon le chargeur) à l'enlèvement et à la livraison pour lier les colis au chargement. Deux candidats ont été comparés sur les quatre téléphones de la ferme de test avec 30 étiquettes réelles imprimées en 300 dpi, dont 10 abîmées (froissées, sales, en partie déchirées).

## Résultats

| Critère | Décodeur Dart pur | Plugin natif (ML sur l'appareil) |
|---|---|---|
| Médiane de décodage, Galaxy A25 | 2,8 s | 0,4 s |
| Médiane, iPhone 12 | 1,1 s | 0,3 s |
| Taux de lecture sur les 10 étiquettes abîmées | 4 / 10 | 9 / 10 |
| Taille ajoutée à l'APK arm64 | 0,3 Mo | 2,6 Mo |
| Fonctionne sans services Google | oui | oui (modèle embarqué, pas la variante téléchargée à la demande) |

Le plugin natif a gagné sur le critère qui compte : un chauffeur ne va pas tenir le téléphone stable 3 secondes devant chaque palette. Les 2,6 Mo ont été acceptés avec le label `size-ok` (voir [[app-size-budget]]).

## Configuration retenue

- Formats limités à `code128` et `qrCode`. Activer tous les formats doublait le temps de décodage et provoquait des faux positifs sur les EAN des produits visibles sur la palette.

- Résolution de l'aperçu caméra à 1280×720, pas plus. Le 1080p n'améliorait pas le taux de lecture et chauffait le téléphone.

- Zone de scan restreinte à un rectangle central de 70 % de largeur, avec un cadre visible. Les étiquettes voisines sont ainsi ignorées.

- Torche activable d'un appui, et activée automatiquement si la luminosité mesurée par le capteur est sous 10 lux (entrepôts sombres).

- Bip et vibration à la lecture, obligatoire : les chauffeurs ne regardent pas l'écran.

- Anti-doublon : un même code lu deux fois en moins de 3 secondes est ignoré.

## Ce qui a été refusé

- Le mode "scan en rafale" qui liste tout ce qui passe devant la caméra. Testé, trop de lectures parasites, les chauffeurs préfèrent viser une étiquette.

- La saisie manuelle du code comme repli est présente mais cachée derrière un bouton "Saisir", parce qu'une saisie manuelle produit une erreur sur 8 en moyenne (chiffres transposés).

## Intégration avec la synchronisation

Un scan produit une mutation `scan_package` dans l'outbox avec le code, le type, l'arrêt et l'horodatage corrigé (voir [[lesson-never-trust-device-clock]]). Un code inconnu du chargement côté serveur n'est pas une erreur : il est enregistré avec `matched: false` et le dispatcher le voit. On a choisi ça plutôt qu'un refus, parce que les listes de colis fournies par les chargeurs sont incomplètes une fois sur trois.
