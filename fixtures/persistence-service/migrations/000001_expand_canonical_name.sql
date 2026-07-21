-- phase: expand
BEGIN;
SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '60s';
SELECT pg_advisory_xact_lock(741004001);

CREATE SCHEMA IF NOT EXISTS persistence_fixture;
CREATE TABLE IF NOT EXISTS persistence_fixture.records (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    legacy_name text NOT NULL,
    canonical_name text,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp()
);
ALTER TABLE persistence_fixture.records
    ADD COLUMN IF NOT EXISTS canonical_name text;
COMMIT;
