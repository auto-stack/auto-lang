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
fn media_plain(status: u16, msg: String) -> axum::response::Response {
    axum::response::Response::builder()
        .status(status)
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(axum::body::Body::from(msg))
        .expect("media plain response is valid")
}

fn media_json_str(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    o.push('"');
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

static MEDIA_INDEX: std::sync::OnceLock<auto_lang::ui::media_service::MediaIndex> =
    std::sync::OnceLock::new();

fn media_index() -> Option<&'static auto_lang::ui::media_service::MediaIndex> {
    let root = auto_lang::ui::media_service::resolve_root(None)?;
    Some(MEDIA_INDEX.get_or_init(|| {
        auto_lang::ui::media_service::index_directory(&root).unwrap_or_default()
    }))
}

async fn auto_media_scan() -> axum::response::Response {
    // PLAN-617 T-10: 三态如实回传 —— 根目录未配置 / 已配置但不存在 /
    // 正常。`root_missing` 让前端能把「目录不存在」与「目录为空」分开说；
    // 绝对路径本身不出后端（SD-02 安全边界）。
    let Some(root) = auto_lang::ui::media_service::resolve_root(None) else {
        // No configured root is an honest empty list, not a 500.
        return axum::response::Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from("{\"entries\":[],\"root_missing\":false}"))
            .expect("media scan response is valid");
    };
    if !root.exists() {
        return axum::response::Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from("{\"entries\":[],\"root_missing\":true}"))
            .expect("media scan response is valid");
    }
    let index = MEDIA_INDEX.get_or_init(|| {
        auto_lang::ui::media_service::index_directory(&root).unwrap_or_default()
    });
    let mut out = String::from("{\"entries\":[");
    for (i, e) in index.entries.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!(
            "{{\"id\":{},\"title\":{},\"name\":{},\"rel_dir\":{},\"relative_path\":{},\"extension\":{},\"bytes\":{},\"size_str\":{},\"video_url\":\"/api/media/stream/{}\"}}",
            media_json_str(&e.id),
            media_json_str(auto_lang::ui::media_service::display_title(&e.name)),
            media_json_str(&e.name),
            media_json_str(&e.rel_dir),
            media_json_str(&e.relative_path),
            media_json_str(&e.extension),
            e.bytes,
            media_json_str(&auto_lang::ui::media_service::human_size(e.bytes)),
            &e.id
        ));
    }
    out.push_str("],\"root_missing\":false}");
    axum::response::Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(out))
        .expect("media scan response is valid")
}

async fn auto_media_stream(
    axum::extract::Path(id): axum::extract::Path<String>,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    use auto_lang::ui::media_service as ms;
    use tokio::io::{AsyncReadExt, AsyncSeekExt};

    let Some(index) = media_index() else {
        return media_plain(503, "no media root configured".to_string());
    };
    let Some(entry) = ms::find(index, &id) else {
        return media_plain(404, "unknown media id".to_string());
    };
    // Re-join under the index root: the caller only ever supplied a token.
    let Ok(root) = std::env::var("AUTO_MEDIA_ROOT").map(std::path::PathBuf::from) else {
        return media_plain(503, "no media root configured".to_string());
    };
    let path = ms::entry_path(&root, entry);
    let Ok(meta) = tokio::fs::metadata(&path).await else {
        return media_plain(404, "media file missing".to_string());
    };
    let len = meta.len();
    let range = headers
        .get(axum::http::header::RANGE)
        .and_then(|v| v.to_str().ok());
    let plan = ms::parse_range(range, len);

    if plan == ms::StreamPlan::Unsatisfiable {
        return axum::response::Response::builder()
            .status(416)
            .header("Content-Range", format!("bytes */{}", len))
            .header("Content-Length", "0")
            .body(axum::body::Body::empty())
            .expect("media 416 response is valid");
    }

    let (start, end) = match &plan {
        ms::StreamPlan::Full => (0u64, len.saturating_sub(1)),
        ms::StreamPlan::Partial { start, end } => (*start, *end),
        ms::StreamPlan::Unsatisfiable => unreachable!("handled above"),
    };

    let mut file = match tokio::fs::File::open(&path).await {
        Ok(f) => f,
        Err(_) => return media_plain(404, "media file unreadable".to_string()),
    };
    if start > 0 && file.seek(std::io::SeekFrom::Start(start)).await.is_err() {
        return media_plain(500, "seek failed".to_string());
    }
    // Bounded chunks: the whole (possibly multi-GB) file is never resident.
    let window = file.take(end - start + 1);
    let body = axum::body::Body::from_stream(tokio_util::io::ReaderStream::with_capacity(
        window,
        256 * 1024,
    ));

    let mut builder = axum::response::Response::builder()
        .status(plan.status())
        .header("Content-Type", ms::content_type(&entry.extension))
        .header("Accept-Ranges", "bytes")
        .header("Content-Length", (end - start + 1).to_string());
    if let ms::StreamPlan::Partial { start: s, end: e } = plan {
        builder = builder.header("Content-Range", ms::content_range(s, e, len));
    }
    builder.body(body).expect("media stream response is valid")
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
        .route("/api/media/scan", axum::routing::get(auto_media_scan))
        .route("/api/media/stream/:id", axum::routing::get(auto_media_stream).head(auto_media_stream))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
