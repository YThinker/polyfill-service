#![warn(clippy::all, clippy::pedantic, clippy::cargo)]
#![allow(clippy::missing_docs_in_private_items)]
mod pages;
mod polyfill;

use axum::{
    extract::Path,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use pages::home;
use polyfill::polyfill_handler;
pub use polyfill_library::Env;
use std::sync::Arc;

const APPLICATION_JSON: &str = "application/json";

pub fn router(env: Arc<Env>) -> Router {
    Router::new()
        .route("/", get(home_handler))
        .route("/img/logo.svg", get(logo))
        .route("/robots.txt", get(robots))
        .route("/v2/polyfill.js", get(polyfill_handler))
        .route("/v2/polyfill.min.js", get(polyfill_handler))
        .route("/v3/polyfill.js", get(polyfill_handler))
        .route("/v3/polyfill.min.js", get(polyfill_handler))
        .route("/v3/json/:file", get(library_json))
        .with_state(env)
}

async fn home_handler() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        "content-type",
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    headers.insert("x-compress-hint", HeaderValue::from_static("on"));
    headers.insert(
        "X-XSS-Protection",
        HeaderValue::from_static("1; mode=block"),
    );
    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("origin-when-cross-origin"),
    );
    headers.insert(
        "Strict-Transport-Security",
        HeaderValue::from_static("max-age=31536000; includeSubdomains; preload"),
    );
    headers.insert(
        "Cache-Control",
        HeaderValue::from_static("max-age=60, stale-while-revalidate=60, stale-if-error=86400"),
    );

    (headers, home())
}

async fn logo() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert("content-type", HeaderValue::from_static("image/svg+xml"));
    headers.insert("x-compress-hint", HeaderValue::from_static("on"));
    headers.insert("surrogate-key", HeaderValue::from_static("website"));

    (headers, include_str!("logo.svg"))
}

async fn robots() -> impl IntoResponse {
    "User-agent: *\nDisallow:"
}

fn library_json_body(version: &str) -> Option<&'static str> {
    match version {
        "3.101.0" => Some(include_str!("json/library-3.101.0.json")),
        "3.103.0" => Some(include_str!("json/library-3.103.0.json")),
        "3.104.0" => Some(include_str!("json/library-3.104.0.json")),
        "3.108.0" => Some(include_str!("json/library-3.108.0.json")),
        "3.109.0" => Some(include_str!("json/library-3.109.0.json")),
        "3.110.1" => Some(include_str!("json/library-3.110.1.json")),
        "3.111.0" => Some(include_str!("json/library-3.111.0.json")),
        "3.27.4" => Some(include_str!("json/library-3.27.4.json")),
        "3.34.0" => Some(include_str!("json/library-3.34.0.json")),
        "3.39.0" => Some(include_str!("json/library-3.39.0.json")),
        "3.40.0" => Some(include_str!("json/library-3.40.0.json")),
        "3.41.0" => Some(include_str!("json/library-3.41.0.json")),
        "3.42.0" => Some(include_str!("json/library-3.42.0.json")),
        "3.46.0" => Some(include_str!("json/library-3.46.0.json")),
        "3.48.0" => Some(include_str!("json/library-3.48.0.json")),
        "3.50.2" => Some(include_str!("json/library-3.50.2.json")),
        "3.51.0" => Some(include_str!("json/library-3.51.0.json")),
        "3.52.0" => Some(include_str!("json/library-3.52.0.json")),
        "3.52.1" => Some(include_str!("json/library-3.52.1.json")),
        "3.52.2" => Some(include_str!("json/library-3.52.2.json")),
        "3.52.3" => Some(include_str!("json/library-3.52.3.json")),
        "3.53.1" => Some(include_str!("json/library-3.53.1.json")),
        "3.89.4" => Some(include_str!("json/library-3.89.4.json")),
        "3.96.0" => Some(include_str!("json/library-3.96.0.json")),
        "3.98.0" => Some(include_str!("json/library-3.98.0.json")),
        "4.8.0" => Some(include_str!("json/library-4.8.0.json")),
        _ => None,
    }
}

async fn library_json(Path(file): Path<String>) -> impl IntoResponse {
    let version = file
        .strip_prefix("library-")
        .and_then(|v| v.strip_suffix(".json"));

    let Some(version) = version else {
        return StatusCode::NOT_FOUND.into_response();
    };

    match library_json_body(version) {
        Some(body) => {
            let mut headers = HeaderMap::new();
            headers.insert("content-type", HeaderValue::from_static(APPLICATION_JSON));
            headers.insert("x-compress-hint", HeaderValue::from_static("on"));
            headers.insert("surrogate-key", HeaderValue::from_static("website"));
            headers.insert(
                "Cache-Control",
                HeaderValue::from_static(
                    "max-age=86400, stale-while-revalidate=86400, stale-if-error=86400",
                ),
            );

            (headers, body.to_owned()).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
