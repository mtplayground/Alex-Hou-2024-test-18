mod config;
mod db;
mod error;
mod router;
mod state;
mod todo_repository;

use config::Config;
use db::Db;
use error::AppError;
use state::AppState;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let config = Config::from_env()?;
    init_tracing(&config.rust_log)?;
    let db = Db::connect(&config.database_url).await?;
    let state = AppState::new(db);

    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
    let local_addr = listener.local_addr()?;

    info!(
        %local_addr,
        static_dir = config
            .static_dir
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "disabled".to_owned()),
        database_pool_ready = true,
        "backend listening"
    );

    let router = router::build_router(state, &config)?;

    axum::serve(listener, router).await?;

    Ok(())
}

fn init_tracing(rust_log: &str) -> Result<(), AppError> {
    let env_filter = EnvFilter::new(rust_log.to_owned());

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer())
        .try_init()
        .map_err(|error| AppError::TracingInit(error.to_string()))?;

    Ok(())
}
