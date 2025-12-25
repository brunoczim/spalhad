use anyhow::anyhow;
use axum::{
    Json,
    Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use spalhad_spec::cluster::{
    ActivateRequest,
    ActivateResponse,
    IsActiveResponse,
    RunIdResponse,
};

use crate::{
    actor::bouncer::{self, Activated},
    http::{
        App,
        error::{self, HttpResult},
    },
};

pub fn router() -> Router<App> {
    Router::new()
        .route("/runid", get(run_id))
        .route("/activate", post(activate))
        .route("/active", get(is_active))
}

pub async fn run_id(State(app): State<App>) -> HttpResult<RunIdResponse> {
    let is_active = app
        .bouncer()
        .send(bouncer::IsActive)
        .await
        .map_err(error::make_response(StatusCode::INTERNAL_SERVER_ERROR))?;
    if is_active {
        let response = RunIdResponse { run_id: app.self_run_id() };
        Ok(Json(response))
    } else {
        Err(anyhow!("not active yet"))
            .map_err(error::make_response(StatusCode::FORBIDDEN))
    }
}

pub async fn activate(
    State(app): State<App>,
    Json(body): Json<ActivateRequest>,
) -> HttpResult<ActivateResponse> {
    app.bouncer()
        .send(bouncer::Activate { run_id: body.run_id })
        .await
        .map_err(error::when_not_bouncer(StatusCode::INTERNAL_SERVER_ERROR))
        .map(|Activated| ActivateResponse { is_active: true })
        .map(Json)
}

pub async fn is_active(State(app): State<App>) -> HttpResult<IsActiveResponse> {
    app.bouncer()
        .send(bouncer::IsActive)
        .await
        .map_err(error::make_response(StatusCode::INTERNAL_SERVER_ERROR))
        .map(|is_active| IsActiveResponse { is_active })
        .map(Json)
}
