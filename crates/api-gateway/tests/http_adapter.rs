#![allow(clippy::unwrap_used, missing_docs)]
use api_gateway::http::{EmptyReadModels, StaticTokenVerifier, router};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use std::sync::Arc;
use tower::ServiceExt;
fn app() -> axum::Router {
    router(
        Arc::new(
            StaticTokenVerifier::new(
                "valid".into(),
                "p".into(),
                "tenant-a".into(),
                "ca-central".into(),
            )
            .unwrap(),
        ),
        Arc::new(EmptyReadModels),
    )
}
#[tokio::test]
async fn health_and_openapi_are_available() {
    assert_eq!(
        app()
            .oneshot(Request::get("/health/ready").body(Body::empty()).unwrap())
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        app()
            .oneshot(Request::get("/openapi/v1").body(Body::empty()).unwrap())
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
}
#[tokio::test]
async fn protected_routes_require_verified_token() {
    let response = app()
        .oneshot(
            Request::get("/api/v1/incident/incidents")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let response = app()
        .oneshot(
            Request::get("/ogc/features/v1/collections/incidents/items")
                .header("authorization", "Bearer valid")
                .header("x-tenant-id", "tenant-b")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
#[tokio::test]
async fn internal_grpc_has_no_public_route() {
    let response = app()
        .oneshot(
            Request::get("/grpc/incident.Command")
                .header("authorization", "Bearer valid")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
