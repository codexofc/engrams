---
name: complaint-rate-and-feedback-loops
description: Boucles de rétroaction des grands fournisseurs de boîtes mail, taux de plainte cible sous 0,03 %, List-Unsubscribe obligatoire, réputation par IP dédiée
type: reference
status: active
verified: 2026-05-05
---

## Ce qu'est une boucle de rétroaction

Quand un destinataire clique « spam » chez un grand fournisseur de boîtes mail, celui-ci nous renvoie un rapport si nous sommes inscrits à sa boucle de rétroaction (feedback loop) pour notre domaine d'envoi. Sans inscription, la plainte compte contre notre réputation et nous ne le savons pas. Courrix gère les inscriptions pour `mail.halden.example` et nous transmet chaque plainte en événement `complained` dans le webhook, avec le fournisseur d'origine dans `source`.

Nous sommes inscrits chez les quatre fournisseurs qui pèsent dans nos envois (ensemble 58 % des adresses de `users`), le reste étant des domaines d'entreprise avec leurs propres passerelles, qui ne renvoient rien et bloquent en silence. Pour ceux-là le seul signal est un taux de `delivered` qui baisse par domaine destinataire, panneau « livraison par domaine » du tableau de bord, seuil d'alerte à 90 % sur 24 h pour tout domaine de plus de 200 destinataires.

## Taux de plainte

Cible interne : sous 0,03 % des livrés sur 7 jours glissants. Réalité 2026 : entre 0,010 et 0,022 % selon la semaine. L'alerte est à 0,08 % sur 24 h ([[bounce-handling-and-suppression]] pour l'effet sur l'adresse). Les fournisseurs publient une tolérance autour de 0,1 % au-delà de laquelle le domaine est ralenti puis filtré.

Ce qui génère des plaintes chez nous, par ordre : les notifications d'enchère aux expéditeurs qui reçoivent trop (le digest de [[shipper-bid-digest-batching]] a divisé leur part par trois), les invitations à rejoindre la plateforme envoyées par un transporteur à un contact qui ne le connaît pas (type `carrier.invite`, 0,3 % de plaintes à lui seul, désormais limité à 20 invitations par jour et par compte), et les rappels de facture.

## En-têtes

Chaque e-mail sortant porte :

- `List-Unsubscribe: <https://app.halden.example/notifications/unsubscribe?t=...>, <mailto:unsub@mail.halden.example>`

- `List-Unsubscribe-Post: List-Unsubscribe=One-Click`

- `Feedback-ID: <event_type>:<owner_team>:halden` pour que les rapports par type d'événement soient lisibles dans les outils des fournisseurs.

Le désabonnement en un clic met `all_email = false` dans les préférences ([[notification-preferences-schema]]) sauf pour les événements `critical`, et l'utilisateur voit un bandeau à la prochaine connexion lui expliquant ce qu'il ne recevra plus. Sur du transactionnel pur, ces en-têtes ne sont pas exigés ; nous les mettons parce que le bouton « se désabonner » à côté du bouton « spam » détourne une partie des clics du second vers le premier. Mesuré sur mars 2026 : 410 désabonnements en un clic contre 190 plaintes.

## Réputation par IP

Depuis avril 2026 nous envoyons depuis une IP dédiée chez Courrix, après l'épisode décrit dans [[incident-2026-04-dedicated-ip-blocklist]]. La réputation de cette IP et celle du domaine sont relevées chaque nuit par le script `reputation-probe` sur les trois listes de blocage publiques que les grands fournisseurs consultent, plus le tableau de bord de réputation de Courrix via leur API. Un changement d'état (apparition sur une liste, passage de « bonne » à « moyenne ») ouvre un ticket automatiquement dans la file plateforme, sans page : la nuit, personne ne peut rien y faire, et le matin il faut de toute façon attendre la réponse des listes.

## Ce qu'on ne fait pas

- Pas de « réchauffement » automatique avec des envois artificiels vers des boîtes à nous. Le volume réel suffit et les envois artificiels sont repérés par les fournisseurs.

- Pas de second domaine d'envoi transactionnel « au cas où ». Un domaine vierge n'a pas de réputation et serait pire que le nôtre le jour où on en aurait besoin.
