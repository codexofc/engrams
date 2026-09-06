---
name: geo-distance-postgis-vs-haversine
description: Load radius search uses PostGIS geography with a GiST index (ST_DWithin), the earlier haversine-in-SQL version was 40x slower, road distance still comes from the routing service
type: project
status: active
verified: 2026-04-09
---

# Distance search: PostGIS decision (HF-1330)

## Before

`GET /v2/loads?near=45.76,4.83&radius_km=150` computed the haversine formula inline in SQL over `pickup_lat`, `pickup_lng` (two `double precision` columns). No index can help that. On 2.2 M loads with a `status` filter first it took 900 ms at p95. Carriers use this search constantly (it is the default view of the carrier app), so it was the second most expensive query after search by text.

## After

- Extension `postgis` (3.4) enabled on the primary. The CloudNativePG image already had it, no operator change.

- Column `loads.pickup_geog geography(Point, 4326)`, generated: `GENERATED ALWAYS AS (ST_SetSRID(ST_MakePoint(pickup_lng, pickup_lat), 4326)::geography) STORED`. The old columns stay, the mobile app still writes them.

- `CREATE INDEX CONCURRENTLY idx_loads_pickup_geog ON loads USING gist (pickup_geog);`

- Query: `WHERE l.status = ANY(:statuses) AND ST_DWithin(l.pickup_geog, ST_MakePoint(:lng, :lat)::geography, :radius_m)`, ordered by `ST_Distance` for the first 200 then paginated by cursor. The order clause uses the same expression so the planner can use the index for the filter and sort the small candidate set in memory.

Measured on prod in March 2026: p95 22 ms, p99 60 ms. Index size 140 MB.

`geography`, not `geometry`: distances in metres without projection games, and 150 km across Europe is enough for the spheroid approximation to matter. `geometry` with a Web Mercator projection would have been faster but wrong by up to 8 % at Scandinavian latitudes, and we have Swedish carriers.

## What this is not

This is straight-line distance. Road distance and driving time come from the routing service (`routing-svc`, OSRM based, separate deployment) and are computed only for the 50 loads shown, never for the filter. Carriers asked for "within 2 hours of driving" as a filter. Refused for now: it would mean pre-computing isochrones per depot and refreshing them, and the product value over "within 150 km" is unclear.

## Doctrine

No Doctrine type for `geography`. The column is declared in the entity as `#[ORM\Column(type: 'string', insertable: false, updatable: false, columnDefinition: 'geography(Point,4326)')]` so `schema:validate` passes, and it is never read through the ORM. Queries are native SQL with a `ResultSetMapping`, see [[doctrine-dql-vs-native-sql-advice]].

Test fixtures need PostGIS in the CI PostgreSQL container: image `postgis/postgis:16-3.4`, changed in `.gitlab-ci.yml` at the same time.
