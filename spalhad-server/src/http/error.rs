use axum::{Json, http::StatusCode};

pub use spalhad_spec::kv::Error;

pub type HttpResult<T, E = (StatusCode, Json<Error>)> = Result<Json<T>, E>;

pub fn make_response(
    status: StatusCode,
) -> impl FnOnce(anyhow::Error) -> (StatusCode, Json<Error>) + Send + 'static {
    move |error| {
        let trace = error.chain().map(ToString::to_string).collect();
        (status, Json(Error { trace }))
    }
}
