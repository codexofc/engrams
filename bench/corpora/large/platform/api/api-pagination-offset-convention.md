---
name: api-pagination-offset-convention
description: Historical offset pagination (page=, per_page=) of the v1 API, replaced by cursors in HF-1402
type: reference
status: archived
superseded_by: [[api-pagination-cursor-convention]]
verified: 2025-10-06
---

# Pagination par offset (v1, obsolète)

Les endpoints de collection de l'API v1 paginaient avec `page` (à partir de 1) et `per_page` (max 100, défaut 25). La réponse contenait `meta.total`, `meta.page`, `meta.per_page` et `meta.total_pages`.

Problèmes rencontrés qui ont mené au remplacement :

- `OFFSET 40000` sur `bids` prenait 1,2 s en prod dès qu'un transporteur avec beaucoup d'historique paginait jusqu'au bout (le client mobile d'un transporteur belge faisait exactement ça toutes les 15 minutes).
- Le `COUNT(*)` pour `meta.total` doublait le coût de chaque page.
- Insertion entre deux pages = doublons ou trous côté client. Le mobile compensait avec un `Set` d'identifiants, ce qui masquait le problème sans le résoudre.

Le remplacement est décrit dans [[api-pagination-cursor-convention]]. Les endpoints v1 répondent encore avec offset jusqu'à la fin de la période de dépréciation, voir [[api-deprecation-policy]].
