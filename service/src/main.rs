use service::{router, Env};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tracing_subscriber::EnvFilter;

fn build_env(polyfill_base: PathBuf) -> Result<Env, Box<dyn std::error::Error>> {
    Ok(Env {
        polyfill_base,
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
