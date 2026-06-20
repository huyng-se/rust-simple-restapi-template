use rust_simple_restapi_templ::core::{
    app::{build_router, build_state},
    config::load_config,
    error::AppResult,
    telemetry,
};
use std::net::SocketAddr;
use tokio::signal::{
    self,
    unix::{self, SignalKind},
};

#[tokio::main]
async fn main() -> AppResult<()> {
    let config = load_config()?;

    telemetry::init_telemetry(&config)?;

    let state = build_state(&config).await?;
    let app = build_router(state, &config);

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    tracing::info!(%addr, "starting http server");

    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    tracing::info!("Waiting for shutdown signal (Ctrl+C or SIGTERM)");
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
        "SIGINT"
    };

    #[cfg(unix)]
    let terminate = async {
        unix::signal(SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
        "SIGTERM"
    };

    let signal = tokio::select! {
        sig = ctrl_c => sig,
        sig = terminate => sig,
    };

    tracing::info!(signal, "Received termination signal, shutting down...");
}
