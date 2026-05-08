mod error;
mod router;

use std::{env, net::SocketAddr};

use error::AppError;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

const DEFAULT_BIND_ADDR: &str = "0.0.0.0:8080";

#[tokio::main]
async fn main() -> Result<(), AppError> {
    init_tracing()?;

    let bind_addr = read_bind_addr()?;
    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    let local_addr = listener.local_addr()?;

    info!(%local_addr, "backend listening");

    axum::serve(listener, router::build_router()).await?;

    Ok(())
}

fn init_tracing() -> Result<(), AppError> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("backend=info,tower_http=info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer())
        .try_init()
        .map_err(|error| AppError::TracingInit(error.to_string()))?;

    Ok(())
}

fn read_bind_addr() -> Result<SocketAddr, AppError> {
    let value = env::var("BIND_ADDR").unwrap_or_else(|_| DEFAULT_BIND_ADDR.to_owned());

    value.parse().map_err(|source| AppError::InvalidBindAddress {
        value,
        source,
    })
}
