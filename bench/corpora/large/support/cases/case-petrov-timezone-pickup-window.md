---
name: case-petrov-timezone-pickup-window
description: Décembre 2025, Petrov Trans en retard sur onze chargements Lisbonne-Madrid : fenêtres saisies dans le fuseau du navigateur, une heure d'écart
type: project
status: active
verified: 2026-02-18
---

# Cas : Petrov Trans, une heure de retard systématique

Transporteur fictif bulgare, 40 camions, beaucoup de lignes ibériques, Business. Ticket du 2025-12-09, catégorie `other:rating`, après une baisse de la ponctualité au pickup.

## Ce qui s'est passé

Un chargeur basé à Paris publiait des chargements au départ de Lisbonne avec une fenêtre de pickup « 08:00-10:00 ». Le formulaire web interprétait la saisie dans le fuseau du navigateur du chargeur (Europe/Paris) et stockait en UTC, comme la règle de la plateforme le veut. L'app chauffeur affichait la fenêtre en heure locale du lieu de pickup (Europe/Lisbon), soit 07:00-09:00. Le chauffeur de Petrov arrivait à 08:30 heure de Lisbonne, dans la fenêtre telle que le chargeur la pensait, hors fenêtre telle qu'elle était stockée : 30 minutes de retard, tolérance dépassée.

Onze chargements, onze retards, la ponctualité de Petrov passe de 96 % à 87 %. Le chargeur, lui, ne voyait aucun retard sur son écran, parce que son écran affichait en heure de Paris et que le camion arrivait avant 10:00 heure de Paris... non, il arrivait à 09:30 heure de Paris, donc dans la fenêtre côté chargeur aussi. Personne ne comprenait le retard sauf le calcul.

## Ce qu'on a trouvé

L2 a mis deux jours, parce que tous les outils affichaient une heure différente : `hfctl load get` en UTC, le back-office en heure de l'agent (Paris), l'app en heure du lieu. La comparaison à la main a fini par montrer l'écart d'une heure. La règle de la plateforme est claire (UTC en base, conversion à l'affichage) ; le problème était le fuseau de **saisie** : celui du navigateur au lieu de celui du lieu de pickup.

## Ce qu'on a changé

- HF-3055 : le formulaire de chargement interprète les fenêtres dans le fuseau du lieu (déduit du géocodage de l'adresse) et l'affiche à côté du champ (« heure de Lisbonne »). Livré janvier 2026. Le fuseau du navigateur ne sert plus qu'à l'affichage des listes.

- Les onze retards ont été requalifiés ; la ponctualité de Petrov est remontée le lendemain.

- `hfctl load get` affiche depuis 1.11 les fenêtres en UTC **et** en heure du lieu, sur deux colonnes, pour que le support n'ait plus à convertir.

- Une ligne dans le playbook de contestation de note : « si tous les retards sont du même chargeur et de la même durée, chercher un fuseau ».

## Ce qu'on a appris

- Une règle « tout en UTC » ne dit rien du fuseau de saisie, et c'est là que l'erreur vit.

- Trois outils, trois fuseaux d'affichage, c'est trois heures perdues par ticket. Les outils du support affichent deux fuseaux maintenant, UTC et local, jamais celui de l'agent.

- Un chargeur qui « ne voit pas de retard » et un calcul qui en voit ont forcément un référentiel différent. La question à poser tout de suite : dans quel fuseau ?

Le cas est dans la liste de [[case-lessons-recurring-themes-2026-h1]] parce qu'il a coûté deux jours de L2 pour une heure de décalage.
