mod api;
mod types;
mod events;

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
        AuthUser {
            username: "Sample".into(),
            role: "Sample".into()
        },
        AuthUser {
            username: "Sample".into(),
            role: "Sample".into()
        },
        AuthUser {
            username: "Sample".into(),
            role: "Sample".into()
        }
    ]));

    // Enable CORS for frontend development
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = axum::Router::new()
        .route("/api/chats/session/{id}/stream", axum::routing::get(api::chat_stream))
        .route("/api/auth/login", axum::routing::post(api::auth_login))
        .route("/api/auth/register", axum::routing::post(api::auth_register))
        .route("/api/auth/me", axum::routing::get(api::auth_me))
        .route("/api/auth/logout", axum::routing::post(api::auth_logout))
        .route("/api/specs", axum::routing::get(api::specs_list))
        .route("/api/specs/overview", axum::routing::get(api::specs_overview))
        .route("/api/specs/item", axum::routing::post(api::specs_save_item))
        .route("/api/specs/item/{section_id}/{id}", axum::routing::delete(api::specs_delete_item))
        .route("/api/specs/rebuild-relations", axum::routing::post(api::specs_rebuild_relations))
        .route("/api/specs/related/{id}", axum::routing::get(api::specs_related))
        .route("/api/specs/drift-check", axum::routing::post(api::specs_drift_check))
        .route("/api/specs/tree", axum::routing::get(api::specs_tree))
        .route("/api/specs/file/{path}", axum::routing::get(api::specs_get_file))
        .route("/api/plans", axum::routing::get(api::plans_list))
        .route("/api/plans/{seq}", axum::routing::get(api::plans_get))
        .route("/api/plans", axum::routing::post(api::plans_create))
        .route("/api/plans/{seq}", axum::routing::put(api::plans_update))
        .route("/api/plans/{seq}/transition", axum::routing::post(api::plans_transition))
        .route("/api/plans/{seq}/archive", axum::routing::post(api::plans_archive))
        .route("/api/plans/{seq}/merge", axum::routing::post(api::plans_merge))
        .route("/api/forge/wiki/{project}/pages", axum::routing::get(api::wiki_list_pages))
        .route("/api/forge/wiki/{project}/page/{slug}", axum::routing::get(api::wiki_get_page))
        .route("/api/forge/wiki/{project}/pages", axum::routing::post(api::wiki_create_page))
        .route("/api/forge/wiki/{project}/page/{slug}", axum::routing::put(api::wiki_update_page))
        .route("/api/forge/wiki/{project}/page/{slug}", axum::routing::delete(api::wiki_delete_page))
        .route("/api/forge/wiki/{project}/search", axum::routing::post(api::wiki_search))
        .route("/api/forge/wiki/{project}/tree", axum::routing::get(api::wiki_tree))
        .route("/api/forge/raw/{project}/tree", axum::routing::get(api::wiki_raw_tree))
        .route("/api/forge/raw/{project}/upload", axum::routing::post(api::wiki_raw_upload))
        .route("/api/forge/raw/{project}/mkdir", axum::routing::post(api::wiki_raw_mkdir))
        .route("/api/forge/raw/{project}/file/{path}", axum::routing::get(api::wiki_raw_file))
        .route("/api/forge/raw/{project}/file/{path}", axum::routing::delete(api::wiki_raw_delete_file))
        .route("/api/forge/mode", axum::routing::get(api::forge_mode_get))
        .route("/api/forge/mode", axum::routing::put(api::forge_mode_set))
        .route("/api/chats/sessions", axum::routing::get(api::chats_list_sessions))
        .route("/api/chats/session", axum::routing::post(api::chats_create_session))
        .route("/api/chats/session/{id}", axum::routing::get(api::chats_get_session))
        .route("/api/chats/session/{id}", axum::routing::delete(api::chats_delete_session))
        .route("/api/chats/sessions", axum::routing::delete(api::chats_delete_all_sessions))
        .route("/api/chats/session/{id}", axum::routing::patch(api::chats_rename_session))
        .route("/api/chats/session/{id}/message", axum::routing::post(api::chats_send_message))
        .route("/api/chats/session/{id}/approve/{index}", axum::routing::post(api::chats_approve))
        .route("/api/chats/session/{id}/reject-all", axum::routing::post(api::chats_reject_all))
        .route("/api/forge/relay/professions", axum::routing::get(api::relay_professions))
        .route("/api/forge/relay/runs", axum::routing::post(api::relay_start_run))
        .route("/api/forge/relay/runs/{run_id}", axum::routing::get(api::relay_get_run))
        .route("/api/forge/relay/runs/{run_id}/advance", axum::routing::post(api::relay_advance_run))
        .route("/api/forge/relay/runs/{run_id}/gate", axum::routing::post(api::relay_resolve_gate))
        .route("/api/workspace/list", axum::routing::get(api::workspace_list))
        .route("/api/workspace/status", axum::routing::get(api::workspace_status))
        .route("/api/workspace/open", axum::routing::post(api::workspace_open))
        .route("/api/workspace/browse", axum::routing::get(api::workspace_browse))
        .with_state(data)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
