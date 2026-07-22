#![forbid(unsafe_code)]
//! External HTTP adapter executable for the API gateway policy core.
use api_gateway::http::{EmptyReadModels, StaticTokenVerifier, router};
use std::{env, sync::Arc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = env::var("WR_API_TOKEN")?;
    let tenant = env::var("WR_TENANT")?;
    let region = env::var("WR_REGION")?;
    let verifier = StaticTokenVerifier::new(token, "gateway-principal".into(), tenant, region)?;
    let app = router(Arc::new(verifier), Arc::new(EmptyReadModels));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
