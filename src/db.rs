use std::{fs, path::PathBuf, str::FromStr, time::Duration};

use sqlx::{
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    ConnectOptions, SqlitePool,
};

use crate::{
    config::{DatabaseConfig, DatabaseKind},
    error::AppError,
};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub async fn initialize(config: &DatabaseConfig) -> Result<SqlitePool, AppError> {
    match config.kind {
        DatabaseKind::Sqlite => initialize_sqlite(config).await,
    }
}

async fn initialize_sqlite(config: &DatabaseConfig) -> Result<SqlitePool, AppError> {
    ensure_sqlite_parent_directory(&config.url)?;

    let options = SqliteConnectOptions::from_str(&config.url)
        .map_err(|source| AppError::InvalidDatabaseUrl {
            value: config.url.clone(),
            reason: source.to_string(),
        })?
        .create_if_missing(true)
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(5))
        .disable_statement_logging();

    let pool = SqlitePoolOptions::new()
        .max_connections(config.max_connections)
        .connect_with(options)
        .await
        .map_err(AppError::Database)?;

    MIGRATOR.run(&pool).await.map_err(AppError::Migration)?;
    Ok(pool)
}

fn ensure_sqlite_parent_directory(url: &str) -> Result<(), AppError> {
    let Some(path) = sqlite_path_from_url(url) else {
        return Ok(());
    };

    let Some(parent) = path.parent() else {
        return Ok(());
    };

    if parent.as_os_str().is_empty() {
        return Ok(());
    }

    fs::create_dir_all(parent).map_err(|source| AppError::CreateDirectory {
        path: parent.to_path_buf(),
        source,
    })?;

    Ok(())
}

fn sqlite_path_from_url(url: &str) -> Option<PathBuf> {
    if url == "sqlite::memory:" {
        return None;
    }

    let path = url.strip_prefix("sqlite://")?;
    let path = path.split('?').next().unwrap_or(path);

    if path.is_empty() {
        return None;
    }

    Some(PathBuf::from(path))
}

#[cfg(test)]
mod tests {
    use sqlx::query_scalar;
    use tempfile::tempdir;

    use super::*;

    fn temp_database_config() -> (tempfile::TempDir, DatabaseConfig) {
        let temp_dir = tempdir().unwrap();
        let database_path = temp_dir.path().join("memos-rs.db");

        (
            temp_dir,
            DatabaseConfig {
                kind: DatabaseKind::Sqlite,
                url: format!("sqlite://{}", database_path.display()),
                max_connections: 5,
            },
        )
    }

    #[tokio::test]
    async fn initialize_runs_migrations() {
        let (_temp_dir, config) = temp_database_config();

        let pool = initialize(&config).await.unwrap();

        let tables: Vec<String> = query_scalar(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name IN ('users', 'memos') ORDER BY name",
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        assert_eq!(tables, vec!["memos".to_owned(), "users".to_owned()]);
    }
}
