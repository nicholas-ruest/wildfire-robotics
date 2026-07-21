-- phase: contract
BEGIN;
SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '60s';
SELECT pg_advisory_xact_lock(741004001);

-- Deployment automation substitutes reviewed evidence IDs into session settings.
DO $$
BEGIN
    IF current_setting('wr.fleet_version_evidence', true) IS NULL THEN
        RAISE EXCEPTION 'fleet_version_evidence is required';
    END IF;
    IF current_setting('wr.backup_evidence', true) IS NULL THEN
        RAISE EXCEPTION 'backup_evidence is required';
    END IF;
    IF EXISTS (SELECT 1 FROM persistence_fixture.records WHERE canonical_name IS NULL)
       OR EXISTS (SELECT 1 FROM persistence_fixture.records WHERE canonical_name <> legacy_name) THEN
        RAISE EXCEPTION 'backfill reconciliation is incomplete';
    END IF;
END $$;

ALTER TABLE persistence_fixture.records ALTER COLUMN canonical_name SET NOT NULL;
ALTER TABLE persistence_fixture.records DROP COLUMN legacy_name;
UPDATE persistence_fixture.schema_compatibility
SET phase = 'new_authoritative', switched_at = statement_timestamp()
WHERE singleton;
COMMIT;
