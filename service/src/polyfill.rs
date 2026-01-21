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
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;
use tokio::fs;
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
    // Error responses should have shorter cache time
    headers.insert(
        "Cache-Control",
        HeaderValue::from_static(
            "public, max-age=300, stale-while-revalidate=300, stale-if-error=86400",
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

fn generate_cache_key(
    params: &polyfill_library::polyfill_parameters::PolyfillParameters,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(params.version.as_bytes());
    hasher.update(b"\0");
    hasher.update(params.ua_string.as_bytes());
    hasher.update(b"\0");
    hasher.update(if params.minify { b"1" } else { b"0" });
    hasher.update(b"\0");
    hasher.update(params.unknown.as_bytes());
    hasher.update(b"\0");
    hasher.update(params.strict.to_string().as_bytes());
    hasher.update(b"\0");

    // Serialize features
    let mut features_vec: Vec<_> = params.features.iter().collect();
    features_vec.sort_by_key(|(k, _)| *k);
    for (k, v) in features_vec {
        hasher.update(k.as_bytes());
        hasher.update(b"=");
        let mut flags_vec: Vec<_> = v.iter().collect();
        flags_vec.sort();
        for flag in flags_vec {
            hasher.update(flag.as_bytes());
            hasher.update(b",");
        }
        hasher.update(b"\0");
    }

    // Serialize excludes
    let mut excludes = params.excludes.clone();
    excludes.sort();
    for exclude in excludes {
        hasher.update(exclude.as_bytes());
        hasher.update(b"\0");
    }

    // Serialize callback
    if let Some(ref callback) = params.callback {
        hasher.update(callback.as_bytes());
    }
    hasher.update(b"\0");

    format!("{:x}", hasher.finalize())
}

#[derive(Clone)]
enum CacheResult {
    Empty,
    HasContent(String),
    Miss,
}

async fn read_cache(
    empty_cache_keys: &Arc<RwLock<HashSet<String>>>,
    cache_dir: Option<&std::path::Path>,
    key: &str,
) -> CacheResult {
    // First check memory set for empty result (fastest path)
    {
        let empty_keys = empty_cache_keys.read().unwrap();
        if empty_keys.contains(key) {
            return CacheResult::Empty;
        }
    }

    // Then check disk cache for regular content
    if let Some(cache_dir) = cache_dir {
        let cache_path = cache_dir.join(format!("{}.js", key));
        match fs::read_to_string(&cache_path).await {
            Ok(content) => return CacheResult::HasContent(content),
            Err(_) => {}
        }
    }

    CacheResult::Miss
}

fn write_cache_async(
    empty_cache_keys: Arc<RwLock<HashSet<String>>>,
    cache_dir: Option<PathBuf>,
    key: String,
    content: String,
    is_empty: bool,
) {
    if is_empty {
        // Store empty result key in memory set (fast, no I/O)
        let empty_keys = empty_cache_keys.clone();
        tokio::spawn(async move {
            let mut keys = empty_keys.write().unwrap();
            keys.insert(key);
        });
    } else {
        // Write content to disk cache in background (non-blocking)
        if let Some(cache_dir) = cache_dir {
            tokio::spawn(async move {
                // Ensure cache directory exists
                if let Err(err) = fs::create_dir_all(&cache_dir).await {
                    tracing::warn!("Failed to create cache directory: {}", err);
                    return;
                }

                let cache_path = cache_dir.join(format!("{}.js", key));
                if let Err(err) = fs::write(&cache_path, content).await {
                    tracing::warn!("Failed to write cache for key {}: {}", key, err);
                }
            });
        }
    }
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
    headers.insert(
        "Content-Type",
        HeaderValue::from_static("text/javascript; charset=UTF-8"),
    );
    headers.insert(
        "Cache-Control",
        HeaderValue::from_static(
            "public, max-age=2592000, stale-while-revalidate=2592000, stale-if-error=2592000, immutable",
        ),
    );
    headers.insert(
        "Vary",
        HeaderValue::from_static("User-Agent, Accept-Encoding"),
    );
    // if let Ok(val) = HeaderValue::from_str(&version) {
    //     headers.insert("Cf-Polyfill-Version", val);
    // }

    // Try to read from cache
    let cache_key = generate_cache_key(&params);
    match read_cache(&env.empty_cache_keys, env.cache_dir.as_deref(), &cache_key).await {
        CacheResult::Empty => {
            // Cache hit: empty result - directly return without any processing
            let mut empty_content = String::from("/*\n");
            empty_content.push_str(" * Polyfill service v");
            empty_content.push_str(&version);
            empty_content.push_str("\n");
            if !params.minify {
                empty_content.push_str(
                        " * For detailed credits and licence information see https://cdnjs.cloudflare.com/polyfill.\n",
                    );
                empty_content.push_str(" *\n");
                let mut features: Vec<String> = params
                    .features
                    .keys()
                    .map(std::clone::Clone::clone)
                    .collect();
                features.sort();
                empty_content.push_str(" * Features requested: ");
                empty_content.push_str(&features.join(","));
                empty_content.push_str("\n *\n");
                empty_content.push_str(" * No polyfills needed for current settings and browser\n");
            } else {
                empty_content.push_str(
                    " * Disable minification (remove `.min` from URL path) for more info\n",
                );
            }
            empty_content.push_str(" */\n\n");

            if let Some(ref callback) = params.callback {
                empty_content.push_str("\ntypeof ");
                empty_content.push_str(callback);
                empty_content.push_str("==='function' && ");
                empty_content.push_str(callback);
                empty_content.push_str("();");
            }

            let mut response = Response::new(Body::from(empty_content));
            *response.status_mut() = StatusCode::OK;
            *response.headers_mut() = headers;
            return response;
        }
        CacheResult::HasContent(cached_content) => {
            // Cache hit: has content - return cached result
            let mut response = Response::new(Body::from(cached_content));
            *response.status_mut() = StatusCode::OK;
            *response.headers_mut() = headers;
            return response;
        }
        CacheResult::Miss => {
            // Cache miss: continue to generate
        }
    }

    // Generate polyfill bundle
    let mut res_body = Buffer::new();

    if let Err(err) =
        get_polyfill_string_stream(&mut res_body, &params, env.clone(), &version).await
    {
        return into_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to get polyfill bundle: {err}"),
        );
    }

    let content = res_body.into_str();

    // Check if result is empty (no polyfills needed)
    // Empty result typically contains only comments and optional callback
    // We consider it empty if it doesn't contain the polyfill wrapper function
    let is_empty = !content.contains("(function(self, undefined) {");

    // Write to cache asynchronously (empty keys in memory, content on disk)
    write_cache_async(
        env.empty_cache_keys.clone(),
        env.cache_dir.clone(),
        cache_key,
        content.clone(),
        is_empty,
    );

    let mut response = Response::new(Body::from(content));
    *response.status_mut() = StatusCode::OK;
    *response.headers_mut() = headers;
    response
}
