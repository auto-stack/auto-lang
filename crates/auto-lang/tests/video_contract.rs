//! PLAN-617 T-19：`video` 元素的 VM 侧受控媒体契约回归。
//!
//! 断言的是**接线是活的**——`video` 节点经 AURA → `AuraViewBuilder` 真的产出
//! [`View::Video`]，且 §2.3 的「作者面状态值 → 节点字段」翻译正确：
//! `paused` 由 `is_playing == false` 求值而来、`volume` 0..100 原样透传、
//! `rate <= 0` 被换成 1.0（0 倍速会冻住播放）、缺省值符合契约。
//!
//! 与 `image_surface_contract.rs` 同一套驱动（fixture 走真实文件解析 → VM bridge
//! → builder），因此这里验的是**真实管线**，不是手搭的测试树。

#![cfg(feature = "ui-iced")]

use auto_lang::ui::aura_view_builder::AuraViewBuilder;
use auto_lang::ui::view::View;
use auto_lang::ui::vm_bridge::VmBridge;
use auto_lang::ui_gen::{generate_component_from_file, ComponentGenOptions};

fn fixture_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("video_contract.at")
}

/// 走「文件解析 → VM bridge → builder」，取指定 widget 的 View。
fn build(name: &str) -> View<auto_lang::ui::interpreter::DynamicMessage> {
    let generated =
        generate_component_from_file(&fixture_path(), ComponentGenOptions::default())
            .expect("fixture 必须能解析与生成");
    let widget = generated
        .widgets
        .iter()
        .find(|w| w.name == name)
        .unwrap_or_else(|| panic!("fixture 缺少 widget {name}"));
    let bridge = VmBridge::new(widget).expect("VM bridge");
    AuraViewBuilder::new(&bridge, &widget.name).build(&widget.view_tree)
}

/// 全量契约：每个下行字段都从作者面状态里取到，且单位/语义正确。
#[test]
fn video_props_map_onto_the_contract_fields() {
    let view = build("VideoPlayer");
    let View::Video {
        src,
        paused,
        position,
        volume,
        muted,
        rate,
        label,
        ..
    } = view
    else {
        panic!("video 节点应产出 View::Video，实际 {view:?}");
    };

    assert_eq!(src, "/api/media/stream/abc123");
    // `paused: .is_playing == false` 且 is_playing = true → 不暂停。
    assert!(!paused, "is_playing=true 应翻成 paused=false");
    assert_eq!(position, Some(12.5), "seek 目标（秒）应原样透传");
    assert_eq!(volume, 37, "volume 是作者面 0..100，原样透传（与 mpv 同刻度）");
    assert!(muted);
    assert_eq!(rate, 1.5);
    assert_eq!(label, "现场", "title 优先作为显示名");
}

/// 缺省值矩阵：只写 `src` 时的行为必须与契约默认一致——
/// **暂停**（不会莫名其妙自动播）、满音量、不静音、原速、不 seek。
#[test]
fn video_defaults_follow_the_contract() {
    let view = build("VideoDefaults");
    let View::Video {
        src,
        paused,
        position,
        volume,
        muted,
        rate,
        label,
        ..
    } = view
    else {
        panic!("video 节点应产出 View::Video，实际 {view:?}");
    };

    assert_eq!(src, r"E:\Video\caelestia.mp4");
    assert!(paused, "未声明 paused 时应为暂停（绝不默认自动播）");
    assert_eq!(position, None, "未声明 position 时不下发位置");
    assert_eq!(volume, 100, "默认满音量");
    assert!(!muted);
    assert_eq!(rate, 1.0, "默认原速（0.0 会冻住播放）");
    // 未给 title/label/alt → 退回源路径末段，降级面板要能显示真实名字。
    assert_eq!(label, "caelestia.mp4", "显示名应退回源路径末段");
}

/// `rate` 的护栏与 `volume` 的夹紧：越界值不得原样送到播放内核。
#[test]
fn video_clamps_out_of_range_values() {
    // 用一个内联 widget 直接钉边界（复用 fixture 的驱动路径）。
    let src = r#"
widget VideoClamp {
    msg { Nop }
    model {
        var src str = "x.mp4"
        var volume int = 250
        var rate float = 0.0
    }
    view {
        video {
            src: .src
            volume: .volume
            rate: .rate
        }
    }
    on { .Nop -> { } }
}
"#;
    let dir = std::env::temp_dir().join("auto-lang-video-clamp");
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("video_clamp.at");
    std::fs::write(&path, src).expect("write fixture");

    let generated =
        generate_component_from_file(&path, ComponentGenOptions::default()).expect("解析与生成");
    let widget = generated
        .widgets
        .iter()
        .find(|w| w.name == "VideoClamp")
        .expect("VideoClamp widget");
    let bridge = VmBridge::new(widget).expect("VM bridge");
    let view = AuraViewBuilder::new(&bridge, &widget.name).build(&widget.view_tree);

    let View::Video { volume, rate, .. } = view else {
        panic!("应产出 View::Video，实际 {view:?}");
    };
    assert_eq!(volume, 100, "越界音量应夹到 100，不得原样下发给 mpv");
    assert_eq!(rate, 1.0, "rate<=0 应换成 1.0，而不是把播放冻住");
}
