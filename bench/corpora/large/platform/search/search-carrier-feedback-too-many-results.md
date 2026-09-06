---
name: search-carrier-feedback-too-many-results
description: Retours transporteurs mars à juin 2026 : la liste bouge au rafraîchissement, longs trajets en tête pour les régionaux, filtres manquants, et les suites
type: feedback
status: active
verified: 2026-07-15
---

# Retours transporteurs : « la liste bouge » et « trop de résultats »

Après la bascule sur haystack en mars 2026, la recherche est devenue rapide, et les transporteurs ont commencé à se plaindre de ce qu'elle renvoyait plutôt que du temps qu'elle mettait. C'est un progrès. Synthèse des 58 tickets `load:search` de mars à juin qui ne concernaient pas un chargement précis, et des réponses libres de l'enquête d'avril.

## Ce qu'ils ont dit

**« La liste change quand je rafraîchis. »** 19 tickets. Un dispatcheur regarde la liste, passe un appel, rafraîchit, et le chargement qu'il voulait montrer à son chauffeur est passé de la 2e à la 7e place. Cause : la composante fraîcheur de la v1 ([[search-relevance-rules-v1]]) avec son échelle de 6 heures faisait qu'un chargement publié entre deux rafraîchissements grimpait au-dessus de tout ce qui était à moins de 50 km. Le classement était « juste » et inutilisable pour quelqu'un qui travaille avec la liste ouverte.

**« Je fais du régional et on me met des Espagne-Pologne en haut. »** 14 tickets, presque tous de transporteurs de moins de dix camions. Cause : le prix au km normalisé globalement favorisait les longs trajets bien payés, quel que soit le profil ([[search-relevance-rules-v1]]). Un transporteur régional voyait la moitié de sa première page en chargements de plus de 500 km.

**« Il me manque un filtre. »** 12 tickets, cinq demandes distinctes : hayon élévateur, longueur utile (ldm) précise, chargement latéral, plusieurs points de livraison, « départ aujourd'hui uniquement ». Les trois premiers n'étaient pas dans le mapping.

**« Trop de résultats, je ne vais pas à la page 10. »** 8 tickets, en général avec un rayon de 500 km ou plus et sans filtre de véhicule. Le retour Ravello du support fait partie de ceux-là.

**« Je ne trouve pas le chargement que le chargeur m'a dit. »** 5 tickets, tous des cas de visibilité `PRIVATE` ou de délai d'indexation, traités par le playbook du support ; pas un problème de pertinence.

## Ce qu'on a fait

- **Fraîcheur** : échelle 12 h, poids divisé par deux dans la v2 ([[search-relevance-rules-v2]]). Depuis juillet, le tri entre deux rafraîchissements à cinq minutes d'écart est stable dans 94 % des cas (mesuré en rejouant des paires de recherches), contre 71 % en v1. Les tickets « la liste bouge » : 2 en juillet.

- **Prix relatif à la ligne** et composante d'adéquation avec les recherches enregistrées : les longs trajets restent visibles mais ne dominent plus la première page d'un régional. Mesuré dans l'expérience de juin ([[search-ab-testing-ranking]]).

- **Règle des 300 km** : au-delà, la distance pèse moins, le prix plus.

- **Filtres** : hayon (`tail_lift`) et chargement latéral (`side_loading`) ajoutés au mapping en juillet ; `ldm` existait, le filtre était caché dans « plus de critères », remonté au premier niveau. « Départ aujourd'hui » est un raccourci de dates, ajouté. Les points de livraison multiples ne sont pas modélisés dans le chargement lui-même, hors périmètre de la recherche, remonté au produit.

- **Avertissement à l'enregistrement** d'une recherche trop large (plus de 200 résultats sur 7 jours), et le point de recherche visible sur la carte avec le rayon.

## Ce qu'on n'a pas fait

- Figer la liste pendant une session (« ne rien bouger tant que je n'ai pas relancé »). Discuté, refusé : un chargement pris par un autre doit disparaître. Le compromis est la fraîcheur adoucie.

- Un tri « par prix » ou « par distance » pur en plus du classement. Le tri par distance existe déjà comme option ; le tri par prix a été refusé parce qu'il ferait ignorer tout le reste, et parce que les chargeurs le savent et gonfleraient les prix indicatifs pour apparaître en haut.

- Pagination au-delà de la page 10 (200 résultats). Au-delà, on propose de réduire le rayon ou d'ajouter un filtre. Personne ne s'en est plaint.

## Ce qu'on a appris

Une recherche lente cache une recherche médiocre. Tant que le spinner tournait, personne ne regardait l'ordre. La v2 a été possible parce que les transporteurs ont enfin eu le temps de nous dire ce qui n'allait pas dans les résultats.

## Verbatims reformulés qui ont pesé

- Un dispatcheur de dix camions dans le Nord : « je cherche 150 km autour de Lille, vous me mettez un Lisbonne-Varsovie en deuxième parce qu'il paie bien ; je n'irai jamais à Lisbonne ». Le cas d'école du prix normalisé globalement.

- Un artisan : « quand le chargement que je regarde descend, je crois qu'il a été pris, alors je clique vite sur un autre, et en fait il était toujours là ». La fraîcheur trop agressive faisait prendre de mauvaises décisions, pas seulement perdre du temps.

- Une transporteur frigo : « le filtre température existe, mais il me montre quand même les chargements frigo à 22 degrés parce qu'ils sont marqués température contrôlée ». `temperature_controlled` est un booléen ; la plage de température n'est pas dans le mapping. Remonté au produit, pas fait.

- Un intégrateur de TMS : « votre classement est très bien, mais mon client veut le même ordre dans son outil et votre API ne renvoie pas le score ». Refusé : le score n'a pas de sens hors contexte, et on ne veut pas que des outils tiers construisent dessus une logique qu'on ne pourra plus changer.

## Suivi

Les tickets `load:search` hors chargement précis sont relus chaque mois par la paire recherche avec le support ; la synthèse suivante est prévue pour décembre 2026, après six mois de v2.
