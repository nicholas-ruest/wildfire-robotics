//! Axum transport adapter. TLS is terminated by the managed ingress.
use crate::{AuthContext, GatewayError};
use axum::{
    Json, Router,
    body::Body,
    extract::{Path, Request, State},
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use serde_json::{Value, json};
use std::{collections::HashSet, sync::Arc};

pub trait TokenVerifier: Send + Sync + 'static {
    fn verify(&self, bearer: &str) -> Result<AuthContext, GatewayError>;
}
pub trait ReadModelPort: Send + Sync + 'static {
    fn query(
        &self,
        context: &str,
        resource: &str,
        auth: &AuthContext,
    ) -> Result<Value, GatewayError>;
}
#[derive(Clone)]
struct AppState {
    verifier: Arc<dyn TokenVerifier>,
    models: Arc<dyn ReadModelPort>,
}

pub fn router(verifier: Arc<dyn TokenVerifier>, models: Arc<dyn ReadModelPort>) -> Router {
    let state = AppState { verifier, models };
    let protected = Router::new()
        .route("/api/v1/{context}/{resource}", get(rest_query))
        .route(
            "/ogc/features/v1/collections/{collection}/items",
            get(ogc_items),
        )
        .route_layer(middleware::from_fn_with_state(state.clone(), authenticate));
    Router::new()
        .route("/health/live", get(|| async { StatusCode::OK }))
        .route("/health/ready", get(|| async { StatusCode::OK }))
        .route("/openapi/v1", get(openapi))
        .merge(protected)
        .with_state(state)
}
async fn authenticate(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Response {
    let token = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));
    let Some(token) = token else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    match state.verifier.verify(token) {
        Ok(auth) => {
            request.extensions_mut().insert(auth);
            next.run(request).await
        }
        Err(_) => StatusCode::UNAUTHORIZED.into_response(),
    }
}
async fn rest_query(
    State(state): State<AppState>,
    Path((context, resource)): Path<(String, String)>,
    request: Request<Body>,
) -> Response {
    query_response(&state, &context, &resource, &request)
}
async fn ogc_items(
    State(state): State<AppState>,
    Path(collection): Path<String>,
    request: Request<Body>,
) -> Response {
    query_response(&state, "ogc", &collection, &request)
}
fn query_response(
    state: &AppState,
    context: &str,
    resource: &str,
    request: &Request<Body>,
) -> Response {
    let Some(auth) = request.extensions().get::<AuthContext>() else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    match state.models.query(context, resource, auth) {
        Ok(value) => Json(value).into_response(),
        Err(_) => StatusCode::FORBIDDEN.into_response(),
    }
}
async fn openapi() -> Json<Value> {
    Json(
        json!({"openapi":"3.1.0","info":{"title":"Wildfire Robotics External API","version":"v1"},"paths":{"/health/live":{"get":{}},"/health/ready":{"get":{}},"/api/v1/{context}/{resource}":{"get":{"security":[{"bearerAuth":[]}]}},"/ogc/features/v1/collections/{collection}/items":{"get":{"security":[{"bearerAuth":[]}]}}},"components":{"securitySchemes":{"bearerAuth":{"type":"http","scheme":"bearer"}}}}),
    )
}

pub struct StaticTokenVerifier {
    token: String,
    principal: String,
    tenant: String,
    region: String,
}
impl StaticTokenVerifier {
    pub fn new(
        token: String,
        principal: String,
        tenant: String,
        region: String,
    ) -> Result<Self, GatewayError> {
        if token.is_empty() || principal.is_empty() || tenant.is_empty() || region.is_empty() {
            return Err(GatewayError::InvalidRequest);
        }
        Ok(Self {
            token,
            principal,
            tenant,
            region,
        })
    }
}
impl TokenVerifier for StaticTokenVerifier {
    fn verify(&self, bearer: &str) -> Result<AuthContext, GatewayError> {
        if bearer != self.token {
            return Err(GatewayError::Forbidden);
        }
        AuthContext::new(
            &self.principal,
            &self.tenant,
            &self.region,
            HashSet::from(["operator".into()]),
            "external-api",
        )
    }
}
pub struct EmptyReadModels;
impl ReadModelPort for EmptyReadModels {
    fn query(
        &self,
        context: &str,
        resource: &str,
        auth: &AuthContext,
    ) -> Result<Value, GatewayError> {
        Ok(
            json!({"context":context,"resource":resource,"tenant":auth.tenant(),"region":auth.region(),"freshness":"unknown","data":null}),
        )
    }
}
