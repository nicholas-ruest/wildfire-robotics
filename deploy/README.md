# Declarative deployment contract

All environments reconcile by pull from reviewed Git state. Production promotes OCI digests only; admission verifies signatures and SLSA provenance. Cloud/provider resources implement the portable PostgreSQL/PostGIS, JetStream, immutable object, KMS/HSM and telemetry contracts in `data-service-contracts`; credentials are injected from an external manager and never committed.

Build the gateway image with `docker build --build-arg PACKAGE=api-gateway --build-arg BINARY=api-gateway -f deploy/oci/Dockerfile .`. The container serves HTTP on port 8080; authenticated TLS is terminated by the managed ingress/service-mesh boundary. `api-gateway-runtime` is materialized by the approved external secret provider, never stored in Git.

Production and non-production require separate accounts, clusters, trust roots, keys and data. Canadian protected data and backups are constrained to approved Canadian regions. Rollback changes a manifest digest to a previously approved release; schema changes normally roll forward under ADR-041. GitOps drift is alerted and reconciled. Emergency mutation requires separately audited JIT access and is overwritten by reconciliation.
