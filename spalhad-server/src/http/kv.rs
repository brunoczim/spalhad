use anyhow::anyhow;
use axum::{
    Json,
    Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use spalhad_spec::{GetResponse, Key, PutResponse};

use crate::storage::StorageHandle;

use super::error::{self, HttpResult};

pub fn router(storage: StorageHandle) -> Router {
    Router::new()
        .route("/{key}", get(get_by_key))
        .route("/{key}", post(put_by_key))
        .with_state(storage)
}

async fn get_by_key(
    State(storage): State<StorageHandle>,
    Path(key): Path<Key>,
) -> HttpResult<GetResponse<serde_json::Value>> {
    let output = storage
        .get(key)
        .await
        .map_err(error::make_response(StatusCode::INTERNAL_SERVER_ERROR))?;
    match output {
        Some(value) => Ok(Json(GetResponse { value })),
        None => Err(anyhow!("key not found"))
            .map_err(error::make_response(StatusCode::NOT_FOUND)),
    }
}

async fn put_by_key(
    State(storage): State<StorageHandle>,
    Path(key): Path<Key>,
    Json(value): Json<serde_json::Value>,
) -> HttpResult<PutResponse> {
    storage
        .put(key, value)
        .await
        .map_err(error::make_response(StatusCode::INTERNAL_SERVER_ERROR))
        .map(|new| PutResponse { new })
        .map(Json)
}
