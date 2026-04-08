use std::future::pending;

use tokio::net::TcpListener;
use tracing::{info, warn};

use crate::{app, error::AppError, state::AppState};

pub async fn serve(state: AppState) -> Result<(), AppError> {
    let listener = TcpListener::bind((
        state.config().server.host.as_str(),
        state.config().server.port,
    ))
    .await
    .map_err(|source| AppError::ServerIo {
        action: "bind TCP listener",
        source,
    })?;

    let local_addr = listener.local_addr().map_err(|source| AppError::ServerIo {
        action: "read bound address",
        source,
    })?;

    info!(address = %local_addr, "HTTP server listening");

    axum::serve(listener, app::build_router(state))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|source| AppError::ServerIo {
            action: "run HTTP server",
            source,
        })?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            warn!(error = %error, "failed to listen for CTRL+C");
            pending::<()>().await;
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(error) => {
                warn!(error = %error, "failed to listen for SIGTERM");
                pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = pending::<()>();

    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }

    info!("shutdown signal received");
}
