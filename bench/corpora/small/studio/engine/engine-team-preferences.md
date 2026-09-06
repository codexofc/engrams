---
name: engine-team-preferences
description: Engine team preferences: measure on the low-end target first, unsafe with a named invariant, no dependency without a build figure
type: user
status: active
verified: 2026-02-12
---

Préférences de l'équipe moteur (quatre personnes), telles qu'on les applique en revue.

- **On mesure sur la cible basse d'abord.** Un chiffre de performance donné sans préciser la machine ne compte pas. La scène de référence est `port_nuit`, et le tableau de budget est celui de [[ecs-scheduler]] pour le CPU et [[renderer-frame-graph]] pour le GPU.
- **`unsafe` avec un commentaire qui nomme l'invariant**, pas « SAFETY: ok ». Le lint maison compte les blocs `unsafe` par crate et la revue regarde le compteur quand il monte.
- **Pas de nouvelle dépendance sans le temps de build qu'elle ajoute** (`cargo build --timings` avant et après). Une dépendance qui ajoute plus de 5 s au build froid a besoin d'une raison écrite. Voir [[build-times-incremental]].
- **L'overlay `F3` avant le profileur.** Budgets, allocations par système, fallbacks de shaders, variantes manquantes, underruns audio : si ce n'est pas dans l'overlay, on l'y ajoute.
- **Un test qui joue des images** vaut mieux qu'un test unitaire pour le moteur : `no_alloc_in_hot_systems`, les 40 captures de référence du rendu, le replay des bugs QA.
- **Français ou anglais** au choix dans les notes et les commentaires ; les identifiants en anglais ; les messages d'erreur du moteur en anglais.
- **Revue** : une personne suffit pour le code moteur, deux pour tout ce qui touche `unsafe`, les allocateurs ou le backend GPU.
- **Vendredi après-midi** : pas de fusion dans `givre-render` ni `givre-platform`.
