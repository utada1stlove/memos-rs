use std::path::PathBuf;

use axum::routing::{get_service, MethodRouter};
use tower_http::services::{ServeDir, ServeFile};

pub fn static_assets_service(static_dir: &str) -> MethodRouter {
    let static_dir = PathBuf::from(static_dir);
    let index_path = static_dir.join("index.html");

    get_service(ServeDir::new(static_dir).fallback(ServeFile::new(index_path)))
}
