#![forbid(unsafe_code)]
#![allow(missing_docs)]
//! Framework-neutral external API policy boundary (ADR-022, ADR-044).

use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum GatewayError {
    #[error("invalid request boundary value")]
    InvalidRequest,
    #[error("authenticated scope does not match resource scope")]
    ScopeMismatch,
    #[error("authorization denied by the owning context policy")]
    Forbidden,
    #[error("internal gRPC is never a public surface")]
    InternalSurfaceForbidden,
    #[error("request body exceeds its lane limit")]
    BodyTooLarge,
    #[error("request deadline is invalid or excessive")]
    DeadlineExceeded,
    #[error("request schema is invalid")]
    SchemaInvalid,
    #[error("traffic lane rate quota exhausted")]
    RateLimited,
    #[error("traffic lane concurrency quota exhausted")]
    ConcurrencyLimited,
    #[error("an effectful request requires an idempotency key")]
    IdempotencyRequired,
    #[error("idempotency key was reused with different content")]
    IdempotencyConflict,
    #[error("audit persistence failed closed")]
    AuditUnavailable,
    #[error("downstream context failed")]
    Downstream,
    #[error("downstream response violated gateway bounds")]
    InvalidResponse,
    #[error("outcome audit failed after an effect may have occurred")]
    OutcomeAuditUnavailable,
}

fn required(value: &str) -> Result<String, GatewayError> {
    let value = value.trim();
    if value.is_empty() {
        Err(GatewayError::InvalidRequest)
    } else {
        Ok(value.to_owned())
    }
}
fn digest(bytes: &[u8]) -> Result<String, GatewayError> {
    let value: Value = serde_json::from_slice(bytes).map_err(|_| GatewayError::SchemaInvalid)?;
    let canonical = serde_json::to_vec(&value).map_err(|_| GatewayError::SchemaInvalid)?;
    Ok(format!("sha256:{:x}", Sha256::digest(canonical)))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ApiSurface {
    RestV1,
    OgcApiFeaturesV1,
    InternalGrpc,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrafficLane {
    Command,
    Query,
    Bulk,
    Export,
    Public,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandOutcome {
    TransportAcknowledged,
    Accepted,
    Rejected,
    ExecutionStarted,
    ExecutionCompleted,
    PhysicalOutcomeConfirmed,
    PhysicalOutcomeUnknown,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Freshness {
    Fresh,
    Stale,
    Gap,
    Degraded,
    Unknown,
}
impl Freshness {
    #[must_use]
    pub fn classify(
        observed: Option<DateTime<Utc>>,
        expires: Option<DateTime<Utc>>,
        now: DateTime<Utc>,
        gap: bool,
        degraded: bool,
    ) -> Self {
        if gap {
            return Self::Gap;
        }
        if degraded {
            return Self::Degraded;
        }
        match observed.zip(expires) {
            None => Self::Unknown,
            Some((_, expiry)) if now >= expiry => Self::Stale,
            Some(_) => Self::Fresh,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthContext {
    principal: String,
    tenant: String,
    region: String,
    roles: HashSet<String>,
    purpose: String,
}
impl AuthContext {
    pub fn new(
        principal: &str,
        tenant: &str,
        region: &str,
        roles: HashSet<String>,
        purpose: &str,
    ) -> Result<Self, GatewayError> {
        if roles.is_empty() {
            return Err(GatewayError::InvalidRequest);
        }
        Ok(Self {
            principal: required(principal)?,
            tenant: required(tenant)?,
            region: required(region)?,
            roles,
            purpose: required(purpose)?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct GatewayRequest {
    id: String,
    surface: ApiSurface,
    lane: TrafficLane,
    tenant: String,
    region: String,
    operation: String,
    body: Vec<u8>,
    issued_at: DateTime<Utc>,
    deadline: DateTime<Utc>,
    idempotency_key: Option<String>,
}
impl GatewayRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: &str,
        surface: ApiSurface,
        lane: TrafficLane,
        tenant: &str,
        region: &str,
        operation: &str,
        body: &[u8],
        issued_at: DateTime<Utc>,
        deadline: DateTime<Utc>,
        idempotency_key: Option<&str>,
    ) -> Result<Self, GatewayError> {
        Ok(Self {
            id: required(id)?,
            surface,
            lane,
            tenant: required(tenant)?,
            region: required(region)?,
            operation: required(operation)?,
            body: body.to_vec(),
            issued_at,
            deadline,
            idempotency_key: idempotency_key.map(required).transpose()?,
        })
    }
}
#[derive(Debug, Clone)]
pub struct ValidatedRequest(GatewayRequest, String);
impl ValidatedRequest {
    #[must_use]
    pub fn body_digest(&self) -> &str {
        &self.1
    }
    #[must_use]
    pub fn operation(&self) -> &str {
        &self.0.operation
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GatewayResponse {
    pub status: u16,
    pub content_type: String,
    pub body: String,
    pub command_outcome: Option<CommandOutcome>,
    pub freshness: Option<Freshness>,
}
impl GatewayResponse {
    #[must_use]
    pub fn json(status: u16, body: String) -> Self {
        Self {
            status,
            content_type: "application/json".into(),
            body,
            command_outcome: None,
            freshness: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GatewayLimits {
    pub max_body_bytes: usize,
    pub max_response_body_bytes: usize,
    pub max_deadline: TimeDelta,
    pub requests_per_window: u32,
    pub max_concurrent: u32,
}
pub trait AuthorizationPort {
    fn authorize(&self, context: &AuthContext, operation: &str, lane: TrafficLane) -> bool;
}
pub trait ContextPort {
    fn invoke(&self, request: &ValidatedRequest) -> Result<GatewayResponse, GatewayError>;
}
pub trait AuditPort {
    fn append(&self, record: AuditRecord) -> Result<(), GatewayError>;
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditRecord {
    pub request_id: String,
    pub principal: String,
    pub tenant: String,
    pub region: String,
    pub purpose: String,
    pub operation: String,
    pub lane: TrafficLane,
    pub decision: String,
    pub body_digest: String,
}

pub struct Gateway<'a, A, C, U> {
    limits: GatewayLimits,
    authorization: A,
    context: C,
    audit: &'a U,
    rates: HashMap<(String, TrafficLane, i64), u32>,
    active: HashMap<TrafficLane, u32>,
    idempotency: HashMap<IdempotencyScope, IdempotencyRecord>,
}
type IdempotencyScope = (String, String, String, String, String);
type IdempotencyRecord = (String, GatewayResponse);
impl<'a, A: AuthorizationPort, C: ContextPort, U: AuditPort> Gateway<'a, A, C, U> {
    pub fn new(limits: GatewayLimits, authorization: A, context: C, audit: &'a U) -> Self {
        Self {
            limits,
            authorization,
            context,
            audit,
            rates: HashMap::new(),
            active: HashMap::new(),
            idempotency: HashMap::new(),
        }
    }
    pub fn execute(
        &mut self,
        auth: &AuthContext,
        request: GatewayRequest,
    ) -> Result<GatewayResponse, GatewayError> {
        let now = request.issued_at;
        self.execute_at(auth, request, now)
    }
    pub fn execute_at(
        &mut self,
        auth: &AuthContext,
        request: GatewayRequest,
        now: DateTime<Utc>,
    ) -> Result<GatewayResponse, GatewayError> {
        if request.body.len() > self.limits.max_body_bytes {
            return Err(GatewayError::BodyTooLarge);
        }
        let request_id = request.id.clone();
        let operation = request.operation.clone();
        let lane = request.lane;
        let body_digest = digest(&request.body)?;
        let effectful = matches!(lane, TrafficLane::Command | TrafficLane::Bulk);
        if effectful {
            self.audit
                .append(AuditRecord {
                    request_id: request_id.clone(),
                    principal: auth.principal.clone(),
                    tenant: auth.tenant.clone(),
                    region: auth.region.clone(),
                    purpose: auth.purpose.clone(),
                    operation: operation.clone(),
                    lane,
                    decision: "admission-intent".into(),
                    body_digest: body_digest.clone(),
                })
                .map_err(|_| GatewayError::AuditUnavailable)?;
        }
        let result = self.execute_inner(auth, request, body_digest.clone(), now);
        let record = AuditRecord {
            request_id,
            principal: auth.principal.clone(),
            tenant: auth.tenant.clone(),
            region: auth.region.clone(),
            purpose: auth.purpose.clone(),
            operation,
            lane,
            decision: if result.is_ok() {
                "allowed".into()
            } else {
                "denied".into()
            },
            body_digest,
        };
        self.audit.append(record).map_err(|_| {
            if effectful {
                GatewayError::OutcomeAuditUnavailable
            } else {
                GatewayError::AuditUnavailable
            }
        })?;
        result
    }
    fn execute_inner(
        &mut self,
        auth: &AuthContext,
        request: GatewayRequest,
        body_digest: String,
        now: DateTime<Utc>,
    ) -> Result<GatewayResponse, GatewayError> {
        if request.surface == ApiSurface::InternalGrpc {
            return Err(GatewayError::InternalSurfaceForbidden);
        }
        if auth.tenant != request.tenant || auth.region != request.region {
            return Err(GatewayError::ScopeMismatch);
        }
        if request.body.len() > self.limits.max_body_bytes {
            return Err(GatewayError::BodyTooLarge);
        }
        let duration = request.deadline - request.issued_at;
        if request.issued_at > now
            || request.deadline <= now
            || duration <= TimeDelta::zero()
            || duration > self.limits.max_deadline
        {
            return Err(GatewayError::DeadlineExceeded);
        }
        let _: Value =
            serde_json::from_slice(&request.body).map_err(|_| GatewayError::SchemaInvalid)?;
        if request.surface == ApiSurface::OgcApiFeaturesV1
            && !request.operation.starts_with("/collections/")
        {
            return Err(GatewayError::SchemaInvalid);
        }
        if !operation_allowed(request.surface, request.lane, &request.operation) {
            return Err(GatewayError::Forbidden);
        }
        if !self
            .authorization
            .authorize(auth, &request.operation, request.lane)
        {
            return Err(GatewayError::Forbidden);
        }
        let effectful = matches!(request.lane, TrafficLane::Command | TrafficLane::Bulk);
        if effectful && request.idempotency_key.is_none() {
            return Err(GatewayError::IdempotencyRequired);
        }
        if let Some(key) = &request.idempotency_key
            && let Some((prior, response)) = self.idempotency.get(&(
                auth.tenant.clone(),
                auth.region.clone(),
                auth.principal.clone(),
                request.operation.clone(),
                key.clone(),
            ))
        {
            return if prior == &body_digest {
                Ok(response.clone())
            } else {
                Err(GatewayError::IdempotencyConflict)
            };
        }
        let window = request.issued_at.timestamp() / 60;
        let rate = self
            .rates
            .entry((auth.tenant.clone(), request.lane, window))
            .or_default();
        if *rate >= self.limits.requests_per_window {
            return Err(GatewayError::RateLimited);
        }
        *rate += 1;
        let active = self.active.entry(request.lane).or_default();
        if *active >= self.limits.max_concurrent {
            return Err(GatewayError::ConcurrencyLimited);
        }
        *active += 1;
        let validated = ValidatedRequest(request.clone(), body_digest.clone());
        let result = self.context.invoke(&validated);
        *active -= 1;
        let response = result?;
        if response.body.len() > self.limits.max_response_body_bytes
            || response.content_type != "application/json"
            || serde_json::from_str::<Value>(&response.body).is_err()
        {
            return Err(GatewayError::InvalidResponse);
        }
        if let Some(key) = request.idempotency_key {
            self.idempotency.insert(
                (
                    auth.tenant.clone(),
                    auth.region.clone(),
                    auth.principal.clone(),
                    request.operation.clone(),
                    key,
                ),
                (body_digest, response.clone()),
            );
        }
        Ok(response)
    }
}

#[allow(clippy::case_sensitive_file_extension_comparisons)]
fn operation_allowed(surface: ApiSurface, lane: TrafficLane, operation: &str) -> bool {
    match surface {
        ApiSurface::InternalGrpc => false,
        ApiSurface::OgcApiFeaturesV1 => {
            lane == TrafficLane::Query && operation.starts_with("/collections/")
        }
        ApiSurface::RestV1 => match lane {
            TrafficLane::Query => operation.ends_with(".read"),
            TrafficLane::Command => operation.ends_with(".command"),
            TrafficLane::Bulk => operation.ends_with(".bulk"),
            TrafficLane::Export => operation.ends_with(".export"),
            TrafficLane::Public => operation.ends_with(".public"),
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OgcQuery {
    pub limit: u16,
    pub offset: u32,
    pub bbox: Option<[f64; 4]>,
}
impl OgcQuery {
    pub fn new(limit: u16, offset: u32, bbox: Option<[f64; 4]>) -> Result<Self, GatewayError> {
        if limit == 0
            || limit > 1000
            || bbox.is_some_and(|b| {
                b.iter().any(|v| !v.is_finite())
                    || b[0] >= b[2]
                    || b[1] >= b[3]
                    || b[0] < -180.0
                    || b[2] > 180.0
                    || b[1] < -90.0
                    || b[3] > 90.0
            })
        {
            return Err(GatewayError::InvalidRequest);
        }
        Ok(Self {
            limit,
            offset,
            bbox,
        })
    }
}
