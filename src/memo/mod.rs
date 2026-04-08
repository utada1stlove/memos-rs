use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteRow, Row};

use crate::{auth::AuthenticatedUser, error::AppError, state::AppState};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoResponse {
    memo: Memo,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListMemosResponse {
    memos: Vec<Memo>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Memo {
    id: i64,
    creator_id: i64,
    content: String,
    visibility: String,
    pinned: bool,
    archived: bool,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMemoRequest {
    content: String,
    visibility: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMemoRequest {
    content: Option<String>,
    visibility: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListMemosQuery {
    creator_id: Option<i64>,
    order: Option<String>,
}

#[derive(Clone, Copy, Debug)]
enum MemoVisibility {
    Private,
    Public,
    Unlisted,
}

#[derive(Clone, Copy, Debug)]
enum MemoOrder {
    Asc,
    Desc,
}

pub async fn create_memo(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(payload): Json<CreateMemoRequest>,
) -> Result<impl IntoResponse, AppError> {
    let content = validate_content(&payload.content)?;
    let visibility = parse_visibility(payload.visibility.as_deref())?;

    let result =
        sqlx::query("INSERT INTO memos (creator_id, content, visibility) VALUES (?, ?, ?)")
            .bind(user.id)
            .bind(content)
            .bind(visibility.as_str())
            .execute(state.pool())
            .await
            .map_err(AppError::Database)?;

    let memo = fetch_memo_by_id(state.pool(), result.last_insert_rowid())
        .await?
        .ok_or_else(|| AppError::Internal("created memo could not be reloaded".to_owned()))?;

    Ok((StatusCode::CREATED, Json(MemoResponse { memo })))
}

pub async fn list_memos(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(query): Query<ListMemosQuery>,
) -> Result<Json<ListMemosResponse>, AppError> {
    let order = parse_order(query.order.as_deref())?;
    let creator_filter = effective_creator_filter(&user, query.creator_id)?;

    let sql = match creator_filter {
        Some(_) => format!(
            "SELECT id, creator_id, content, visibility, pinned, archived, created_at, updated_at FROM memos WHERE creator_id = ? ORDER BY created_at {}",
            order.as_sql()
        ),
        None => format!(
            "SELECT id, creator_id, content, visibility, pinned, archived, created_at, updated_at FROM memos ORDER BY created_at {}",
            order.as_sql()
        ),
    };

    let rows = match creator_filter {
        Some(creator_id) => sqlx::query(&sql)
            .bind(creator_id)
            .fetch_all(state.pool())
            .await
            .map_err(AppError::Database)?,
        None => sqlx::query(&sql)
            .fetch_all(state.pool())
            .await
            .map_err(AppError::Database)?,
    };

    let memos = rows.into_iter().map(memo_from_row).collect();

    Ok(Json(ListMemosResponse { memos }))
}

pub async fn get_memo(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(memo_id): Path<i64>,
) -> Result<Json<MemoResponse>, AppError> {
    let memo = fetch_memo_by_id(state.pool(), memo_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("memo {memo_id} was not found")))?;

    ensure_can_access_memo(&user, &memo)?;

    Ok(Json(MemoResponse { memo }))
}

pub async fn update_memo(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(memo_id): Path<i64>,
    Json(payload): Json<UpdateMemoRequest>,
) -> Result<Json<MemoResponse>, AppError> {
    let existing_memo = fetch_memo_by_id(state.pool(), memo_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("memo {memo_id} was not found")))?;

    ensure_can_access_memo(&user, &existing_memo)?;

    if payload.content.is_none() && payload.visibility.is_none() {
        return Err(AppError::Validation(
            "at least one of content or visibility must be provided".to_owned(),
        ));
    }

    let next_content = match payload.content.as_deref() {
        Some(content) => validate_content(content)?.to_owned(),
        None => existing_memo.content.clone(),
    };

    let next_visibility = match payload.visibility.as_deref() {
        Some(visibility) => parse_visibility(Some(visibility))?.as_str().to_owned(),
        None => existing_memo.visibility.clone(),
    };

    let updated_rows = sqlx::query(
        "UPDATE memos SET content = ?, visibility = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
    )
    .bind(next_content)
    .bind(next_visibility)
    .bind(memo_id)
    .execute(state.pool())
    .await
    .map_err(AppError::Database)?
    .rows_affected();

    if updated_rows == 0 {
        return Err(AppError::NotFound(format!("memo {memo_id} was not found")));
    }

    let memo = fetch_memo_by_id(state.pool(), memo_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("memo {memo_id} was not found")))?;

    Ok(Json(MemoResponse { memo }))
}

pub async fn delete_memo(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(memo_id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let memo = fetch_memo_by_id(state.pool(), memo_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("memo {memo_id} was not found")))?;

    ensure_can_access_memo(&user, &memo)?;

    sqlx::query("DELETE FROM memos WHERE id = ?")
        .bind(memo_id)
        .execute(state.pool())
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

async fn fetch_memo_by_id(pool: &sqlx::SqlitePool, memo_id: i64) -> Result<Option<Memo>, AppError> {
    sqlx::query(
        "SELECT id, creator_id, content, visibility, pinned, archived, created_at, updated_at FROM memos WHERE id = ?",
    )
    .bind(memo_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)
    .map(|row| row.map(memo_from_row))
}

fn memo_from_row(row: SqliteRow) -> Memo {
    Memo {
        id: row.get("id"),
        creator_id: row.get("creator_id"),
        content: row.get("content"),
        visibility: row.get("visibility"),
        pinned: row.get::<i64, _>("pinned") != 0,
        archived: row.get::<i64, _>("archived") != 0,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn validate_content(content: &str) -> Result<&str, AppError> {
    if content.trim().is_empty() {
        return Err(AppError::Validation(
            "memo content must not be empty".to_owned(),
        ));
    }

    Ok(content)
}

fn parse_visibility(value: Option<&str>) -> Result<MemoVisibility, AppError> {
    match value
        .unwrap_or("private")
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "private" => Ok(MemoVisibility::Private),
        "public" => Ok(MemoVisibility::Public),
        "unlisted" => Ok(MemoVisibility::Unlisted),
        _ => Err(AppError::Validation(
            "visibility must be one of: private, public, unlisted".to_owned(),
        )),
    }
}

fn parse_order(value: Option<&str>) -> Result<MemoOrder, AppError> {
    match value.unwrap_or("desc").trim().to_ascii_lowercase().as_str() {
        "asc" => Ok(MemoOrder::Asc),
        "desc" => Ok(MemoOrder::Desc),
        _ => Err(AppError::Validation(
            "order must be one of: asc, desc".to_owned(),
        )),
    }
}

fn effective_creator_filter(
    user: &AuthenticatedUser,
    requested_creator_id: Option<i64>,
) -> Result<Option<i64>, AppError> {
    match requested_creator_id {
        Some(creator_id) if user.is_admin() || creator_id == user.id => Ok(Some(creator_id)),
        Some(_) => Err(AppError::Forbidden(
            "you can only filter by your own creatorId unless you are an admin".to_owned(),
        )),
        None if user.is_admin() => Ok(None),
        None => Ok(Some(user.id)),
    }
}

fn ensure_can_access_memo(user: &AuthenticatedUser, memo: &Memo) -> Result<(), AppError> {
    if user.is_admin() || memo.creator_id == user.id {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "you do not have access to this memo".to_owned(),
        ))
    }
}

impl MemoVisibility {
    fn as_str(self) -> &'static str {
        match self {
            Self::Private => "private",
            Self::Public => "public",
            Self::Unlisted => "unlisted",
        }
    }
}

impl MemoOrder {
    fn as_sql(self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}
