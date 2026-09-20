//! Button variant/size preset table — 单一事实源（PLAN-571）。
//!
//! 设计语义：PLAN-080（2026-09-20 用户裁定，桌面实机验收）起，**未指定
//! variant/size 的裸 button 一律 chromeless**——对齐 vue 版 Tailwind
//! preflight 基线（浏览器 UA 对 `<button>` 的 preflight 重置：透明底、无
//! 边框、内容贴边），外观只来自显式 variant 或显式 class；三端
//! （VM 解释器臂 / Rust transpile 臂 / Vue cva）缺省同步为无预设。
//! 此前 PLAN-571 把缺省当 `default`（UA 预填等价：muted 填充 + 发丝描边）
//! ——任何带自定义类（VM 不识别的非 Tailwind 词表类被丢弃）的按钮在 VM
//! 漏出该预设（紫灰底 + h-10 px-4），web 侧却可被自定义 CSS 全量覆盖，
//! 双轨观感分叉（musk 收缩 rail 图标钮实锚）。`default` 键保留为**显式**
//! UA 基线档：写 `variant="default"` 才得到 muted 填充。`primary` 是显式
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
        // PLAN-080: 缺省（无 variant prop）= preflight 等价 chromeless——
        // 无底色/无边框/无预设，外观完全归显式类。原 PLAN-571 缺省=default
        // 的行为改为显式 "default" 档保留。
        "" => "",
        // PLAN-571: 显式 default = UA 预填等价基线（中性填充 + 发丝描边 + 圆角）。
        "default" => {
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

/// `button` 的 size preset 类（按 `size` prop 取键）。
/// PLAN-080: 缺省（无 size prop）= 无预设（内容贴边，preflight 等价）——
/// 显式 `"default"` 才得 h-10 px-4；未知键不再回落 default。
pub fn button_size_preset(size: &str) -> &'static str {
    match size {
        "sm" => "h-9 px-3",
        "lg" => "h-11 px-8",
        "icon" => "h-10 w-10",
        "default" => "h-10 px-4",
        _ => "", // 未指定/未知：无预设
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── PLAN-080 核心：缺省 chromeless（preflight 等价） ────────────────

    #[test]
    fn absent_variant_is_chromeless_preflight_equivalent() {
        // 无 variant prop = 透明底/无边框/无预设——外观只来自显式类。
        // 双轨同源：Vue cva defaultVariants 同步移除（ui_gen/vue.rs 模板）。
        assert_eq!(button_variant_preset(""), "");
    }

    #[test]
    fn absent_size_is_chromeless_content_hug() {
        assert_eq!(button_size_preset(""), "");
        assert_eq!(button_size_preset("nonsense"), "");
        // 显式 "default" 才得 h-10 px-4。
        assert_eq!(button_size_preset("default"), "h-10 px-4");
    }

    // ── PLAN-571 显式档（显式 "default" = UA 等价基线） ─────────────────

    #[test]
    fn explicit_default_is_neutral_baseline_with_hairline_border() {
        let p = button_variant_preset("default");
        assert!(p.contains("bg-muted"), "default 用 muted 中性填充: {p}");
        assert!(p.contains("border"), "default 必须有发丝描边(UA 等价): {p}");
        assert!(p.contains("rounded-md"), "default 圆角: {p}");
        assert!(!p.contains("bg-primary"), "default 不得再用主题色填充: {p}");
        assert!(!p.contains("bg-secondary"), "default 不借 secondary 皮: {p}");
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
        for v in ["", "text", "nonsense"] {
            assert_eq!(button_variant_preset(v), "", "{v:?} 无 preset（chromeless）");
        }
    }

    // ── size preset（与旧 VM 臂行为逐字对齐）─────────────────────

    #[test]
    fn size_presets_match_table() {
        assert_eq!(button_size_preset("sm"), "h-9 px-3");
        assert_eq!(button_size_preset("lg"), "h-11 px-8");
        assert_eq!(button_size_preset("icon"), "h-10 w-10");
        assert_eq!(button_size_preset("default"), "h-10 px-4");
        // PLAN-080：缺省/未知 = 无预设（原 h-10 px-4 缺省退役）。
        assert_eq!(button_size_preset(""), "");
        assert_eq!(button_size_preset("nonsense"), "");
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
