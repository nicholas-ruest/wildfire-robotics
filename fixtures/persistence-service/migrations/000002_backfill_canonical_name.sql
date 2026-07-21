-- phase: backfill
BEGIN;
SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '60s';
SELECT pg_advisory_xact_lock(741004001);

CREATE TABLE IF NOT EXISTS persistence_fixture.backfill_checkpoint (
    migration_version integer PRIMARY KEY,
    last_id bigint NOT NULL DEFAULT 0,
    rows_migrated bigint NOT NULL DEFAULT 0,
    mismatches bigint NOT NULL DEFAULT 0,
    completed_at timestamptz
);
INSERT INTO persistence_fixture.backfill_checkpoint (migration_version)
VALUES (2) ON CONFLICT (migration_version) DO NOTHING;

WITH batch AS (
    SELECT id FROM persistence_fixture.records
    WHERE canonical_name IS NULL
      AND id > (SELECT last_id FROM persistence_fixture.backfill_checkpoint WHERE migration_version = 2)
    ORDER BY id LIMIT 1000 FOR UPDATE SKIP LOCKED
), migrated AS (
    UPDATE persistence_fixture.records AS target
    SET canonical_name = target.legacy_name
    FROM batch WHERE target.id = batch.id
    RETURNING target.id
)
UPDATE persistence_fixture.backfill_checkpoint
SET last_id = COALESCE((SELECT max(id) FROM migrated), last_id),
    rows_migrated = rows_migrated + (SELECT count(*) FROM migrated)
WHERE migration_version = 2;
COMMIT;
