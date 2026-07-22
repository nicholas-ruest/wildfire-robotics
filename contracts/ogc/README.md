# OGC external interface v1

The external geospatial API is rooted at `/ogc/v1` and is tenant- and region-scoped by the authenticated access token. URL parameters never expand that scope. The API claims only the OGC API - Features Core and GeoJSON conformance classes listed by its `/conformance` response; Maps, Tiles, Processes, CQL, and CRS extension conformance are not claimed.

All collections use WGS 84 longitude/latitude coordinates in GeoJSON, bounded pages of at most 500 features, opaque scope-bound cursors, bounded `bbox` and `datetime` inputs, and `application/geo+json` item responses. Feature properties must include the same provenance, uncertainty, and freshness semantics as REST projections. Stale, gapped, degraded, and unknown data remain visible as such and cannot authorize a command.

The gateway applies collection authorization before querying, counting, filtering, or generating links. It must not reveal inaccessible collection or feature existence. The owning context remains responsible for field classification, redaction, licensing, and authorization.

## Claimed conformance URIs

- `http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/core`
- `http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/oas30`
- `http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/html`
- `http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/geojson`

HTML is a claimed representation only when the gateway implementation and conformance suite supply it; until then deployments must omit that URI from the runtime response.
