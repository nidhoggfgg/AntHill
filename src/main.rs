#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod api;
mod config;
mod error;
mod executor;
mod models;
mod paths;
mod repository;
mod services;
#[cfg(target_os = "windows")]
mod windows_tray;

use crate::config::Config;
use crate::repository::{ExecutionRepository, PluginRepository, establish_connection};
use crate::services::{ExecutionService, PluginService, UpdateService};
use api::create_router;
use std::future::Future;
use std::net::SocketAddr;
use std::path::PathBuf;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn prepend_bin_to_path() -> anyhow::Result<()> {
    let bin_dir = crate::paths::install_root()?.join("bin");
    let mut paths: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).collect())
        .unwrap_or_default();

    if !paths.iter().any(|path| path == &bin_dir) {
        paths.insert(0, bin_dir);
        let new_path = std::env::join_paths(paths)?;
        // SAFETY: We only mutate PATH at startup before spawning child processes.
        unsafe {
            std::env::set_var("PATH", new_path);
        }
    }

    Ok(())
}

async fn run_server<F>(shutdown: F) -> anyhow::Result<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "anthill=debug,tower_http=debug,axum=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    prepend_bin_to_path()?;

    if let Err(err) = UpdateService::apply_pending_update() {
        tracing::error!("Failed to apply pending update: {}", err);
    }

    // Load configuration
    let config = Config::from_env()?;
    tracing::info!("Starting anthill with config: {:?}", config);

    if let Some(path) = config.database_url.strip_prefix("sqlite:") {
        let path = std::path::Path::new(path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // Establish database connection
    let db_pool = establish_connection(&config.database_url).await?;
    tracing::info!("Database connected: {}", config.database_url);

    // Initialize repositories
    let plugin_repo = PluginRepository::new(db_pool.clone());
    let execution_repo = ExecutionRepository::new(db_pool);

    // Initialize services
    let plugin_service = PluginService::new(plugin_repo.clone(), config.uv_path.clone());
    let execution_service = ExecutionService::new(execution_repo, plugin_repo);

    // Create router
    let app = create_router(plugin_service, execution_service);
    let app = app.layer(TraceLayer::new_for_http());

    // Start server
    let addr = format!("{}:{}", config.host, config.port);
    let addr = addr.parse::<SocketAddr>()?;
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await?;

    Ok(())
}

#[cfg(not(target_os = "windows"))]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run_server(std::future::pending::<()>()).await
}

#[cfg(target_os = "windows")]
fn main() -> anyhow::Result<()> {
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    let server_handle = runtime.spawn(run_server(async move {
        let _ = shutdown_rx.await;
    }));

    let _tray_thread = std::thread::spawn(move || {
        if let Err(err) = windows_tray::run_tray_loop(shutdown_tx) {
            eprintln!("tray loop failed: {err}");
        }
    });

    match runtime.block_on(async { server_handle.await }) {
        Ok(Ok(())) => Ok(()),
        Ok(Err(err)) => Err(err),
        Err(err) => Err(anyhow::anyhow!(err)),
    }
}
