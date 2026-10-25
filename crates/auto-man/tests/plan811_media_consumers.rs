//! PLAN-811 V-01/V-04/V-06: auto-man 侧媒体基础真消费方（Man consumer）。
//!
//! 两层证据：
//! 1. **真编译实跑消费**（不以字符串 assert 替代）：本测试 crate 以
//!    dev-dependency 直接依赖 `auto-media-core`（与生成应用同形——生成
//!    manifest 现在携带 `auto-media-core` path/workspace 依赖），实跑
//!    media_service 索引/Range 面、image_pipeline registry/HTTP/runtime
//!    面与真实图像解码（fixtures 真字节）。
//! 2. **生成面锚**：生成器模板发射 `auto_media_core::` 公开入口、交付
//!    闭包含 auto-media-core。生成 handler 的编译 oracle 在
//!    auto-man in-crate 测试（api_gen 50 项，含 media handler 编译面）；
//!    交付 fresh/locked 真构建由 792 driver 验收（plan811 T-07 跑档）。
//!
//! 注：auto-man test-http-e2e wire 档在 master 基线即编译红
//! （generate_api_rs 签名漂移 + plan734 tauri fixture），与本计划无关；
//! 生成 TCP 实机证据由 back_proxy th 档（031 native-ns + 字节保真）与
//! 792 真构建共同覆盖。

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/ui/031-image-viewer/tests/fixtures")
}

// ── 真消费方：media_service（索引/token/Range/content-type） ────────────

#[test]
fn man_consumer_drives_media_service_index_and_range() {
    let index =
        auto_media_core::media_service::index_directory(fixtures_root()).expect("fixtures index");
    // 白名单：目录里的 notes/corrupt 家族之外的可播放扩展；图像扩展
    // （png/jpg）不在 media_service 白名单——两套清单不合并（合同基线 §3）。
    assert!(index.entries.iter().all(|e| {
        auto_media_core::media_service::SUPPORTED_EXTENSIONS.contains(&e.extension.as_str())
            || e.extension.is_empty()
    }));
    // token 反查：id 是 blake3 相对路径 token，绝不携带绝对路径。
    for entry in &index.entries {
        assert_eq!(entry.id.len(), 64, "blake3 hex token");
        assert!(!entry.id.contains(':') && !entry.id.contains('\\'));
        assert!(
            auto_media_core::media_service::find(&index, &entry.id).is_some(),
            "token 反查命中"
        );
    }
    assert!(
        auto_media_core::media_service::find(&index, "deadbeef").is_none(),
        "未知 token miss"
    );
    // Range 面：后缀窗/越界 clamp/416。
    use auto_media_core::media_service::{parse_range, StreamPlan};
    assert_eq!(
        parse_range(Some("bytes=-100"), 1000),
        StreamPlan::Partial {
            start: 900,
            end: 999
        }
    );
    assert_eq!(
        parse_range(Some("bytes=1000-"), 1000),
        StreamPlan::Unsatisfiable
    );
    assert_eq!(
        parse_range(Some("bytes=0-99999"), 1000).content_length(1000),
        1000
    );
    assert_eq!(
        auto_media_core::media_service::content_type("mp4"),
        "video/mp4"
    );
}

// ── 真消费方：image_pipeline（registry/HTTP/runtime/真解码） ────────────

#[test]
fn man_consumer_drives_pipeline_registry_http_and_runtime() {
    use auto_media_core::image_pipeline::{
        media_http_response, MediaAssetKey, MediaAssetState, MediaLookup, MediaMetadata,
        MediaOrientation, MediaPriority, MediaRuntime, RenditionSpec,
    };

    // 数据面：registry 生命周期 + HTTP 状态 oracle（同一套规则喂生成后端）。
    let runtime = MediaRuntime::new(Duration::from_secs(60));
    let registry = runtime.registry();
    let ticket = registry.queue(
        MediaAssetKey {
            source_fingerprint: "man-consumer".into(),
            orientation: MediaOrientation::Normal,
            rendition: RenditionSpec::original(),
            revision: 5,
        },
        MediaMetadata {
            mime_type: "image/png".into(),
            ..Default::default()
        },
    );
    let uri = format!("/api/__auto/media/{}/{}", ticket.id, ticket.revision);
    assert_eq!(media_http_response(registry, "GET", &uri, None).status, 503);
    for state in [
        MediaAssetState::Reading,
        MediaAssetState::Decoding,
        MediaAssetState::Transforming,
    ] {
        registry.transition(ticket.id, state).unwrap();
    }
    registry
        .publish_ready(ticket.id, ticket.revision, Arc::<[u8]>::from([1, 2, 3, 4]))
        .unwrap();
    let response = media_http_response(registry, "GET", &uri, None);
    assert_eq!(response.status, 200);
    assert_eq!(
        response.headers.get("content-type"),
        Some(&"image/png".into())
    );
    let etag = response.headers["etag"].clone();
    assert_eq!(
        media_http_response(registry, "GET", &uri, Some(&etag)).status,
        304
    );
    assert_eq!(media_http_response(registry, "HEAD", &uri, None).body, None);

    // 运行时面：真解码（fixtures 真字节）→ Ready + URI 可解析。
    let fixture = fixtures_root().join("rgb-1x1.png");
    let decode_ticket = runtime.queue_path(fixture.clone(), MediaPriority::Current);
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match registry.lookup(decode_ticket.id, decode_ticket.revision) {
            MediaLookup::Ready(_) => break,
            MediaLookup::Failed(e) => panic!("decode failed: {e}"),
            _ if Instant::now() > deadline => panic!("decode timeout"),
            _ => std::thread::sleep(Duration::from_millis(10)),
        }
    }
    let thumb_uri = runtime.queue_thumbnail(&fixture, 64);
    assert!(thumb_uri.starts_with("/api/__auto/media/"), "{thumb_uri}");
    assert!(!thumb_uri.contains("fixtures"), "thumbnail 不泄源路径");
    assert_eq!(runtime.queue_thumbnail("no-such.png", 64), "");

    // 会话面：open→names→close 与未知会话语义。
    let session = runtime.open_session(fixtures_root());
    let deadline = Instant::now() + Duration::from_secs(2);
    while runtime.session_names(&session).is_empty() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    let names = runtime.session_names(&session);
    assert!(names.iter().any(|n| n == "rgb-1x1.png"), "{names:?}");
    assert!(runtime.close_session(&session));
    assert!(!runtime.close_session(&session), "双 close=false");
    assert_eq!(
        runtime.session_snapshot(&session),
        "{\"error\":\"unknown_session\"}"
    );

    // owned runtime 关闭：终态化 + 拒绝迟到发布（生命周期反例在此重放）。
    runtime.shutdown();
    assert_eq!(registry.pending_count(), 0);
    assert!(registry.is_shutdown());
}

// ── 生成面锚：模板发射 core 入口 + 交付闭包登记 ────────────────────────

#[test]
fn generators_emit_core_paths_and_delivery_closure_registers_media_core() {
    let api_gen_rs = include_str!("../../auto-api-gen/src/api_gen.rs");
    assert!(
        api_gen_rs.contains("auto_media_core::media_service::resolve_root"),
        "生成媒体 handler 直接消费 core 公开入口（api_gen）"
    );
    assert!(
        api_gen_rs.contains("auto_media_core::image_pipeline::media_http_response"),
        "生成的 /api/__auto/media 直查走 core HTTP 规则"
    );
    assert!(
        !api_gen_rs.contains("auto_lang::ui::image_pipeline"),
        "模板不再经 auto_lang::ui 路径（PLAN-811 剪断）"
    );
    let rust_ui_rs = include_str!("../../../../auto-ui/crates/auto-ui-tools/src/rust_ui.rs");
    assert!(
        rust_ui_rs.contains("auto_media_core::image_pipeline::open_media_session"),
        "viewer 桥函数直调 core（rust_ui）"
    );
    assert!(rust_ui_rs.contains("auto-media-core = {{ path = \"../crates/auto-media-core\" }}"));

    // 交付闭包：DELIVERY_CRATES 登记 auto-media-core（792 fresh 解析合同）。
    assert!(
        auto_man::rust_ui::DELIVERY_CRATES.contains(&"auto-media-core"),
        "DELIVERY_CRATES 必须含 auto-media-core，否则交付根 fresh 解析红"
    );
    assert!(
        auto_man::rust_ui::DELIVERY_PATCHES.contains(&"iced_widget")
            || auto_man::rust_ui::DELIVERY_PATCHES.len() >= 1,
        "交付 patch 面保持登记"
    );
}
