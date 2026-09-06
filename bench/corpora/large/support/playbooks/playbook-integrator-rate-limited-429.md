---
name: playbook-integrator-rate-limited-429
description: Intégrateur en 429 : limites par organisation (600 lecture, 120 écriture, 20 recherches par minute), Retry-After, webhooks, exemption par flag
type: reference
status: active
verified: 2026-06-18
---

# L'intégrateur reçoit des 429

Catégorie `integration:webhook` (sous-tag `ratelimit`), faute de mieux. Un chargeur ou un transporteur qui a écrit sa propre intégration voit des `429 Too Many Requests` et pense que « l'API est en panne ».

## Les limites

Par organisation, toutes clés API confondues, fenêtre glissante d'une minute :

- 600 requêtes de lecture (`GET`)

- 120 requêtes d'écriture (`POST`, `PATCH`, `DELETE`)

- 20 appels à `GET /v2/loads/search` (la recherche coûte cher, elle a sa propre limite)

Chaque réponse porte `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`. Un 429 porte `Retry-After` en secondes. C'est dans la doc, et personne ne la lit avant le premier 429.

## Vérifications

1. Grafana, tableau « Org overview », `org_id` de l'intégrateur, 24 h. Le panneau « requests by route » montre ce qu'ils font. Neuf fois sur dix : un `GET /v2/loads/{id}` toutes les 5 secondes sur chaque chargement actif, ou `GET /v2/loads?status=DISPATCHED` toutes les 10 secondes. C'est du polling qui devrait être un webhook.

2. Le panneau « 429 by route » confirme la route qui sature.

3. Si c'est la recherche : 20 par minute, un tableau de bord qui rafraîchit toutes les 3 secondes explose la limite tout seul.

## Réponse

Macro `api-rate-limit-webhooks` : les limites, la lecture de `Retry-After`, et la recommandation de s'abonner aux webhooks pour les changements d'état (voir [[playbook-webhook-not-received]] s'ils en ont déjà un qui ne marche pas). Le polling reste acceptable en secours, à deux minutes d'intervalle, pas cinq secondes.

Pour un intégrateur qui fait une migration de données (import initial de milliers de chargements), l'exemption temporaire existe : flag `ratelimit.exempt_orgs`, posé par le backend sur ticket HF, pour une durée écrite (une semaine en général). Le support ne le pose pas, il ouvre le ticket avec l'`org_id`, la raison et la date de fin. On l'a fait cinq fois en 2026, jamais plus de trois semaines.

## Ce qu'on n'accorde pas

- Une limite plus haute « parce qu'on est Enterprise ». Les limites sont les mêmes pour tous, l'exemption est temporaire et motivée.

- Une clé API supplémentaire pour doubler la limite. La limite est par organisation, pas par clé, précisément pour ça.

- Un débit garanti. On ne promet pas de capacité.

## Écritures en 429

Plus rare et plus grave : un intégrateur qui poste des offres en rafale (un transporteur qui mise sur tout ce qui bouge) atteint 120 écritures par minute. C'est en général voulu de leur côté, et ça pose la question commerciale du bot de mise, qui n'est pas au support. Signaler au responsable de compte du transporteur, ne rien changer.

## Escalade

Ticket HF pour l'exemption. Backend si des 429 apparaissent chez plusieurs organisations qui ne polluent pas : la limite se calcule dans Redis et un problème de Redis fait des faux 429, vu une fois en avril 2026.
