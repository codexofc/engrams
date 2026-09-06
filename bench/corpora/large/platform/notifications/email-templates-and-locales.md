---
name: email-templates-and-locales
description: Templates e-mail en MJML dans templates/notifications/<event>/<locale>.mjml, 11 locales avec repli sur en, compilées au build, variables typées par un schéma par événement
type: reference
status: active
verified: 2026-05-20
---

## Où vivent les templates

Un dossier par type d'événement, un fichier par locale et par canal :

```
templates/notifications/
  bid.received/
    en.mjml  fr.mjml  de.mjml  pl.mjml  ...
    sms.en.txt  sms.fr.txt
    push.en.yaml  push.fr.yaml
    schema.yaml
```

`schema.yaml` liste les variables que le payload doit fournir, avec leur type (`string`, `money`, `datetime`, `url`, `list<...>`). Le `TemplateRenderer` refuse un payload qui ne respecte pas le schéma, en dev comme en prod (erreur `TemplateVariableMissing`, la livraison passe en `failed` sans retry, puisque réessayer ne fera pas apparaître la variable).

## Locales

Onze locales servies : `en`, `fr`, `de`, `nl`, `pl`, `es`, `it`, `pt`, `ro`, `cs`, `hu`. La locale vient de `users.locale`, sinon du pays de l'entreprise, sinon `en`. Un template manquant dans la locale demandée bascule sur `en` et incrémente `notify.template_fallback{locale, event}`. En mai 2026 le repli le plus fréquent est `hu` sur les événements de facturation (12 % des envois en `hu`), la traduction est en cours (HF-4188).

Les traductions passent par le fichier d'export `bin/console notifications:export-strings --locale pl`, qui produit un CSV pour l'agence, et l'import inverse vérifie que chaque `{{ variable }}` du texte source existe dans la traduction. Un `{{ montant }}` traduit en `{{ amount }}` a coûté une après-midi en 2025, d'où la vérification.

## Compilation

MJML est compilé en HTML au build de l'image (`make templates`), pas au moment de l'envoi : le rendu MJML prenait 80 ms par message et le compilateur avait besoin de Node dans l'image du worker. Le HTML compilé est mis dans `var/templates-compiled/` et versionné avec l'image. Au moment de l'envoi il ne reste que la substitution des variables par Twig (`strict_variables: true`) : 2 ms.

Conséquence : changer un template demande un déploiement. C'est voulu, un template est du code, il passe en revue ([[template-review-checklist]]).

## Règles de contenu

- Le sujet est une seule ligne, sans variable de montant (un sujet avec « 1 240,00 EUR » a été signalé comme hameçonnage par deux filtres d'entreprise en 2025).

- Un seul lien d'action par message, vers `app.halden.example`, jamais vers un domaine tiers. Les liens portent un `?ref=<event>` pour la mesure d'ouverture côté application, on ne charge pas de pixel de suivi.

- La version texte n'est pas générée depuis le HTML : elle est écrite à part, parce que la génération automatique produisait des URLs de 200 caractères au milieu des phrases.

- Le pied de page porte l'adresse légale et le lien vers les préférences ([[notification-preferences-schema]]), obligatoire même pour du transactionnel, par choix, pas par obligation.

- Pas d'image en pièce jointe. Les factures sont un lien vers le PDF dans l'espace client, pas une pièce jointe : les pièces jointes de 400 kB multipliaient le volume sortant par 20 et étaient la première cause de rejet par les passerelles de PME.

## Variables communes

Toutes les templates reçoivent `recipient` (prénom, nom, entreprise), `app_url`, `support_email`, `unsubscribe_url` et `locale`. Les dates sont formatées dans le fuseau de l'utilisateur (`users.timezone`, par défaut le fuseau du pays de l'entreprise) par le filtre `|hf_datetime`, les montants par `|hf_money(currency)`. Un template qui formate une date à la main ne passe pas la revue.

## Tester un template

`bin/console notifications:preview bid.received --locale fr --payload fixtures/bid.received.json` écrit le HTML et le texte dans `var/preview/` et les ouvre. Les fixtures par événement sont dans `fixtures/notifications/`, elles sont aussi utilisées par `notifications:lint-templates` en CI, qui rend chaque template dans chaque locale et échoue sur une variable manquante ou un lien vers un domaine non autorisé.
