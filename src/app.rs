use axum::{
    http::StatusCode,
    middleware,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use tower_http::trace::TraceLayer;

use crate::{auth, bootstrap, frontend, memo, state::AppState};

pub fn build_router(state: AppState) -> Router {
    let auth_state = state.clone();
    let memo_state = state.clone();
    let static_dir = state.config().frontend.static_dir.clone();
    let protected_auth_routes =
        Router::new()
            .route("/auth/me", get(auth::me))
            .route_layer(middleware::from_fn_with_state(
                auth_state,
                auth::require_auth,
            ));
    let protected_memo_routes = Router::new()
        .route("/memos", get(memo::list_memos).post(memo::create_memo))
        .route(
            "/memos/{id}",
            get(memo::get_memo)
                .patch(memo::update_memo)
                .delete(memo::delete_memo),
        )
        .route_layer(middleware::from_fn_with_state(
            memo_state,
            auth::require_auth,
        ));
    let api_routes = Router::new().nest(
        "/v1",
        Router::new()
            .route("/bootstrap", post(bootstrap::bootstrap_admin))
            .route("/auth/login", post(auth::login))
            .merge(protected_auth_routes)
            .merge(protected_memo_routes),
    );
    let api_routes = api_routes.fallback(api_not_found);

    let app = Router::new()
        .route("/healthz", get(healthz))
        .nest("/api", api_routes);

    let app = match static_dir.as_deref() {
        Some(static_dir) => {
            app.merge(Router::new().fallback_service(frontend::static_assets_service(static_dir)))
        }
        None => app,
    };

    app.layer(TraceLayer::new_for_http()).with_state(state)
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

async fn healthz() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn api_not_found() -> StatusCode {
    StatusCode::NOT_FOUND
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
    };
    use serde_json::{json, Value};
    use sqlx::{query, query_as};
    use tempfile::tempdir;
    use tower::util::ServiceExt;

    use super::*;
    use crate::{config::AppConfig, db, state::AppState};

    #[tokio::test]
    async fn healthz_returns_ok_json() {
        let (_temp_dir, state) = setup_state().await;
        let app = build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let headers = response.headers().clone();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();

        assert_eq!(headers["content-type"], "application/json");
        assert_eq!(body.as_ref(), br#"{"status":"ok"}"#);
    }

    #[tokio::test]
    async fn bootstrap_creates_first_admin_user() {
        let (_temp_dir, state) = setup_state().await;
        let pool = state.pool().clone();
        let app = build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/bootstrap")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "username": "admin",
                            "displayName": "Admin User",
                            "email": "admin@example.com",
                            "password": "supersecret"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["user"]["username"], "admin");
        assert_eq!(payload["user"]["role"], "admin");

        let (role, password_hash): (String, String) =
            query_as("SELECT role, password_hash FROM users WHERE username = ?")
                .bind("admin")
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(role, "admin");
        assert_ne!(password_hash, "supersecret");
    }

    #[tokio::test]
    async fn bootstrap_conflicts_after_first_user_exists() {
        let (_temp_dir, state) = setup_state().await;
        let app = build_router(state);

        let first_response = app
            .clone()
            .oneshot(bootstrap_request("admin-one", "Admin One"))
            .await
            .unwrap();
        assert_eq!(first_response.status(), StatusCode::CREATED);

        let second_response = app
            .oneshot(bootstrap_request("admin-two", "Admin Two"))
            .await
            .unwrap();
        assert_eq!(second_response.status(), StatusCode::CONFLICT);

        let body = to_bytes(second_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert!(payload["error"]
            .as_str()
            .unwrap()
            .contains("bootstrap is already complete"));
    }

    #[tokio::test]
    async fn login_returns_token_for_valid_credentials() {
        let (_temp_dir, state) = setup_state().await;
        let app = build_router(state);

        let bootstrap_response = app
            .clone()
            .oneshot(bootstrap_request("admin", "Admin User"))
            .await
            .unwrap();
        assert_eq!(bootstrap_response.status(), StatusCode::CREATED);

        let login_response = app
            .clone()
            .oneshot(login_request("admin", "supersecret"))
            .await
            .unwrap();
        assert_eq!(login_response.status(), StatusCode::OK);

        let body = to_bytes(login_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["tokenType"], "Bearer");
        assert_eq!(payload["user"]["username"], "admin");

        let me_response = app
            .oneshot(me_request(payload["token"].as_str().unwrap()))
            .await
            .unwrap();
        assert_eq!(me_response.status(), StatusCode::OK);

        let body = to_bytes(me_response.into_body(), usize::MAX).await.unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["user"]["username"], "admin");
        assert_eq!(payload["user"]["role"], "admin");
    }

    #[tokio::test]
    async fn login_rejects_invalid_credentials() {
        let (_temp_dir, state) = setup_state().await;
        let app = build_router(state);

        let bootstrap_response = app
            .clone()
            .oneshot(bootstrap_request("admin", "Admin User"))
            .await
            .unwrap();
        assert_eq!(bootstrap_response.status(), StatusCode::CREATED);

        let login_response = app
            .oneshot(login_request("admin", "wrong-password"))
            .await
            .unwrap();
        assert_eq!(login_response.status(), StatusCode::UNAUTHORIZED);

        let body = to_bytes(login_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["error"], "invalid username or password");
    }

    #[tokio::test]
    async fn protected_route_rejects_missing_bearer_token() {
        let (_temp_dir, state) = setup_state().await;
        let app = build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/auth/me")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert!(payload["error"]
            .as_str()
            .unwrap()
            .contains("Authorization: Bearer"));
    }

    #[tokio::test]
    async fn protected_route_accepts_case_insensitive_bearer_scheme() {
        let (_temp_dir, state) = setup_state().await;
        let app = build_router(state);
        let token = bootstrap_and_login(app.clone()).await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/auth/me")
                    .header("authorization", format!("bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn memo_crud_flow_works_for_authenticated_user() {
        let (_temp_dir, state) = setup_state().await;
        let app = build_router(state);
        let token = bootstrap_and_login(app.clone()).await;

        let create_response = app
            .clone()
            .oneshot(create_memo_request(&token, "first memo", Some("private")))
            .await
            .unwrap();
        assert_eq!(create_response.status(), StatusCode::CREATED);

        let body = to_bytes(create_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        let memo_id = payload["memo"]["id"].as_i64().unwrap();
        assert_eq!(payload["memo"]["content"], "first memo");
        assert_eq!(payload["memo"]["visibility"], "private");

        let list_response = app
            .clone()
            .oneshot(list_memos_request(&token, None, None))
            .await
            .unwrap();
        assert_eq!(list_response.status(), StatusCode::OK);

        let body = to_bytes(list_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["memos"].as_array().unwrap().len(), 1);

        let get_response = app
            .clone()
            .oneshot(get_memo_request(&token, memo_id))
            .await
            .unwrap();
        assert_eq!(get_response.status(), StatusCode::OK);

        let update_response = app
            .clone()
            .oneshot(update_memo_request(
                &token,
                memo_id,
                Some("updated memo"),
                Some("public"),
            ))
            .await
            .unwrap();
        assert_eq!(update_response.status(), StatusCode::OK);

        let body = to_bytes(update_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["memo"]["content"], "updated memo");
        assert_eq!(payload["memo"]["visibility"], "public");

        let delete_response = app
            .clone()
            .oneshot(delete_memo_request(&token, memo_id))
            .await
            .unwrap();
        assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

        let missing_response = app
            .oneshot(get_memo_request(&token, memo_id))
            .await
            .unwrap();
        assert_eq!(missing_response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn memo_list_supports_creator_filter_and_order() {
        let (_temp_dir, state) = setup_state().await;
        let pool = state.pool().clone();
        let app = build_router(state);
        let token = bootstrap_and_login(app.clone()).await;

        let second_user_id = query(
            "INSERT INTO users (username, display_name, email, password_hash, role) VALUES (?, ?, ?, ?, 'user')",
        )
        .bind("writer")
        .bind("Writer User")
        .bind(Option::<&str>::None)
        .bind("placeholder-hash")
        .execute(&pool)
        .await
        .unwrap()
        .last_insert_rowid();

        insert_memo_row(&pool, 1, "older admin memo", "2026-01-01 00:00:00").await;
        insert_memo_row(
            &pool,
            second_user_id,
            "other user memo",
            "2026-01-02 00:00:00",
        )
        .await;
        insert_memo_row(&pool, 1, "newer admin memo", "2026-01-03 00:00:00").await;

        let response = app
            .oneshot(list_memos_request(&token, Some(1), Some("asc")))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        let memos = payload["memos"].as_array().unwrap();
        assert_eq!(memos.len(), 2);
        assert_eq!(memos[0]["content"], "older admin memo");
        assert_eq!(memos[1]["content"], "newer admin memo");
    }

    #[tokio::test]
    async fn serves_static_frontend_without_breaking_api_routes() {
        let frontend_dir = tempdir().unwrap();
        write_static_file(frontend_dir.path(), "index.html", "<html>frontend</html>");
        write_static_file(frontend_dir.path(), "app.js", "console.log('frontend');");

        let (_db_temp_dir, state) = setup_state_with_static_dir(frontend_dir.path()).await;
        let app = build_router(state);

        let root_response = app
            .clone()
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(root_response.status(), StatusCode::OK);
        let body = to_bytes(root_response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body.as_ref(), b"<html>frontend</html>");

        let asset_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/app.js")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(asset_response.status(), StatusCode::OK);
        let body = to_bytes(asset_response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body.as_ref(), b"console.log('frontend');");

        let spa_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/notes/123")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(spa_response.status(), StatusCode::OK);
        let body = to_bytes(spa_response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body.as_ref(), b"<html>frontend</html>");

        let api_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/memos")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(api_response.status(), StatusCode::UNAUTHORIZED);

        let missing_api_response = app
            .oneshot(
                Request::builder()
                    .uri("/api/not-a-route")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(missing_api_response.status(), StatusCode::NOT_FOUND);
    }

    async fn setup_state() -> (tempfile::TempDir, AppState) {
        let temp_dir = tempdir().unwrap();
        let database_path = temp_dir.path().join("memos-rs.db");

        let mut config = AppConfig::default();
        config.database.url = format!("sqlite://{}", database_path.display());

        let pool = db::initialize(&config.database).await.unwrap();

        (temp_dir, AppState::new(config, pool))
    }

    async fn setup_state_with_static_dir(static_dir: &Path) -> (tempfile::TempDir, AppState) {
        let temp_dir = tempdir().unwrap();
        let database_path = temp_dir.path().join("memos-rs.db");

        let mut config = AppConfig::default();
        config.database.url = format!("sqlite://{}", database_path.display());
        config.frontend.static_dir = Some(static_dir.display().to_string());

        let pool = db::initialize(&config.database).await.unwrap();

        (temp_dir, AppState::new(config, pool))
    }

    async fn bootstrap_and_login(app: Router) -> String {
        let bootstrap_response = app
            .clone()
            .oneshot(bootstrap_request("admin", "Admin User"))
            .await
            .unwrap();
        assert_eq!(bootstrap_response.status(), StatusCode::CREATED);

        let login_response = app
            .oneshot(login_request("admin", "supersecret"))
            .await
            .unwrap();
        assert_eq!(login_response.status(), StatusCode::OK);

        let body = to_bytes(login_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();

        payload["token"].as_str().unwrap().to_owned()
    }

    async fn insert_memo_row(
        pool: &sqlx::SqlitePool,
        creator_id: i64,
        content: &str,
        timestamp: &str,
    ) {
        query(
            "INSERT INTO memos (creator_id, content, visibility, pinned, archived, created_at, updated_at) VALUES (?, ?, 'private', 0, 0, ?, ?)",
        )
        .bind(creator_id)
        .bind(content)
        .bind(timestamp)
        .bind(timestamp)
        .execute(pool)
        .await
        .unwrap();
    }

    fn write_static_file(static_dir: &Path, relative_path: &str, contents: &str) {
        let path = static_dir.join(relative_path);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        fs::write(path, contents).unwrap();
    }

    fn bootstrap_request(username: &str, display_name: &str) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri("/api/v1/bootstrap")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "username": username,
                    "displayName": display_name,
                    "password": "supersecret"
                })
                .to_string(),
            ))
            .unwrap()
    }

    fn login_request(username: &str, password: &str) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri("/api/v1/auth/login")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "username": username,
                    "password": password
                })
                .to_string(),
            ))
            .unwrap()
    }

    fn me_request(token: &str) -> Request<Body> {
        Request::builder()
            .uri("/api/v1/auth/me")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap()
    }

    fn create_memo_request(token: &str, content: &str, visibility: Option<&str>) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri("/api/v1/memos")
            .header("authorization", format!("Bearer {token}"))
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "content": content,
                    "visibility": visibility
                })
                .to_string(),
            ))
            .unwrap()
    }

    fn list_memos_request(
        token: &str,
        creator_id: Option<i64>,
        order: Option<&str>,
    ) -> Request<Body> {
        let mut uri = String::from("/api/v1/memos");
        let mut params = Vec::new();

        if let Some(creator_id) = creator_id {
            params.push(format!("creatorId={creator_id}"));
        }

        if let Some(order) = order {
            params.push(format!("order={order}"));
        }

        if !params.is_empty() {
            uri.push('?');
            uri.push_str(&params.join("&"));
        }

        Request::builder()
            .uri(uri)
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap()
    }

    fn get_memo_request(token: &str, memo_id: i64) -> Request<Body> {
        Request::builder()
            .uri(format!("/api/v1/memos/{memo_id}"))
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap()
    }

    fn update_memo_request(
        token: &str,
        memo_id: i64,
        content: Option<&str>,
        visibility: Option<&str>,
    ) -> Request<Body> {
        Request::builder()
            .method("PATCH")
            .uri(format!("/api/v1/memos/{memo_id}"))
            .header("authorization", format!("Bearer {token}"))
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "content": content,
                    "visibility": visibility
                })
                .to_string(),
            ))
            .unwrap()
    }

    fn delete_memo_request(token: &str, memo_id: i64) -> Request<Body> {
        Request::builder()
            .method("DELETE")
            .uri(format!("/api/v1/memos/{memo_id}"))
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap()
    }
}
