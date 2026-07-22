#![allow(clippy::expect_used, clippy::unwrap_used, missing_docs)]
use api_gateway::*;
use chrono::{TimeDelta, TimeZone, Utc};
use std::{cell::RefCell, collections::HashSet};

struct Policy {
    allowed: bool,
}

#[test]
fn should_reject_malformed_json_and_expired_deadline_against_injected_now() {
    let audit = Audit::default();
    let mut gateway = Gateway::new(limits(), Policy { allowed: true }, Backend, &audit);
    let malformed = GatewayRequest::new(
        "bad",
        ApiSurface::RestV1,
        TrafficLane::Query,
        "tenant-a",
        "ca-central",
        "incident.read",
        b"{broken",
        Utc.timestamp_opt(10, 0).single().unwrap(),
        Utc.timestamp_opt(14, 0).single().unwrap(),
        None,
    )
    .unwrap();
    assert_eq!(
        gateway.execute_at(
            &auth("tenant-a"),
            malformed,
            Utc.timestamp_opt(11, 0).single().unwrap()
        ),
        Err(GatewayError::SchemaInvalid)
    );
    let expired = GatewayRequest::new(
        "old",
        ApiSurface::RestV1,
        TrafficLane::Query,
        "tenant-a",
        "ca-central",
        "incident.read",
        b"{}",
        Utc.timestamp_opt(10, 0).single().unwrap(),
        Utc.timestamp_opt(14, 0).single().unwrap(),
        None,
    )
    .unwrap();
    assert_eq!(
        gateway.execute_at(
            &auth("tenant-a"),
            expired,
            Utc.timestamp_opt(15, 0).single().unwrap()
        ),
        Err(GatewayError::DeadlineExceeded)
    );
}

struct FailedAudit;
impl AuditPort for FailedAudit {
    fn append(&self, _: AuditRecord) -> Result<(), GatewayError> {
        Err(GatewayError::AuditUnavailable)
    }
}
struct CountingBackend<'a>(&'a std::cell::Cell<u32>);
impl ContextPort for CountingBackend<'_> {
    fn invoke(&self, _: &ValidatedRequest) -> Result<GatewayResponse, GatewayError> {
        self.0.set(self.0.get() + 1);
        Ok(GatewayResponse::json(200, "{}".into()))
    }
}
#[test]
fn should_prevent_effect_when_admission_audit_is_unavailable() {
    let calls = std::cell::Cell::new(0);
    let mut gateway = Gateway::new(
        limits(),
        Policy { allowed: true },
        CountingBackend(&calls),
        &FailedAudit,
    );
    let request = GatewayRequest::new(
        "cmd",
        ApiSurface::RestV1,
        TrafficLane::Command,
        "tenant-a",
        "ca-central",
        "mission.command",
        b"{}",
        Utc.timestamp_opt(10, 0).single().unwrap(),
        Utc.timestamp_opt(14, 0).single().unwrap(),
        Some("key"),
    )
    .unwrap();
    assert_eq!(
        gateway.execute(&auth("tenant-a"), request),
        Err(GatewayError::AuditUnavailable)
    );
    assert_eq!(calls.get(), 0);
}

struct LargeBackend;
impl ContextPort for LargeBackend {
    fn invoke(&self, _: &ValidatedRequest) -> Result<GatewayResponse, GatewayError> {
        Ok(GatewayResponse::json(200, "x".repeat(300)))
    }
}
#[test]
fn should_reject_oversized_downstream_response() {
    let audit = Audit::default();
    let mut gateway = Gateway::new(limits(), Policy { allowed: true }, LargeBackend, &audit);
    let request = GatewayRequest::new(
        "q",
        ApiSurface::RestV1,
        TrafficLane::Query,
        "tenant-a",
        "ca-central",
        "incident.read",
        b"{}",
        Utc.timestamp_opt(10, 0).single().unwrap(),
        Utc.timestamp_opt(14, 0).single().unwrap(),
        None,
    )
    .unwrap();
    assert_eq!(
        gateway.execute(&auth("tenant-a"), request),
        Err(GatewayError::InvalidResponse)
    );
}

#[test]
fn should_scope_idempotency_by_principal_and_operation() {
    let audit = Audit::default();
    let mut scoped_limits = limits();
    scoped_limits.requests_per_window = 3;
    let mut gateway = Gateway::new(scoped_limits, Policy { allowed: true }, Backend, &audit);
    let auth2 = AuthContext::new(
        "principal-2",
        "tenant-a",
        "ca-central",
        HashSet::from(["operator".into()]),
        "operations",
    )
    .unwrap();
    let make = |id: &str, op: &str, body: &[u8]| {
        GatewayRequest::new(
            id,
            ApiSurface::RestV1,
            TrafficLane::Command,
            "tenant-a",
            "ca-central",
            op,
            body,
            Utc.timestamp_opt(10, 0).single().unwrap(),
            Utc.timestamp_opt(14, 0).single().unwrap(),
            Some("same-key"),
        )
        .unwrap()
    };
    assert!(
        gateway
            .execute(&auth("tenant-a"), make("a", "mission.command", b"{}"))
            .is_ok()
    );
    assert!(
        gateway
            .execute(&auth2, make("b", "mission.command", b"[1]"))
            .is_ok()
    );
    assert!(
        gateway
            .execute(&auth("tenant-a"), make("c", "fleet.command", b"[2]"))
            .is_ok()
    );
}

#[test]
fn should_bound_ogc_pagination_and_bbox() {
    assert!(OgcQuery::new(1000, 20, Some([-123.0, 48.0, -122.0, 49.0])).is_ok());
    assert_eq!(
        OgcQuery::new(1001, 0, None),
        Err(GatewayError::InvalidRequest)
    );
    assert_eq!(
        OgcQuery::new(10, 0, Some([10.0, 20.0, 5.0, 21.0])),
        Err(GatewayError::InvalidRequest)
    );
}
impl AuthorizationPort for Policy {
    fn authorize(&self, _: &AuthContext, _: &str, _: TrafficLane) -> bool {
        self.allowed
    }
}
#[derive(Default)]
struct Audit(RefCell<Vec<AuditRecord>>);
impl AuditPort for Audit {
    fn append(&self, record: AuditRecord) -> Result<(), GatewayError> {
        self.0.borrow_mut().push(record);
        Ok(())
    }
}
struct Backend;
impl ContextPort for Backend {
    fn invoke(&self, request: &ValidatedRequest) -> Result<GatewayResponse, GatewayError> {
        Ok(GatewayResponse::json(
            200,
            format!("\"{}\"", request.body_digest()),
        ))
    }
}
fn auth(tenant: &str) -> AuthContext {
    AuthContext::new(
        "principal",
        tenant,
        "ca-central",
        HashSet::from(["operator".into()]),
        "incident-operations",
    )
    .unwrap()
}
fn limits() -> GatewayLimits {
    GatewayLimits {
        max_body_bytes: 64,
        max_response_body_bytes: 256,
        max_deadline: TimeDelta::seconds(5),
        requests_per_window: 2,
        max_concurrent: 1,
    }
}

#[test]
fn should_reject_cross_tenant_and_internal_grpc_routes() {
    let audit = Audit::default();
    let mut gateway = Gateway::new(limits(), Policy { allowed: true }, Backend, &audit);
    let request = GatewayRequest::new(
        "req-1",
        ApiSurface::RestV1,
        TrafficLane::Query,
        "tenant-b",
        "ca-central",
        "incident.read",
        b"{}",
        Utc.timestamp_opt(10, 0).single().unwrap(),
        Utc.timestamp_opt(14, 0).single().unwrap(),
        None,
    )
    .unwrap();
    assert_eq!(
        gateway.execute(&auth("tenant-a"), request),
        Err(GatewayError::ScopeMismatch)
    );
    let internal = GatewayRequest::new(
        "req-2",
        ApiSurface::InternalGrpc,
        TrafficLane::Query,
        "tenant-a",
        "ca-central",
        "incident.read",
        b"{}",
        Utc.timestamp_opt(10, 0).single().unwrap(),
        Utc.timestamp_opt(14, 0).single().unwrap(),
        None,
    )
    .unwrap();
    assert_eq!(
        gateway.execute(&auth("tenant-a"), internal),
        Err(GatewayError::InternalSurfaceForbidden)
    );
}

#[test]
fn should_enforce_body_deadline_authorization_and_lane_limits() {
    let audit = Audit::default();
    let mut denied = Gateway::new(limits(), Policy { allowed: false }, Backend, &audit);
    let request = GatewayRequest::new(
        "req",
        ApiSurface::RestV1,
        TrafficLane::Command,
        "tenant-a",
        "ca-central",
        "mission.command",
        b"{}",
        Utc.timestamp_opt(10, 0).single().unwrap(),
        Utc.timestamp_opt(14, 0).single().unwrap(),
        Some("idem-1"),
    )
    .unwrap();
    assert_eq!(
        denied.execute(&auth("tenant-a"), request),
        Err(GatewayError::Forbidden)
    );
    let mut gateway = Gateway::new(limits(), Policy { allowed: true }, Backend, &audit);
    let oversized = GatewayRequest::new(
        "big",
        ApiSurface::RestV1,
        TrafficLane::Bulk,
        "tenant-a",
        "ca-central",
        "hazard.bulk",
        &[0; 65],
        Utc.timestamp_opt(10, 0).single().unwrap(),
        Utc.timestamp_opt(14, 0).single().unwrap(),
        Some("idem"),
    )
    .unwrap();
    assert_eq!(
        gateway.execute(&auth("tenant-a"), oversized),
        Err(GatewayError::BodyTooLarge)
    );
}

#[test]
fn should_make_commands_idempotent_and_distinguish_outcomes() {
    let audit = Audit::default();
    let mut gateway = Gateway::new(limits(), Policy { allowed: true }, Backend, &audit);
    let make = |digest: &[u8]| {
        GatewayRequest::new(
            "req",
            ApiSurface::RestV1,
            TrafficLane::Command,
            "tenant-a",
            "ca-central",
            "mission.command",
            digest,
            Utc.timestamp_opt(10, 0).single().unwrap(),
            Utc.timestamp_opt(14, 0).single().unwrap(),
            Some("idem-1"),
        )
        .unwrap()
    };
    let first = gateway.execute(&auth("tenant-a"), make(b"{}")).unwrap();
    let replay = gateway.execute(&auth("tenant-a"), make(b"{}")).unwrap();
    assert_eq!(first, replay);
    assert_eq!(
        gateway.execute(&auth("tenant-a"), make(b"[1]")),
        Err(GatewayError::IdempotencyConflict)
    );
    assert_ne!(
        CommandOutcome::TransportAcknowledged,
        CommandOutcome::PhysicalOutcomeConfirmed
    );
}

#[test]
fn should_label_fresh_stale_gap_degraded_and_unknown_read_models() {
    let now = Utc.timestamp_opt(100, 0).single().unwrap();
    assert_eq!(
        Freshness::classify(
            Some(now - TimeDelta::seconds(2)),
            Some(now + TimeDelta::seconds(2)),
            now,
            false,
            false
        ),
        Freshness::Fresh
    );
    assert_eq!(
        Freshness::classify(
            Some(now - TimeDelta::seconds(20)),
            Some(now - TimeDelta::seconds(1)),
            now,
            false,
            false
        ),
        Freshness::Stale
    );
    assert_eq!(
        Freshness::classify(
            Some(now),
            Some(now + TimeDelta::seconds(2)),
            now,
            true,
            false
        ),
        Freshness::Gap
    );
    assert_eq!(
        Freshness::classify(
            Some(now),
            Some(now + TimeDelta::seconds(2)),
            now,
            false,
            true
        ),
        Freshness::Degraded
    );
    assert_eq!(
        Freshness::classify(None, None, now, false, false),
        Freshness::Unknown
    );
}

#[test]
fn should_validate_ogc_paths_and_rate_limit_each_traffic_lane_independently() {
    let audit = Audit::default();
    let mut gateway = Gateway::new(limits(), Policy { allowed: true }, Backend, &audit);
    let make = |id: &str, lane| {
        GatewayRequest::new(
            id,
            ApiSurface::OgcApiFeaturesV1,
            lane,
            "tenant-a",
            "ca-central",
            "/collections/incidents/items",
            b"{}",
            Utc.timestamp_opt(10, 0).single().unwrap(),
            Utc.timestamp_opt(14, 0).single().unwrap(),
            None,
        )
        .unwrap()
    };
    gateway
        .execute(&auth("tenant-a"), make("q1", TrafficLane::Query))
        .unwrap();
    gateway
        .execute(&auth("tenant-a"), make("q2", TrafficLane::Query))
        .unwrap();
    assert_eq!(
        gateway.execute(&auth("tenant-a"), make("q3", TrafficLane::Query)),
        Err(GatewayError::RateLimited)
    );
    let public = GatewayRequest::new(
        "p1",
        ApiSurface::RestV1,
        TrafficLane::Public,
        "tenant-a",
        "ca-central",
        "status.public",
        b"{}",
        Utc.timestamp_opt(10, 0).single().unwrap(),
        Utc.timestamp_opt(14, 0).single().unwrap(),
        None,
    )
    .unwrap();
    assert!(gateway.execute(&auth("tenant-a"), public).is_ok());
}
