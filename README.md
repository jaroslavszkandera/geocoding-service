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
curl http://localhost:3000/geocoding/Praha.json
curl http://localhost:3000/geocoding/14.4016,50.0910.json
```


TODO

## Architectural decisions

- Rust because of familiarity and time frame of implementation.
- Copy the current MapTiler API:
    - (resp: 200/400/403)
    - https://api.maptiler.com/geocoding/Zurich.json?key=YOUR_SECRET_TOKEN (secret token does not matter here much)

FeatureCollection where every item is represented as a GeoJSON Feature
```json
{
  "type": "FeatureCollection",
  "features": [
    {
      "id": "municipality.46425",
      "text": "Paris",
      "language": "en",
      "^text_(\\w\\w)": "string",
      "^language_(\\w\\w)": "string",
      "additionalProperty": "anything",
      "type": "Feature",
      "properties": {
        "ref": "osm:r71525",
        "country_code": "fr",
        "kind": "road",
        "categories": [
          "restaurant"
        ],
        "feature_tags": {
          "additionalProperty": "string"
        },
        "place_designation": "city",
        "additionalProperty": "anything"
      },
      "geometry": {
        "type": "Point",
        "coordinates": [
          8.528509,
          47.3774434
        ]
      },
      "bbox": [
        5.9559,
        45.818,
        10.4921,
        47.8084
      ],
      "center": "[Circular]",
      "place_name": "string",
      "matching_place_name": "string",
      "matching_text": "string",
      "place_type": [
        "continental_marine"
      ],
      "place_type_name": [
        "string"
      ],
      "relevance": 1,
      "context": [
        {
          "ref": "osm:r71525",
          "country_code": "fr",
          "kind": "road",
          "categories": [
            "restaurant"
          ],
          "feature_tags": {
            "additionalProperty": "string"
          },
          "place_designation": "city",
          "additionalProperty": "anything",
          "id": "municipality.46425",
          "text": "Paris",
          "language": "en",
          "^text_(\\w\\w)": "string",
          "^language_(\\w\\w)": "string"
        }
      ],
      "address": "string",
      "^place_name_(\\w\\w)": "string"
    }
  ],
  "query": [
    "string"
  ],
  "attribution": "<a href=\"https://www.maptiler.com/copyright/\" target=\"_blank\">&copy; MapTiler</a> <a href=\"https://www.openstreetmap.org/copyright\" target=\"_blank\">&copy; OpenStreetMap contributors</a>"
}
```

## Rationale behind the chosen tech stack

## Future improvements or scalability strategies

## Consideration
### One of our input database is OpenStreetMap
### The server is now using TypeScript
### Data are being pre-processed by Java

### Data sources (Research)

**OpenStreetMap (geofabrik)**
https://download.geofabrik.de/

**Nomatim**
- web API: rate limited 1 req/s
- local: only openstreetmaps for data sources, if address is slightly wrong, then no result (https://jeremymax.com/blog/nominatim-self-hosted-geocoding)

**Pelias**
- https://www.pelias.io/
- Elastic search
- different data sources
