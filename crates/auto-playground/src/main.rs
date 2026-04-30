mod code_runner;
mod debugger;
mod error;
mod routes;
mod vm_runner;

use axum::extract::ws::WebSocketUpgrade;
use axum::http::Method;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;
use std::path::PathBuf;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("auto_playground=debug,tower_http=debug")
        .init();

    let frontend_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("frontend");
    let dist_dir = frontend_dir.join("dist");

    // Spawn frontend dev server if no production build
    let mut frontend_child: Option<tokio::process::Child> = None;
    if !dist_dir.exists() {
        frontend_child = spawn_frontend_dev(&frontend_dir);
    }

    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(tower_http::cors::Any);

    let api_routes = Router::new()
        .route("/api/run", post(routes::run::run_handler))
        .route("/api/run_abt", post(routes::run_abt::run_abt_handler))
        .route("/api/run_code", post(routes::run_code::run_code_handler))
        .route("/api/trans", post(routes::trans::trans_handler))
        .route("/api/examples", get(routes::examples::examples_handler))
        .route("/api/debug/ws", get(debug_ws_handler));

    let app = if dist_dir.exists() {
        api_routes.fallback_service(tower_http::services::ServeDir::new(&dist_dir))
    } else {
        api_routes
    };

    let app = app.layer(cors);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3030));
    tracing::info!("Auto Playground server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    drop(frontend_child);
}

async fn debug_ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(debugger::session::run_debug_session)
}

fn spawn_frontend_dev(frontend_dir: &std::path::Path) -> Option<tokio::process::Child> {
    let cmd = which_frontend_cmd();

    let child = match tokio::process::Command::new(&cmd)
        .args(["run", "dev"])
        .current_dir(frontend_dir)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(
                "Failed to start frontend dev server ({}): {}. Start it manually with `cd frontend && {} run dev`",
                cmd, e, cmd
            );
            return None;
        }
    };

    tracing::info!(
        "Frontend dev server started (PID: {:?}) — {}",
        child.id(),
        cmd
    );
    Some(child)
}

fn which_frontend_cmd() -> &'static str {
    // Prefer bun (faster), fall back to npm
    if std::process::Command::new("bun")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
    {
        "bun"
    } else {
        "npm"
    }
}
