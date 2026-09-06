---
name: case-nordvik-gps-battery-complaint
description: Retour de Nordvik Frakt (avril 2026) : chauffeurs coupant la localisation, 9 % de batterie par heure mesurés sur vieux téléphones, d'où HF-3125
type: feedback
status: active
verified: 2026-06-02
---

# Retour : Nordvik Frakt, la batterie et la localisation

Transporteur fictif norvégien, 20 camions, Business. Pas un ticket au départ : un e-mail du gérant au responsable de compte, transmis au support, avril 2026.

## Ce qu'il a dit

Ses chauffeurs désactivaient la localisation du téléphone en hiver parce que l'app « mangeait la batterie » et qu'ils avaient besoin du téléphone pour le reste de la journée. Résultat : des chargements en transit sans position, des chargeurs qui appellent, et le gérant coincé entre les deux. Il ne demandait pas une correction précise, il décrivait le problème et demandait si c'était normal.

## Ce qu'on a vérifié

- Les tickets `driver:sync` sous-tag `gps` de Nordvik : 11 sur l'hiver, tous fermés avec la macro sur les permissions en arrière-plan, sans que personne ne fasse le lien.

- Les appareils (`hfctl driver devices` sur leurs 26 chauffeurs) : 19 téléphones de plus de quatre ans, Android 10 et 11, batteries d'origine.

- La mobile a mesuré sur un appareil du même modèle, à 0 °C, avec le suivi actif : 9 % de batterie par heure pour l'app, contre 4 % sur un appareil récent à température ambiante. Le budget de la note mobile sur la batterie était calibré sur des appareils récents.

## Ce qu'on a répondu

Que c'était normal dans le sens où l'app fait ce qu'elle doit, et anormal dans le sens où 9 % par heure n'est pas tenable pour une journée de 10 h. Sans promesse de délai, avec le ticket HF-3125 en référence.

## Ce qui en est sorti

- HF-3125 (livré en 4.9, juin 2026) : la fréquence de position passe de 2 minutes à 5 minutes quand la batterie est sous 30 % et que le téléphone n'est pas en charge, avec un message dans l'app. Le chargeur voit « position toutes les 5 min, batterie faible » sur la carte. Mesuré : 6 % par heure dans les mêmes conditions.

- La macro `gps-permission-background` a gagné un paragraphe sur la batterie et la charge en cabine.

- Le support a une nouvelle habitude : un même transporteur avec plus de trois tickets sur le même sous-tag en un trimestre est remonté au tri, même si chaque ticket a été « résolu ». Onze tickets fermés correctement peuvent cacher un seul problème.

## Ce qu'on n'a pas fait

Pas de désactivation du suivi par transporteur. Pas de subvention de téléphones. Le gérant a fini par équiper ses camions de chargeurs, ce qu'il aurait pu faire avant, mais il fallait qu'on lui dise que le problème était réel et mesuré. Voir [[case-lessons-recurring-themes-2026-h1]].
