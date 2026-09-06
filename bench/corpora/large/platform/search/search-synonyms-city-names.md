---
name: search-synonyms-city-names
description: Fichier de synonymes des villes et de la marchandise, rechargé sans réindexation depuis HF-3165, maintenu à la main, 40 groupes issus des tickets
type: project
status: active
verified: 2026-06-19
---

# Synonymes : villes et marchandise

Un transporteur belge tape « Anvers », un chargeur flamand a saisi « Antwerpen », un chargeur allemand « Antwerpen » aussi, un Français « Anvers ». Sans synonymes, la recherche par ville exacte (le repli quand le rayon ne s'applique pas, [[search-geo-radius-queries]]) et le texte de la marchandise ratent la moitié des cas. Le rayon géographique ne sait pas lire, mais la ville tapée doit d'abord être géocodée, et le géocodeur a le même problème.

## Le fichier

`synonyms/cities.txt` dans le dépôt de l'indexeur, format Solr, une ligne par groupe :

```
köln, koln, cologne, colonia, keulen
anvers, antwerpen, antwerp, amberes
milan, milano, mailand
liège, liege, luik, lüttich
aix-la-chapelle, aachen, aken
```

Environ 320 groupes en juin 2026 : les grandes villes européennes avec leurs exonymes dans les langues de nos utilisateurs (français, allemand, néerlandais, polonais, tchèque, italien, espagnol, portugais, anglais), plus les graphies sans accent (le filtre `asciifolding` en couvre l'essentiel, mais « ß » vers « ss » et « ł » vers « l » ne suffisent pas pour « Wrocław » que les Allemands tapent « Breslau »).

Second fichier, `synonyms/goods.txt`, une centaine de groupes pour le champ `goods` : « palettes europe, europalettes, epal, euro pallets », « frigo, réfrigéré, reefer, kühl », « big bag, bigbag, fibc ». Moins critique, le texte de marchandise est rarement le critère principal.

## Chargement

Les deux fichiers sont des filtres `synonym_graph` dans les analyseurs `city` et `goods` du mapping ([[search-index-mapping-loads]]). Jusqu'en mai 2026, un changement de synonymes exigeait un nouvel index et une réindexation ([[search-reindex-runbook]]) : 40 minutes pour ajouter une ligne, donc on attendait d'en avoir dix.

HF-3165 : les filtres sont déclarés `updateable: true` et utilisés **uniquement au moment de la recherche** (`search_analyzer`), pas à l'indexation. Un changement se déploie par `POST loads-v7/_reload_search_analyzers` après avoir poussé le fichier sur les nœuds (un ConfigMap monté, un job qui appelle le rechargement). Deux minutes, pas de réindexation. La contrepartie : les synonymes ne s'appliquent qu'à la requête, ce qui est correct pour notre usage (le document contient « Antwerpen », la requête « anvers » est étendue en « anvers, antwerpen, antwerp, amberes » et trouve).

Le même fichier de villes alimente le géocodeur de l'API (résolution d'une ville tapée en coordonnées), chargé au démarrage et rechargé toutes les heures. Une seule source, deux consommateurs.

## Qui maintient

La paire recherche. Les ajouts viennent de trois sources :

- Les tickets `load:search` du support où la cause est une graphie : 40 groupes ajoutés depuis mars 2026, dont « Bruxelles/Brussel/Brüssel/Brussels » qui manquait, ce qui a fait rire tout le monde et pas le transporteur.

- Le journal des recherches sans résultat par ville (`search_zero_results` dans l'entrepôt, agrégé par terme tapé) : chaque mois, les 20 termes les plus fréquents sont relus ; s'ils sont une ville mal orthographiée ou un exonyme, ajout.

- Les chargeurs qui saisissent des noms locaux inattendus (« Lisboa » chez un chargeur portugais, « Lisbonne » chez le transporteur français).

Pas d'ajout automatique. Une pull request, une ligne, un test : `tests/synonyms/cities_test.yaml` liste pour chaque groupe une requête et le nom attendu, et le test les passe dans l'analyseur d'un nœud local.

## Ce qu'on ne fait pas

- Pas de correction orthographique floue (`fuzziness`) sur les villes : « Metz » et « Mety » sont proches, « Lens » et « Lenz » aussi, et les faux positifs sur des noms courts coûtent plus que les fautes ratées. Le géocodeur propose une correction à la saisie, avant la recherche.

- Pas de synonymes de pays ou de régions : « Bavière » n'est pas une ville, le filtre par pays et le rayon couvrent.

- Pas de traduction automatique de la marchandise : « steel coils » et « bobines d'acier » sont dans le fichier parce qu'un humain les a mises.

## Chiffres

Recherches par ville sans résultat : 6,1 % en mars 2026, 2,3 % en juin. Les 3,8 points de différence sont pour l'essentiel les 40 groupes ajoutés. Le reste est des villes réellement sans chargement ce jour-là, ce qui n'est pas un problème de synonymes.

## Cas particuliers dans le fichier

- **Villes homonymes** : « Frankfurt » désigne Frankfurt am Main et Frankfurt (Oder), à 550 km l'une de l'autre. Le fichier ne lie pas « frankfurt » à l'une des deux ; le géocodeur propose les deux à la saisie et le transporteur choisit. Idem « Neustadt » (une douzaine), « Villeneuve ». Un synonyme ne doit jamais rapprocher deux lieux distincts.

- **Villes fusionnées ou renommées** : « Charleroi » et ses sections, « Saint-Étienne » et « Saint-Etienne » (couvert par `asciifolding`), les communes néerlandaises fusionnées en 2019 et 2022 dont les chargeurs utilisent encore l'ancien nom. Ajoutées quand un ticket les remonte, jamais par anticipation.

- **Zones industrielles et ports** : « Port de Rotterdam », « Rotterdam Maasvlakte », « Europoort » sont dans le même groupe que « Rotterdam » parce qu'un transporteur qui cherche Rotterdam veut voir le port. Le rayon fait le reste. Idem « Le Havre » et « Port 2000 », « Antwerpen » et « Zeebrugge » **non** (deux ports, 90 km).

- **Abréviations** : « Ffm » pour Frankfurt am Main, « Bxl » pour Bruxelles, « Mtp » pour Montpellier, ajoutées parce que des chargeurs les tapent dans le champ ville de leur import.

## Procédure d'ajout

1. Une ligne dans `synonyms/cities.txt`, dans l'ordre alphabétique du premier terme.

2. Une entrée dans `tests/synonyms/cities_test.yaml` : le terme tapé et le `pickup.city.raw` qu'il doit trouver.

3. `make synonyms-test`, qui monte un nœud local, charge le fichier et passe tous les cas (environ 8 s).

4. Pull request relue par l'autre personne de la paire, fusion, déploiement du ConfigMap, job de rechargement. Le fichier du géocodeur se recharge dans l'heure.

Une modification qui retire un terme suit le même chemin et le test doit alors vérifier que l'ancien terme ne trouve plus.
