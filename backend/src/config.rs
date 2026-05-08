use std::{
    env,
    net::SocketAddr,
    path::PathBuf,
};

use crate::error::AppError;

const DEFAULT_BIND_ADDR: &str = "0.0.0.0:8080";
const DEFAULT_RUST_LOG: &str = "backend=info,tower_http=info";
const DEFAULT_STATIC_DIR: &str = "frontend/dist";

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: SocketAddr,
    pub rust_log: String,
    pub static_dir: PathBuf,
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
        let static_dir =
            PathBuf::from(optional_var("STATIC_DIR").unwrap_or_else(|| DEFAULT_STATIC_DIR.to_owned()));

        Ok(Self {
            database_url,
            bind_addr,
            rust_log,
            static_dir,
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

    value.parse().map_err(|source| AppError::InvalidBindAddress {
        value,
        source,
    })
}
