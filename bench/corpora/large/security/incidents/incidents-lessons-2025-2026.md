---
name: incidents-lessons-2025-2026
description: Across 6 incidents in 12 months: duplicated security code paths, test secrets treated as harmless, detection by outsiders, and old cheap controls that held
type: feedback
status: active
verified: 2026-07-10
---

# Ce que douze mois d'incidents nous ont appris

Relecture transversale des six post-mortems d'octobre 2025 à juin 2026, faite en juillet 2026 pour la revue semestrielle. Ce ne sont pas les causes de chaque incident (elles sont dans chaque note) mais ce qui revient.

## Les schémas qui se répètent

**Le même comportement de sécurité implémenté à deux endroits.** Les URL présignées ([[incident-2026-01-presigned-url-ttl-leak]]) : deux fabriques, une corrigée. Les buckets ([[incident-2026-02-pod-bucket-public]]) : deux modules copiés, différenciés par une ligne. La leçon tient en une règle qu'on applique maintenant en revue : un comportement de sécurité a **une** implémentation, et un test d'architecture empêche la seconde. `DependencyRulesTest` en compte sept en juillet 2026.

**« Test », « sandbox », « staging » lus comme « sans importance ».** Le log de build ([[incident-2025-12-test-credentials-build-log]]) exposait un secret de sandbox de paiement et un mot de passe de staging. Le `staff_admin` périmé (projet IAM) avait une clé sur une organisation de test. À chaque fois, quelqu'un avait pensé « c'est du test ». La règle : un secret est un secret de production jusqu'à preuve du contraire, et la preuve du contraire n'est jamais « c'est dans le nom ».

**On apprend l'exposition de l'extérieur.** Bucket public : un chercheur. URL présignées : l'équipe IT d'un client. Typosquat ([[incident-2026-03-dependency-typosquat]]) : un relecteur attentif, ce qui est interne mais pas automatisé. Dans trois cas sur six, la détection était humaine et externe. Les actions « améliore la détection » des post-mortems (sonde de bucket externe, scanner de sortie CI, vérification de dérive) sont la réponse, et la règle du gabarit ([[postmortem-template-rules]]) qui exige des actions de détection vient de là.

**Les mesures qui ont tenu étaient anciennes et bon marché.** Le refroidissement de 72 h sur l'IBAN a bloqué la prise de contrôle de compte ([[incident-2026-05-support-account-takeover]]). `previous_secret_hash` a permis de revenir en arrière sur la rotation des webhooks ([[incident-2026-06-webhook-secret-rotation-gap]]). Les MR du bot qui ne lançaient pas `npm ci` (une mesure d'économie de CI) ont empêché l'exécution du paquet malveillant. Aucune de ces mesures n'était « de la sécurité » au moment où elle a été prise. Conclusion : un délai, un chemin de retour et un « on n'exécute rien avant approbation » valent plus que la plupart des outils.

## Ce qui a changé dans la façon de répondre

- Le temps de containment médian est passé de 40 minutes (deux incidents de 2025, reconstruits) à 14 minutes (quatre incidents de 2026). La différence est le processus ([[incident-process-severity-levels]]) : un commandant nommé et une chronologie tenue par quelqu'un qui ne répare pas.

- On notifie plus, et plus vite : 51 h pour l'autorité en février, l'exercice de mars ([[security-incident-drill-2026-03]]) a ramené la cible à 48 h avec les outils prêts.

- Les actions sont fermées : 34 actions issues des six post-mortems, 31 closes au 2026-07-10, 3 en cours avec une échéance. Contre 12 sur 31 pour toute la période 2024 à 2025.

## Ce qu'on n'a pas encore résolu

- **Le second facteur côté client.** Deux incidents (phishing côté staff, prise de contrôle côté client) auraient été stoppés par un second facteur. Le staff en a depuis janvier ; les clients, non. Ce n'est plus une question de sécurité, c'est une question de produit (comptes partagés par agence) et elle est ouverte depuis un an.

- **La dépendance à la vigilance humaine** pour le typosquat et le log de build. Les outils ajoutés réduisent la surface, mais le prochain incident de cette classe sera un cas que le motif ne connaît pas.

## La phrase qu'on garde

Du post-mortem du phishing, à propos de la personne qui a écrit « je crois que j'ai cliqué » 33 minutes après : c'est la meilleure détection qu'on ait eue en douze mois, et elle ne coûte rien tant qu'on ne punit personne pour l'avoir dit.
