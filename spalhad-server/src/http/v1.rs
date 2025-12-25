use axum::Router;

use crate::http::App;

pub mod kv;
pub mod sync;
pub mod internal;

pub fn router() -> Router<App> {
    Router::new()
        .nest("/kv", kv::router())
        .nest("/sync", sync::router())
        .nest("/internal/kv", internal::router())
}
