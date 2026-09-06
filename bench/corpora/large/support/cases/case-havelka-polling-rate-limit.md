---
name: case-havelka-polling-rate-limit
description: Février 2026, l'intégration de Havelka Doprava interrogeait chaque chargement toutes les 3 s, 100 % de 429 ; webhooks et exemption d'une semaine
type: project
status: active
verified: 2026-04-10
---

# Cas : Havelka Doprava, le polling toutes les trois secondes

Transporteur fictif tchèque, 70 camions, Business, intégration écrite par leur propre développeur. Ticket du 2026-02-24 : « votre API est en panne depuis hier ».

## Ce qui s'est passé

Leur intégration faisait `GET /v2/loads/{id}` toutes les 3 secondes pour chacun de leurs chargements actifs, environ 80 en journée, soit 1 600 requêtes par minute pour une limite de 600. Depuis le 23 février, quand ils sont passés de 30 à 80 chargements actifs (nouveau contrat), tout dépassait, et leur code ne lisait ni le code 429 ni `Retry-After` : il réessayait immédiatement, ce qui aggravait. Leur TMS affichait tous les chargements « en erreur ». Ils ont conclu à une panne chez nous.

## Ce qu'on a fait

- Grafana « Org overview » : 1 600 requêtes par minute, 99 % de 429, une seule route. Diagnostic en cinq minutes.

- Réponse avec la macro qui n'existait pas encore (`api-rate-limit-webhooks` est née là) : les limites, `Retry-After`, et la recommandation des webhooks.

- Leur développeur a d'abord demandé une limite à 2 000 par minute. Refusé, avec l'explication : la limite protège tout le monde, et le besoin réel (savoir quand un chargement change d'état) est couvert par les webhooks avec zéro requête.

- Exemption temporaire d'une semaine par le flag `ratelimit.exempt_orgs` (ticket HF-3078), le temps qu'il implémente l'abonnement webhook. Il l'a fait en trois jours. L'exemption a été retirée le 3 mars.

- Depuis, leur intégration fait environ 40 requêtes par minute (des `GET` de vérification toutes les deux minutes en secours) et reçoit les changements d'état par webhook.

## Ce qu'on a appris

- « Votre API est en panne » avec un seul client touché, c'est presque toujours le client. Le premier réflexe est le tableau Grafana de l'org, pas la page de statut.

- Une exemption courte avec une date de fin écrite dans le ticket est acceptable ; une limite relevée est une dette qu'on ne récupère jamais.

- Leur code ne lisait pas `Retry-After` parce que la doc le mentionnait en bas d'une page. Il est en haut maintenant, avec un exemple.

## Suite

Havelka n'a plus dépassé la limite. Leur développeur a signalé deux imprécisions dans le guide webhook (le `driver` à `null` avant assignation, déjà vu chez Brenner, et l'ordre non garanti des événements). Les deux sont corrigées. Voir [[case-lessons-recurring-themes-2026-h1]] pour la place de ce cas.
