# Restore procedure

Before contract, verify a content-addressed encrypted PostgreSQL backup and complete a timed restore rehearsal in an isolated environment. Record backup digest, WAL recovery point, schema version, row counts, tenant sampling results, restore duration, and reviewer approval. Contract deployment supplies that evidence ID as `wr.backup_evidence`.

After destructive contract, recovery restores the verified backup to a new database, applies forward migrations, reconciles it, and switches traffic through the deployment runbook. In-place down migrations are intentionally unsupported.
