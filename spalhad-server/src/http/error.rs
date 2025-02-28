use axum::{Json, http::StatusCode};

pub use spalhad_spec::kv::Error;

use crate::actor::bouncer;

pub type HttpResult<T, E = (StatusCode, Json<Error>)> = Result<Json<T>, E>;

pub fn make_response(
    status: StatusCode,
) -> impl FnOnce(anyhow::Error) -> (StatusCode, Json<Error>) + Send + 'static {
    move |error| {
        let trace = error.chain().map(ToString::to_string).collect();
        (status, Json(Error { trace }))
    }
}

pub fn catch_bouncer(
    fallback_status: StatusCode,
) -> impl FnOnce(anyhow::Error) -> (StatusCode, Json<Error>) + Send + 'static {
    move |error| {
        if error.is::<bouncer::Error>() {
            make_response(StatusCode::BAD_REQUEST)(error)
        } else {
            make_response(fallback_status)(error)
        }
    }
}
