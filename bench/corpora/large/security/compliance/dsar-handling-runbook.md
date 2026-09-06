---
name: dsar-handling-runbook
description: Data subject requests go to privacy@halden.example, are logged in dsar_requests, verified proportionately and answered within 30 days by export or erase
type: reference
status: active
verified: 2026-03-12
---

# Traitement des demandes de personnes concernées (DSAR)

Accès, rectification, effacement, portabilité, opposition. En 2025 : 31 demandes, dont 19 d'effacement, 9 d'accès, 3 d'opposition au suivi. Délai médian de réponse 11 jours.

## Réception

- Une seule porte : `privacy@halden.example`, boîte partagée lue par la rota conformité chaque jour ouvré. Une demande arrivée par le support, par un commercial ou par un chauffeur dans le chat est transférée à cette boîte, jamais traitée en direct.

- Chaque demande devient une ligne `dsar_requests (id, received_at, channel, type, subject_type, subject_user_id, requester_email, status, due_at, closed_at, notes)` créée par `bin/console compliance:dsar:open`. `due_at` est `received_at + 30 jours`. Une prolongation à 90 jours est possible pour les demandes complexes, il faut l'écrire dans `notes` et en informer la personne avant le 30e jour ; utilisée deux fois en 2025.

## Vérification d'identité

Proportionnée à ce qui est demandé :

- Accès ou effacement par un utilisateur connecté depuis `/settings/privacy` : identité vérifiée par la session, rien d'autre.

- Demande par e-mail depuis l'adresse du compte : on répond à cette adresse et on demande une confirmation par un lien signé (valide 48 h). La confirmation vaut vérification.

- Demande depuis une autre adresse, ou pour une personne sans compte (un contact de livraison, un chauffeur sans compte nommé) : on demande de quoi rapprocher sans exiger de pièce d'identité (le numéro de téléphone tel qu'il figure chez nous, le nom du transporteur employeur, une référence de chargement). Pas de scan de carte d'identité : on n'en veut pas, on ne saurait pas le garder proprement.

## Cas du chauffeur

Si la personne est un chauffeur, le transporteur employeur est responsable de traitement pour une partie des données (voir [[dpa-carriers-template]]). On informe le transporteur de la demande sous 48 h comme le prévoit l'accord, et on traite nous-mêmes la partie dont nous sommes responsables (positions collectées par notre application, compte, journaux). Le transporteur décide pour le reste (affectations, documents qu'il a téléversés). Trois demandes de chauffeurs en 2025, deux fois le transporteur a demandé qu'on efface tout, une fois il a demandé de garder les affectations pour un litige en cours, ce qui est un motif légitime qu'on a accepté par écrit.

## Exécution

**Accès.** `compliance:dsar:export --request <id>` produit une archive ZIP : un JSON par catégorie de [[gdpr-data-map]], un PDF récapitulatif lisible, et la liste des destinataires (chargeurs qui ont vu la position, fournisseurs). L'archive va dans `hf-exports` avec un lien signé valable 7 jours, envoyé à l'adresse vérifiée. Les positions sont incluses en résumé (trajets) au-delà de 90 jours puisque le brut n'existe plus, voir [[data-retention-matrix]].

**Effacement.** `compliance:dsar:erase --request <id> --dry-run` liste ce qui sera touché, puis sans `--dry-run`. La commande pseudonymise plutôt que supprimer là où une obligation de conservation existe (un chargement facturé garde un `user_id` pseudonymisé), supprime ailleurs, pousse une demande d'effacement à l'entrepôt de données et aux fournisseurs concernés (Payla, fournisseur d'e-mail), et écrit une ligne `dsar_erasure_proofs` avec les comptes par table. Les sauvegardes ne sont pas modifiées ; elles expirent sous 35 jours et on le dit dans la réponse.

**Opposition au suivi.** Pour un chauffeur : désactivation de la collecte de positions par l'application pour ce compte, information au transporteur. Ce que ça implique pour le service est décrit dans [[driver-position-legal-basis]].

## Réponse

Un e-mail depuis la boîte partagée, en français ou en anglais selon la demande, qui dit ce qui a été fait, ce qui n'a pas été fait et pourquoi (obligation de conservation, litige), et comment contester. Le modèle est dans `halden-legal/templates/dsar-*.md`. On ferme la ligne `dsar_requests` avec `closed_at`.

## Ce qui a mal tourné une fois

Décembre 2025 : un effacement lancé sans `--dry-run` a pseudonymisé le compte d'un homonyme (même nom, autre transporteur) parce que la demande avait été ouverte avec le mauvais `subject_user_id`. Détecté en 2 heures par le transporteur, restauré depuis la sauvegarde du matin pour cet utilisateur uniquement. Depuis, `compliance:dsar:erase` refuse de tourner sans que le `--dry-run` de la même demande ait été exécuté dans les 24 heures par la même personne, et affiche l'e-mail et le transporteur du compte visé avant de continuer.
