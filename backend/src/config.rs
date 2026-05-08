use std::{env, net::SocketAddr, path::PathBuf};

use crate::error::AppError;

const DEFAULT_BIND_ADDR: &str = "0.0.0.0:8080";
const DEFAULT_RUST_LOG: &str = "backend=info,tower_http=info";
const DEFAULT_CORS_ALLOWED_ORIGINS: [&str; 2] = ["http://127.0.0.1:8080", "http://localhost:8080"];

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: SocketAddr,
    pub rust_log: String,
    pub static_dir: Option<PathBuf>,
    pub cors_allowed_origins: Vec<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        match dotenvy::dotenv() {
            Ok(_) => {}
            Err(error) if error.not_found() => {}
            Err(error) => return Err(AppError::Dotenv(error)),
        }

        let database_url = required_var("DATABASE_URL")?;
        let bind_addr = parse_bind_addr(optional_var("BIND_ADDR"), DEFAULT_BIND_ADDR)?;
        let rust_log = optional_var("RUST_LOG").unwrap_or_else(|| DEFAULT_RUST_LOG.to_owned());
        let static_dir = parse_static_dir(optional_var("STATIC_DIR"));
        let cors_allowed_origins = parse_cors_allowed_origins(optional_var("CORS_ALLOWED_ORIGINS"));

        Ok(Self {
            database_url,
            bind_addr,
            rust_log,
            static_dir,
            cors_allowed_origins,
        })
    }
}

fn required_var(name: &'static str) -> Result<String, AppError> {
    env::var(name).map_err(|source| AppError::MissingEnvVar { name, source })
}

fn optional_var(name: &'static str) -> Option<String> {
    env::var(name).ok()
}

fn parse_bind_addr(value: Option<String>, default: &str) -> Result<SocketAddr, AppError> {
    let value = value.unwrap_or_else(|| default.to_owned());

    value
        .parse()
        .map_err(|source| AppError::InvalidBindAddress { value, source })
}

fn parse_cors_allowed_origins(value: Option<String>) -> Vec<String> {
    match value {
        Some(value) => value
            .split(',')
            .map(str::trim)
            .filter(|origin| !origin.is_empty())
            .map(ToOwned::to_owned)
            .collect(),
        None => DEFAULT_CORS_ALLOWED_ORIGINS
            .iter()
            .map(|origin| (*origin).to_owned())
            .collect(),
    }
}

fn parse_static_dir(value: Option<String>) -> Option<PathBuf> {
    value.and_then(|value| {
        let trimmed = value.trim();

        if trimmed.is_empty() {
            None
        } else {
            Some(PathBuf::from(trimmed))
        }
    })
}
