-- Run this manually AFTER `osm2pgsql --create` has imported data.
--   psql "$DATABASE_URL" -f schema.sql

ALTER TABLE places
    ADD COLUMN IF NOT EXISTS bbox_west DOUBLE PRECISION GENERATED ALWAYS AS (ST_X(geom) - CASE feature_type WHEN 'city' THEN 0.18 WHEN 'town' THEN 0.045 WHEN 'village' THEN 0.009 WHEN 'suburb' THEN 0.018 ELSE 0.0009 END) STORED,
    ADD COLUMN IF NOT EXISTS bbox_south DOUBLE PRECISION GENERATED ALWAYS AS (ST_Y(geom) - CASE feature_type WHEN 'city' THEN 0.18 WHEN 'town' THEN 0.045 WHEN 'village' THEN 0.009 WHEN 'suburb' THEN 0.018 ELSE 0.0009 END) STORED,
    ADD COLUMN IF NOT EXISTS bbox_east DOUBLE PRECISION GENERATED ALWAYS AS (ST_X(geom) + CASE feature_type WHEN 'city' THEN 0.18 WHEN 'town' THEN 0.045 WHEN 'village' THEN 0.009 WHEN 'suburb' THEN 0.018 ELSE 0.0009 END) STORED,
    ADD COLUMN IF NOT EXISTS bbox_north DOUBLE PRECISION GENERATED ALWAYS AS (ST_Y(geom) + CASE feature_type WHEN 'city' THEN 0.18 WHEN 'town' THEN 0.045 WHEN 'village' THEN 0.009 WHEN 'suburb' THEN 0.018 ELSE 0.0009 END) STORED,
    ADD COLUMN IF NOT EXISTS search_vector tsvector GENERATED ALWAYS AS (to_tsvector('simple', name)) STORED;

-- Spatial index: powers bbox filtering and KNN ordering.
CREATE INDEX IF NOT EXISTS idx_places_geom ON places USING GIST (geom);
-- Trigram GIN: fuzzy filtering and pattern filtering.
CREATE INDEX IF NOT EXISTS idx_places_name_trgm ON places USING GIN (name gin_trgm_ops);
-- Trigram GIST: similarity-ranking.
CREATE INDEX IF NOT EXISTS idx_places_name_trgm_gist ON places USING GIST (name gist_trgm_ops);
-- Full-text GIN: search_vector.
CREATE INDEX IF NOT EXISTS idx_places_search ON places USING GIN (search_vector);
-- exact-match, a plain btree on lower(name)
CREATE INDEX IF NOT EXISTS idx_places_name_lower ON places (lower(name));
