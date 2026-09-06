---
name: webhook-team-preferences
description: Préférences de l'équipe intégrations : l'outbox seule porte, additif par version, fixtures manuelles, 30 jours d'annonce, le client voit ce qu'on voit
type: user
status: active
verified: 2026-07-01
---

Préférences de l'équipe intégrations (trois personnes, dont une à mi-temps sur les partenaires), après six mois d'outbox et deux incidents.

- **L'outbox est la seule porte.** Aucun code métier ne fait de HTTP vers un client. Il écrit une ligne dans `sys_outbox` dans sa transaction, point. Une pull request qui instancie un client HTTP hors du relais est refusée en revue sans discussion.

- **Additif dans une version, jamais autre chose.** Un champ s'ajoute. Il ne se renomme pas, ne change pas de type, ne devient pas un objet. Ce qui change de forme attend une version ([[webhook-payload-versioning-v2-v3]]).

- **Les fixtures se tapent à la main.** Une fixture régénérée par le code qu'elle teste ne teste rien. Depuis juin 2026 c'est écrit dans le README des tests et vérifié en revue.

- **Trente jours d'annonce** pour tout ce qui touche à la forme d'un payload, aux adresses de sortie, au calendrier des relances. Le changelog intégrateur est un fichier du dépôt, publié à chaque déploiement ; on le relit avant de merger.

- **Le client voit ce qu'on voit.** Une erreur stockée dans `last_error` s'affiche dans son écran, en mots. Un compteur qu'on a, il l'a. Pas d'information « support seulement » sur ses propres livraisons.

- **Une seule manière d'authentifier** : la signature. Pas de mTLS, pas d'en-tête sur mesure, pas d'option par client ([[webhook-mtls-request-declined]]). Chaque option est un chemin de code de plus dans le relais, et le relais doit rester ennuyeux.

- **La sonde, c'est le travail qui avance**, pas le processus qui tourne. L'âge du plus vieux `PENDING` est la métrique de santé, tout le reste est du confort.

- **On mesure avant de changer un nombre.** Le passage de 7 à 10 tentatives a attendu quatre mois de données sur les livraisons mortes. Un chiffre choisi au doigt mouillé se change au doigt mouillé six mois plus tard.

- **Le support est notre premier utilisateur.** Si le support ne peut pas diagnostiquer un problème de livraison avec `hfctl` et l'écran, c'est nous qui avons un problème, pas eux.

- **Langue** : les notes internes en français ou en anglais selon l'auteur, le guide intégrateur en anglais uniquement, les identifiants et les codes d'erreur en anglais toujours.

- **Pas de fonctionnalité pour un seul client.** Trois demandes, ou une obligation réglementaire, sinon c'est une lettre d'explication et pas du code.
