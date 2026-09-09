//! PLAN-593（Design 29 Phase 1：token 单源化）零漂移基线测试。
//!
//! 方法论「先钉后改」：本文件把迁移前（2026-09-09 master）的语义色值与
//! CSS 产物固化为期望表/金样——此时被测函数尚是旧实现，测试**直接绿**
//! （旧实现即真值）；S4/S5 改造（resolve_semantic_rgb match 臂→registry 查表、
//! generate_base_css 变量块→registry 渲染）完成后同一组断言即零漂移证明。
//!
//! 词表/投影/内置表的完备性断言见 S8 补入的 T-c 组。

use crate::ui::style::theme;
use crate::ui::style::Color;

/// 旧实现静态语义色期望表（抄自 theme.rs resolve_semantic_rgb match 臂，
/// Plan 518 stella 双主题校准值）。Primary 独立断言（accent 驱动）。
fn assert_static_arms(dark: bool, expected: &[(Color, (u8, u8, u8))]) {
    theme::set_dark_mode(dark);
    for (color, want) in expected {
        assert_eq!(
            theme::resolve_semantic_rgb(color),
            Some(*want),
            "{color:?} @ dark={dark} 漂移（迁移前基线被破坏）"
        );
    }
}

#[test]
fn zero_drift_semantic_table_light() {
    assert_static_arms(
        false,
        &[
            (Color::Secondary, (227, 221, 209)),   // #e3ddd1 暖灰一档深
            (Color::Background, (245, 241, 232)),  // #f5f1e8 暖纸
            (Color::Surface, (251, 248, 242)),     // #fbf8f2 卡片微浮
            (Color::Muted, (240, 235, 226)),       // #f0ebe2 暖 muted
            (Color::Error, (239, 68, 68)),
            (Color::Warning, (234, 179, 8)),
            (Color::Success, (34, 197, 94)),
            (Color::Info, (59, 130, 246)),
            (Color::OnPrimary, (248, 250, 252)),
            (Color::OnSecondary, (42, 39, 35)),
            (Color::OnDestructive, (248, 250, 252)),
            (Color::OnBackground, (42, 39, 35)),
            (Color::OnSurface, (125, 119, 109)),
            (Color::Border, (227, 221, 209)),
        ],
    );
    assert_eq!(theme::resolve_border_rgb(), (227, 221, 209));
}

#[test]
fn zero_drift_semantic_table_dark() {
    assert_static_arms(
        true,
        &[
            (Color::Secondary, (51, 65, 85)),      // #334155 slate-700
            (Color::Background, (20, 26, 41)),     // #141a29 深蓝黑
            (Color::Surface, (26, 34, 53)),        // #1a2235 面板
            (Color::Muted, (30, 41, 59)),          // slate-800
            (Color::Error, (239, 68, 68)),
            (Color::Warning, (234, 179, 8)),
            (Color::Success, (34, 197, 94)),
            (Color::Info, (59, 130, 246)),
            (Color::OnPrimary, (15, 23, 42)),      // #0f172a 黑字
            (Color::OnSecondary, (248, 250, 252)),
            (Color::OnDestructive, (248, 250, 252)),
            (Color::OnBackground, (248, 250, 252)),
            (Color::OnSurface, (151, 163, 181)),
            (Color::Border, (40, 49, 70)),         // #283146
        ],
    );
    assert_eq!(theme::resolve_border_rgb(), (40, 49, 70));
}

/// Primary（accent 驱动臂）双态基线：默认 indigo，dark L+10。
#[test]
fn zero_drift_primary_indigo() {
    theme::set_accent_name("indigo");
    theme::set_dark_mode(false);
    assert_eq!(theme::resolve_semantic_rgb(&Color::Primary), Some((100, 102, 241)));
    theme::set_dark_mode(true);
    assert_eq!(theme::resolve_semantic_rgb(&Color::Primary), Some((147, 148, 245)));
    theme::set_accent_name("indigo"); // 还原默认，防污染其他用例
}

/// accent 5 预设 × 双态的 `accent_primary_rgb` 基线（index.html bootstrap 与
/// iced 窗口调色板消费面）。E4/E5 迁 registry 后此表不松。
#[test]
fn zero_drift_accent_presets() {
    for (name, light, dark) in [
        ("indigo", (100, 102, 241), (147, 148, 245)),
        ("coral", (195, 111, 105), (209, 146, 141)), // 既有 coral 测试同值互证
        ("ocean", (60, 131, 245), (108, 162, 248)),
        ("sage", (15, 182, 127), (19, 229, 159)),
        ("amber", (244, 158, 10), (246, 178, 59)),
    ] {
        assert_eq!(
            theme::accent_primary_rgb(name, false),
            Some(light),
            "accent {name} light 漂移"
        );
        assert_eq!(
            theme::accent_primary_rgb(name, true),
            Some(dark),
            "accent {name} dark 漂移"
        );
    }
}

/// T-b：`generate_base_css()` 全文金样（S5 V1 改造的逐字节零漂移证明）。
/// 金样 = 迁移前模板逐字提取（tests/fixtures/plan593/base_css.golden）。
#[test]
fn base_css_golden() {
    let css = crate::ui_gen::vue::VueGenerator::generate_base_css();
    let golden = include_str!("../tests/fixtures/plan593/base_css.golden");
    assert_eq!(css, golden, "generate_base_css 输出与迁移前金样不一致");
}
