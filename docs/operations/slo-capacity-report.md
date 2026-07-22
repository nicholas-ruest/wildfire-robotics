# Release-candidate SLO and capacity report

This report records software evidence, not deployed-service or field performance.

The Prompt 30 reproducible qualification exercises one million registered and concurrently connected assets, one million 1 Hz summaries, a ten-million reconnect burst, hot-region skew, partition split/merge, relay loss, regional isolation, rolling upgrade and recovery. The checked-in result reports p50/p95/p99 latency of 49/79/81 microseconds, zero modeled lag and loss, 80% saturation, 997,500 ppm availability, 2,100 ms bounded recovery, and 99% measured processing headroom. It also exercises the declared suppression, charging, logistics, useful-arrival, and robot-hospital workload mix. These are deterministic in-process qualification measurements on the recorded test platform; they are not extrapolated cloud-service claims.

The deployment recovery fixture records local simulated recovery evidence and remains distinct from a production regional exercise. Runtime SLO ownership and definitions remain in `operations-core` and ADR-039. Production promotion requires measured canary and endurance evidence from the target environment, alert and error-budget verification, exercised rollback, and an accountable operations approval. Those external operational gates remain outstanding in `release/field-evidence-status.toml`.

