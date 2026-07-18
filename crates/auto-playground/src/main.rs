mod agent_debug;
mod code_runner;
mod debugger;
mod error;
mod notebook;
mod project;
mod routes;

mod vm_runner;

use axum::extract::ws::WebSocketUpgrade;
use axum::extract::FromRef;
use axum::http::Method;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use axum::Router;
use std::path::PathBuf;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
struct AppState {
    agent_sessions: routes::agent_debug::AgentDebugSessions,
    notebook_state: notebook::NotebookState,
}

impl FromRef<AppState> for routes::agent_debug::AgentDebugSessions {
    fn from_ref(state: &AppState) -> Self {
        state.agent_sessions.clone()
    }
}

impl FromRef<AppState> for notebook::NotebookActor {
    fn from_ref(state: &AppState) -> Self {
        state.notebook_state.clone()
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("auto_playground=debug,tower_http=debug")
        .init();

    let frontend_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("frontend");
    let dist_dir = frontend_dir.join("dist");

    // AutoLab UI static files
    let lab_dist_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("packages")
        .join("auto-lab-ui")
        .join("dist");
    let lab_dist_dir = lab_dist_dir.canonicalize().unwrap_or(lab_dist_dir);

    // Spawn frontend dev server if no production build
    let mut frontend_child: Option<tokio::process::Child> = None;
    if !dist_dir.exists() {
        frontend_child = spawn_frontend_dev(&frontend_dir);
    }

    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(tower_http::cors::Any);

    let app_state = AppState {
        agent_sessions: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        notebook_state: notebook::NotebookActor::new(),
    };

    let api_routes = Router::new()
        .route("/api/run", post(routes::run::run_handler))
        .route("/api/run_abt", post(routes::run_abt::run_abt_handler))
        .route("/api/run_code", post(routes::run_code::run_code_handler))
        .route("/api/trans", post(routes::trans::trans_handler))
        .route("/api/examples", get(routes::examples::examples_handler))
        .route("/api/notebook/session", post(routes::notebook::create_session_handler))
        .route("/api/notebook/{sid}/execute", post(routes::notebook::execute_handler))
        .route("/api/notebook/{sid}/status", get(routes::notebook::status_handler))
        .route("/api/notebook/{sid}/variables", get(routes::notebook::variables_handler))
        .route("/api/notebook/{sid}/transpile", post(routes::notebook::transpile_handler))
        .route("/api/notebook/{sid}", delete(routes::notebook::delete_session_handler))
        .route("/api/debug/ws", get(debug_ws_handler))
        .route("/api/agent-debug/start", post(routes::agent_debug::start_handler))
        .route("/api/agent-debug/sessions", get(routes::agent_debug::sessions_handler))
        .route(
            "/api/agent-debug/{id}/breakpoints",
            post(routes::agent_debug::breakpoints_handler),
        )
        .route(
            "/api/agent-debug/{id}/command",
            post(routes::agent_debug::command_handler),
        )
        .route("/api/agent-debug/{id}/state", get(routes::agent_debug::state_handler))
        .route("/api/agent-debug/{id}", delete(routes::agent_debug::delete_handler))
        .with_state(app_state);

    let mut app = api_routes;
    if lab_dist_dir.exists() {
        app = app.nest_service("/lab", tower_http::services::ServeDir::new(&lab_dist_dir));
        tracing::info!("AutoLab UI served at /lab ({})", lab_dist_dir.display());
    }
    if dist_dir.exists() {
        app = app.fallback_service(tower_http::services::ServeDir::new(&dist_dir));
    }

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
