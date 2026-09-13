mod api;
mod types;
mod db;
async fn auto_media(
    method: axum::http::Method,
    uri: axum::http::Uri,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    let mut response = auto_lang::ui::image_pipeline::media_http_response(
        auto_lang::ui::image_pipeline::global_media_registry(),
        method.as_str(),
        uri.path(),
        headers.get(axum::http::header::IF_NONE_MATCH).and_then(|value| value.to_str().ok()),
    );
    // A freshly queued rendition is expected to be pending for a short time.
    // Give the bounded worker a small grace window so browser <img> loads do
    // not turn a transient 503 into a terminal error callback.
    if response.status == 503 {
        tokio::time::sleep(std::time::Duration::from_millis(120)).await;
        response = auto_lang::ui::image_pipeline::media_http_response(
            auto_lang::ui::image_pipeline::global_media_registry(),
            method.as_str(),
            uri.path(),
            headers.get(axum::http::header::IF_NONE_MATCH).and_then(|value| value.to_str().ok()),
        );
    }
    let mut builder = axum::response::Response::builder().status(response.status);
    for (name, value) in response.headers {
        builder = builder.header(name, value);
    }
    builder
        .body(axum::body::Body::from(response.body.map(|bytes| bytes.to_vec()).unwrap_or_default()))
        .expect("media response is valid")
}
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
        .route("/api/__auto/media/{id}/{revision}", axum::routing::get(auto_media).head(auto_media))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
