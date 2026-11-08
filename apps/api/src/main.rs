use std::net::SocketAddr;

use lend_rescue_api::config::AppConfig;
use lend_rescue_api::observability::init_observability;
use lend_rescue_api::router::create_router;
use lend_rescue_api::state::AppState;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::from_env().unwrap_or_else(|e| {
        eprintln!(
            "Failed to load configuration: {}. Falling back to default.",
            e
        );
        AppConfig::for_test()
    });

    init_observability(&config.log_level);

    info!(
        "Starting LendRescue API service on {}:{}",
        config.bind_host, config.bind_port
    );

    let state = AppState::create(config.clone()).await?;
    let router = create_router(state);

    let addr = SocketAddr::new(config.bind_host, config.bind_port);
    let listener = tokio::net::TcpListener::bind(addr).await?;

    info!("LendRescue API listening on http://{}", addr);
    info!(
        "OpenAPI documentation available at http://{}/swagger-ui",
        addr
    );

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("LendRescue API shutdown completed");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
