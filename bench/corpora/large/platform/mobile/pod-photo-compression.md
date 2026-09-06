---
name: pod-photo-compression
description: POD photos are resized to 1600 px on the long edge and JPEG quality 82 on the device before upload, average file went from 4.3 MB to 380 KB, done in a background isolate, originals never kept
type: project
status: active
verified: 2026-01-27
---

# Compression des photos de POD (HF-1380, app 4.7)

## Avant

La caméra des téléphones actuels produit des JPEG de 3 à 12 Mo. Un chauffeur envoie 2 à 6 photos par livraison. Sur une connexion de dépôt en zone industrielle (souvent 1 à 2 Mbit/s montant), une livraison c'était 30 à 120 s d'envoi, souvent en échec, ce qui a alimenté [[incident-2026-01-duplicate-pod-uploads]]. Et côté stockage, 4,3 Mo en moyenne par document sur 2025.

## Ce qu'on fait maintenant

Dans `ImagePreparer.prepare()`, exécuté dans un isolate séparé (via `compute`) pour ne pas bloquer l'interface :

1. Décodage avec le package `image`, lecture de l'orientation EXIF et rotation appliquée aux pixels (sinon certains lecteurs affichent la photo couchée, et le back-office a eu le problème).

2. Redimensionnement à 1600 px sur le grand côté, jamais d'agrandissement.

3. Encodage JPEG qualité 82.

4. Suppression de toutes les métadonnées EXIF sauf l'orientation (mise à 1) : les coordonnées GPS de la photo sont une donnée personnelle du chauffeur et on n'en a pas besoin, la position de livraison est déjà dans l'événement `deliver`.

5. Calcul du SHA-256 du résultat, qui est celui envoyé à l'API à la création du document.

L'original pris par la caméra est supprimé du cache immédiatement après. On ne garde pas de copie haute résolution : aucun cas d'usage remonté en 18 mois qui demande plus de 1600 px pour lire une signature ou un numéro de colis.

## Chiffres (janvier 2026, 40 000 photos)

- Taille moyenne : 380 Ko (médiane 310 Ko, p95 720 Ko).

- Temps de préparation sur Galaxy A25 : 600 ms par photo. Sur iPhone 12 : 250 ms.

- Temps d'envoi moyen par livraison : 4 s au lieu de 50 s.

- Échecs d'envoi (avant reprise) : 1,1 % au lieu de 9 %.

## Le choix 1600 / 82

Testé avec 20 PODs réels (bons de livraison manuscrits, étiquettes de palettes) lus par 3 personnes du support. À 1200 px, deux signatures étaient jugées illisibles. À 1600, aucune. La qualité 82 est le point où la taille cesse de baisser de façon utile (75 donnait 320 Ko avec des artefacts visibles sur le texte manuscrit).

## Ce qui reste

Les documents PDF (CMR scannés par certains transporteurs) ne passent pas par ce chemin, ils sont envoyés tels quels avec une limite de 10 Mo. Et l'app ne fait pas de recadrage automatique des documents (détection des bords), c'était dans le ticket initial et retiré parce que la bibliothèque candidate ajoutait 9 Mo à l'APK, voir [[app-size-budget]].
