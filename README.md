# Geoconding service - Maptiler task

## Setup instructions

Download necessary packages
- osm2pgsql
- docker
- lua
- cargo

Download a map for testing (e.g. czech-republic-latest), launch the db and init the db.
```sh
mkdir -p data
wget -c -P data https://download.geofabrik.de/europe/czech-republic-latest.osm.pbf
docker compose up -d
PGPASSWORD=test osm2pgsql -d geocoding -U test -H localhost -p 5432 \
  --create --slim -O flex -S db/osm2pgsql/preprocess_osm.lua \
  data/czech-republic-latest.osm.pbf
psql "postgres://test:test@localhost:5432/geocoding" -f db/migrations/01_schema.sql
```
> This is meant to be a replacement for the Java preprocessing. Takes < 5 minutes for Czechia.

Add line to `.env`:
```sh
echo 'DATABASE_URL=postgres://test:test@localhost:5432/geocoding' > .env
```

Run the service:
```sh
RUST_LOG=info cargo r
```

Query the service:
```sh
curl 'http://localhost:3000/geocoding/Prague.json?key=test&limit=3'
curl 'http://localhost:3000/geocoding/18.6274503,49.6279094.json?key=test'
```

## Architectural decisions

- Standalone Rust binary with Tokio + Axum + sqlx, asynchronous service
- PostgreSQL + PostGIS + pg_trgm
    - PostGIS gives native spatial types, GiST indexes, bbox filtering, NN search (used by Nominatim)
    - pg_trgm: fuzzy search
    - pregenerated bbox for city, village and search vector for text
- Inspired by the current MapTiler API:
    - (resp: 200/400/403 (+500))
    - https://api.maptiler.com/geocoding/Zurich.json?key=YOUR_SECRET_TOKEN (secret token does not matter here much)

FeatureCollection where every item is represented as a GeoJSON Feature (e.g. reverse geocoding resp):
```json
{
  "type": "FeatureCollection",
  "features": [
    {
      "id": "mountain_rescue.-98416612",
      "text": "Horská služba Javorový",
      "type": "Feature",
      "geometry": {
        "type": "Point",
        "coordinates": [
          18.627200443200163,
          49.62821274643137
        ]
      },
      "bbox": [
        18.62630044320016,
        49.62731274643137,
        18.628100443200164,
        49.629112746431375
      ],
      "center": [
        18.627200443200163,
        49.62821274643137
      ],
      "place_name": "Horská služba Javorový",
      "place_type": [
        "mountain_rescue"
      ],
      "place_type_name": [
        "mountain_rescue"
      ],
      "relevance": 0.9631458505177869,
      "properties": {
        "ref": "osm:-98416612",
        "postcode": null
      },
      "context": [],
      "address": null
    }
  ],
  "query": [
    "18.6274503,49.6279094"
  ],
  "attribution": "<a href=\"https://www.openstreetmap.org/copyright\" target=\"_blank\">&copy; OpenStreetMap contributors</a>"
}
```

## Rationale behind the chosen tech stack

- Rust because of familiarity and time frame of implementation. Would have chosen TypeScript otherwise based on the considerations section
- PostgreSQL + PostGIS is proven technology concerning spatial search

## Future improvements or scalability strategies

- Read replicas (better read speed)
- Connection pool sizing tied to server core count
- Cache common queries for forward queries to RAM - Redis with LRU or TTL
- Partition/shard by country/region
- Better osm2pgsql preprocessing (administrative bounderies, POIs, importance score, ...) or using a search engine like Nominatim

## Consideration

### One of our input database is OpenStreetMap
- Used in my implementation
- Regional quality varies, potentially inconsistent tagging, free, crowd-sourced

### The server is now using TypeScript
- Rust chosen only for familiarity, best to adapt to TypeScript after a while

### Data are being pre-processed by Java
- Switch to Osmosis from osm2pgsql

### Data sources (Research)

**OpenStreetMap (geofabrik)**
https://download.geofabrik.de/

**Nominatim**
- web API: rate limited 1 req/s
- local: only openstreetmaps for data sources, if address is slightly wrong, then no result (https://jeremymax.com/blog/nominatim-self-hosted-geocoding)

**Pelias**
- https://www.pelias.io/
- Elastic search
- different data sources

**GeoNames**

**OpenAddresses.io**

**Who's on First (Mapzen/Overture heritage)**

