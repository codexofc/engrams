---
name: partner-feedback-sales-wants-more-boards
description: Revue commerciale de juin 2026 : partenaires à 11 % du volume et 23 transporteurs convertis, demande de trois bourses de plus, une seule retenue
type: feedback
status: active
verified: 2026-07-09
---

# Retour commercial : « il nous faut plus de bourses »

Revue trimestrielle du 2026-06-24, commerciaux, produit, intégrations. Le point partenaires a pris la moitié de la réunion.

## Ce que le commercial voit

- **Volume** : 11 % du volume dispatché passe par Cargolink, 8 % par Fretzone (dans les deux sens). Sans les partenaires, une partie de ces chargements aurait trouvé preneur chez nous quand même, mais plus tard et plus cher ; l'estimation produit est un gain net de 6 à 8 % de volume.

- **Acquisition** : 23 transporteurs fantômes sont devenus des transporteurs Halden à part entière depuis mai ([[partner-bid-relay-and-shadow-carriers]]). Le commercial les compte comme des leads gratuits qualifiés, ce qu'ils sont.

- **Argument de vente** : « publiez une fois, touchez trois bourses » ferme des contrats chargeurs, en particulier en Allemagne et en France. Les chargeurs polonais et italiens demandent leurs bourses locales.

- **Demande** : trois bourses de plus d'ici fin 2026, une polonaise, une italienne, une ibérique. Noms fictifs dans le compte rendu, pas de contrat signé.

## Ce que l'équipe intégrations a répondu

- Une intégration coûte six semaines à deux personnes pour arriver en production, et deux mois de plus pour être stable, avec la checklist ([[partner-onboarding-checklist-new-board]]). L'équipe est de trois personnes dont une à mi-temps. Trois bourses en six mois, c'est toute l'équipe et rien d'autre, y compris les webhooks.

- Chaque bourse apporte ses règles contractuelles dans le code, et chaque règle est un incident potentiel. Les deux incidents de l'année ([[partner-incident-2025-12-cargolink-mass-expiry]], [[partner-incident-2026-04-fretzone-price-drift]]) venaient de règles de partenaire.

- La demande locale est réelle mais inégale : la bourse polonaise citée n'a pas d'API publique, la bourse italienne en a une sans identifiant d'annonce stable. Deux critères de refus de la checklist. La bourse ibérique passe les critères.

## Décision

- **Une bourse en 2026**, l'ibérique, si le contrat se signe avant septembre, démarrage en octobre, production en décembre.

- Les deux autres : le commercial ouvre la discussion avec les bourses sur les prérequis techniques (bac à sable, identifiants stables, statut lisible) et on réévalue en janvier 2027. Le commercial a demandé une fiche d'une page « ce qu'on attend d'une API de bourse » à leur envoyer ; écrite depuis, elle reprend la section « quand dire non » de la checklist.

- Le produit met en avant les 23 conversions dans le rapport au comité de direction, parce que c'est l'argument qui justifiera la quatrième personne dans l'équipe en 2027.

## Ce que le commercial a appris

Que « ajouter une bourse » n'est pas « ajouter un bouton ». Le chiffre qui a marqué : les deux intégrations existantes ont consommé 40 % du temps de l'équipe en 2026 en maintenance et incidents, hors développement. La question suivante a été « et si on retirait Fretzone ? », à laquelle personne n'a voulu répondre, parce que 8 % du volume, ça ne se retire pas.

## Ce que l'équipe intégrations a appris

Que le commercial ne connaissait pas les 23 conversions avant la réunion, ni la règle de non-démarchage de Fretzone qui en limite d'autres. Un tableau de bord partenaires pour le commercial (volume, conversions, commissions, incidents) est en cours, HF-3225, pour que la prochaine revue parte des mêmes chiffres.
