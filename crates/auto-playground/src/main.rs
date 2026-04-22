mod error;
mod routes;
mod vm_runner;

use axum::http::{HeaderValue, Method};
use axum::routing::{get, post};
use axum::Router;
use std::path::PathBuf;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("auto_playground=debug,tower_http=debug")
        .init();

    let cors = CorsLayer::new()
        .allow_origin([
            HeaderValue::from_static("http://localhost:5173"),
            HeaderValue::from_static("http://localhost:3000"),
        ])
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(tower_http::cors::Any);

    let api_routes = Router::new()
        .route("/api/run", post(routes::run::run_handler))
        .route("/api/trans", post(routes::trans::trans_handler))
        .route("/api/examples", get(routes::examples::examples_handler));

    // Serve frontend static files in production
    let frontend_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("frontend/dist");
    let app = if frontend_dir.exists() {
        api_routes.fallback_service(tower_http::services::ServeDir::new(&frontend_dir))
    } else {
        api_routes
    };

    let app = app.layer(cors);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3030));
    tracing::info!("Auto Playground server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
