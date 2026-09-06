---
name: testing-pyramid-rules
description: Platform-wide testing rules, every behaviour change ships with a test at the lowest layer that can observe it, contract tests between API and clients, no test hits a third party, flaky tests are quarantined for one month max
type: reference
status: active
verified: 2026-04-20
---

# Règles de test communes à la plateforme

Chaque projet a sa note détaillée (PHPUnit côté API, widget et intégration côté mobile, Playwright côté web). Cette note est ce qui est commun et qui s'applique aux trois.

## Le test au niveau le plus bas qui peut l'observer

Un changement de comportement est accompagné d'un test, au niveau le plus bas où le comportement est observable :

- une règle métier (calcul de score, transition d'état) : test unitaire, sans base ni réseau ;

- une requête SQL ou une migration : test d'intégration avec la vraie base ;

- un contrat HTTP (forme de réponse, code d'erreur) : test d'API ;

- un parcours utilisateur qui traverse plusieurs écrans : test de bout en bout, et seulement s'il n'est pas déjà couvert par les niveaux inférieurs.

Un test de bout en bout qui vérifie une règle de calcul est renvoyé en revue : il est lent, il casse pour de mauvaises raisons, et il ne dit pas où est le bug.

## Contrats entre l'API et ses clients

- Le document OpenAPI de l'API est généré depuis le code et comparé en CI à la version commitée. Un changement non commité échoue.

- Le front régénère ses types depuis ce document chaque nuit et ouvre une MR si le diff n'est pas vide.

- L'app mobile a des fixtures JSON capturées depuis staging, et une fixture de plus de 6 mois échoue.

- Les payloads de webhook ont un test de contrat côté API (`WebhookPayloadContractTest`) qui compare à des exemples figés par version de schéma.

Ces quatre mécanismes forment la détection de dérive. Aucun d'eux n'empêche un changement, ils le rendent visible avant la production.

## Aucun test ne touche un tiers

Fournisseur d'e-mail, de push, tuiles de carte, service de routage, stockage objet : tous remplacés par des doubles en test. Le stockage objet a un serveur compatible S3 en conteneur dans le CI, les autres sont des faux en mémoire. Un test qui a besoin d'un vrai tiers est un test manuel de la checklist de release, pas un test automatisé.

## Instabilité

Un test instable (échoue puis passe sans changement) est mis en quarantaine le jour même : il continue à tourner sans bloquer, avec le ticket dans le fichier. Un mois maximum en quarantaine, ensuite réparé ou supprimé. Le nombre de tests en quarantaine par projet est dans le résumé de chaque MR.

## Données de test

Des constructeurs (`aLoad()->open()->build()`) plutôt que des fixtures partagées, dans les trois projets. Les fixtures partagées créent un couplage entre tests que personne ne voit avant de modifier une ligne et de casser trente tests.

## Ce qu'on ne mesure pas

La couverture globale. Elle est affichée, elle n'est pas un objectif. Les seuils existent seulement sur les zones critiques nommées par chaque projet (la couche de synchronisation mobile, les repositories de l'API). Voir [[definition-of-done]] pour ce qui est exigé avant de fermer un ticket.
