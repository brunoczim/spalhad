use anyhow::Result;
use axum::Router;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use crate::storage::StorageHandle;

pub mod error;
pub mod kv;

pub fn router(kv_storage: StorageHandle) -> Router {
    Router::new().nest("/kv", kv::router(kv_storage))
}

pub async fn serve(bind_address: &str, router: Router) -> Result<()> {
    tracing::info!(%bind_address, "binding server socket listener...");
    let listener = TcpListener::bind(bind_address).await?;
    tracing::info!("socket bound.");
    tracing::info!("serving...");
    let router = router.route_layer(TraceLayer::new_for_http());
    axum::serve(listener, router).await?;
    Ok(())
}
