---
name: notifications-purge-and-retention
description: Rétention des tables de notification: 180 jours pour notifications et deliveries, 90 pour les événements fournisseur, rendu jamais stocké, purge par lots
type: reference
status: active
verified: 2026-04-22
---

## Ce qu'on garde, combien de temps

| Table | Rétention | Lignes (avril 2026) | Pourquoi cette durée |
|---|---|---|---|
| `notifications` | 180 jours | 20 M | le support remonte rarement au-delà de 3 mois, 6 laisse de la marge |
| `notification_deliveries` | 180 jours | 45 M | même chose, alignée sur l'intention |
| `notification_provider_events` | 90 jours | 31 M | ne sert qu'au rejeu ([[provider-webhook-verification-and-replay]]) |
| `notification_batches` | 30 jours après ouverture | 400 k | les digests envoyés n'ont plus d'intérêt |
| `suppressed_recipients` | pas de purge | 39 k | une suppression est une décision, pas un journal |
| `notification_audit` | 5 ans | 9 k | actes du support, durée légale interne |

## Ce qu'on ne stocke pas

Le contenu rendu. Ni le HTML de l'e-mail, ni le texte du SMS, ni le corps du push. La base garde le type d'événement, la locale, le payload d'entrée (JSONB, souvent des identifiants et des montants) et la version du template (`template_version`, le hash du fichier compilé). `notifications:trace` re-rend le message à la demande à partir du payload et de la version du template, qui est retrouvable dans l'historique Git de l'image. Cela divise la taille de `notification_deliveries` par huit par rapport à un stockage du rendu, et surtout cela évite de garder 45 millions de textes contenant des noms, des adresses et des montants.

Exception : les 20 premiers caractères du sujet e-mail, dans `subject_prefix`, parce que le support cherche par « facture F-2026-… » et que re-rendre pour chercher ne marche pas.

## La purge

`bin/console notifications:purge` tourne à 03:40 en CronJob dans `platform-prod`, après le nettoyage des documents. Suppression par lots de 20 000 lignes (`DELETE ... WHERE id IN (SELECT id ... ORDER BY created_at LIMIT 20000)`), une pause de 200 ms entre les lots, sur l'index `(status, updated_at)` pour `notification_deliveries` et `(created_at)` pour les autres. Une nuit normale supprime 280 000 livraisons en 4 minutes. Le retard de purge (plus vieille ligne au-delà de la rétention + 3 jours) est un `warn`.

La purge respecte l'ordre : livraisons, puis intentions dont aucune livraison ne reste, puis événements fournisseur. Un `ON DELETE CASCADE` avait été envisagé et écarté : une purge par cascade de 20 M de lignes bloque plus longtemps qu'une purge par lots explicite, et la nuit n'est pas si calme (les factures partent à 04:00).

## Ce qui part dans l'entrepôt

Avant purge, un agrégat par jour, type d'événement, canal, statut, pays SMS et motif de suppression est écrit dans `analytics.notification_daily` de l'entrepôt par le job marmot `notifications_daily` (lecture de la réplique PostgreSQL, 02:30). Pas de ligne par message dans l'entrepôt : la question « combien d'e-mails de facture ont rebondi en mars » a une réponse, la question « quel e-mail a reçu cet expéditeur le 12 mars » a une réponse dans PostgreSQL pendant 180 jours et pas au-delà. C'est assumé et écrit dans la documentation du support.

## Demandes de suppression de données

Une demande d'effacement d'un utilisateur passe par le processus général de la plateforme ; côté notifications, `notifications:forget <user_id>` remplace `recipient_id` par une valeur de tombstone et vide `payload` sur les lignes restantes, et ajoute l'adresse dans `suppressed_recipients` avec le motif `forgotten` pour qu'aucun envoi futur ne parte vers elle par un chemin anonyme (invitation par un tiers, par exemple). La suppression n'est pas levable par le support, seulement par le responsable des données avec une ligne d'audit.
