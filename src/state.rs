use std::sync::Arc;

use sqlx::SqlitePool;

use crate::config::AppConfig;

#[derive(Clone, Debug)]
pub struct AppState {
    config: Arc<AppConfig>,
    pool: SqlitePool,
}

impl AppState {
    pub fn new(config: AppConfig, pool: SqlitePool) -> Self {
        Self {
            config: Arc::new(config),
            pool,
        }
    }

    pub fn config(&self) -> &AppConfig {
        self.config.as_ref()
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
