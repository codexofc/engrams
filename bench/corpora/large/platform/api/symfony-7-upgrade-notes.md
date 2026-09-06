---
name: symfony-7-upgrade-notes
description: Symfony 6.4 LTS to 7.2 in HF-1440 (Jan 2026), main pain points were the Serializer default context changes, removed Form usage, and the Messenger AmqpExt stamp rename
type: project
status: active
verified: 2026-01-30
---

# Symfony 6.4 → 7.2 (HF-1440)

Déployé le 2026-01-22. Environ 6 jours de travail répartis sur 3 semaines, une seule PR de 340 fichiers modifiés, dont 280 sont des ajouts de types de retour.

## Préparation qui a payé

- Toutes les dépréciations 6.4 traitées en amont sur trois PR séparées (`deprecation-contracts` à zéro dans les logs de test, vérifié par `SYMFONY_DEPRECATIONS_HELPER=max[self]=0`). C'est ce qui a rendu l'upgrade lui-même mécanique.
- Les Forms Symfony avaient déjà été retirés de l'API en 2025 (tout passe par `#[MapRequestPayload]` et des DTO readonly). Ça a évité le gros des changements de signatures.

## Ce qui a demandé du travail

**Serializer.** Le contexte par défaut de `ObjectNormalizer` a changé : `skip_null_values` était forcé à `true` dans notre `serializer.yaml` mais deux normaliseurs custom construisaient leur propre contexte et ont perdu l'option. Résultat : des `null` en plus dans les payloads de webhook (voir [[webhook-delivery-outbox]]), attrapé par le test de contrat des payloads. Le `AbstractObjectNormalizer::SKIP_NULL_VALUES` est maintenant posé dans un `SerializerContextBuilder` unique.

**Messenger AMQP.** `AmqpStamp` a bougé de namespace avec le split du bridge, et l'attribut de routage qu'on utilisait pour `async_priority` a changé de nom. Une heure de recherche, deux lignes de correctif. Voir [[messenger-transports-and-retries]].

**Types de retour.** 7.0 exige les types de retour sur toutes les méthodes surchargées des classes Symfony (`getSubscribedEvents(): array`, `process(ContainerBuilder $container): void`). Rector `SymfonySetList::SYMFONY_70` a tout fait, mais le diff est énorme et la revue a pris une après-midi.

**Security.** `Symfony\Component\Security\Core\Security` supprimé au profit de `SecurityBundle\Security`. 40 fichiers. Et notre `JwtAuthenticator` custom avait une signature `supports()` sans type de retour, corrigée.

**Validator.** Les contraintes en annotations Doctrine-style étaient déjà en attributs. Rien à faire.

**Clock.** On utilisait déjà `symfony/clock` depuis 6.3. Rien à faire, mais c'est le moment où on a ajouté la règle PHPStan `NoNativeDateTimeRule` (voir [[api-tests-phpunit-conventions]]).

## Ce qui n'a pas changé

Doctrine ORM est resté en 3.3, DBAL en 4.2, on ne mélange pas deux upgrades. `api-platform` n'est pas utilisé, donc pas de contrainte de ce côté.

## Mesures

Aucune différence de performance mesurable sur `app:bench:api`. Taille du container compilé (`var/cache/prod/App_KernelProdContainer.php`) : 3,9 Mo, était 4,1 Mo.

## Prochaine étape

7.4 LTS en fin d'année 2026, ça devrait être une PR de dépendances. La leçon : traiter les dépréciations au fil de l'eau, chaque sprint, et l'upgrade majeur n'est pas un événement. Suite avec [[php-upgrade-8-3-to-8-4]].
