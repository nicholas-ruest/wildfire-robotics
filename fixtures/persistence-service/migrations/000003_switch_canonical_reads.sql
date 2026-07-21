-- phase: switch
BEGIN;
SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '60s';
SELECT pg_advisory_xact_lock(741004001);

CREATE TABLE IF NOT EXISTS persistence_fixture.schema_compatibility (
    singleton boolean PRIMARY KEY DEFAULT true CHECK (singleton),
    phase text NOT NULL CHECK (phase IN ('dual_read', 'new_authoritative')),
    switched_at timestamptz NOT NULL,
    reconciliation_mismatches bigint NOT NULL CHECK (reconciliation_mismatches >= 0)
);
INSERT INTO persistence_fixture.schema_compatibility
    (singleton, phase, switched_at, reconciliation_mismatches)
VALUES (true, 'dual_read', statement_timestamp(), 0)
ON CONFLICT (singleton) DO UPDATE
SET phase = 'dual_read', switched_at = EXCLUDED.switched_at;
COMMIT;
