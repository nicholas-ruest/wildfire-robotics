# Roll-forward procedure

If a phase fails, retain the last compatible application version, inspect the recorded checkpoint and PostgreSQL error, correct the failure with a new immutable migration, and rehearse it against the anonymized production-scale snapshot. Never edit an applied migration or reverse a destructive DDL transaction. Resume backfill after the durable `last_id`, reconcile old/new values, and switch readers only at zero mismatches.

The operator records migration version, release digest, checkpoint, row counts, duration, lock waits, reconciliation result, and approval in the deployment evidence record.
