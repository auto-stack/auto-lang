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
        Note {
            id: 0,
            title: "Welcome".into(),
            body: "This is your notes app. Click on any note to view it.".into(),
            time: "Just now".into(),
            pinned: true,
            tags: vec!["intro".into()],
            folder: "".into()
        },
        Note {
            id: 1,
            title: "Quick Ideas".into(),
            body: "Build a markdown editor with live preview.".into(),
            time: "10 min ago".into(),
            pinned: false,
            tags: vec!["ideas".into()],
            folder: "".into()
        },
        Note {
            id: 2,
            title: "Shopping List".into(),
            body: "Milk, Eggs, Bread, Cheese".into(),
            time: "2 hours ago".into(),
            pinned: false,
            tags: vec!["home".into()],
            folder: "personal".into()
        },
        Note {
            id: 3,
            title: "Recipe: Pasta".into(),
            body: "200g flour, 2 eggs, pinch of salt.".into(),
            time: "Last week".into(),
            pinned: false,
            tags: vec!["home".into()],
            folder: "personal".into()
        },
        Note {
            id: 4,
            title: "Meeting Notes".into(),
            body: "Q3 roadmap discussion.".into(),
            time: "Yesterday".into(),
            pinned: false,
            tags: vec!["work".into()],
            folder: "work".into()
        },
        Note {
            id: 5,
            title: "Sprint Planning".into(),
            body: "Sprint 12 planning notes.".into(),
            time: "3 days ago".into(),
            pinned: false,
            tags: vec!["work".into()],
            folder: "work".into()
        }
    ]));

    // Enable CORS for frontend development
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
        .with_state(data)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
