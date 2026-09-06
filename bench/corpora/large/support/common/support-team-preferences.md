---
name: support-team-preferences
description: Préférences de l'équipe support (six personnes) : identifiants avant tout, hfctl plutôt que SQL, réponses courtes en langue du client, pas de promesse de délai technique, playbooks courts
type: user
status: active
verified: 2026-04-08
---

Préférences de l'équipe support, six personnes, écrites après un an de Deskline et deux changements d'outillage.

- **Un identifiant d'abord.** Pas de diagnostic sans `org_id` et sans l'identifiant de l'objet (chargement, facture, chauffeur). La macro `need-reference` n'est pas impolie, elle est honnête.

- **`hfctl` plutôt que SQL, SQL plutôt que supposition.** Si la commande existe, on l'utilise, elle laisse une trace. Si elle n'existe pas, on regarde la réplique, et on note la requête pour HF-3140. On ne devine jamais l'état d'un objet.

- **On répond dans la langue du ticket**, français, anglais, allemand. Polonais et tchèque en anglais, avec la phrase d'excuse standard. Pas de traducteur automatique dans une réponse client sans relecture.

- **Court.** Une réponse client tient en cinq lignes : ce qu'on a vu, ce qu'on a fait, ce que le client doit faire, quand on revient vers lui. Le détail technique reste dans la note interne.

- **Aucune promesse de délai technique.** On dit « c'est chez l'équipe technique avec la référence HF-xxxx », jamais « ce sera corrigé demain ». Le seul délai qu'on promet est celui du SLA ([[support-sla-tiers]]).

- **Les playbooks se lisent en deux minutes.** Un playbook, c'est une liste de vérifications dans l'ordre, chaque vérification avec la commande et ce qu'on attend, et une sortie claire : résolu, macro, escalade. Le pourquoi tient en trois lignes en bas, pas en haut.

- **On ne clique pas pour le client.** Depuis HF-3001, l'impersonation est en lecture seule et ça nous convient. Si le client doit faire une action, on lui dit où est le bouton.

- **On écrit les cas mémorables** dans `support/cases` dans la semaine, anonymisés, avec ce que ça nous a appris. Pas de nom de client, un nom de transporteur inventé.

- **Le lundi 14 h est sacré** ([[support-weekly-triage-ritual]]). On ne prend pas de rendez-vous client à cette heure.

- **Le vendredi après 16 h, on n'applique rien** qui touche un chargement `IN_TRANSIT`. Ça attend lundi ou l'astreinte décide.
