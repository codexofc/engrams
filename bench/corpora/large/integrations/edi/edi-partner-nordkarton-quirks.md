---
name: edi-partner-nordkarton-quirks
description: Nordkarton (1 400 loads/month) sends replacements without FTX, rejects unlisted IFTSTA codes, pads site codes, wants INVOIC in 48 h and closes AS2 at night
type: project
status: active
verified: 2026-06-17
---

# Nordkarton : le profil du plus gros partenaire

Fabricant d'emballages, cinq usines, un TMS ancien et une équipe EDI de deux personnes joignables par ticket. Premier partenaire EDIFACT (septembre 2024) et celui pour lequel la moitié des mécanismes du projet ont été construits. Vue générale des partenaires : [[edi-partners-overview]].

## Ce qui les distingue

**Remplacements sans texte libre.** Leur TMS envoie un IFTMIN de remplacement (`BGM.1001 = 5`) à chaque modification, même mineure (une heure décalée de 15 minutes), et ce remplacement ne contient **pas** les segments `FTX` de l'original. Un remplacement appliqué littéralement vidait `loads.instructions`. Surcharge `replacement_keeps_ftx = true` dans [[edifact-iftmin-mapping]] : un remplacement sans `FTX` conserve le texte existant. Découvert en octobre 2024 quand un dispatcher a perdu la mention « quai 4, sonner deux fois » sur 30 chargements.

**Volume de remplacements** : 2,3 remplacements par chargement en moyenne, jusqu'à 11 sur un même chargement. Chacun est un message, un accusé, une ligne dans `edi_messages`. Un remplacement identique au précédent (même contenu après normalisation) est accepté et ignoré, avec un APERAK positif, sans événement plateforme ; ça évite de notifier le transporteur pour rien. Environ 40 % de leurs remplacements sont dans ce cas.

**Codes de statut fermés.** Leur TMS rejette par CONTRL négatif tout IFTSTA avec un code `4405` qu'ils n'ont pas configuré. Les codes `20` (ETA) et `17` (retard) sont désactivés pour eux dans le jeu de statuts ([[edi-status-messages-iftsta]]). Ils ont demandé l'ETA « par e-mail » à la place ; refusé, ils ont la page de suivi.

**Codes de site avec espaces finaux.** `LOC+9+NK-WERK-2   ` : trois espaces, longueur fixe héritée de leur ancien système. Le mapper fait un `rtrim` sur les codes de lieu pour tout le monde depuis cet incident (novembre 2024) ; ça ne peut casser personne.

**INVOIC par chargement, sous 48 h.** Leur comptabilité fournisseurs veut une facture par chargement, dans les 48 heures de la livraison, avec leur référence dans `RFF+ON`. Le mode de facturation à l'échéance de la plateforme n'est pas possible pour eux ; c'est une des raisons de l'INVOIC sortant ([[edi-invoice-invoic-outbound]]) et de son délai.

**Fenêtre AS2.** Leur endpoint AS2 est éteint de 22:00 à 05:00 heure de Paris pour maintenance quotidienne. Nos IFTSTA de nuit (livraisons de nuit, 15 % de leur volume) sont réessayés selon le calendrier standard ([[as2-transport-setup]]) et arrivent à 05:00. On a ajouté une surcharge `as2_quiet_hours` pour ne pas compter ces échecs dans l'alerte « 3 MDN échoués de suite ».

**Fichier de comptage à 04:00**, avant que leur queue du matin soit partie, donc avec des `unexpected_inbound` systématiques dans la réconciliation ([[edi-reconciliation-daily]]) qui disparaissent le lendemain. Accepté, documenté dans le rapport.

## Les 7 surcharges

`replacement_keeps_ftx`, `date_format_delivery = 203`, `weight_unit = KGM`, `status_codes_closed = true`, `as2_transfer_encoding = binary`, `as2_quiet_hours = 22:00-05:00 Europe/Paris`, `invoice_mode = per_load_48h`. Chacune a le ticket qui l'a créée.

## Ce qu'on a obtenu d'eux

En dix-huit mois, deux changements de leur côté : l'activation du code `21` (transporteur affecté, juin 2025, parce que leur service achat le voulait) et le passage à AS2 depuis SFTP (mars 2025, [[edi-sftp-legacy-transport]]). Délai moyen d'une demande : 6 semaines. Tout le reste est chez nous, et c'est assumé : ils font 1 400 chargements par mois.

## Chiffres

Rejets : 1,4 % (sous la moyenne, grâce aux tables de sites complètes après deux ans). Litiges de fin de mois : 2 par mois depuis le rapport mensuel de réconciliation, 15 avant. Délai de résolution d'un rejet : 3 h en médiane, presque tous des codes de site d'une usine nouvellement équipée.

## Ce qu'on refuse

Une connexion directe de leur TMS à notre base « pour aller plus vite », proposée par leur intégrateur en 2025. Non. L'API JSON ([[edi-api-json-alternative]]) existe s'ils veulent du synchrone.
