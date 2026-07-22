# ruv-drone evaluation status

Evaluated upstream `main` revision `691ce38e83ebccf01f846926ba4eef56b6468fd1` under Apache-2.0. Adoption is deferred: the repository has no releases, tags, or lockfile; its `ruvie` package is unpublished and depends on `w-swarm`; stated performance targets have not been independently verified.

No dependency was added. `UavCoordinationPort` is platform-owned, the deterministic conventional coordinator is enabled, and both ruv-drone and MAPPO remain `DisabledPendingPromotion`. Enabling an adapter requires immutable dependency evidence, representative cohort/relay and fault benchmarks, safety review, and independent promotion.
