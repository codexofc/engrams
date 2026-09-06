---
name: php-upgrade-8-3-to-8-4
description: halden-api moved to PHP 8.4 in HF-1470 (Feb 2026), 3 % faster on the API benchmark, property hooks not adopted yet, the implicit nullable deprecation touched 212 signatures
type: project
status: active
verified: 2026-02-26
---

# PHP 8.3 to 8.4 (HF-1470)

Done in two PRs over two weeks, deployed 2026-02-19.

## What broke or warned

- **Implicitly nullable parameters** (`function f(Foo $x = null)`) are deprecated in 8.4. 212 signatures. Fixed with Rector's `ExplicitNullableParamTypeRector`, reviewed the diff by eye in about an hour because Rector also wanted to touch unrelated things. The PR was `chore(php): explicit nullable types`, merged first, on PHP 8.3, so the upgrade PR itself stayed small.

- `DOMDocument`, `DOMElement` and friends moved to `Dom\` namespace in 8.4 with the old classes kept as aliases. Nothing to do, but PHPStan 1.12 complained until we bumped to 2.0.

- `E_STRICT` constant removed. One `error_reporting(E_ALL & ~E_STRICT)` in a legacy script, deleted.

- The `pdo_pgsql` driver in 8.4 changed how it reports `SQLSTATE` on connection loss, which broke one test asserting on the exception message. Assert on `SQLSTATE` code instead.

- `mbstring` functions with `null` input: our `PiiScrubber` passed `null` in one branch and 8.4 throws. Found by the integration suite.

## What we measured

`bin/console app:bench:api` (200 requests against the 12 hottest endpoints, in staging, 3 runs) : mean latency 3.1 % lower, p99 unchanged. Memory per php-fpm worker about the same (54 MB against 55 MB). OPcache with JIT was already on and stays on (`opcache.jit=1255`, `opcache.jit_buffer_size=128M`).

Not a reason to upgrade on its own. The reason was the support window: 8.3 gets security fixes until end of 2027, and we prefer being one version behind at most.

## What we did not adopt yet

- **Property hooks**. Tempting for entities (`public string $reference { set => strtoupper($value); }`), but Doctrine 3.3 did not support hooked properties on mapped fields at the time, and the lazy-loading proxies choke on them. Revisit when Doctrine ORM says it is fine. Tracked as HF-1471.

- **Asymmetric visibility** (`public private(set)`). Same Doctrine question. Would remove a lot of getters.

- `array_find()` and friends: allowed, no rule against them.

- `new` without parentheses in chains: allowed, php-cs-fixer 3.65 handles it.

## Image

`php:8.4-fpm-bookworm` base, extensions built in the Dockerfile the same way as before: `pdo_pgsql`, `intl`, `opcache`, `sodium`, `bcmath`, `pcntl`, `redis` (PECL 6.1), `amqp` (PECL 2.1.2, needed a bump for 8.4). Image size 210 MB, was 205 MB.

Dev environment: `docker-compose.yml` bumped at the same time, everyone had to rebuild. Announced in the team channel 3 days ahead, which turned out to be enough.

See also [[symfony-7-upgrade-notes]], done the month before, which is why this one was small.
