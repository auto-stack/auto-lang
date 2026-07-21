mod api;
mod types;

use api::Db;
use crate::types::*;
use std::sync::{Arc, Mutex};
use tower_http::cors::{CorsLayer, Any};

#[tokio::main]
async fn main() {
    println!("Server running on http://127.0.0.1:8080");
    println!("CORS enabled for all origins");

    // Initial data
    let data: Db = Arc::new(Mutex::new(vec![
        ToolCall {
            tool: "Sample".into(),
            args: Default::default(),
            result: "Sample".into()
        },
        ToolCall {
            tool: "Sample".into(),
            args: Default::default(),
            result: "Sample".into()
        },
        ToolCall {
            tool: "Sample".into(),
            args: Default::default(),
            result: "Sample".into()
        }
    ]));

    // Enable CORS for frontend development
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = axum::Router::new()
        .route("/api/run", axum::routing::post(api::run_agent))
        .route("/api/professions", axum::routing::get(api::list_professions))
        .route("/api/config", axum::routing::get(api::get_config))
        .route("/api/skills", axum::routing::get(api::list_skills))
        .route("/api/modes", axum::routing::get(api::list_modes))
        .route("/api/workflows", axum::routing::get(api::list_workflows))
        .route("/api/workflow/run", axum::routing::post(api::run_workflow))
        .route("/api/auth/login", axum::routing::post(api::login))
        .route("/api/auth/me", axum::routing::get(api::me))
        .route("/api/specs", axum::routing::get(api::get_specs))
        .route("/api/specs/item", axum::routing::post(api::upsert_spec))
        .route("/api/specs/transition", axum::routing::post(api::transition_spec))
        .with_state(data)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
