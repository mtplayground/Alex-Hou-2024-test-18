use std::{env::VarError, fmt, net::AddrParseError};

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Debug)]
pub enum AppError {
    Dotenv(dotenvy::Error),
    MissingEnvVar {
        name: &'static str,
        source: VarError,
    },
    InvalidBindAddress {
        value: String,
        source: AddrParseError,
    },
    Io(std::io::Error),
    TracingInit(String),
}

impl AppError {
    fn code(&self) -> &'static str {
        match self {
            Self::Dotenv(_) => "dotenv_load_failed",
            Self::MissingEnvVar { .. } => "missing_env_var",
            Self::InvalidBindAddress { .. } => "invalid_bind_address",
            Self::Io(_) => "io_error",
            Self::TracingInit(_) => "tracing_init_failed",
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::Dotenv(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::MissingEnvVar { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InvalidBindAddress { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Io(_) | Self::TracingInit(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dotenv(error) => write!(f, "failed to load .env file: {error}"),
            Self::MissingEnvVar { name, .. } => {
                write!(f, "missing required environment variable `{name}`")
            }
            Self::InvalidBindAddress { value, .. } => {
                write!(f, "failed to parse BIND_ADDR value `{value}`")
            }
            Self::Io(error) => write!(f, "i/o error: {error}"),
            Self::TracingInit(error) => write!(f, "failed to initialize tracing: {error}"),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Dotenv(error) => Some(error),
            Self::MissingEnvVar { source, .. } => Some(source),
            Self::InvalidBindAddress { source, .. } => Some(source),
            Self::Io(error) => Some(error),
            Self::TracingInit(_) => None,
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct ErrorBody<'a> {
            error: ErrorPayload<'a>,
        }

        #[derive(Serialize)]
        struct ErrorPayload<'a> {
            code: &'a str,
            message: String,
        }

        let status = self.status_code();
        let body = ErrorBody {
            error: ErrorPayload {
                code: self.code(),
                message: self.to_string(),
            },
        };

        (status, Json(body)).into_response()
    }
}
