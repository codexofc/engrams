---
name: gitleaks-precommit-feedback
description: Three months of gitleaks pre-commit on 40 machines blocked 11 real secrets and 130 false positives; tuning IBAN and UUID rules cut noise from 22 to 3 a week
type: feedback
status: active
verified: 2026-03-31
---

# Retour sur trois mois de gitleaks en pre-commit

Déployé sur les postes en décembre 2025 (HF-2085) via le script d'installation du dépôt `halden-dev-tools`, après l'incident du log de build. Bilan à fin mars 2026, à partir des remontées dans `#signalement-securite` et du compteur de contournements.

## Ce qui a été bloqué

**Onze vrais secrets** en trois mois, tous avant de quitter le poste :

- 4 clés `hfk_` de staging dans des fichiers de test ou des collections de requêtes HTTP exportées ;

- 3 secrets de fournisseurs (2 sandbox de paiement, 1 clé de test d'un fournisseur télématique) dans des `.env` non ignorés d'un nouveau sous-projet ;

- 2 jetons JWT de staging collés dans un fichier de notes personnel ajouté par erreur au dépôt ;

- 1 clé privée de compte de service de dev dans un script ;

- 1 mot de passe de base de données locale, sans importance, mais le motif générique l'a pris.

Aucun n'aurait été un incident majeur (staging, sandbox, dev), et c'est exactement le raisonnement qu'on refuse (voir [[secrets-handling-conventions]]) : ils auraient tous fini dans l'historique Git, et deux des dépôts concernés sont clonés par des prestataires.

## Les faux positifs

**130 en trois mois**, 22 par semaine au début, 3 par semaine en mars. Les causes, dans l'ordre :

1. **IBAN dans les fixtures** (48 cas). La règle maison sur les IBAN attrapait les IBAN de test des fixtures de facturation. Correction : les fixtures utilisent des IBAN de test officiels (structure valide, banque fictive) listés dans une allow-list du `gitleaks.toml`, et la règle ignore le dossier `fixtures/`.

2. **UUID v4 lus comme des secrets hexadécimaux** (39 cas). La règle générique « 32 hexadécimaux » prenait les identifiants. Désactivée au profit de nos préfixes et des formats fournisseurs, qui suffisent : un secret sans forme reconnaissable n'est pas censé exister chez nous.

3. **Hash de lockfiles** (21 cas) : `integrity`, `sha512-...`. Chemins `*.lock`, `package-lock.json` exclus.

4. **Exemples dans la documentation** (14 cas) : `hfk_EXAMPLE_...`. Convention : les exemples de doc utilisent `hfk_000000_` suivi de zéros, allow-listé.

5. **Divers** (8) : une clé publique PEM (règle trop large corrigée pour ne prendre que `PRIVATE KEY`), des jetons d'exemple dans une spec OpenAPI.

## Le compteur de contournements

`git commit --no-verify` marche, on ne l'empêche pas : un hook qu'on ne peut pas contourner est un hook qu'on désinstalle. Mais le hook écrit un fichier local à chaque contournement et le script `dev-tools doctor` (lancé par la CI sur le poste, c'est-à-dire jamais, et par les gens eux-mêmes) l'affiche. Plus utile : la CI refait la même analyse sur la MR, donc un contournement local est rattrapé au push, et le message de la CI dit « ce secret a été contourné localement le <date> ».

Quatorze contournements déclarés en trois mois, 11 pendant la période des faux positifs sur IBAN, 3 en mars pour un même développeur qui travaillait hors ligne sur des fixtures et a fait ajouter l'allow-list ensuite.

## Ce qu'on retient

- Un scanner de secrets vaut par la précision de ses règles, pas par leur nombre. Les règles génériques « quelque chose qui ressemble à de l'entropie » ont produit 60 % des faux positifs et zéro vrai secret que nos règles spécifiques n'auraient pas eu.

- Le rythme d'ajustement compte : les deux premières semaines, quelqu'un de la rota ([[security-oncall-rota]]) regardait chaque remontée le jour même et modifiait la règle. Sans ça, le hook aurait été désinstallé par la moitié de l'équipe avant Noël.

- Il faut dire aux gens que le hook bloquera des choses inoffensives et que c'est le prix. Le message d'erreur du hook a été réécrit en janvier : ce qu'il a trouvé, pourquoi c'est peut-être un faux positif, comment demander une exception, et la phrase « si c'est un vrai secret, il est déjà considéré fuité, écris dans #signalement-securite ».

- Onze secrets en trois mois pour quarante développeurs : le problème existe même chez des gens prudents, et il existe en amont de Git.
