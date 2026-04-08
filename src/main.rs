mod app;
mod auth;
mod bootstrap;
mod cli;
mod config;
mod db;
mod error;
mod frontend;
mod memo;
mod server;
mod state;

use std::sync::OnceLock;

use clap::Parser;
use cli::{Cli, Command};
use config::AppConfig;
use error::AppError;
use state::AppState;
use tracing_subscriber::EnvFilter;

static TRACING_INIT: OnceLock<Result<(), String>> = OnceLock::new();

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(error) = run(cli).await {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<(), AppError> {
    match cli.command {
        Command::Serve(args) => {
            let config = AppConfig::load(&args.config)?;
            init_tracing(&config.logging.level)?;
            let pool = db::initialize(&config.database).await?;

            tracing::info!(
                config_path = %args.config.display(),
                host = %config.server.host,
                port = config.server.port,
                log_level = %config.logging.level,
                database_kind = %config.database.kind,
                database_max_connections = config.database.max_connections,
                "starting memos-rs"
            );

            let state = AppState::new(config, pool);
            server::serve(state).await
        }
    }
}

fn init_tracing(level: &str) -> Result<(), AppError> {
    let result = TRACING_INIT.get_or_init(|| {
        let filter = EnvFilter::try_new(level).map_err(|error| error.to_string())?;
        let subscriber = tracing_subscriber::fmt()
            .with_env_filter(filter)
            .json()
            .with_current_span(true)
            .with_span_list(true)
            .finish();

        tracing::subscriber::set_global_default(subscriber).map_err(|error| error.to_string())
    });

    match result {
        Ok(()) => Ok(()),
        Err(message) => Err(AppError::TracingInitialization(message.clone())),
    }
}
