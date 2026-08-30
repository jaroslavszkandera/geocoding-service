CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE TABLE IF NOT EXISTS places (
    id            BIGSERIAL PRIMARY KEY,
    osm_id        BIGINT,
    name          TEXT NOT NULL,
    feature_type  TEXT NOT NULL,
    geom          GEOMETRY(Point, 4326) NOT NULL,
    source        TEXT NOT NULL DEFAULT 'osm'
);

CREATE INDEX IF NOT EXISTS idx_places_geom ON places USING GIST (geom);
CREATE INDEX IF NOT EXISTS idx_places_name_trgm ON places USING GIN (name gin_trgm_ops);
