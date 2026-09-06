---
name: template-review-checklist
description: Ce que le relecteur vérifie sur un nouveau template ou type d'événement de notification, tiré des erreurs de 2025 et 2026, avant d'approuver la MR
type: feedback
status: active
verified: 2026-05-20
---

## Pourquoi cette liste

Chaque point vient d'une erreur passée en production. Un template est du code déployé, il passe en revue comme du code, et le relecteur n'est jamais l'auteur du texte. La liste est dans `docs/notifications/REVIEW.md` du dépôt API et reprise ici pour la mémoire.

## Type d'événement

- Il a un `owner` dans `event_types.yaml`, un `group` pour les préférences ([[notification-preferences-schema]]), un `ttl` si le message a une durée de vie (un code, une fenêtre horaire), un `reputation_tier`. Un type sans `ttl` sur un contenu périssable, c'est l'erreur de janvier ([[incident-2026-01-otp-sms-retry-loop]]).

- Il est déclaré dans `senders.yaml` si l'adresse d'envoi ou le nom affiché diffère du défaut. Un nouveau `From` non aligné a coûté six jours de spam en novembre 2025.

- Si le type est émis en masse (import, batch de nuit), le producteur passe `bulk(true)`. Demander « combien par heure au pire » et vérifier que la réponse est dans la MR.

- S'il est déclaré `critical`, la justification est dans la MR et la liste des critiques a été relue en entier : il y en a trois, il ne devrait pas y en avoir dix.

## Contenu e-mail

- Sujet : une ligne, pas de montant, pas de mot en majuscules, pas de « URGENT ». Les filtres d'entreprise l'ont appris avant nous.

- Un seul lien d'action, vers `app.halden.example`, avec `?ref=<event>`. Aucun lien vers un domaine tiers, même le site d'un partenaire.

- Version texte écrite à part, pas dérivée du HTML.

- Dates par `|hf_datetime`, montants par `|hf_money`, jamais formatés à la main. Un montant polonais en `1,240.00` a été lu comme 1,24 par un comptable.

- Pas de pièce jointe. Les documents sont des liens.

- Pied de page avec le lien de préférences, même sur du transactionnel.

- Le rendu dans les 11 locales passe `notifications:lint-templates` ; si une locale manque, c'est un repli sur `en` assumé et noté dans la MR, pas un oubli.

## Contenu SMS

- Un segment GSM-7 sur la fixture la plus longue. Le lint le dit, le relecteur vérifie que la fixture est réaliste (une adresse allemande longue, un nom de transporteur de 40 caractères).

- Pas de lien raccourci par un service tiers. Nos liens SMS sont `hf.example/<code>` court, servi par nous.

- Le message dit qui écrit dans les premiers mots (« Halden : … ») parce que l'expéditeur est un numéro long dans quatre pays ([[sms-country-routing-and-unit-costs]]).

## Push

- Titre sous 40 caractères, corps sous 120, sinon tronqué sur la moitié des téléphones.

- Le `data` du push contient l'identifiant de l'objet (chargement, facture), jamais son contenu : la notification est le pointeur, l'application charge le reste après authentification.

## Producteur

- L'appel à `dispatch()` est idempotent côté appelant : rejouer le même événement métier produit la même `idempotency_key`. Vérifier comment la clé est construite, pas juste qu'elle existe.

- Pas d'appel à `dispatch()` dans une boucle sans regroupement ni limite explicite. Si la boucle est légitime (un digest), le regroupement passe par `batch` dans le yaml, pas par du code dans le producteur.

## Ce que le relecteur ne fait pas

Il ne réécrit pas le texte. Si la formulation est mauvaise, il le dit à l'auteur et au produit, mais le ton appartient au produit. La revue porte sur ce qui casse, pas sur ce qui plaît.
