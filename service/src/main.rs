use service::{router, Env};
use std::collections::HashSet;
use std::sync::RwLock;
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tracing_subscriber::EnvFilter;

fn build_env(polyfill_base: PathBuf) -> Result<Env, Box<dyn std::error::Error>> {
    // Use CACHE_DIR env var if set, otherwise default to "./cache-dir" directory
    let cache_dir = std::env::var("CACHE_DIR")
        .ok()
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
        .or_else(|| Some(PathBuf::from("cache-dir")));

    Ok(Env {
        polyfill_base,
        cache_dir,
        empty_cache_keys: Arc::new(RwLock::new(HashSet::new())),
        up_to_date_ua_metric: prometheus::IntCounter::new(
            "polyfill_up_to_date_ua_total",
            "User agents that do not need polyfills",
        )?,
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let polyfill_base = std::env::var("POLYFILL_BASE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("polyfill-libraries"));
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8787);

    let env = Arc::new(build_env(polyfill_base)?);
    let app = router(Arc::clone(&env));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Starting polyfill service on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
