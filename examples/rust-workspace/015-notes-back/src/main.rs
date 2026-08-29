mod api;
mod types;
mod db;

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

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = axum::Router::new()
        .route("/api/notes", axum::routing::get(api::list_notes))
        .route("/api/notes/:id", axum::routing::get(api::get_note))
        .route("/api/notes", axum::routing::post(api::create_note))
        .route("/api/notes/:id", axum::routing::put(api::update_note))
        .route("/api/notes/:id", axum::routing::delete(api::delete_note))
        .route("/api/notes/:id/pin", axum::routing::patch(api::toggle_pin))
        .route("/api/notes/:id/tags", axum::routing::put(api::update_tags))
        .route("/api/notes/search", axum::routing::get(api::search_notes))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
