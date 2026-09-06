//! Button variant/size preset table — 单一事实源（PLAN-571）。
//!
//! 设计语义：`default` 承载 Web UA stylesheet 的显式等价物（裸 `<button>` 在
//! 浏览器里有 UA 预填：中性填充+边框+圆角；VM(iced) 没有这层兜底，因此必须
//! 由 variant 表显式给出）——中性填充 `bg-muted` + `border-border` 发丝描边，
//! 在任意表面（含 dark 主题低对比表面）保有按钮可辨识度。`primary` 是显式
//! 醒目档（主题色填充），留给 CTA/主操作；`submit` 是行为语义，视觉跟随
//! primary。`secondary` 是深一档的纯填充 chip、无边框（"有边框"归 outline
//! 专属，四档中性梯子 default/secondary/outline/ghost 两两可辨）。
//!
//! 消费方：VM 解释器臂（aura_view_builder::convert_button）与 Rust transpile
//! 臂（ui_gen::rust button 代码生成）直接调用本表；Vue 臂的 variants.ts cva
//! 是生成到前端工程的文本，无法共享 Rust 代码，由 ui_gen::rust 的
//! `variants_cva_parity_tests` 与本表互锁——三表改任一须同步。
//! 规约见 docs/design/autoui/base-styles-and-visual-parity.md §3。

/// `button` 的 variant preset 类（按 `variant` prop 取键）。
/// 返回空串 = 无 preset（chromeless，由 user class 主导）。
pub fn button_variant_preset(variant: &str) -> &'static str {
    match variant {
        // PLAN-571: 缺省/default = UA 预填等价基线（中性填充 + 发丝描边 + 圆角）。
        "" | "default" => {
            "bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70"
        }
        // 显式醒目档：主题色填充，留给 CTA/主操作；submit 是行为语义，视觉跟随。
        "primary" | "submit" => {
            "bg-primary text-primary-foreground font-medium rounded-md hover:bg-primary/90"
        }
        // 深一档纯填充 chip、无边框（"有边框"归 outline 专属）。
        "secondary" => {
            "bg-secondary text-secondary-foreground font-medium rounded-md hover:bg-secondary/80"
        }
        "destructive" => {
            "bg-destructive text-destructive-foreground font-medium rounded-md hover:bg-destructive/90"
        }
        "outline" => {
            "border border-input bg-background text-foreground rounded-md hover:bg-secondary hover:text-secondary-foreground"
        }
        "ghost" => "rounded-md hover:bg-secondary hover:text-secondary-foreground",
        // Plan 414 R13: icon button - chromeless SQUARE (w follows h)。
        "icon" => "h-7 w-7 px-0 py-0",
        "link" => "text-primary",
        // "text" 及未知 variant：无 preset — chromeless（由 user class 主导）。
        _ => "",
    }
}

/// `button` 的 size preset 类（按 `size` prop 取键）。缺省 `h-10 px-4`。
pub fn button_size_preset(size: &str) -> &'static str {
    match size {
        "sm" => "h-9 px-3",
        "lg" => "h-11 px-8",
        "icon" => "h-10 w-10",
        _ => "h-10 px-4", // default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── PLAN-571 核心：default 一等化 ─────────────────────────────

    #[test]
    fn default_is_neutral_baseline_with_hairline_border() {
        for v in ["", "default"] {
            let p = button_variant_preset(v);
            assert!(p.contains("bg-muted"), "default({v:?}) 用 muted 中性填充: {p}");
            assert!(p.contains("border"), "default({v:?}) 必须有发丝描边(UA 等价): {p}");
            assert!(p.contains("rounded-md"), "default({v:?}) 圆角: {p}");
            assert!(!p.contains("bg-primary"), "default({v:?}) 不得再用主题色填充: {p}");
            assert!(!p.contains("bg-secondary"), "default({v:?}) 不借 secondary 皮: {p}");
        }
    }

    #[test]
    fn primary_is_explicit_accent_fill_without_border() {
        for v in ["primary", "submit"] {
            let p = button_variant_preset(v);
            assert!(p.contains("bg-primary"), "{v:?} 主题色填充: {p}");
            assert!(p.contains("text-primary-foreground"), "{v:?} 反色前景: {p}");
            assert!(p.contains("hover:bg-primary/90"), "{v:?} hover 压暗: {p}");
            assert!(!p.contains("border"), "{v:?} 填充档无边框: {p}");
        }
    }

    #[test]
    fn secondary_is_deeper_fill_without_border() {
        let p = button_variant_preset("secondary");
        assert!(p.contains("bg-secondary"), "secondary 纯填充: {p}");
        assert!(p.contains("text-secondary-foreground"), "secondary 前景: {p}");
        // 无边框——"有边框"归 outline 专属，default 的 border 不得出现。
        assert!(!p.contains("border "), "secondary 无边框（outline 专属）: {p}");
    }

    #[test]
    fn existing_variants_unchanged() {
        assert_eq!(
            button_variant_preset("destructive"),
            "bg-destructive text-destructive-foreground font-medium rounded-md hover:bg-destructive/90"
        );
        assert_eq!(
            button_variant_preset("outline"),
            "border border-input bg-background text-foreground rounded-md hover:bg-secondary hover:text-secondary-foreground"
        );
        assert_eq!(
            button_variant_preset("ghost"),
            "rounded-md hover:bg-secondary hover:text-secondary-foreground"
        );
        assert_eq!(button_variant_preset("icon"), "h-7 w-7 px-0 py-0");
        assert_eq!(button_variant_preset("link"), "text-primary");
    }

    #[test]
    fn text_and_unknown_are_chromeless() {
        for v in ["text", "nonsense"] {
            assert_eq!(button_variant_preset(v), "", "{v:?} 无 preset（chromeless）");
        }
    }

    // ── size preset（与旧 VM 臂行为逐字对齐）─────────────────────

    #[test]
    fn size_presets_match_legacy_table() {
        assert_eq!(button_size_preset("sm"), "h-9 px-3");
        assert_eq!(button_size_preset("lg"), "h-11 px-8");
        assert_eq!(button_size_preset("icon"), "h-10 w-10");
        assert_eq!(button_size_preset(""), "h-10 px-4");
        assert_eq!(button_size_preset("nonsense"), "h-10 px-4");
    }

    // ── 三表互锁锚点（ui_gen::rust 的 cva parity 测试对齐此串）───

    #[test]
    fn interlock_default_recipe_anchor() {
        assert_eq!(
            button_variant_preset("default"),
            "bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70"
        );
    }

    #[test]
    fn interlock_primary_recipe_anchor() {
        assert_eq!(
            button_variant_preset("primary"),
            "bg-primary text-primary-foreground font-medium rounded-md hover:bg-primary/90"
        );
    }
}
