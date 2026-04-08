use std::{ffi::OsString, path::PathBuf};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("config file does not exist: {0}")]
    MissingConfig(PathBuf),

    #[error("failed to read config file {path}: {source}")]
    ConfigRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse config file {path}: {source}")]
    ConfigParse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("invalid config value for {field}: {value} ({reason})")]
    InvalidConfigValue {
        field: &'static str,
        value: String,
        reason: String,
    },

    #[error("invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("invalid environment override {key}={value}: {reason}")]
    InvalidEnvironment {
        key: &'static str,
        value: String,
        reason: String,
    },

    #[error("environment override {key} is not valid UTF-8: {value:?}")]
    NonUnicodeEnvironment { key: &'static str, value: OsString },

    #[error("failed to initialize tracing: {0}")]
    TracingInitialization(String),

    #[error("failed to create directory {path}: {source}")]
    CreateDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid database URL {value}: {reason}")]
    InvalidDatabaseUrl { value: String, reason: String },

    #[error("database query failed: {0}")]
    Database(#[source] sqlx::Error),

    #[error("database migration failed: {0}")]
    Migration(#[source] sqlx::migrate::MigrateError),

    #[error("password hashing failed: {0}")]
    PasswordHash(String),

    #[error("password verification failed: {0}")]
    PasswordVerification(String),

    #[error("token encoding failed: {0}")]
    TokenEncoding(String),

    #[error("time handling failed: {0}")]
    Time(String),

    #[error("{0}")]
    Validation(String),

    #[error("{0}")]
    Conflict(String),

    #[error("{0}")]
    Unauthorized(String),

    #[error("{0}")]
    Forbidden(String),

    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    Internal(String),

    #[error("failed to {action}: {source}")]
    ServerIo {
        action: &'static str,
        #[source]
        source: std::io::Error,
    },
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::MissingConfig(_)
            | Self::ConfigRead { .. }
            | Self::ConfigParse { .. }
            | Self::InvalidConfigValue { .. }
            | Self::InvalidConfiguration(_)
            | Self::InvalidEnvironment { .. }
            | Self::NonUnicodeEnvironment { .. }
            | Self::InvalidDatabaseUrl { .. }
            | Self::Validation(_)
            | Self::TracingInitialization(_) => StatusCode::BAD_REQUEST,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::CreateDirectory { .. }
            | Self::Database(_)
            | Self::Migration(_)
            | Self::PasswordHash(_)
            | Self::PasswordVerification(_)
            | Self::TokenEncoding(_)
            | Self::Time(_)
            | Self::Internal(_)
            | Self::ServerIo { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
        };

        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}
