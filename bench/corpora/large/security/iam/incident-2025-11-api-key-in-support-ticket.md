---
name: incident-2025-11-api-key-in-support-ticket
description: On 2025-11-18 a support agent pasted a carrier's full API key into a shared ticket; rotated in 2 h, led to hashed storage and the ticket-scrubbing hook
type: project
status: active
verified: 2025-12-15
---

# Clé API d'un transporteur collée dans un ticket (novembre 2025)

## Chronologie

- **2025-11-18 10:42** : un transporteur écrit au support parce que son TMS reçoit des 401. L'agent L1 ouvre la page de l'organisation dans le back-office, qui à l'époque affichait les clés en clair (voir [[api-keys-legacy-plaintext]]), copie la clé complète et la colle dans le ticket avec « voici votre clé, vérifiez qu'elle est bien celle configurée ». Le ticket est dans l'outil de support partagé, lisible par les 40 comptes staff qui y ont accès, et par le prestataire de support de premier niveau externalisé le week-end (3 personnes).

- **10:58** : le transporteur répond, le problème était une faute de frappe de son côté, résolu.

- **14:20** : un développeur qui cherche autre chose dans l'outil de support tombe sur le ticket, reconnaît une clé (40 hexadécimaux, voir la note archivée), prévient le canal sécurité.

- **14:35** : la clé est désactivée et une nouvelle est créée par le support avec le transporteur au téléphone. Le transporteur la configure à 15:10. Coupure d'environ 35 minutes pour son intégration, prévenue et acceptée.

- **15:00** : le ticket est édité pour retirer la clé. L'outil de support garde un historique des modifications, la clé y reste lisible pour les administrateurs de l'outil. Demande au fournisseur de l'outil de purger l'historique, faite le 2025-11-20.

- **Vérification** : `audit_events` et les logs d'accès ne montrent aucun appel avec l'ancienne clé entre 10:42 et 14:35 depuis une IP autre que celles habituelles du transporteur. Rien n'indique une utilisation abusive.

## Cause

La cause immédiate est un geste de support parfaitement compréhensible : l'outil affichait la clé, l'agent voulait aider. La cause réelle est que l'outil affichait la clé. Le stockage en clair rendait ce geste possible, et rien dans le processus ne disait de ne pas le faire.

## Correctifs

1. **Stockage haché, affichage du préfixe seul** : HF-2087, voir [[api-key-hashing-and-prefix]]. Depuis, le back-office ne peut physiquement plus afficher une clé.

2. **Permission `apikey.reveal`** réservée à `staff_admin`, qui ne révèle que le préfixe, tracée dans l'audit. Le L1 n'a plus besoin de voir quoi que ce soit : `apikey:lookup <préfixe>` répond à la question « est-ce la bonne clé ».

3. **Hook de nettoyage dans l'outil de support** : un webhook sortant sur chaque commentaire créé passe le texte à `SecretPatternScanner` (les mêmes motifs que notre configuration gitleaks : `hfk_`, clés du fournisseur de paiement, JWT). Si un motif correspond, le commentaire est remplacé par « [contenu retiré : secret détecté, voir canal sécurité] » et une alerte part. Depuis décembre : 6 déclenchements, dont 4 étaient des clients qui collaient leur propre clé dans le ticket, ce qu'on ne peut pas empêcher mais qu'on peut nettoyer.

4. **Fiche support** « Un client dit que sa clé ne marche pas » : réécrite. On ne renvoie jamais une clé, on fait créer une nouvelle clé au client par la rotation décrite dans [[api-keys-lifecycle]].

## Ce qui n'a pas été fait

On n'a pas sanctionné l'agent. La note interne le dit explicitement : le système permettait le geste, le geste a été fait. La conversation post-incident a duré 30 minutes et a produit les quatre points ci-dessus.

Ticket : HF-2079.
