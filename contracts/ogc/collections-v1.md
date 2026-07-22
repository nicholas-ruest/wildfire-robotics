# OGC collection catalog v1

| Collection | Owning context | Content | Required scope | Freshness policy |
|---|---|---|---|---|
| `hazard-pictures` | Hazard Intelligence | Published hazard footprints and gaps | `read:geospatial` | Preserve source validity, confidence, expiry, gaps, and lineage |
| `incident-restrictions` | Incident Command | Authorized incident restrictions | `read:geospatial` | Expired or superseded restrictions remain explicitly labelled |
| `mission-operating-areas` | Mission Control | Advisory mission and allocation projections | `read:geospatial` | Projection is non-authoritative; command reloads owning aggregate |
| `station-coverage` | Station Operations | Connectivity and station service coverage | `read:geospatial` | Unknown or stale coverage cannot be treated as available capacity |

Every feature is tenant/region filtered before spatial evaluation. Sensitive locations, people, residences, critical infrastructure, restricted imagery, and licensed source attributes are minimized or redacted according to classification and purpose. Collection bounds and counts are themselves protected data.
