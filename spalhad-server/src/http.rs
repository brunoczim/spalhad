use anyhow::Result;
use axum::Router;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

pub use app::App;

mod error;
mod app;

pub mod kv;
pub mod sync;
pub mod internal;

pub fn router() -> Router<App> {
    Router::new()
        .nest("/kv", kv::router())
        .nest("/sync", sync::router())
        .nest("/internal-kv", internal::router())
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
