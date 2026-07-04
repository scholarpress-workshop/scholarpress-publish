mod compile;
mod extract;
mod institutions;
mod spec;
mod template;
mod validate;

use crate::institutions::Registry;
use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};

pub fn router() -> Router<Registry> {
    Router::new()
        .route("/extract", post(extract::handler))
        .route("/compile", post(compile::handler))
        .route(
            "/validate",
            post(validate::handler).layer(DefaultBodyLimit::max(50 * 1024 * 1024)),
        )
        .route("/health", get(|| async { "ok" }))
        .route("/institutions", get(institutions::handler))
        .route("/institutions/:id/spec", get(spec::handler))
        .route("/institutions/:id/template", get(template::handler))
}
