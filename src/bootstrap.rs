use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};

use crate::{error::AppError, state::AppState};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapRequest {
    username: String,
    display_name: String,
    email: Option<String>,
    password: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapResponse {
    user: BootstrapUser,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BootstrapUser {
    id: i64,
    username: String,
    display_name: String,
    email: Option<String>,
    role: String,
}

pub async fn bootstrap_admin(
    State(state): State<AppState>,
    Json(payload): Json<BootstrapRequest>,
) -> Result<impl IntoResponse, AppError> {
    let payload = ValidatedBootstrapRequest::try_from(payload)?;
    let password_hash = hash_password(&payload.password)?;

    let result = sqlx::query(
        "INSERT INTO users (username, display_name, email, password_hash, role)
         SELECT ?, ?, ?, ?, 'admin'
         WHERE NOT EXISTS (SELECT 1 FROM users LIMIT 1)",
    )
    .bind(&payload.username)
    .bind(&payload.display_name)
    .bind(payload.email.as_deref())
    .bind(&password_hash)
    .execute(state.pool())
    .await
    .map_err(AppError::Database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::Conflict(
            "bootstrap is already complete; at least one user already exists".to_owned(),
        ));
    }

    let response = BootstrapResponse {
        user: BootstrapUser {
            id: result.last_insert_rowid(),
            username: payload.username,
            display_name: payload.display_name,
            email: payload.email,
            role: "admin".to_owned(),
        },
    };

    Ok((StatusCode::CREATED, Json(response)))
}

#[derive(Debug)]
struct ValidatedBootstrapRequest {
    username: String,
    display_name: String,
    email: Option<String>,
    password: String,
}

impl TryFrom<BootstrapRequest> for ValidatedBootstrapRequest {
    type Error = AppError;

    fn try_from(value: BootstrapRequest) -> Result<Self, Self::Error> {
        let username = value.username.trim().to_owned();
        if username.is_empty() {
            return Err(AppError::Validation(
                "username must not be empty".to_owned(),
            ));
        }

        let display_name = value.display_name.trim().to_owned();
        if display_name.is_empty() {
            return Err(AppError::Validation(
                "displayName must not be empty".to_owned(),
            ));
        }

        if value.password.len() < 8 {
            return Err(AppError::Validation(
                "password must be at least 8 characters long".to_owned(),
            ));
        }

        let email = value
            .email
            .map(|email| email.trim().to_owned())
            .filter(|email| !email.is_empty());

        Ok(Self {
            username,
            display_name,
            email,
            password: value.password,
        })
    }
}

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);

    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|source| AppError::PasswordHash(source.to_string()))
}
