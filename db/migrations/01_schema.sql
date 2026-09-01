-- Run this manually AFTER `osm2pgsql --create` has imported data.
--   psql "$DATABASE_URL" -f db/migrations/01_schema.sql

-- Extension for diacritic-insensitive search.
CREATE EXTENSION IF NOT EXISTS unaccent;

-- unaccent() is STABLE by default; GENERATED ALWAYS AS columns require IMMUTABLE.
-- Wrap it so generated columns and indexes are usable.
CREATE OR REPLACE FUNCTION immutable_unaccent(text)
RETURNS text AS $$ SELECT unaccent('unaccent', $1) $$
LANGUAGE sql IMMUTABLE PARALLEL SAFE STRICT;

ALTER TABLE places
    ADD COLUMN IF NOT EXISTS bbox_west DOUBLE PRECISION GENERATED ALWAYS AS (ST_X(geom) - CASE feature_type WHEN 'city' THEN 0.18 WHEN 'town' THEN 0.045 WHEN 'village' THEN 0.009 WHEN 'suburb' THEN 0.018 WHEN 'state' THEN 1.5 WHEN 'region' THEN 1.5 WHEN 'country' THEN 5.0 WHEN 'park' THEN 0.01 WHEN 'forest' THEN 0.02 WHEN 'nature_reserve' THEN 0.02 WHEN 'cemetery' THEN 0.005 WHEN 'industrial' THEN 0.005 WHEN 'commercial' THEN 0.005 WHEN 'residential' THEN 0.005 ELSE 0.0009 END) STORED,
    ADD COLUMN IF NOT EXISTS bbox_south DOUBLE PRECISION GENERATED ALWAYS AS (ST_Y(geom) - CASE feature_type WHEN 'city' THEN 0.18 WHEN 'town' THEN 0.045 WHEN 'village' THEN 0.009 WHEN 'suburb' THEN 0.018 WHEN 'state' THEN 1.5 WHEN 'region' THEN 1.5 WHEN 'country' THEN 5.0 WHEN 'park' THEN 0.01 WHEN 'forest' THEN 0.02 WHEN 'nature_reserve' THEN 0.02 WHEN 'cemetery' THEN 0.005 WHEN 'industrial' THEN 0.005 WHEN 'commercial' THEN 0.005 WHEN 'residential' THEN 0.005 ELSE 0.0009 END) STORED,
    ADD COLUMN IF NOT EXISTS bbox_east DOUBLE PRECISION GENERATED ALWAYS AS (ST_X(geom) + CASE feature_type WHEN 'city' THEN 0.18 WHEN 'town' THEN 0.045 WHEN 'village' THEN 0.009 WHEN 'suburb' THEN 0.018 WHEN 'state' THEN 1.5 WHEN 'region' THEN 1.5 WHEN 'country' THEN 5.0 WHEN 'park' THEN 0.01 WHEN 'forest' THEN 0.02 WHEN 'nature_reserve' THEN 0.02 WHEN 'cemetery' THEN 0.005 WHEN 'industrial' THEN 0.005 WHEN 'commercial' THEN 0.005 WHEN 'residential' THEN 0.005 ELSE 0.0009 END) STORED,
    ADD COLUMN IF NOT EXISTS bbox_north DOUBLE PRECISION GENERATED ALWAYS AS (ST_Y(geom) + CASE feature_type WHEN 'city' THEN 0.18 WHEN 'town' THEN 0.045 WHEN 'village' THEN 0.009 WHEN 'suburb' THEN 0.018 WHEN 'state' THEN 1.5 WHEN 'region' THEN 1.5 WHEN 'country' THEN 5.0 WHEN 'park' THEN 0.01 WHEN 'forest' THEN 0.02 WHEN 'nature_reserve' THEN 0.02 WHEN 'cemetery' THEN 0.005 WHEN 'industrial' THEN 0.005 WHEN 'commercial' THEN 0.005 WHEN 'residential' THEN 0.005 ELSE 0.0009 END) STORED,
    ADD COLUMN IF NOT EXISTS name_unaccent text GENERATED ALWAYS AS (immutable_unaccent(name)) STORED,
    ADD COLUMN IF NOT EXISTS search_vector tsvector GENERATED ALWAYS AS (to_tsvector('simple', immutable_unaccent(name))) STORED;

-- Spatial index: powers bbox filtering and KNN ordering.
CREATE INDEX IF NOT EXISTS idx_places_geom ON places USING GIST (geom);
-- Trigram GIN on diacritic-stripped name: fuzzy filtering and pattern filtering.
CREATE INDEX IF NOT EXISTS idx_places_name_unaccent_trgm ON places USING GIN (name_unaccent gin_trgm_ops);
-- Trigram GIST on diacritic-stripped name: similarity-ranking.
CREATE INDEX IF NOT EXISTS idx_places_name_unaccent_trgm_gist ON places USING GIST (name_unaccent gist_trgm_ops);
-- Full-text GIN: search_vector.
CREATE INDEX IF NOT EXISTS idx_places_search ON places USING GIN (search_vector);
-- exact-match, a plain btree on lower(name_unaccent) for normalized equality.
CREATE INDEX IF NOT EXISTS idx_places_name_unaccent_lower ON places (lower(name_unaccent));
