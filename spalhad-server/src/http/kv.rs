use anyhow::anyhow;
use axum::{
    Json,
    Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use spalhad_spec::kv::{GetResponse, Key, PutRequest, PutResponse};

use crate::actor::storage;

use super::{
    App,
    error::{self, HttpResult},
};

pub fn router() -> Router<App> {
    Router::new()
        .route("/{key}", get(get_by_key))
        .route("/{key}", post(put_by_key))
}

async fn get_by_key(
    State(app): State<App>,
    Path(key): Path<Key>,
) -> HttpResult<GetResponse<serde_json::Value>> {
    app.bouncer()
        .send(storage::Get { key })
        .await
        .map_err(error::catch_bouncer(StatusCode::INTERNAL_SERVER_ERROR))?
        .ok_or_else(|| anyhow!("key not found"))
        .map_err(error::make_response(StatusCode::NOT_FOUND))
        .map(|value| GetResponse { value })
        .map(Json)
}

async fn put_by_key(
    State(app): State<App>,
    Path(key): Path<Key>,
    Json(body): Json<PutRequest<serde_json::Value>>,
) -> HttpResult<PutResponse> {
    app.bouncer()
        .send(storage::Put { key, value: body.value })
        .await
        .map_err(error::catch_bouncer(StatusCode::INTERNAL_SERVER_ERROR))
        .map(|new| PutResponse { new })
        .map(Json)
}
