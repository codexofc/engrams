---
name: support-escalation-levels-l1-l2-l3
description: Escalade support en trois niveaux (L1 Deskline, L2 technique, astreinte backend) et les cinq critères qui justifient un page
type: reference
status: active
verified: 2026-05-12
---

# Escalade support : qui, quand, comment

Trois niveaux, un seul canal d'escalade (`#support-escalation`), et une règle simple : on escalade un fait constaté, pas une impression.

## L1 : la file Deskline

L1 répond dans Deskline (voir [[support-tooling-backoffice-console]]). Il a le back-office en lecture, l'impersonation en lecture seule, et les playbooks. Il ne touche jamais la base. Un ticket reste en L1 tant qu'un playbook le couvre. Si le playbook se termine par « escalader », le ticket passe en L2 avec le formulaire rempli : identifiant de l'organisation, identifiant du chargement ou de la facture, ce qui a été vérifié, ce qui a été vu.

Un ticket L1 sans identifiant de chargement, de facture ou de chauffeur est renvoyé au client avec la macro `need-reference`. Ça paraît sec, mais depuis qu'on le fait (HF-3002, octobre 2025) le temps de première réponse utile a baissé de 41 min à 19 min en médiane.

## L2 : support technique

Deux personnes en rotation hebdomadaire. L2 a `hfctl` avec le rôle `support-write`, un accès SQL à la réplique `pg-ro.hf.internal` et la lecture des logs dans Grafana. L2 peut : rejouer un webhook, forcer une resynchronisation partenaire, débloquer un compte chauffeur, corriger une livraison déclarée par erreur ([[support-sla-tiers]] dit dans quel délai). L2 ne peut pas : modifier une facture finalisée, toucher aux paiements Payla, écrire en base autrement que par `hfctl`.

Quand L2 a besoin d'un `UPDATE` direct, il ouvre un ticket HF avec la requête proposée et le backend d'astreinte l'exécute ou le refuse. Ça prend en général moins d'une heure en journée.

## L3 : astreinte backend

On page (`hfctl page backend --reason ...`) uniquement si l'un des critères est rempli :

- plus de 10 organisations touchées par le même symptôme en moins de 30 minutes

- un chargement `IN_TRANSIT` dont le chauffeur est injoignable et le chargeur réclame une annulation (seul le support peut faire `cancel_in_transit`, mais la décision se prend à deux)

- une facture visiblement fausse déjà envoyée à un client Enterprise

- une fuite de données possible (un client voit les chargements d'un autre)

- une alerte plateforme visible côté client depuis plus de 15 minutes sans incident déclaré

Tout le reste attend 9 h. C'est écrit ici parce qu'en novembre 2025 L2 a pagé trois fois en une semaine pour des webhooks `DEAD` chez un seul intégrateur, ce qui se rejoue tout seul le matin.

## Ce que l'escalade emporte

Le message dans `#support-escalation` suit le gabarit `esc-template` : symptôme en une phrase, périmètre (combien d'orgs, depuis quand), identifiants, ce qui a été tenté, lien Deskline. Pas de capture d'écran seule. Pas de « c'est urgent » sans dire pour qui et pourquoi.

## Retour vers le client

Celui qui a le ticket Deskline garde la parole avec le client, même quand L3 travaille. Le backend ne répond jamais directement au client, sauf pour les intégrateurs Enterprise qui ont un canal partagé (trois à ce jour).

## Incidents déclarés

Si le backend déclare un incident, L1 bascule sur la macro `incident-open` avec le lien de la page de statut `status.halden.example` et arrête de traiter les tickets un par un : on les tague `incident:<numéro>` et on répond en masse à la clôture. Le tri hebdomadaire ([[support-weekly-triage-ritual]]) reprend les tickets tagués pour vérifier qu'aucun n'était en fait un autre problème.
