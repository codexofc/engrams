---
name: incident-2025-10-phishing-finance
description: On 2025-10-21 a finance team member entered SSO credentials on a look-alike page; attacker session lasted 27 min without MFA; triggered the MFA rollout
type: project
status: active
verified: 2025-11-28
---

# Phishing sur l'équipe finance (octobre 2025)

Traité avec l'ancien processus ([[incident-process-v1]]), reconstruit a posteriori. Post-mortem `2025-10-21-phishing-finance.md`. Sévérité rétroactive : SEV1 (compte staff compromis).

## Chronologie (UTC)

- **2025-10-21 06:48** `[log]` : e-mail reçu par 6 membres de l'équipe finance, expéditeur usurpé « payouts@halden-freight.example » (domaine proche du nôtre, enregistré 3 jours avant), objet « Virement transporteur en attente de validation », lien vers `sso-halden.example/validate`. La page reproduisait la mire de connexion d'Idento.

- **07:02** `[log]` : une personne clique et saisit identifiant et mot de passe. Le compte n'avait pas de second facteur (voir la note IAM sur le déploiement MFA, 23 comptes sur 84 en avaient un à l'époque).

- **07:11** `[audit]` : `auth.login_succeeded` pour ce compte depuis une IP d'hébergeur, pays différent de l'habituel. Aucune alerte n'existait sur ce signal.

- **07:14 à 07:38** `[audit]` : 41 requêtes de lecture dans le back-office, section finance : liste des virements du jour, détail de 8 transporteurs (IBAN masqué à l'affichage, 4 derniers chiffres visibles), page des paramètres de paiement. Aucune écriture. Une tentative d'export CSV des virements, refusée : la permission `export.payouts` n'était pas dans `staff_finance` à ce moment-là, par oubli plus que par design.

- **07:35** `[chat]` : la personne, prise d'un doute en voyant l'URL dans son historique, écrit dans le canal de l'équipe « je crois que j'ai cliqué sur un truc ».

- **07:41** `[chat]` : un développeur voit le message, vérifie `audit_events`, voit la session inconnue, révoque toutes les sessions du compte et force un changement de mot de passe. Fin de l'accès de l'attaquant : 40 minutes après la saisie, 27 minutes de session active.

- **07:50** : le domaine usurpé est signalé au registrar et au fournisseur d'hébergement. Coupé le 2025-10-23.

- **09:30** : les 5 autres destinataires sont contactés un par un. Aucun n'avait cliqué. Deux avaient déjà supprimé l'e-mail « parce que ça ressemblait à du spam ».

- **2025-10-22** : revue des `audit_events` de la session par deux personnes. Conclusion : lecture seule, pas d'export réussi, pas de modification. Le DPO est informé le 22 au matin, décide qu'il n'y a pas de notification à faire (IBAN masqués, pas de données de personnes physiques hors noms de contacts déjà publics sur les factures).

## Cause

Le compte était protégé par un mot de passe seul, et la page de connexion Idento n'avait aucune caractéristique qu'un faux ne puisse imiter. La personne a fait ce que la page lui demandait ; le système permettait qu'un mot de passe seul suffise.

Facteurs contributifs : aucune alerte sur une connexion staff depuis un pays inhabituel ; le domaine `halden-freight.example` n'était pas surveillé ; l'équipe finance recevait légitimement des e-mails « virement en attente » de notre propre plateforme, donc le leurre était crédible.

## Ce qui a bien marché

La personne a parlé, 33 minutes après avoir cliqué, dans un canal où quelqu'un a réagi en 6 minutes. C'est la seule raison pour laquelle l'accès a duré 27 minutes et pas une journée. Le post-mortem le dit en première ligne, et c'est la raison pour laquelle il n'y a jamais eu de discussion sur une sanction.

## Actions

- MFA obligatoire pour tout le staff : décidé le 2025-10-22, livré le 2026-01-15 (projet IAM).

- Alerte sur `auth.login_succeeded` staff depuis un pays non vu pour ce compte dans les 90 jours : livrée le 2025-11-04, 3 déclenchements depuis, tous des déplacements.

- Surveillance des domaines proches (`halden-freight`, `haldenfreight`, `halden-fright`, etc.) par un service de veille, alerte à l'enregistrement : livrée novembre 2025, 4 domaines signalés depuis, 2 coupés.

- `export.payouts` ajouté à `staff_finance` **et** soumis à ré-authentification. Le fait que la permission manquait nous a sauvés par accident ; on ne compte pas sur les accidents.

- Un exercice de phishing interne par trimestre, annoncé comme pratique générale, jamais comme piège individuel. Premier en janvier 2026 : 9 % de clics, 60 % de signalements.

- Un canal `#signalement-securite` où « je crois que j'ai cliqué » est la phrase attendue, avec la promesse écrite qu'elle n'a aucune conséquence pour la personne. Les modèles de messages sont dans [[incident-comms-templates]].

Ticket : HF-2046.

## Ce que l'attaquant a regardé, précisément

La liste des 41 requêtes, reconstruite depuis `audit_events` et les logs d'accès du back-office, parce qu'on nous a demandé après coup ce qui avait été exposé :

- 07:14 à 07:19 : page d'accueil finance, liste des virements du jour (28 lignes : transporteur, montant, statut, IBAN masqué).

- 07:19 à 07:31 : détail de 8 transporteurs parmi les 28, dans l'ordre décroissant des montants. Sur chaque fiche : raison sociale, adresse, numéro de TVA, IBAN avec les 4 derniers chiffres, contact (nom, e-mail, téléphone du gérant).

- 07:31 : page des paramètres de paiement (délai de virement, compte émetteur, avec l'IBAN de notre compte émetteur masqué de la même façon).

- 07:33 : tentative d'export CSV, refusée.

- 07:34 à 07:38 : retour sur la liste, tri par montant, ouverture de 3 fiches supplémentaires.

Aucune requête vers les chargements, les utilisateurs, les clés API ou l'administration. Le comportement est celui d'une préparation de fraude au virement (identifier les gros fournisseurs, obtenir des contacts pour un faux changement d'IBAN), ce qui a été communiqué aux 11 transporteurs concernés avec une mise en garde sur les demandes de changement de coordonnées bancaires. Aucun d'eux n'a signalé de tentative dans les trois mois suivants.
