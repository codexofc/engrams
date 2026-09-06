---
name: dispatchers-want-dense-ui
description: Dispatchers asked for a dense mode (32 px rows, 8 px padding, 12 px text), keyboard shortcuts for everything they do more than 20 times a day, and no confirmation dialogs except for cancellation
type: user
status: active
verified: 2026-02-17
---

# Ce que les dispatchers veulent de l'interface

Retours de 4 sessions d'observation sur poste (2025 et janvier 2026, 3 chargeurs, 9 dispatchers) et de l'enquête trimestrielle. À l'opposé des chauffeurs sur mobile : ici l'utilisateur est assis, avec deux écrans, huit heures par jour, et il veut voir le plus de choses possible.

- **Mode dense par défaut pour les utilisateurs expérimentés.** Lignes de 32 px au lieu de 44, remplissage à 8 px, texte à 12 px. Activé par `data-density="dense"` (voir [[design-tokens-and-theming]]), mémorisé dans le profil. 80 % des dispatchers actifs l'ont activé dans le mois qui a suivi. Les cibles cliquables restent à 32 px, ce qui est le minimum accepté par l'audit d'accessibilité.

- **Raccourcis clavier** pour tout ce qui se fait plus de 20 fois par jour : `/` pour le filtre rapide, `A` pour affecter le chargement sélectionné, `J`/`K` pour passer d'un chargement à l'autre, `O` pour ouvrir le panneau, `Échap` pour le fermer, `G` puis `B` pour aller au tableau, `G` puis `M` pour la carte. La liste est affichée par `?`. Les raccourcis ne se déclenchent pas quand un champ a le focus.

- **Pas de boîte de confirmation** sauf pour l'annulation d'un chargement et le retrait d'un transporteur. Tout le reste est réversible ou annulable par un toast "Annuler" pendant 6 s (voir [[error-boundary-and-toasts]]). Les dispatchers cliquaient "Oui" sans lire, la confirmation ne protégeait rien.

- **Deux écrans** : le tableau sur l'un, la carte ou la conversation sur l'autre. Donc les routes `/map` et `/messages` doivent fonctionner dans un onglet séparé sans perdre le contexte, ce qui est une des raisons de la synchronisation entre onglets par `BroadcastChannel` (voir [[websocket-live-updates]]).

- **Le nombre de résultats compte**, même approximatif. "50+" a été accepté, "beaucoup" non.

- **Les couleurs de statut doivent être les mêmes partout**, tableau, carte, panneau, e-mails. Un dispatcher a demandé pourquoi "en transit" était bleu sur le tableau et vert sur la carte, il avait raison.

- **Pas d'animation** au-delà de 150 ms. Le panneau qui glissait en 300 ms a été jugé lent par 7 personnes sur 9.

Ce que les dispatchers ne veulent pas : un moteur de suggestion de transporteurs (testé en 2025 sous forme de suggestion, ignoré par 90 % des utilisateurs qui ont leurs habitudes), et des tableaux de bord de statistiques sur la page d'accueil (ils vont directement au tableau).
