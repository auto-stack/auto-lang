mod api;
mod types;

use api::Db;
use crate::types::*;
use std::sync::{Arc, Mutex};
use tower_http::cors::{CorsLayer, Any};

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("AUTO_HTTP_PORT")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(8080);
    let addr = format!("127.0.0.1:{}", port);
    println!("Server running on http://{}", addr);
    println!("CORS enabled for all origins");

    // Initial data
    let data: Db = Arc::new(Mutex::new(vec![
        Status {
            ok: false,
            message: "Sample".into()
        },
        Status {
            ok: false,
            message: "Sample".into()
        },
        Status {
            ok: false,
            message: "Sample".into()
        }
    ]));

    // Enable CORS for frontend development
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = axum::Router::new()
        .route("/api/status", axum::routing::get(api::status))
        .with_state(data)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
