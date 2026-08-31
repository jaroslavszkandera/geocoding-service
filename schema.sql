CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE TABLE IF NOT EXISTS places (
    id            BIGSERIAL PRIMARY KEY,
    osm_id        BIGINT,
    name          TEXT NOT NULL,
    feature_type  TEXT NOT NULL,
    geom          GEOMETRY(Point, 4326) NOT NULL,
    source        TEXT NOT NULL DEFAULT 'osm',
    housenumber   TEXT,
    street        TEXT,
    city          TEXT,
    state         TEXT,
    country_code  TEXT,
    postcode      TEXT
);

-- Spatial index for faster geospatial range, bounding box, and distance queries
CREATE INDEX IF NOT EXISTS idx_places_geom ON places USING GIST (geom);
-- Inverted trigram index for faster text matching and substring/wildcard searches
CREATE INDEX IF NOT EXISTS idx_places_name_trgm ON places USING GIN (name gin_trgm_ops);
-- Tree-based trigram index optimized for nearest-neighbor similarity sorting
CREATE INDEX IF NOT EXISTS idx_places_name_trgm_gist ON places USING GIST (name gist_trgm_ops);
