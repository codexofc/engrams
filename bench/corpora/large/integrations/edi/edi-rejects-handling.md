---
name: edi-rejects-handling
description: A rejected inbound gets an APERAK with one of 12 reason codes and a row in edi_rejects; support fixes 60 % from mapping data, median 4 h, 2.1 % reject rate
type: project
status: active
verified: 2026-05-26
---

# Traitement des rejets EDI (HF-2064)

Un IFTMIN qui ne devient pas un chargement n'est pas une erreur silencieuse : le partenaire attend un accusé, le chargement attend d'exister, et le commercial du compte attend qu'on ne perde pas la commande. Le flux ci-dessous est en place depuis décembre 2025.

## Deux niveaux d'accusé

- **CONTRL** : syntaxe. Le message est illisible (segment manquant, compteur `UNT` faux, émetteur inconnu). Généré par le parseur avant tout mapping, renvoyé sous 60 s. Rare : 0,1 % des messages, presque toujours une nouvelle version du TMS du partenaire.

- **APERAK** : application. Le message est lisible mais ne peut pas devenir un chargement. `BGM.1225 = 27` (non accepté) avec un `FTX+AAO` qui contient **notre code de motif et une phrase en anglais**, et `RFF+ACW` qui reprend la référence du partenaire. Renvoyé sous 5 minutes.

Les douze codes de motif (`edi_reject_reasons`) : `unknown_location`, `missing_reference`, `duplicate_reference`, `slot_order`, `slot_in_past`, `multi_stop_unsupported`, `load_already_dispatched`, `unknown_package_type`, `weight_out_of_range`, `adr_incomplete`, `unknown_site_timezone`, `partner_inactive`. Ajouter un code demande un cas réel et une phrase que le partenaire comprendra sans nous appeler.

## Répartition (mai 2026, 3 100 messages, 65 rejets)

| Motif | Part | Qui corrige |
|---|---|---|
| `unknown_location` | 46 % | nous (ajout du code site dans la table) |
| `slot_in_past` | 17 % | partenaire (message envoyé en retard) ou nous (fuseau) |
| `duplicate_reference` | 12 % | personne : c'est un vrai doublon, le rejet est la bonne réponse |
| `weight_out_of_range` | 9 % | nous (unité) ou partenaire (faute de saisie) |
| `load_already_dispatched` | 8 % | partenaire (remplacement trop tard) |
| autres | 8 % | |

## La file de rejets

Chaque rejet est une ligne `edi_rejects (id, partner_id, message_ref, reason, raw_key, received_at, resolved_at, resolution, resolved_by)` et apparaît dans `/backoffice/edi/rejects`, avec le message brut lisible (segments décodés, champs surlignés) et les actions :

- **Corriger et rejouer** : pour `unknown_location`, l'écran propose l'adresse `NAD` du message et les sites du partenaire à moins de 2 km ; l'agent choisit ou crée le site, le code est ajouté à `edi_partner_locations`, le message est rejoué depuis le brut (`edi:replay <id>`) et devient un chargement. Le partenaire ne fait rien et ne renvoie rien ; il a reçu un APERAK négatif puis, au rejeu, un APERAK positif (`BGM.1225 = 29`) pour la même référence, ce que les cinq TMS acceptent.

- **Demander au partenaire** : un e-mail depuis un gabarit avec la référence, le motif et la phrase, au contact EDI du partenaire (`edi_partners.reconciliation_contact`). La ligne reste ouverte jusqu'à réception d'un nouveau message pour la même référence, qui la ferme automatiquement.

- **Fermer sans action** : pour `duplicate_reference`, avec une raison.

Résolution médiane : 4 heures en mai 2026, 60 % des rejets résolus par nous sans le partenaire. Un rejet ouvert depuis plus de 24 heures ouvrées remonte au commercial du compte ; plus de 72 heures, au responsable intégrations.

## Ce qui n'est pas un rejet

Un IFTMIN accepté avec un avertissement (`BGM.1225 = 29` et un `FTX+AAO` qui commence par `WARN:`) : par exemple un poids manquant remplacé par une valeur par défaut, ou une heure de livraison en format date seule complétée. Le chargement existe, l'avertissement est dans `loads.edi_warnings` et visible par le dispatcher du partenaire sur le web. Un partenaire qui accumule des avertissements du même type reçoit un e-mail mensuel ; Vestaflor a corrigé son format de date après le troisième.

## Ce qu'on a écarté

- **Créer le chargement quand même** en état brouillon pour les rejets « légers ». Testé en 2025 : les brouillons n'étaient jamais complétés et polluaient la liste. Un chargement existe ou n'existe pas.

- **Appeler le partenaire par téléphone**. Leurs équipes EDI répondent par ticket, en jours ; l'e-mail depuis le gabarit est ce qui marche.

Le rapport quotidien de réconciliation ([[edi-reconciliation-daily]]) reprend les rejets ouverts, pour qu'un rejet non traité ne devienne pas un écart de fin de mois. Les codes de statut sortants, eux, sont un autre sujet : [[edi-status-messages-iftsta]].
