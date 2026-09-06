---
name: incident-comms-templates
description: Six bilingual incident message templates (declaration, staff, controller notice, individuals, researcher reply, status page); every fact from the timeline
type: reference
status: active
verified: 2026-05-25
---

# Modèles de communication d'incident

Écrire sous pression produit des phrases qu'on regrette (« aucune donnée n'a été compromise » avant d'avoir lu les logs). Les modèles sont dans `halden-security/comms/`, en français et en anglais, et la personne au rôle communication ([[incident-process-severity-levels]]) part toujours d'un modèle.

## Les six modèles

**1. `declare.md`, déclaration interne.** Le message épinglé du canal `#inc-HF-xxxx` : titre, sévérité provisoire, commandant, scribe, ce qu'on sait, ce qu'on ne sait pas, prochain point dans 30 minutes. Généré par la commande `/incident`, complété à la main.

**2. `staff-headsup.md`, information à tout le staff.** Pour un SEV1 ou un SEV2 visible (un bucket public, un phishing en cours). Trois paragraphes : ce qui se passe en une phrase, ce qu'on attend de chacun (ne pas cliquer, signaler, ne pas répondre à la presse), où suivre. Envoyé sur le canal général par le commandant. Pas de détail technique, pas de nom.

**3. `customer-processor.md`, avis au responsable de traitement.** Quand nous sommes sous-traitant (données des chauffeurs téléversées par un transporteur, personnes de contact EDI d'un chargeur). Structure imposée par le DPA : nature de la violation, catégories et volume approximatif, conséquences probables, mesures prises, point de contact, et la phrase « vous êtes responsable de traitement pour ces données ; nous vous fournissons ci-joint un texte que vous pouvez utiliser pour informer les personnes ». Le texte pour les personnes est le modèle 4 adapté. Envoyé sous 48 h (24 h pour les variantes de DPA), depuis `privacy@halden.example`, avec copie au commercial du compte. Utilisé pour la première fois lors de [[incident-2026-02-pod-bucket-public]] (41 transporteurs).

**4. `customer-controller.md`, avis aux personnes ou aux clients quand nous sommes responsable.** Ce qui s'est passé, quelles données, ce qu'on a fait, ce que la personne peut faire, comment nous joindre, et comment contester. En français et en anglais dans le même e-mail pour les chauffeurs, la moitié ne lit pas le français.

**5. `researcher-reply.md`, réponse à un signalement externe.** Accusé de réception sous 4 heures ouvrées : merci, on a reproduit ou pas, ce qu'on fait, on revient vers vous sous 5 jours ouvrés avec le résultat. Pas de « nous prenons la sécurité très au sérieux ». Question posée à la fin : souhaitez-vous être cité dans la note de remerciement publique. Depuis février 2026 : 4 signalements, 2 fondés.

**6. `status.md`, page de statut.** Pour un incident qui dégrade le service (une rotation de clé qui casse les liens pendant une heure, voir [[incident-2026-01-presigned-url-ttl-leak]]). Court, factuel, horodaté, mis à jour toutes les 30 minutes tant que ça dure.

## Règles d'écriture

- **Chaque fait vient d'une ligne de la chronologie** du scribe ([[incident-timeline-tooling]]). Si le fait n'y est pas, il ne va pas dans le message. C'est ce qui empêche « aucune donnée n'a été accédée » : la chronologie dit « journaux d'accès relus par deux personnes, 212 lectures », le message dit ça.

- **« Nous n'avons pas de preuve de » plutôt que « il n'y a pas eu de »**, jusqu'à ce que deux personnes aient lu les journaux et que le commandant l'écrive.

- **Pas de nom de personne**, ni chez nous ni chez l'attaquant. Des rôles.

- **Pas de secret**, même révoqué. Préfixe et quatre derniers caractères au maximum.

- **Un seul point de contact** par message, une adresse partagée, jamais l'adresse personnelle du commandant.

- **Relu par le commandant** avant envoi, et par le DPO pour les modèles 3 et 4. Le DPO a une heure ; passé ce délai en SEV1, le commandant envoie et le dit.

## Ce qu'on a ajouté après les incidents

- Après le phishing d'octobre 2025 : le modèle 2 et la phrase attendue « je crois que j'ai cliqué » dans `#signalement-securite`.

- Après le bucket public : le modèle 3 en version bilingue et le texte prêt pour les personnes, parce que les transporteurs ont demandé « qu'est-ce qu'on dit à nos chauffeurs ».

- Après [[incident-2026-05-support-account-takeover]] : un modèle 4 spécifique « votre compte a été utilisé par un tiers », avec la liste de ce que l'attaquant a pu voir et la recommandation de changer les mots de passe réutilisés.
