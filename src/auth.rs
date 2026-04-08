use std::time::{SystemTime, UNIX_EPOCH};

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, HeaderValue},
    middleware::Next,
    response::IntoResponse,
    Extension, Json,
};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteRow, Row};

use crate::{error::AppError, state::AppState};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticatedUser {
    pub id: i64,
    pub username: String,
    pub display_name: String,
    pub email: Option<String>,
    pub role: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    token: String,
    token_type: &'static str,
    expires_at: u64,
    user: AuthenticatedUser,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeResponse {
    user: AuthenticatedUser,
}

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    sub: i64,
    exp: usize,
    iat: usize,
    iss: String,
}

#[derive(Debug)]
struct StoredUser {
    id: i64,
    username: String,
    display_name: String,
    email: Option<String>,
    password_hash: String,
    role: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let username = payload.username.trim();
    if username.is_empty() || payload.password.is_empty() {
        return Err(AppError::Validation(
            "username and password must not be empty".to_owned(),
        ));
    }

    let Some(user) = fetch_user_by_username(state.pool(), username).await? else {
        return Err(AppError::Unauthorized(
            "invalid username or password".to_owned(),
        ));
    };

    let password_is_valid = verify_password(&payload.password, &user.password_hash)?;
    if !password_is_valid {
        return Err(AppError::Unauthorized(
            "invalid username or password".to_owned(),
        ));
    }

    let now = current_unix_timestamp()?;
    let expires_at = now + state.config().auth.token_ttl_seconds;
    let claims = Claims {
        sub: user.id,
        exp: expires_at as usize,
        iat: now as usize,
        iss: state.config().auth.jwt_issuer.clone(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config().auth.jwt_secret.as_bytes()),
    )
    .map_err(|source| AppError::TokenEncoding(source.to_string()))?;

    Ok(Json(LoginResponse {
        token,
        token_type: "Bearer",
        expires_at,
        user: user.into_authenticated_user(),
    }))
}

pub async fn me(Extension(user): Extension<AuthenticatedUser>) -> Json<MeResponse> {
    Json(MeResponse { user })
}

pub async fn require_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<impl IntoResponse, AppError> {
    let token = bearer_token(request.headers().get(AUTHORIZATION))?;
    let claims = decode_token(
        token,
        &state.config().auth.jwt_secret,
        &state.config().auth.jwt_issuer,
    )?;

    let Some(user) = fetch_user_by_id(state.pool(), claims.sub).await? else {
        return Err(AppError::Unauthorized(
            "token subject does not map to an active user".to_owned(),
        ));
    };

    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}

fn bearer_token(header_value: Option<&HeaderValue>) -> Result<&str, AppError> {
    let header_value = header_value.ok_or_else(|| {
        AppError::Unauthorized("missing Authorization: Bearer <token> header".to_owned())
    })?;
    let header_value = header_value.to_str().map_err(|_| {
        AppError::Unauthorized("Authorization header must be valid UTF-8".to_owned())
    })?;
    let mut parts = header_value.split_whitespace();
    let Some(scheme) = parts.next() else {
        return Err(AppError::Unauthorized(
            "Authorization header must use Bearer auth".to_owned(),
        ));
    };
    let Some(token) = parts.next() else {
        return Err(AppError::Unauthorized(
            "bearer token must not be empty".to_owned(),
        ));
    };

    if !scheme.eq_ignore_ascii_case("Bearer") {
        return Err(AppError::Unauthorized(
            "Authorization header must use Bearer auth".to_owned(),
        ));
    }

    if parts.next().is_some() {
        return Err(AppError::Unauthorized(
            "Authorization header must contain exactly one bearer token".to_owned(),
        ));
    }

    Ok(token)
}

fn decode_token(token: &str, secret: &str, issuer: &str) -> Result<Claims, AppError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_issuer(&[issuer]);

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|token_data| token_data.claims)
    .map_err(|_| AppError::Unauthorized("invalid or expired bearer token".to_owned()))
}

fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(password_hash)
        .map_err(|source| AppError::PasswordVerification(source.to_string()))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

async fn fetch_user_by_username(
    pool: &sqlx::SqlitePool,
    username: &str,
) -> Result<Option<StoredUser>, AppError> {
    sqlx::query(
        "SELECT id, username, display_name, email, password_hash, role FROM users WHERE username = ?",
    )
    .bind(username)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)
    .map(|row| row.map(stored_user_from_row))
}

async fn fetch_user_by_id(
    pool: &sqlx::SqlitePool,
    user_id: i64,
) -> Result<Option<AuthenticatedUser>, AppError> {
    sqlx::query("SELECT id, username, display_name, email, role FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
        .map(|row| row.map(authenticated_user_from_row))
}

fn stored_user_from_row(row: SqliteRow) -> StoredUser {
    StoredUser {
        id: row.get("id"),
        username: row.get("username"),
        display_name: row.get("display_name"),
        email: row.get("email"),
        password_hash: row.get("password_hash"),
        role: row.get("role"),
    }
}

fn authenticated_user_from_row(row: SqliteRow) -> AuthenticatedUser {
    AuthenticatedUser {
        id: row.get("id"),
        username: row.get("username"),
        display_name: row.get("display_name"),
        email: row.get("email"),
        role: row.get("role"),
    }
}

fn current_unix_timestamp() -> Result<u64, AppError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|source| AppError::Time(source.to_string()))
}

impl StoredUser {
    fn into_authenticated_user(self) -> AuthenticatedUser {
        AuthenticatedUser {
            id: self.id,
            username: self.username,
            display_name: self.display_name,
            email: self.email,
            role: self.role,
        }
    }
}

impl AuthenticatedUser {
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }
}
