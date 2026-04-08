use std::{env, fs, path::Path};

use serde::Deserialize;

use crate::error::AppError;

const DEFAULT_CONFIG_PATH: &str = "config.toml";
const SYSTEM_CONFIG_PATH: &str = "/etc/memos-rs/config.toml";
const ENV_HOST: &str = "MEMOS_RS_HOST";
const ENV_PORT: &str = "MEMOS_RS_PORT";
const ENV_LOG_LEVEL: &str = "MEMOS_RS_LOG_LEVEL";
const ENV_DATABASE_KIND: &str = "MEMOS_RS_DATABASE_KIND";
const ENV_DATABASE_URL: &str = "MEMOS_RS_DATABASE_URL";
const ENV_DATABASE_MAX_CONNECTIONS: &str = "MEMOS_RS_DATABASE_MAX_CONNECTIONS";
const ENV_AUTH_JWT_SECRET: &str = "MEMOS_RS_AUTH_JWT_SECRET";
const ENV_AUTH_JWT_ISSUER: &str = "MEMOS_RS_AUTH_JWT_ISSUER";
const ENV_AUTH_TOKEN_TTL_SECONDS: &str = "MEMOS_RS_AUTH_TOKEN_TTL_SECONDS";
const ENV_FRONTEND_STATIC_DIR: &str = "MEMOS_RS_FRONTEND_STATIC_DIR";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub logging: LoggingConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub frontend: FrontendConfig,
}

impl AppConfig {
    pub fn load(path: &Path) -> Result<Self, AppError> {
        let allow_missing = path == Path::new(DEFAULT_CONFIG_PATH);
        let effective_path = if allow_missing && !path.exists() {
            let system_path = Path::new(SYSTEM_CONFIG_PATH);

            if system_path.exists() {
                system_path
            } else {
                path
            }
        } else {
            path
        };

        Self::load_inner(effective_path, allow_missing && effective_path == path)
    }

    fn load_inner(path: &Path, allow_missing: bool) -> Result<Self, AppError> {
        let mut config = Self::default();

        if path.exists() {
            let contents = fs::read_to_string(path).map_err(|source| AppError::ConfigRead {
                path: path.to_path_buf(),
                source,
            })?;
            let partial: PartialConfig =
                toml::from_str(&contents).map_err(|source| AppError::ConfigParse {
                    path: path.to_path_buf(),
                    source,
                })?;
            config.merge(partial)?;
        } else if !allow_missing {
            return Err(AppError::MissingConfig(path.to_path_buf()));
        }

        config.apply_env_overrides()?;
        config.validate()?;
        Ok(config)
    }

    fn merge(&mut self, partial: PartialConfig) -> Result<(), AppError> {
        if let Some(server) = partial.server {
            if let Some(host) = server.host {
                self.server.host = host;
            }

            if let Some(port) = server.port {
                self.server.port = port;
            }
        }

        if let Some(logging) = partial.logging {
            if let Some(level) = logging.level {
                self.logging.level = level;
            }
        }

        if let Some(database) = partial.database {
            if let Some(kind) = database.kind {
                self.database.kind =
                    kind.parse()
                        .map_err(|reason| AppError::InvalidConfigValue {
                            field: "database.kind",
                            value: kind,
                            reason,
                        })?;
            }

            if let Some(url) = database.url {
                self.database.url = url;
            }

            if let Some(max_connections) = database.max_connections {
                self.database.max_connections = max_connections;
            }
        }

        if let Some(auth) = partial.auth {
            if let Some(jwt_secret) = auth.jwt_secret {
                self.auth.jwt_secret = jwt_secret;
            }

            if let Some(jwt_issuer) = auth.jwt_issuer {
                self.auth.jwt_issuer = jwt_issuer;
            }

            if let Some(token_ttl_seconds) = auth.token_ttl_seconds {
                self.auth.token_ttl_seconds = token_ttl_seconds;
            }
        }

        if let Some(frontend) = partial.frontend {
            if let Some(static_dir) = frontend.static_dir {
                self.frontend.static_dir = normalize_optional_value(static_dir);
            }
        }

        Ok(())
    }

    fn apply_env_overrides(&mut self) -> Result<(), AppError> {
        if let Some(host) = read_env(ENV_HOST)? {
            self.server.host = host;
        }

        if let Some(port) = read_env(ENV_PORT)? {
            self.server.port = port;
        }

        if let Some(level) = read_env(ENV_LOG_LEVEL)? {
            self.logging.level = level;
        }

        if let Some(kind) = read_env::<String>(ENV_DATABASE_KIND)? {
            self.database.kind = kind
                .parse()
                .map_err(|reason| AppError::InvalidEnvironment {
                    key: ENV_DATABASE_KIND,
                    value: kind,
                    reason,
                })?;
        }

        if let Some(url) = read_env(ENV_DATABASE_URL)? {
            self.database.url = url;
        }

        if let Some(max_connections) = read_env(ENV_DATABASE_MAX_CONNECTIONS)? {
            self.database.max_connections = max_connections;
        }

        if let Some(jwt_secret) = read_env(ENV_AUTH_JWT_SECRET)? {
            self.auth.jwt_secret = jwt_secret;
        }

        if let Some(jwt_issuer) = read_env(ENV_AUTH_JWT_ISSUER)? {
            self.auth.jwt_issuer = jwt_issuer;
        }

        if let Some(token_ttl_seconds) = read_env(ENV_AUTH_TOKEN_TTL_SECONDS)? {
            self.auth.token_ttl_seconds = token_ttl_seconds;
        }

        if let Some(static_dir) = read_env::<String>(ENV_FRONTEND_STATIC_DIR)? {
            self.frontend.static_dir = normalize_optional_value(static_dir);
        }

        Ok(())
    }

    fn validate(&self) -> Result<(), AppError> {
        if self.database.max_connections == 0 {
            return Err(AppError::InvalidConfiguration(
                "database.max_connections must be at least 1".to_owned(),
            ));
        }

        if self.auth.jwt_secret.trim().is_empty() {
            return Err(AppError::InvalidConfiguration(
                "auth.jwt_secret must not be empty".to_owned(),
            ));
        }

        if self.auth.jwt_issuer.trim().is_empty() {
            return Err(AppError::InvalidConfiguration(
                "auth.jwt_issuer must not be empty".to_owned(),
            ));
        }

        if self.auth.token_ttl_seconds == 0 {
            return Err(AppError::InvalidConfiguration(
                "auth.token_ttl_seconds must be at least 1".to_owned(),
            ));
        }

        Ok(())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_owned(),
                port: 5230,
            },
            logging: LoggingConfig {
                level: "info".to_owned(),
            },
            database: DatabaseConfig {
                kind: DatabaseKind::Sqlite,
                url: "sqlite://./data/memos-rs.db".to_owned(),
                max_connections: 5,
            },
            auth: AuthConfig {
                jwt_secret: "change-me-in-production".to_owned(),
                jwt_issuer: "memos-rs".to_owned(),
                token_ttl_seconds: 86_400,
            },
            frontend: FrontendConfig { static_dir: None },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoggingConfig {
    pub level: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DatabaseConfig {
    pub kind: DatabaseKind,
    pub url: String,
    pub max_connections: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_issuer: String,
    pub token_ttl_seconds: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontendConfig {
    pub static_dir: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DatabaseKind {
    Sqlite,
}

impl std::fmt::Display for DatabaseKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sqlite => write!(f, "sqlite"),
        }
    }
}

impl std::str::FromStr for DatabaseKind {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "sqlite" => Ok(Self::Sqlite),
            _ => Err("supported values: sqlite".to_owned()),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct PartialConfig {
    server: Option<PartialServerConfig>,
    logging: Option<PartialLoggingConfig>,
    database: Option<PartialDatabaseConfig>,
    auth: Option<PartialAuthConfig>,
    frontend: Option<PartialFrontendConfig>,
}

#[derive(Debug, Default, Deserialize)]
struct PartialServerConfig {
    host: Option<String>,
    port: Option<u16>,
}

#[derive(Debug, Default, Deserialize)]
struct PartialLoggingConfig {
    level: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct PartialDatabaseConfig {
    kind: Option<String>,
    url: Option<String>,
    max_connections: Option<u32>,
}

#[derive(Debug, Default, Deserialize)]
struct PartialAuthConfig {
    jwt_secret: Option<String>,
    jwt_issuer: Option<String>,
    token_ttl_seconds: Option<u64>,
}

#[derive(Debug, Default, Deserialize)]
struct PartialFrontendConfig {
    static_dir: Option<String>,
}

fn read_env<T>(key: &'static str) -> Result<Option<T>, AppError>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    match env::var(key) {
        Ok(raw) => raw
            .parse::<T>()
            .map(Some)
            .map_err(|error| AppError::InvalidEnvironment {
                key,
                value: raw,
                reason: error.to_string(),
            }),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(raw)) => {
            Err(AppError::NonUnicodeEnvironment { key, value: raw })
        }
    }
}

fn normalize_optional_value(value: String) -> Option<String> {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        sync::{Mutex, OnceLock},
    };

    use tempfile::tempdir;

    use super::*;

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn clear_env() {
        env::remove_var(ENV_HOST);
        env::remove_var(ENV_PORT);
        env::remove_var(ENV_LOG_LEVEL);
        env::remove_var(ENV_DATABASE_KIND);
        env::remove_var(ENV_DATABASE_URL);
        env::remove_var(ENV_DATABASE_MAX_CONNECTIONS);
        env::remove_var(ENV_AUTH_JWT_SECRET);
        env::remove_var(ENV_AUTH_JWT_ISSUER);
        env::remove_var(ENV_AUTH_TOKEN_TTL_SECONDS);
        env::remove_var(ENV_FRONTEND_STATIC_DIR);
    }

    #[test]
    fn loads_defaults_when_default_config_is_missing() {
        let _guard = env_lock().lock().unwrap();
        clear_env();

        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join(DEFAULT_CONFIG_PATH);
        let config = AppConfig::load_inner(&config_path, true).unwrap();

        assert_eq!(config, AppConfig::default());
    }

    #[test]
    fn loads_file_and_environment_overrides() {
        let _guard = env_lock().lock().unwrap();
        clear_env();

        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("custom.toml");
        fs::write(
            &config_path,
            r#"
                [server]
                host = "0.0.0.0"
                port = 8080

                [logging]
                level = "debug"

                [database]
                kind = "sqlite"
                url = "sqlite://./custom/memos-rs.db"
                max_connections = 9

                [auth]
                jwt_secret = "config-secret"
                jwt_issuer = "config-issuer"
                token_ttl_seconds = 7200

                [frontend]
                static_dir = "./frontend/dist"
            "#,
        )
        .unwrap();

        env::set_var(ENV_PORT, "9999");
        env::set_var(ENV_LOG_LEVEL, "trace");
        env::set_var(ENV_DATABASE_URL, "sqlite://./override/memos-rs.db");
        env::set_var(ENV_DATABASE_MAX_CONNECTIONS, "11");
        env::set_var(ENV_AUTH_JWT_SECRET, "override-secret");
        env::set_var(ENV_AUTH_TOKEN_TTL_SECONDS, "3600");
        env::set_var(ENV_FRONTEND_STATIC_DIR, "./frontend/override-dist");

        let config = AppConfig::load(&config_path).unwrap();

        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 9999);
        assert_eq!(config.logging.level, "trace");
        assert_eq!(config.database.kind, DatabaseKind::Sqlite);
        assert_eq!(config.database.url, "sqlite://./override/memos-rs.db");
        assert_eq!(config.database.max_connections, 11);
        assert_eq!(config.auth.jwt_secret, "override-secret");
        assert_eq!(config.auth.jwt_issuer, "config-issuer");
        assert_eq!(config.auth.token_ttl_seconds, 3600);
        assert_eq!(
            config.frontend.static_dir,
            Some("./frontend/override-dist".to_owned())
        );

        clear_env();
    }

    #[test]
    fn missing_custom_config_returns_error() {
        let _guard = env_lock().lock().unwrap();
        clear_env();

        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("missing.toml");

        let error = AppConfig::load(&config_path).unwrap_err();

        assert!(matches!(error, AppError::MissingConfig(path) if path == config_path));
    }

    #[test]
    fn rejects_invalid_database_kind_from_config() {
        let _guard = env_lock().lock().unwrap();
        clear_env();

        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("custom.toml");
        fs::write(
            &config_path,
            r#"
                [database]
                kind = "postgres"
            "#,
        )
        .unwrap();

        let error = AppConfig::load(&config_path).unwrap_err();

        assert!(matches!(
            error,
            AppError::InvalidConfigValue {
                field: "database.kind",
                ..
            }
        ));
    }
}
