use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, StatusCode, Uri},
    response::Response,
};
use polyfill_library::{
    buffer::Buffer,
    get_polyfill_string::get_polyfill_string_stream,
    polyfill_parameters::{get_polyfill_parameters, RequestParts},
    Env,
};
use std::sync::Arc;
use url::form_urlencoded;

const SUPPORTED_VERSIONS: &[&str] = &[
    "3.101.0", "3.103.0", "3.104.0", "3.108.0", "3.109.0", "3.110.1", "3.111.0", "3.25.1",
    "3.27.4", "3.34.0", "3.39.0", "3.40.0", "3.41.0", "3.42.0", "3.46.0", "3.48.0", "3.50.2",
    "3.51.0", "3.52.0", "3.52.1", "3.52.2", "3.52.3", "3.53.1", "3.89.4", "3.96.0", "3.98.0",
    "4.8.0",
];

fn parse_library_version(version: &str) -> String {
    if SUPPORTED_VERSIONS.contains(&version) {
        version.to_owned()
    } else {
        eprintln!("unknown version: {version}, using fallback.");
        "3.111.0".to_owned()
    }
}

fn into_error(status: StatusCode, message: impl AsRef<str>) -> Response {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Cache-Control",
        HeaderValue::from_static(
            "public, s-maxage=31536000, max-age=604800, stale-while-revalidate=604800, stale-if-error=604800, immutable",
        ),
    );
    let mut response = Response::new(Body::from(message.as_ref().to_string()));
    *response.status_mut() = status;
    *response.headers_mut() = headers;
    response
}

fn rewrite_v2_uri(uri: &Uri) -> Option<Uri> {
    let path = uri.path();
    if !(path == "/v2/polyfill.js" || path == "/v2/polyfill.min.js") {
        return None;
    }

    let mut serializer = form_urlencoded::Serializer::new(String::new());
    let mut has_unknown = false;
    if let Some(query) = uri.query() {
        for (k, v) in form_urlencoded::parse(query.as_bytes()) {
            if k == "unknown" {
                has_unknown = true;
            }
            serializer.append_pair(&k, &v);
        }
    }
    serializer.append_pair("version", "3.25.1");
    if !has_unknown {
        serializer.append_pair("unknown", "ignore");
    }
    let new_path = format!("/v3{}", &path[3..]);
    let query = serializer.finish();
    let new_uri = if query.is_empty() {
        new_path
    } else {
        format!("{}?{}", new_path, query)
    };
    new_uri.parse::<Uri>().ok()
}

pub async fn polyfill_handler(State(env): State<Arc<Env>>, req: Request) -> Response {
    let effective_uri = rewrite_v2_uri(req.uri()).unwrap_or_else(|| req.uri().clone());

    let params = get_polyfill_parameters(RequestParts {
        path: effective_uri.path(),
        query: effective_uri.query(),
        headers: req.headers(),
    });

    let version = parse_library_version(&params.version);
    let mut headers = HeaderMap::new();
    headers.insert("Access-Control-Allow-Origin", HeaderValue::from_static("*"));
    headers.insert(
        "Access-Control-Allow-Methods",
        HeaderValue::from_static("GET,HEAD,OPTIONS"),
    );
    headers.insert("X-Compress-Hint", HeaderValue::from_static("on"));
    headers.insert(
        "Content-Type",
        HeaderValue::from_static("text/javascript; charset=UTF-8"),
    );
    headers.insert(
        "Cache-Control",
        HeaderValue::from_static(
            "public, s-maxage=31536000, max-age=604800, stale-while-revalidate=604800, stale-if-error=604800, immutable",
        ),
    );
    headers.insert(
        "Vary",
        HeaderValue::from_static("User-Agent, Accept-Encoding"),
    );
    if let Ok(val) = HeaderValue::from_str(&version) {
        headers.insert("Cf-Polyfill-Version", val);
    }
    let mut res_body = Buffer::new();

    if let Err(err) = get_polyfill_string_stream(&mut res_body, &params, env, &version).await {
        return into_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to get polyfill bundle: {err}"),
        );
    }

    let mut response = Response::new(Body::from(res_body.into_str()));
    *response.status_mut() = StatusCode::OK;
    *response.headers_mut() = headers;
    response
}
