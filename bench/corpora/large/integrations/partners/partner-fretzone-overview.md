---
name: partner-fretzone-overview
description: Fretzone, bourse française depuis 2025-11 : annonces dans les deux sens (leur poll de 10 min, notre pull), acceptation chez eux, exclusivité 2 h
type: reference
status: active
verified: 2026-07-16
---

# Vue d'ensemble de l'intégration Fretzone

Fretzone est une bourse de fret fictive, française, historique, avec beaucoup de petits transporteurs et une API conçue pour le polling. Intégrée en novembre 2025 (HF-3030) parce que nos chargeurs français y publiaient déjà à la main. Contrairement à Cargolink ([[partner-cargolink-overview]]), le flux des annonces va dans **les deux sens**, ce qui est la source de la plupart des complications.

## Les flux

**Nos annonces vers Fretzone.** Fretzone ne propose pas d'API d'écriture temps réel : ils **interrogent** un flux que nous exposons, `GET /v2/partners/fretzone/feed`, toutes les 10 minutes, avec un curseur. Le flux liste les créations, mises à jour et annulations depuis le curseur. Conséquence : un changement de prix met jusqu'à 10 minutes à apparaître chez eux, et on ne sait qu'ils l'ont pris que quand ils rappellent le flux avec un curseur plus avancé. `sync_state = PENDING_PUSH` dans `partner_load_refs` ([[partner-dedup-external-refs-table]]) veut dire « pas encore dans un appel de flux acquitté ».

**Leurs annonces vers nous.** Un worker `app:partners:fretzone:pull` appelle `GET https://api.fretzone.example/v2/offres?since=` toutes les 5 minutes et crée chez nous des chargements `source = fretzone`, visibles dans la recherche transporteur avec la mention « via Fretzone », **sauf** si l'annonce correspond à un chargement direct du même chargeur (règle de déduplication HF-3210, née du cas Bosque du support). Ces chargements ne sont pas modifiables chez nous ; ils suivent l'annonce d'origine.

**Offres.** Un transporteur Halden qui mise sur une annonce Fretzone : l'offre est transmise à Fretzone (`POST .../offres/{id}/propositions`), et l'acceptation se fait **chez Fretzone**, par le chargeur Fretzone, sur leur site. On la reçoit par le pull suivant (`statut = attribuee`, avec l'identifiant de la proposition retenue). Si c'est la nôtre, le chargement passe `DISPATCHED` chez nous et l'exécution se fait sur Halden. Si ce n'est pas la nôtre, le chargement rapatrié disparaît de notre recherche.

Un transporteur Fretzone qui mise sur une de nos annonces : l'offre arrive dans le pull, on la crée sur le chargement avec un transporteur fantôme ([[partner-bid-relay-and-shadow-carriers]]), le chargeur accepte chez nous, et on pousse l'attribution dans le flux.

## La fenêtre d'exclusivité

Clause propre à Fretzone ([[partner-fretzone-contract-quirks]]) : une annonce publiée chez eux par un chargeur Fretzone est réservée à leurs transporteurs pendant **2 heures** avant d'être visible par les partenaires. Le pull ne la voit qu'après. Un transporteur Halden qui la trouve « en retard » n'y peut rien ; c'est contractuel, et c'est expliqué dans l'aide.

## Volumes (juin 2026)

- Annonces à nous poussées (via leur poll) : environ 1 600 par semaine.

- Annonces rapatriées de Fretzone : environ 5 800 par semaine avant déduplication, 4 900 après.

- Offres de nos transporteurs sur leurs annonces : 2 100 par semaine ; attribuées à un transporteur Halden : 9 %.

- Chargements dispatchés via Fretzone (dans les deux sens) : environ 380 par semaine.

## Authentification

Chez eux : clé d'API par compte partenaire, dans le secret store sous `partners/fretzone/*`, envoyée dans un en-tête `X-Api-Key`. Pas de rotation programmée de leur côté ; on demande une rotation tous les six mois par ticket chez eux, et on note la date dans le runbook. Chez nous : notre flux exige une authentification de base sur un compte technique dédié à Fretzone, avec une adresse IP source vérifiée (ils n'en ont qu'une).

## Ce que l'intégration ne couvre pas

- Les documents : le POD d'un chargement exécuté sur Halden reste sur Halden, le chargeur Fretzone le reçoit par e-mail (fonction de leur côté, sur la base d'un lien que nous poussons dans le flux).

- Le suivi : pas de positions transmises à Fretzone.

- La facturation : sur Halden quand le transporteur est à nous, chez Fretzone quand le transporteur est à eux. La réconciliation ([[partner-reconciliation-nightly-job]]) compte les deux.

- Les véhicules `MEGA` et les transports ADR classe 1 : Fretzone n'a pas ces catégories, les annonces ne sont pas poussées (`unsupported_vehicle_type`).

## Propriétaires

Nous : l'équipe intégrations, la personne à mi-temps sur les partenaires en premier. Eux : un contact technique unique, réactif en journée, et un contact commercial. Pas de page de statut publique ; on détecte leurs pannes par les erreurs du pull ([[partner-incident-2026-04-fretzone-price-drift]] raconte une panne qui n'en avait pas l'air).

## Ce qui nous a surpris

- Leur API renvoie les dates en heure de Paris sans décalage explicite, toute l'année. Un chargement portugais est donc annoncé chez eux avec une heure d'avance sur l'heure locale du lieu ; c'est un affichage de leur côté et on ne peut rien y faire, mais le support a la macro.

- Leur poll de notre flux ne suit pas le curseur si notre réponse dépasse 500 éléments : ils prennent les 500 premiers et repartent du même curseur au prochain appel, ce qui rejoue les mêmes. Notre flux pagine à 400 depuis janvier 2026 et le problème a disparu ; leur contact a confirmé la limite sans qu'elle soit documentée.

- Un chargeur Fretzone peut modifier son annonce après qu'un de nos transporteurs a fait une proposition. La proposition reste attachée au nouveau prix chez eux, pas chez nous : on la marque `stale` et on prévient le transporteur, qui peut la retirer.

- Ils n'ont pas de notion d'annulation d'attribution. Un chargeur Fretzone qui « désattribue » supprime l'annonce et la republie. Pour nous, c'est un `CANCELLED` puis un nouveau chargement rapatrié, ce que le transporteur voit comme une annulation avec pénalité potentielle de son côté à lui ; la note du transporteur Halden n'est pas touchée dans ce cas (l'acteur est le chargeur).

## Où regarder quand ça casse

Grafana « Partners », filtre `fretzone` : âge du dernier pull réussi, éléments par pull, propositions envoyées, erreurs, compteur de quota. Alerte `PartnerPullStale` si le dernier pull réussi date de plus de 20 minutes. Ensuite `hfctl partner runs --partner fretzone --since 1h`. Leur contact technique répond en journée ; le soir, on attend le matin, leurs transporteurs aussi.
