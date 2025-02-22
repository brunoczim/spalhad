use axum::{Json, Router, extract::State, routing::get};
use spalhad_spec::cluster::RunIdResponse;

use super::{App, error::HttpResult};

pub fn router() -> Router<App> {
    Router::new().route("/run_id", get(run_id))
}

pub async fn run_id(State(app): State<App>) -> HttpResult<RunIdResponse> {
    let response = RunIdResponse { run_id: app.self_run_id() };
    Ok(Json(response))
}
