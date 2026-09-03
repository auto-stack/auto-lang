// Unified Styling System for AutoUI
//
// This module provides a Tailwind CSS-inspired utility class system that works across
// multiple backends (GPUI, Iced, etc.) through a unified intermediate representation.
//
// ═══════════════════════════════════════════════════════════════════════════
// Plan 527 覆盖契约(VM 轨 Tailwind 全量覆盖)
// ═══════════════════════════════════════════════════════════════════════════
// 覆盖基线 = Tailwind v3.4 core 清单(tests/fixtures/tailwind-v34-utilities.txt,
// 8861 类×15 families,tools/gen_tailwind_manifest.py 可再生)。契约三支柱:
//
// 1. 零静默丢弃 —— `Style::parse_reported` 报告未映射类原文名;常驻审计
//    `tests/style_parity.rs`(cargo t 档)断言白名单外零 missing。
// 2. 覆盖率表 —— `docs/style-coverage.md`(家族×状态矩阵,STYLE_COVERAGE_REGEN=1
//    与断言同源再生):布局/视觉/文本三家族非白名单类必须 iced applied
//    (parsed-only 须 PARSED_ONLY_ALLOWED 在册)。
// 3. 不做/受限台账 —— docs/plans/KNOWN-DEBT-AND-RISKS.md P527 节(原生无
//    语义/宿主上限/近似口径逐族登记),与 coverage 表白名单互链。
//
// 变体管道:`hover:`/`focus:`/`active:`/`disabled:`(Variant)+ responsive
// 断点 `sm/md/lg/xl/2xl:`(Breakpoint,theme::window_width 解析期门控)+
// `dark:`(theme::dark_mode 门控)——命中态进 base,未命中登记 variant_classes
// 可见不静默;resize/主题切换经 view 重建→重解析生效。

mod class;
mod color;
mod layout_extract;
mod parser;

pub use class::{StyleClass, SizeValue, GradientDir, ObjectFit};
pub use color::Color;
pub use layout_extract::BoxLayout;
pub use parser::StyleParser;

// Backend adapters (only compile when the respective backend is enabled)
#[cfg(feature = "ui-gpui")]
pub mod gpui_adapter;

/// Backend-neutral theme state + semantic color resolution (Plan 413/418
/// follow-up: extracted from iced_adapter so `code-editor`-only builds
/// compile — iced_adapter re-exports it).
pub mod theme;

#[cfg(feature = "ui-gpui")]
pub use gpui_adapter::GpuiStyle; // Re-export for backend adapters

#[cfg(feature = "ui-iced")]
pub mod iced_adapter;

#[cfg(feature = "ui-iced")]
pub use iced_adapter::IcedStyle; // Re-export for backend adapters

#[cfg(feature = "ui-headless")]
pub mod headless_adapter;

#[cfg(feature = "ui-headless")]
pub use headless_adapter::HeadlessStyle;

/// Plan 527 T7: Tailwind responsive 断点(min-width 语义,移动端优先)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Breakpoint {
    Sm,
    Md,
    Lg,
    Xl,
    Xxl,
}

impl Breakpoint {
    /// 断点最小宽度(px):sm 640 / md 768 / lg 1024 / xl 1280 / 2xl 1536。
    pub fn min_width(self) -> f32 {
        match self {
            Breakpoint::Sm => 640.0,
            Breakpoint::Md => 768.0,
            Breakpoint::Lg => 1024.0,
            Breakpoint::Xl => 1280.0,
            Breakpoint::Xxl => 1536.0,
        }
    }

    /// 解析 `sm:`/`md:`/`lg:`/`xl:`/`2xl:` 前缀。
    pub fn from_prefix(prefix: &str) -> Option<Self> {
        match prefix {
            "sm" => Some(Breakpoint::Sm),
            "md" => Some(Breakpoint::Md),
            "lg" => Some(Breakpoint::Lg),
            "xl" => Some(Breakpoint::Xl),
            "2xl" => Some(Breakpoint::Xxl),
            _ => None,
        }
    }
}

/// Plan 527 T6: 变体前缀通用管道 —— `hover:`/`focus:`/`active:`/`disabled:`
/// 同构声明,`Style::variant_classes` 按声明序收集。iced 消费沿按钮 hover
/// 先例扩展状态回调面(v1 仅按钮族真消费,其余 widget 登记 parsed-only,
/// coverage 表可见不静默)。
/// Plan 527 T7: Responsive(Breakpoint) —— 宿主窗口宽信号(theme::window_width,
/// renderer view 前回填)解析期门控:命中断点的 responsive 类进 base,
/// 未命中的仅登记 variant(可见不静默;resize→view 重建→重解析既有回路
/// 让断点随窗口宽实时生效,接入点裁定记录复审)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    Hover,
    Focus,
    Active,
    Disabled,
    Responsive(Breakpoint),
    /// Plan 527 T8: dark: 前缀 —— theme::dark_mode 主题态门控(解析期
    /// 过滤,dark 态命中进 base,light 态仅登记;语义色双主题值在 theme.rs)。
    Dark,
}

impl Variant {
    /// 解析 `<variant>:` 前缀;未知前缀返回 None(token 走基类解析或报告通道)。
    pub fn from_prefix(prefix: &str) -> Option<Self> {
        match prefix {
            "hover" => Some(Variant::Hover),
            "focus" => Some(Variant::Focus),
            "active" => Some(Variant::Active),
            "disabled" => Some(Variant::Disabled),
            _ => None,
        }
    }
}

/// Parsed style collection ready to be applied to backend-specific components
#[derive(Debug, Clone, Default)]
pub struct Style {
    pub classes: Vec<StyleClass>,
    /// DEPRECATED(Plan 527 T6 保留转发一版): 读取方应改用
    /// `variant_classes` 的 `Variant::Hover` 分量;parse 双轨写入保持旧
    /// 消费点(iced 按钮/svg hover 臂)兼容,下版移除。
    pub hover_classes: Vec<StyleClass>,
    /// Plan 527 T6: 通用变体管道(hover/focus/active/disabled,按声明序)。
    pub variant_classes: Vec<(Variant, StyleClass)>,
}

impl Style {
    /// 取指定变体的类列表(按声明序)。
    pub fn variant_slice(&self, variant: Variant) -> Vec<StyleClass> {
        self.variant_classes
            .iter()
            .filter(|(v, _)| *v == variant)
            .map(|(_, c)| c.clone())
            .collect()
    }

    /// 是否声明了指定变体。
    pub fn has_variant(&self, variant: Variant) -> bool {
        self.variant_classes.iter().any(|(v, _)| *v == variant)
    }
    /// Parse a style string into a Style collection
    pub fn parse(input: &str) -> Result<Self, String> {
        // Plan 527 T1: route through parse_reported and drop the report —
        // legacy callers keep the exact silent-drop behavior (compat risk 0).
        Ok(Self::parse_reported(input).0)
    }

    /// Plan 527 T1: parse with an explicit report channel — tokens that fail
    /// to map come back by their original name (variant prefix included)
    /// instead of being silently dropped. The audit harness
    /// (tests/style_parity.rs) is the primary consumer; ad-hoc callers can
    /// surface drift (e.g. log unmapped classes) instead of losing them.
    ///
    /// Plan 527 T6: `<variant>:<class>` 前缀通用化(hover/focus/active/
    /// disabled 同构);未知变体前缀(如 group-hover:)按未映射报告。
    /// Plan 527 T7: responsive 断点(sm/md/lg/xl/2xl)经窗口宽信号门控——
    /// 命中断点进 base(Plan 409 md+ 语义升级为真实断点),未命中仅登记
    /// variant_classes(可见不静默;窗口 resize 触发 view 重建→重解析生效)。
    pub fn parse_reported(input: &str) -> (Self, Vec<String>) {
        let mut classes = Vec::new();
        let mut hover_classes = Vec::new();
        let mut variant_classes: Vec<(Variant, StyleClass)> = Vec::new();
        let mut unmapped = Vec::new();
        let width = theme::window_width();
        for token in input.split_whitespace() {
            if let Some((prefix, rest)) = token.split_once(':') {
                // Plan 527 T8: dark: 前缀 —— 主题态门控(dark 态命中进 base,
                // light 态仅登记 variant,可见不静默;主题切换走 view 重建)
                if prefix == "dark" {
                    match StyleClass::parse_single(rest) {
                        Ok(c) => {
                            variant_classes.push((Variant::Dark, c.clone()));
                            if theme::dark_mode() {
                                classes.push(c);
                            }
                        }
                        Err(_) => unmapped.push(token.to_string()),
                    }
                    continue;
                }
                // Plan 527 T7: responsive 断点 —— 窗口宽门控
                if let Some(bp) = Breakpoint::from_prefix(prefix) {
                    match StyleClass::parse_single(rest) {
                        Ok(c) => {
                            variant_classes.push((Variant::Responsive(bp), c.clone()));
                            if width >= bp.min_width() {
                                classes.push(c);
                            }
                        }
                        Err(_) => unmapped.push(token.to_string()),
                    }
                    continue;
                }
                // Plan 527 T6: 状态变体(hover/focus/active/disabled)
                if let Some(variant) = Variant::from_prefix(prefix) {
                    match StyleClass::parse_single(rest) {
                        Ok(c) => {
                            if variant == Variant::Hover {
                                hover_classes.push(c.clone());
                            }
                            variant_classes.push((variant, c));
                        }
                        Err(_) => unmapped.push(token.to_string()),
                    }
                    continue;
                }
            }
            match StyleClass::parse_single(token) {
                Ok(c) => classes.push(c),
                Err(_) => unmapped.push(token.to_string()),
            }
        }
        (Self { classes, hover_classes, variant_classes }, unmapped)
    }

    /// Create an empty style
    pub fn empty() -> Self {
        Self::default()
    }

    /// Add a style class
    pub fn add(mut self, class: StyleClass) -> Self {
        self.classes.push(class);
        self
    }

    /// Plan 409 §10 续: display:none 语义——含 Hidden 且无任何 display class 覆盖。
    /// Tailwind 中 `hidden md:flex` 在 md+ 下 display 胜出(特异性更高);VM 桌面
    /// 窗口按 md+ 语义,所以同时有 display(Flex/Grid/Block/Inline/...)时不隐藏。
    pub fn is_hidden(&self) -> bool {
        let has_hidden = self.classes.iter().any(|c| matches!(c, StyleClass::Hidden));
        if !has_hidden {
            return false;
        }
        let has_display = self.classes.iter().any(|c| matches!(
            c,
            StyleClass::Flex
                | StyleClass::Grid
                | StyleClass::Block
                | StyleClass::Inline
                | StyleClass::InlineBlock
                | StyleClass::InlineFlex
        ));
        !has_display
    }
}

impl From<&str> for Style {
    fn from(input: &str) -> Self {
        Self::parse(input).expect("Failed to parse style string")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let style = Style::parse("p-4 gap-2 bg-white").unwrap();
        assert_eq!(style.classes.len(), 3);
    }

    #[test]
    fn test_from_str() {
        let style: Style = "flex items-center".into();
        assert_eq!(style.classes.len(), 2);
    }

    #[test]
    fn test_hover_classes_split_from_base() {
        let s = Style::parse("h-6 w-6 bg-transparent hover:bg-muted/60 hover:text-foreground")
            .unwrap();
        assert_eq!(s.classes.len(), 3, "base classes: {:?}", s.classes);
        assert_eq!(s.hover_classes.len(), 2, "hover classes: {:?}", s.hover_classes);
        assert!(s.hover_classes.iter().any(|c| matches!(c, StyleClass::BackgroundColor(_))));
        assert!(s.hover_classes.iter().any(|c| matches!(c, StyleClass::TextColor(_))));
    }

    #[test]
    fn test_hover_class_with_alpha_parses() {
        let s = Style::parse("hover:bg-red-500/10").unwrap();
        assert!(matches!(
            s.hover_classes.first(),
            Some(StyleClass::BackgroundColor(Color::Rgba { a, .. })) if (*a as f32 / 255.0 - 0.1).abs() < 0.01
        ));
    }

    // ========== Plan 527 T1: parse_reported 报告通道 ==========

    #[test]
    fn test_parse_reported_returns_unmapped_tokens() {
        let (style, unmapped) = Style::parse_reported("p-4 no-such-class bg-white float-left");
        assert_eq!(style.classes.len(), 2, "mapped: {:?}", style.classes);
        assert_eq!(
            unmapped,
            vec!["no-such-class".to_string(), "float-left".to_string()],
            "未映射类按原文名报告(float-left 为白名单显式豁免类,审计在册)"
        );
    }

    #[test]
    fn test_parse_reported_hover_unmapped_keeps_variant_prefix() {
        let (style, unmapped) = Style::parse_reported("hover:bg-red-500 hover:nope-1");
        assert_eq!(style.hover_classes.len(), 1);
        assert_eq!(style.classes.len(), 0);
        assert_eq!(unmapped, vec!["hover:nope-1".to_string()]);
    }

    #[test]
    fn test_parse_reported_clean_input_empty_report() {
        let (style, unmapped) = Style::parse_reported("flex items-center p-4 bg-white");
        assert_eq!(style.classes.len(), 4);
        assert!(unmapped.is_empty());
    }

    #[test]
    fn test_parse_legacy_drop_behavior_unchanged() {
        // Plan 527 T1 兼容约束:旧 parse 走同路但丢弃未映射 token,行为不变。
        let style = Style::parse("p-4 invalid-class").unwrap();
        assert_eq!(style.classes.len(), 1);
    }

    // ========== Plan 527 T6: variant 管道泛化 ==========

    #[test]
    fn test_variant_pipeline_collects_all_variants() {
        let s = Style::parse(
            "bg-card hover:bg-primary/20 focus:ring-2 active:bg-muted disabled:opacity-50",
        )
        .unwrap();
        assert_eq!(s.classes.len(), 1, "base: {:?}", s.classes);
        assert_eq!(s.variant_classes.len(), 4, "variants: {:?}", s.variant_classes);
        assert!(s.has_variant(super::Variant::Hover));
        assert!(s.has_variant(super::Variant::Focus));
        assert!(s.has_variant(super::Variant::Active));
        assert!(s.has_variant(super::Variant::Disabled));
        // 双轨:hover_classes 兼容旧消费点(iced 按钮/svg 臂)
        assert_eq!(s.hover_classes.len(), 1);
        assert_eq!(s.variant_slice(super::Variant::Focus).len(), 1);
        assert_eq!(s.variant_slice(super::Variant::Hover).len(), 1);
    }

    #[test]
    fn test_variant_unknown_prefix_reported_unmapped() {
        let (s, unmapped) =
            Style::parse_reported("p-4 group-hover:bg-red-500 focus:bg-blue-500");
        assert_eq!(s.classes.len(), 1);
        assert_eq!(s.variant_classes.len(), 1, "focus 应进变体管道");
        assert_eq!(
            unmapped,
            vec!["group-hover:bg-red-500".to_string()],
            "未知变体前缀按未映射原文名报告,不再静默"
        );
    }

    #[test]
    fn test_variant_hover_legacy_behavior_zero_regression() {
        // 既有 hover 行为零回归:类收集与旧字段完全一致
        let s = Style::parse("h-6 w-6 bg-transparent hover:bg-muted/60 hover:text-foreground")
            .unwrap();
        assert_eq!(s.hover_classes.len(), 2);
        assert_eq!(s.variant_slice(super::Variant::Hover).len(), 2);
        assert!(s.hover_classes.iter().all(|c| s
            .variant_slice(super::Variant::Hover)
            .contains(c)));
    }

    // ========== Plan 527 T7: responsive 断点 ==========

    #[test]
    fn test_breakpoint_boundary_values() {
        use super::Breakpoint as Bp;
        assert_eq!(Bp::from_prefix("sm"), Some(Bp::Sm));
        assert_eq!(Bp::from_prefix("md"), Some(Bp::Md));
        assert_eq!(Bp::from_prefix("lg"), Some(Bp::Lg));
        assert_eq!(Bp::from_prefix("xl"), Some(Bp::Xl));
        assert_eq!(Bp::from_prefix("2xl"), Some(Bp::Xxl));
        assert_eq!(Bp::from_prefix("xxl"), None, "非法断点名不收");
        assert_eq!(Bp::from_prefix("hover"), None);
        // Tailwind v3.4 官方断点值
        assert_eq!(Bp::Sm.min_width(), 640.0);
        assert_eq!(Bp::Md.min_width(), 768.0);
        assert_eq!(Bp::Lg.min_width(), 1024.0);
        assert_eq!(Bp::Xl.min_width(), 1280.0);
        assert_eq!(Bp::Xxl.min_width(), 1536.0);
    }

    #[test]
    fn test_responsive_gates_by_window_width() {
        use crate::ui::style::theme;
        // 默认 1024(md/lg 命中,xl/2xl 不命中)
        theme::set_window_width(1024.0);
        let s = Style::parse("hidden md:flex xl:grid 2xl:block").unwrap();
        assert!(s.classes.iter().any(|c| matches!(c, StyleClass::Hidden)));
        assert!(s.classes.iter().any(|c| matches!(c, StyleClass::Flex)), "md@1024 命中");
        assert!(
            !s.classes.iter().any(|c| matches!(c, StyleClass::Grid)),
            "xl@1280 不应命中 1024 窗"
        );
        assert!(!s.classes.iter().any(|c| matches!(c, StyleClass::Block)));
        // 全部 responsive 前缀登记 variant(未命中也可见,不静默)
        assert_eq!(s.variant_classes.len(), 3);
        // 窄窗 500:md 不命中 → hidden 生效
        theme::set_window_width(500.0);
        let s = Style::parse("hidden md:flex").unwrap();
        assert!(s.is_hidden(), "窄窗 md:flex 未命中 → hidden 生效");
        assert!(!s.classes.iter().any(|c| matches!(c, StyleClass::Flex)));
        // 宽窗 800:md 命中 → display 覆盖 hidden
        theme::set_window_width(800.0);
        let s = Style::parse("hidden md:flex").unwrap();
        assert!(!s.is_hidden(), "md@800 命中 → flex 覆盖 hidden");
        // 边界:恰好 768 命中,767.99 不命中
        theme::set_window_width(768.0);
        assert!(Style::parse("md:flex").unwrap().classes.iter().any(|c| matches!(c, StyleClass::Flex)));
        theme::set_window_width(767.9);
        assert!(!Style::parse("md:flex").unwrap().classes.iter().any(|c| matches!(c, StyleClass::Flex)));
        // 还原默认,防污染其他用例
        theme::set_window_width(1024.0);
    }

    // ========== Plan 527 T8: dark: 主题过滤 ==========

    #[test]
    fn test_dark_prefix_gates_by_theme_mode() {
        use crate::ui::style::theme;
        // dark 态:dark: 类命中进 base(后应用胜出)
        theme::set_dark_mode(true);
        let s = Style::parse("bg-card dark:bg-slate-900").unwrap();
        let has_slate = s
            .classes
            .iter()
            .any(|c| matches!(c, StyleClass::BackgroundColor(crate::ui::style::Color::Slate(_))));
        assert!(has_slate, "dark 态 dark:bg-slate-900 应进 base: {:?}", s.classes);
        assert!(s.classes.iter().any(|c| matches!(c, StyleClass::BackgroundColor(crate::ui::style::Color::Surface))));
        assert_eq!(s.variant_classes.len(), 1);

        // light 态:dark: 类不进 base(仅登记),同一样本只有底色
        theme::set_dark_mode(false);
        let s = Style::parse("bg-card dark:bg-slate-900").unwrap();
        assert!(
            !s.classes.iter().any(|c| matches!(c, StyleClass::BackgroundColor(crate::ui::style::Color::Slate(_)))),
            "light 态 dark: 类不进 base: {:?}",
            s.classes
        );
        assert!(s.classes.iter().any(|c| matches!(c, StyleClass::BackgroundColor(crate::ui::style::Color::Surface))));
        assert_eq!(s.variant_classes.len(), 1, "light 态登记可见不静默");

        // 还原默认,防污染其他用例(Plan 408 默认 dark)
        theme::set_dark_mode(true);
    }
}

#[cfg(test)]
mod plan411_tests {
    use super::Style;

    #[test]
    fn test_md_hidden_is_hidden() {
        let s = Style::parse("md:hidden -ml-2").unwrap();
        assert!(s.is_hidden(), "md:hidden should be hidden: {:?}", s.classes);
    }

    #[test]
    fn test_hidden_sm_inline_not_hidden() {
        let s = Style::parse("font-bold text-lg hidden sm:inline").unwrap();
        assert!(!s.is_hidden());
    }

    #[test]
    fn test_responsive_text_size_ladder() {
        // Plan 411 P0-B: responsive prefixes are stripped (desktop semantics),
        // so this ladder must parse to three size classes with the LAST one
        // (lg:text-7xl) winning in sequential adapter application.
        let s = Style::parse("text-4xl md:text-5xl lg:text-7xl").unwrap();
        let has_4xl = s.classes.iter().any(|c| matches!(c, crate::ui::style::StyleClass::Text4Xl));
        assert!(has_4xl, "ladder parse: {:?}", s.classes);
    }

    #[test]
    fn test_md_hidden_classes_parse() {
        // Plan 411: prefix stripping yields Hidden；-ml-2 曾按未知类跳过，
        // class.rs 增 NegativeMargin* 族（pricing-table 打磨）后按序解析为
        // [Hidden, NegativeMarginLeft]（472 T2 修复 stale 断言，master 既有红）。
        let s = Style::parse("md:hidden -ml-2").unwrap();
        assert_eq!(s.classes.len(), 2);
        assert!(matches!(s.classes[0], crate::ui::style::StyleClass::Hidden));
        assert!(matches!(
            s.classes[1],
            crate::ui::style::StyleClass::NegativeMarginLeft(_)
        ));
    }

    #[test]
    fn test_text_5xl_9xl_parse() {
        // Plan 411 P0-B: hero ladder needs text-5xl..text-9xl to exist so
        // "text-4xl md:text-5xl lg:text-7xl" resolves to 72px (last wins).
        use crate::ui::style::StyleClass;
        assert!(matches!(StyleClass::parse_single("text-5xl"), Ok(StyleClass::Text5Xl)));
        assert!(matches!(StyleClass::parse_single("lg:text-7xl"), Ok(StyleClass::Text7Xl)));
        assert!(matches!(StyleClass::parse_single("text-9xl"), Ok(StyleClass::Text9Xl)));
    }
}
