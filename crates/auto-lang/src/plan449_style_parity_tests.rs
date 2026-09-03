//! Plan 049 (auto-musk) T1: class.rs 支持度探针 — 样式迁移映射草案权的逐类断言。
//!
//! auto-musk PLAN-049 把 web 专用 CSS（inject_styles.ts + 组件 style{} 块）迁移为
//! .at 内联 tailwind 工具类（单一样式源），VM 轨由本 crate 的 class.rs 解释同一
//! 类串。迁移前用本探针逐类断言映射草案里每个工具类 token 的解析行为：
//!
//! - `ok`      —— parse_single 必须成功（映射草案可用；断言失败 = class.rs 回归
//!                或草案写错，先修再迁）。
//! - `variant` —— 变体/装饰 token，VM 已知丢弃（hover:/focus:/transition/whitelist）。
//!                断言 = base 部分解析进 hover_classes（Style::parse 语义）或整体
//!                被跳过，绝不混进 classes。
//! - `gap`     —— 当前解析失败，登记为 D3 缺口（auto-lang 补解析臂）；一旦上游
//!                补齐，本断言翻转提醒 musk 侧更新 MIGRATION.md 支持度列。
//!
//! 手动门（跨仓 sibling 布局假设，与 plan442_musk_probe 同款）：
//!   cargo test -p auto-lang --lib --features ui-iced style_migration_probe -- --nocapture
//! auto-musk 侧留档：scripts/lib-parity/style-parity/t1-class-probe.txt

#[cfg(test)]
mod plan449_style_migration_probe {
    use crate::ui::style::Style;

    /// (token, verdict, note) — note 面向 MIGRATION.md 支持度列。
    const PROBE: &[(&str, &str, &str)] = &[
        // ── 048 导航栏试点（app.at 现行类串）──
        ("h-screen", "ok", "高度=视口"),
        ("w-full", "ok", "宽 100%"),
        ("w-48", "ok", "192px"),
        ("shrink-0", "ok", "不收缩"),
        ("bg-card", "ok", "语义卡色"),
        ("bg-secondary", "ok", "语义次面色"),
        ("border-r", "ok", "PLAN-050 T4 已补单侧边框臂(border-b/r/t/l,iced 1px 填充条降级)"),
        ("border-border", "ok", "边框语义色"),
        ("gap-3", "ok", "12px"),
        ("px-3", "ok", "12px"),
        ("pb-3", "ok", ""),
        ("pt-0", "ok", ""),
        ("h-full", "ok", ""),
        ("flex", "ok", ""),
        ("items-baseline", "ok", "D3 已补降级臂(ItemsStart,iced 无基线对齐)"),
        ("gap-2", "ok", ""),
        ("px-0", "ok", ""),
        ("text-base", "ok", "16px"),
        ("font-bold", "ok", ""),
        ("text-primary", "ok", "语义主色"),
        ("text-xs", "ok", "12px"),
        ("text-sm", "ok", "14px"),
        ("text-muted-foreground", "ok", "语义弱化前景"),
        ("rounded-md", "ok", "6px"),
        ("text-left", "ok", ""),
        ("bg-primary/10", "ok", "alpha 语法(主题暗感知)"),
        ("font-medium", "ok", ""),
        ("hover:bg-accent", "variant", "hover 进 hover_classes,iced 按钮消费"),
        ("mt-auto", "ok", ""),
        ("items-center", "ok", ""),
        ("justify-between", "ok", ""),
        ("gap-1.5", "ok", "分数 gap=Pixels(6)"),
        ("px-1", "ok", ""),
        ("flex-1", "ok", ""),
        ("min-h-0", "ok", "MinHeight(0)"),
        ("min-w-0", "ok", ""),
        ("overflow-hidden", "ok", ""),
        // ── 登录页（login.at 现行类串,T5 对拍集）──
        ("min-h-screen", "ok", "f32::MAX 语义"),
        ("justify-center", "ok", ""),
        ("bg-background", "ok", "语义背景"),
        ("max-w-sm", "ok", "384px"),
        ("p-8", "ok", ""),
        ("border", "ok", "默认边框"),
        ("rounded-xl", "ok", "12px"),
        ("shadow-lg", "ok", "L3 阴影"),
        ("gap-6", "ok", ""),
        ("gap-4", "ok", ""),
        ("gap-1", "ok", ""),
        ("mb-2", "ok", ""),
        ("text-2xl", "ok", "24px"),
        ("text-foreground", "ok", ""),
        ("font-semibold", "ok", ""),
        ("px-2.5", "ok", "D3 已补分数步进臂 Pixels(10)"),
        ("py-2.5", "ok", "D3 已补分数步进臂"),
        ("py-2", "ok", ""),
        ("focus:outline-none", "variant", "focus: 非响应式前缀,整体跳过"),
        ("focus:ring-2", "variant", "focus: 跳过"),
        ("focus:ring-ring", "variant", "focus: 跳过"),
        ("px-4", "ok", ""),
        ("bg-primary", "ok", ""),
        ("text-primary-foreground", "ok", ""),
        ("hover:opacity-90", "variant", "hover+opacity 装饰"),
        ("transition-opacity", "variant", "仅 transition-colors 有臂,其余跳过"),
        ("mt-1", "ok", ""),
        ("text-center", "ok", ""),
        ("bg-transparent", "ok", ""),
        ("border-none", "ok", ""),
        // Plan 518 注:underline 臂上游已补(class.rs Underline/NoUnderline),
        // 探针翻 ok——master 预存红,与 518 的 token 重校无关,顺带收口。
        ("underline", "ok", "text-decoration 臂已补(Underline/NoUnderline)"),
        ("cursor-pointer", "ok", ""),
        ("hover:opacity-80", "variant", ""),
        ("bg-destructive/10", "ok", "alpha 语法"),
        ("text-destructive", "ok", ""),
        // ── 后续切片草案 token（T6-T8 映射草案权）──
        ("w-[220px]", "ok", "arbitrary 像素"),
        ("w-[200px]", "ok", ""),
        ("w-[240px]", "ok", ""),
        ("flex-col", "ok", ""),
        ("justify-start", "ok", ""),
        ("justify-end", "ok", ""),
        ("justify-center", "ok", ""),
        ("overflow-y-auto", "ok", ""),
        ("whitespace-nowrap", "ok", ""),
        ("truncate", "ok", ""),
        ("leading-[1.6]", "ok", "arbitrary 行高"),
        ("font-mono", "ok", ""),
        ("break-words", "ok", ""),
        ("rounded", "ok", "4px"),
        ("rounded-full", "ok", ""),
        ("rounded-lg", "ok", ""),
        ("max-h-[300px]", "ok", "arbitrary"),
        ("max-w-[320px]", "ok", ""),
        ("opacity-60", "ok", ""),
        ("relative", "ok", ""),
        ("absolute", "ok", "VM 降级(无绝对定位)"),
        ("z-10", "ok", ""),
        ("z-[100]", "gap", "z>50 拒绝且 arbitrary 不走 pixel 臂——草案避用"),
        ("hidden", "ok", ""),
        ("inline-flex", "ok", ""),
        ("select-none", "gap", "user-select 无臂"),
        ("uppercase", "gap", "text-transform 无臂"),
        ("capitalize", "gap", "text-transform 无臂"),
        ("-ml-2", "ok", "负 margin"),
        ("w-px", "ok", "1px"),
        ("h-px", "ok", "1px"),
        ("inset-0", "ok", "VM 降级"),
        ("grid", "ok", "VM 降级单行"),
        ("grid-cols-2", "ok", "VM 降级"),
        ("items-stretch", "ok", ""),
        ("self-end", "ok", "VM 降级到容器 items"),
        ("gap-x-2", "ok", ""),
        ("outline-none", "ok", ""),
        ("transition-colors", "ok", "有臂(iced 消费)"),
        ("animate-pulse", "gap", "动画无臂——web-only"),
        ("resize-y", "gap", "textarea resize 无臂"),
        ("appearance-none", "gap", "无臂"),
        ("italic", "gap", "无臂"),
        // Plan 527 T5:tracking 全档补臂(gap → ok,支持度列同步)
        ("tracking-wide", "ok", "Plan 527 T5 tracking 全档(iced 无 letter_spacing,渲染分期)"),
    ];

    #[test]
    fn style_migration_probe() {
        let mut ok = 0usize;
        let mut variant = 0usize;
        let mut gap = 0usize;
        for (token, verdict, note) in PROBE {
            let s = Style::parse(token).unwrap();
            match *verdict {
                "ok" => {
                    assert_eq!(
                        s.classes.len(),
                        1,
                        "token `{token}` 标注 ok 但解析 classes={:?}（class.rs 回归或草案写错）",
                        s.classes
                    );
                    ok += 1;
                }
                "variant" => {
                    // hover: 且内层可解析 → hover_classes；其余（focus:/transition-*）
                    // 整体跳过。两种都不得混进 classes。
                    assert!(
                        s.classes.is_empty(),
                        "token `{token}` 标注 variant 却进了 classes={:?}",
                        s.classes
                    );
                    variant += 1;
                }
                "gap" => {
                    assert!(
                        s.classes.is_empty() && s.hover_classes.is_empty(),
                        "token `{token}` 标注 gap 但已能解析（上游已补臂?）——更新 MIGRATION.md 支持度列并翻 ok",
                    );
                    gap += 1;
                }
                other => panic!("未知 verdict: {other}"),
            }
            println!("[style-parity-probe] {verdict}\t{token}\t{note}");
        }
        println!(
            "[style-parity-probe] summary: total={} ok={ok} variant={variant} gap={gap}",
            PROBE.len()
        );
    }
}
use crate::ui::style::{Color, SizeValue, StyleClass};
use serde_json::{json, Map, Value};
use std::path::PathBuf;

/// PLAN-049 (auto-musk) T2: style-parity VM 侧 dump —— 读 musk 夹具 cases.json,
/// 逐 token 走 class.rs 解析,输出属性表 JSON 行（run.mjs 抓取后与 web 侧
/// tailwind 生成 CSS 的静态规则表逐属性 diff）。
///
/// cases.json 定位：env STYLE_PARITY_CASES 优先（run.mjs 传入自身仓路径,不依赖
/// sibling 布局）；缺省回退 sibling auto-musk 主检出。找不到 → SKIP 不判失败
/// （普通 cargo test 无夹具也能跑）。
///
/// 属性表口径（与 run.mjs norm 对齐）：
/// - 长度一律 "Npx"（Tailwind 1 unit=4px;Full=100%,Auto=auto,分数=百分比）。
/// - 颜色一律 "color(<tailwind 色名>@<alpha 0-1>)" —— 双轨以类串里的色名为
///   公共键（VM 语义变体 muted/card 同落 Surface 的合并是 theme.rs 值域问题,
///   进 tokenValues 报告,不进 diff）；"_rgb_" 前缀键为解析 RGB 报告值,diff 忽略。
/// - 布局布尔用 CSS 属性名 + tailwind 语义值（flex-direction:column 等）。
/// - truncate 按语义展开三属性,对齐 web 生成规则。
/// - 阴影/过渡/旋转等 VM 降级项落 "_" 前缀报告键,diff 忽略（web-only 增强）。
#[cfg(test)]
mod plan449_style_parity_dump {
    use super::*;

    fn locate_cases() -> Option<PathBuf> {
        if let Ok(p) = std::env::var("STYLE_PARITY_CASES") {
            let p = PathBuf::from(p);
            if p.exists() {
                return Some(p);
            }
        }
        let rel = "../../../auto-musk/scripts/lib-parity/style-parity/fixtures/cases.json";
        let candidates = [
            std::env::var("CARGO_MANIFEST_DIR")
                .ok()
                .map(|d| PathBuf::from(d).join(rel)),
            Some(PathBuf::from(rel)),
        ];
        candidates.into_iter().flatten().find(|p| p.exists())
    }

    pub(super) fn trim_f32(p: f32) -> String {
        let s = format!("{p}");
        if s.ends_with(".0") {
            s[..s.len() - 2].to_string()
        } else {
            s
        }
    }

    fn px_of(v: &SizeValue) -> String {
        match v {
            SizeValue::Full => "100%".into(),
            SizeValue::Half => "50%".into(),
            SizeValue::Third => "33.333%".into(),
            SizeValue::TwoThirds => "66.666%".into(),
            SizeValue::Quarter => "25%".into(),
            SizeValue::ThreeQuarters => "75%".into(),
            SizeValue::Auto => "auto".into(),
            SizeValue::Fixed(u) => format!("{}px", u * 4),
            SizeValue::Pixels(p) => format!("{}px", trim_f32(*p)),
            // Plan 527 T3: 通用分数按百分比分(Fill-ratio 近似)
            SizeValue::Fraction(n, m) => format!("{}%", n * 100 / m),
        }
    }

    fn fmt_alpha(a: f32) -> String {
        trim_f32(a)
    }

    fn neg_px(v: &SizeValue) -> String {
        format!("-{}", px_of(v))
    }

    /// min-h-screen 在 class.rs 里是 f32::MAX 哨兵——对拍口径折回 web 的 100vh。
    fn vh_or_px(px: f32) -> String {
        if px > 1.0e9 {
            "100vh".into()
        } else {
            format!("{}px", trim_f32(px))
        }
    }

    /// 颜色类 → (色名, alpha, 报告 RGB)。色名取自类串 base（bg-primary/10 →
    /// "primary"），双轨 diff 的公共键;RGB 为 theme.rs 语义解析报告值。
    fn color_entry(base_with_alpha: &str) -> Option<(String, f32, Option<(u8, u8, u8, u8)>)> {
        let (base, alpha) = match base_with_alpha.split_once('/') {
            Some((b, a)) => (b, a.parse::<f32>().unwrap_or(100.0) / 100.0),
            None => (base_with_alpha, 1.0),
        };
        let col = Color::from_tailwind(base).ok()?;
        let (r, g, b) = col.to_rgb8();
        let rgb = crate::ui::style::theme::resolve_semantic_rgb(&col)
            .map(|(r, g, b)| (r, g, b, (alpha * 255.0) as u8))
            .or(Some((r, g, b, (alpha * 255.0) as u8)));
        Some((base.to_string(), alpha, rgb))
    }

    /// 单个 StyleClass → 属性表。未覆盖变体给 {"ir": "<Debug>"}（run.mjs 记
    /// vm-unmapped 进报告）。
    fn class_props(c: &StyleClass, raw: &str) -> Vec<(String, Value)> {
        use StyleClass::*;
        let mut out: Vec<(String, Value)> = Vec::new();
        let color_key = |prop: &str, base_with_alpha: &str| -> Vec<(String, Value)> {
            match color_entry(base_with_alpha) {
                Some((name, a, rgb)) => {
                    let mut v = vec![(
                        prop.to_string(),
                        json!(format!("color({}@{})", name, fmt_alpha(a))),
                    )];
                    if let Some((r, g, b, a8)) = rgb {
                        v.push(("_rgb_".to_string() + prop, json!(format!("{r},{g},{b},{a8}"))));
                    }
                    v
                }
                None => vec![(prop.to_string(), json!(format!("unresolved({base_with_alpha})")))],
            }
        };
        let color_base = |prefixes: &[&str]| -> Option<String> {
            for p in prefixes {
                if let Some(rest) = raw.strip_prefix(p) {
                    return Some(rest.to_string());
                }
            }
            None
        };
        match c {
            Padding(v) => out.push(("padding".into(), json!(px_of(v)))),
            PaddingX(v) => {
                out.push(("padding-left".into(), json!(px_of(v))));
                out.push(("padding-right".into(), json!(px_of(v))));
            }
            PaddingY(v) => {
                out.push(("padding-top".into(), json!(px_of(v))));
                out.push(("padding-bottom".into(), json!(px_of(v))));
            }
            PaddingTop(v) => out.push(("padding-top".into(), json!(px_of(v)))),
            PaddingBottom(v) => out.push(("padding-bottom".into(), json!(px_of(v)))),
            PaddingLeft(v) => out.push(("padding-left".into(), json!(px_of(v)))),
            PaddingRight(v) => out.push(("padding-right".into(), json!(px_of(v)))),
            Margin(v) => out.push(("margin".into(), json!(px_of(v)))),
            NegativeMargin(v) => out.push(("margin".into(), json!(neg_px(v)))),
            NegativeMarginX(v) => {
                out.push(("margin-left".into(), json!(neg_px(v))));
                out.push(("margin-right".into(), json!(neg_px(v))));
            }
            NegativeMarginY(v) => {
                out.push(("margin-top".into(), json!(neg_px(v))));
                out.push(("margin-bottom".into(), json!(neg_px(v))));
            }
            NegativeMarginTop(v) => out.push(("margin-top".into(), json!(neg_px(v)))),
            NegativeMarginBottom(v) => out.push(("margin-bottom".into(), json!(neg_px(v)))),
            NegativeMarginLeft(v) => out.push(("margin-left".into(), json!(neg_px(v)))),
            NegativeMarginRight(v) => out.push(("margin-right".into(), json!(neg_px(v)))),
            MarginX(v) => {
                out.push(("margin-left".into(), json!(px_of(v))));
                out.push(("margin-right".into(), json!(px_of(v))));
            }
            MarginY(v) => {
                out.push(("margin-top".into(), json!(px_of(v))));
                out.push(("margin-bottom".into(), json!(px_of(v))));
            }
            MarginTop(v) => out.push(("margin-top".into(), json!(px_of(v)))),
            MarginBottom(v) => out.push(("margin-bottom".into(), json!(px_of(v)))),
            MarginLeft(v) => out.push(("margin-left".into(), json!(px_of(v)))),
            MarginRight(v) => out.push(("margin-right".into(), json!(px_of(v)))),
            MarginLeftAuto => out.push(("margin-left".into(), json!("auto"))),
            MarginRightAuto => out.push(("margin-right".into(), json!("auto"))),
            MarginXAuto => {
                out.push(("margin-left".into(), json!("auto")));
                out.push(("margin-right".into(), json!("auto")));
            }
            Gap(v) => out.push(("gap".into(), json!(px_of(v)))),
            GapX(v) => out.push(("column-gap".into(), json!(px_of(v)))),
            GapY(v) => out.push(("row-gap".into(), json!(px_of(v)))),
            SpaceX(v) => out.push(("column-gap".into(), json!(px_of(v)))),
            SpaceY(v) => out.push(("row-gap".into(), json!(px_of(v)))),
            BackgroundColor(_) => {
                if let Some(base) = color_base(&["bg-"]) {
                    out.extend(color_key("background-color", &base));
                }
            }
            TextColor(_) => {
                if let Some(base) = color_base(&["text-"]) {
                    out.extend(color_key("color", &base));
                }
            }
            BorderColor(_) => {
                if let Some(base) = color_base(&["border-"]) {
                    out.extend(color_key("border-color", &base));
                }
            }
            AccentColor(_) => {
                if let Some(base) = color_base(&["accent-"]) {
                    out.extend(color_key("accent-color", &base));
                }
            }
            Flex => out.push(("display".into(), json!("flex"))),
            Block => out.push(("display".into(), json!("block"))),
            Inline => out.push(("display".into(), json!("inline"))),
            InlineBlock => out.push(("display".into(), json!("inline-block"))),
            InlineFlex => out.push(("display".into(), json!("inline-flex"))),
            Grid => out.push(("display".into(), json!("grid"))),
            Flex1 => out.push(("flex".into(), json!("1"))),
            FlexAuto => out.push(("flex".into(), json!("auto"))),
            FlexInitial => out.push(("flex".into(), json!("initial"))),
            FlexNone => out.push(("flex".into(), json!("none"))),
            Grow => out.push(("flex-grow".into(), json!("1"))),
            Grow0 => out.push(("flex-grow".into(), json!("0"))),
            Shrink => out.push(("flex-shrink".into(), json!("1"))),
            Shrink0 => out.push(("flex-shrink".into(), json!("0"))),
            FlexRow => out.push(("flex-direction".into(), json!("row"))),
            FlexCol => out.push(("flex-direction".into(), json!("column"))),
            FlexRowReverse => out.push(("flex-direction".into(), json!("row-reverse"))),
            FlexColReverse => out.push(("flex-direction".into(), json!("column-reverse"))),
            FlexWrap => out.push(("flex-wrap".into(), json!("wrap"))),
            FlexWrapReverse => out.push(("flex-wrap".into(), json!("wrap-reverse"))),
            FlexNowrap => out.push(("flex-wrap".into(), json!("nowrap"))),
            ItemsCenter => out.push(("align-items".into(), json!("center"))),
            ItemsStart => out.push(("align-items".into(), json!("flex-start"))),
            ItemsEnd => out.push(("align-items".into(), json!("flex-end"))),
            ItemsStretch => out.push(("align-items".into(), json!("stretch"))),
            JustifyCenter => out.push(("justify-content".into(), json!("center"))),
            JustifyStart => out.push(("justify-content".into(), json!("flex-start"))),
            JustifyEnd => out.push(("justify-content".into(), json!("flex-end"))),
            JustifyBetween => out.push(("justify-content".into(), json!("space-between"))),
            JustifyAround => out.push(("justify-content".into(), json!("space-around"))),
            JustifyEvenly => out.push(("justify-content".into(), json!("space-evenly"))),
            SelfStart => out.push(("align-self".into(), json!("flex-start"))),
            SelfCenter => out.push(("align-self".into(), json!("center"))),
            SelfEnd => out.push(("align-self".into(), json!("flex-end"))),
            SelfStretch => out.push(("align-self".into(), json!("stretch"))),
            Width(v) => out.push(("width".into(), json!(px_of(v)))),
            Height(v) => out.push(("height".into(), json!(px_of(v)))),
            MinWidth(px) => out.push(("min-width".into(), json!(vh_or_px(*px)))),
            MinHeight(px) => out.push(("min-height".into(), json!(vh_or_px(*px)))),
            MaxWidth(px) => out.push(("max-width".into(), json!(format!("{}px", trim_f32(*px))))),
            MaxHeight(px) => out.push(("max-height".into(), json!(format!("{}px", trim_f32(*px))))),
            Rounded => out.push(("border-radius".into(), json!("4px"))),
            RoundedNone => out.push(("border-radius".into(), json!("0px"))),
            RoundedSm => out.push(("border-radius".into(), json!("2px"))),
            RoundedMd => out.push(("border-radius".into(), json!("6px"))),
            RoundedLg => out.push(("border-radius".into(), json!("8px"))),
            RoundedXl => out.push(("border-radius".into(), json!("12px"))),
            Rounded2Xl => out.push(("border-radius".into(), json!("16px"))),
            Rounded3Xl => out.push(("border-radius".into(), json!("24px"))),
            RoundedFull => out.push(("border-radius".into(), json!("9999px"))),
            RoundedT(sz) => {
                let px = sz.map(|x| x.to_pixels()).unwrap_or(4.0);
                out.push(("border-top-left-radius".into(), json!(format!("{}px", trim_f32(px)))));
                out.push(("border-top-right-radius".into(), json!(format!("{}px", trim_f32(px)))));
            }
            RoundedB(sz) => {
                let px = sz.map(|x| x.to_pixels()).unwrap_or(4.0);
                out.push(("border-bottom-left-radius".into(), json!(format!("{}px", trim_f32(px)))));
                out.push(("border-bottom-right-radius".into(), json!(format!("{}px", trim_f32(px)))));
            }
            RoundedL(sz) => {
                let px = sz.map(|x| x.to_pixels()).unwrap_or(4.0);
                out.push(("border-top-left-radius".into(), json!(format!("{}px", trim_f32(px)))));
                out.push(("border-bottom-left-radius".into(), json!(format!("{}px", trim_f32(px)))));
            }
            RoundedR(sz) => {
                let px = sz.map(|x| x.to_pixels()).unwrap_or(4.0);
                out.push(("border-top-right-radius".into(), json!(format!("{}px", trim_f32(px)))));
                out.push(("border-bottom-right-radius".into(), json!(format!("{}px", trim_f32(px)))));
            }
            RoundedTL(sz) => out.push(("border-top-left-radius".into(), json!(format!("{}px", trim_f32(sz.map(|x| x.to_pixels()).unwrap_or(4.0)))))),
            RoundedTR(sz) => out.push(("border-top-right-radius".into(), json!(format!("{}px", trim_f32(sz.map(|x| x.to_pixels()).unwrap_or(4.0)))))),
            RoundedBL(sz) => out.push(("border-bottom-left-radius".into(), json!(format!("{}px", trim_f32(sz.map(|x| x.to_pixels()).unwrap_or(4.0)))))),
            RoundedBR(sz) => out.push(("border-bottom-right-radius".into(), json!(format!("{}px", trim_f32(sz.map(|x| x.to_pixels()).unwrap_or(4.0)))))),
            Border => out.push(("border-width".into(), json!("1px"))),
            Border0 => out.push(("border-width".into(), json!("0px"))),
            BorderWidth(w) => out.push(("border-width".into(), json!(format!("{}px", trim_f32(*w))))),
            TextXs => out.push(("font-size".into(), json!("12px"))),
            TextSm => out.push(("font-size".into(), json!("14px"))),
            TextBase => out.push(("font-size".into(), json!("16px"))),
            TextLg => out.push(("font-size".into(), json!("18px"))),
            TextXl => out.push(("font-size".into(), json!("20px"))),
            Text2Xl => out.push(("font-size".into(), json!("24px"))),
            Text3Xl => out.push(("font-size".into(), json!("30px"))),
            Text4Xl => out.push(("font-size".into(), json!("36px"))),
            Text5Xl => out.push(("font-size".into(), json!("48px"))),
            Text6Xl => out.push(("font-size".into(), json!("60px"))),
            Text7Xl => out.push(("font-size".into(), json!("72px"))),
            Text8Xl => out.push(("font-size".into(), json!("96px"))),
            Text9Xl => out.push(("font-size".into(), json!("128px"))),
            TextArbitrary(px) => out.push(("font-size".into(), json!(format!("{}px", trim_f32(*px))))),
            FontBold => out.push(("font-weight".into(), json!("700"))),
            FontMedium => out.push(("font-weight".into(), json!("500"))),
            FontNormal => out.push(("font-weight".into(), json!("400"))),
            FontLight => out.push(("font-weight".into(), json!("300"))),
            FontExtraLight => out.push(("font-weight".into(), json!("200"))),
            FontSemiBold => out.push(("font-weight".into(), json!("600"))),
            FontSerif => out.push(("font-family".into(), json!("serif"))),
            FontSans => out.push(("font-family".into(), json!("sans"))),
            FontMono => out.push(("font-family".into(), json!("mono"))),
            TextCenter => out.push(("text-align".into(), json!("center"))),
            TextLeft => out.push(("text-align".into(), json!("left"))),
            TextRight => out.push(("text-align".into(), json!("right"))),
            LineHeight(lh) => out.push(("line-height".into(), json!(trim_f32(*lh)))),
            LineHeightNone => out.push(("line-height".into(), json!("1"))),
            WhitespaceNowrap => out.push(("white-space".into(), json!("nowrap"))),
            BreakWords => out.push(("overflow-wrap".into(), json!("break-word"))),
            CursorPointer => out.push(("cursor".into(), json!("pointer"))),
            OutlineNone => out.push(("outline".into(), json!("none"))),
            BorderNone => out.push(("border-style".into(), json!("none"))),
            ListNone => out.push(("list-style".into(), json!("none"))),
            Antialiased => out.push(("_antialiased".into(), json!("1"))),
            Truncate => {
                out.push(("overflow".into(), json!("hidden")));
                out.push(("text-overflow".into(), json!("ellipsis")));
                out.push(("white-space".into(), json!("nowrap")));
            }
            Opacity(v) => out.push(("opacity".into(), json!(fmt_alpha(*v as f32 / 100.0)))),
            Relative => out.push(("position".into(), json!("relative"))),
            Absolute => out.push(("position".into(), json!("absolute"))),
            Fixed => out.push(("position".into(), json!("fixed"))),
            Sticky => out.push(("position".into(), json!("sticky"))),
            ZIndex(z) => out.push(("z-index".into(), json!(z.to_string()))),
            TopOffset(px) => out.push(("top".into(), json!(format!("{}px", trim_f32(*px))))),
            BottomOffset(px) => out.push(("bottom".into(), json!(format!("{}px", trim_f32(*px))))),
            RightOffset(px) => out.push(("right".into(), json!(format!("{}px", trim_f32(*px))))),
            LeftOffset(px) => out.push(("left".into(), json!(format!("{}px", trim_f32(*px))))),
            Inset(px) => {
                for p in ["top", "right", "bottom", "left"] {
                    out.push((p.to_string(), json!(format!("{}px", trim_f32(*px)))));
                }
            }
            OverflowAuto => out.push(("overflow".into(), json!("auto"))),
            OverflowHidden => out.push(("overflow".into(), json!("hidden"))),
            OverflowVisible => out.push(("overflow".into(), json!("visible"))),
            OverflowScroll => out.push(("overflow".into(), json!("scroll"))),
            OverflowXAuto => out.push(("overflow-x".into(), json!("auto"))),
            OverflowYAuto => out.push(("overflow-y".into(), json!("auto"))),
            Hidden => out.push(("display".into(), json!("none"))),
            Shadow => out.push(("_shadow".into(), json!("shadow"))),
            ShadowSm => out.push(("_shadow".into(), json!("sm"))),
            ShadowMd => out.push(("_shadow".into(), json!("md"))),
            ShadowLg => out.push(("_shadow".into(), json!("lg"))),
            ShadowXl => out.push(("_shadow".into(), json!("xl"))),
            Shadow2Xl => out.push(("_shadow".into(), json!("2xl"))),
            ShadowNone => out.push(("box-shadow".into(), json!("none"))),
            TransitionColors => out.push(("_transition".into(), json!("colors"))),
            TransitionDuration(ms) => out.push(("_transition-duration".into(), json!(ms.to_string()))),
            Rotate(deg) => out.push(("_rotate".into(), json!(trim_f32(*deg)))),
            Order(n) => out.push(("order".into(), json!(n.to_string()))),
            CodeLang(_) => {} // 元数据类,非视觉
            other => {
                out.push(("ir".into(), json!(format!("{other:?}"))));
            }
        }
        out
    }

    fn dump_token(raw: &str) -> Value {
        if let Some(rest) = raw.strip_prefix("hover:") {
            let ok = StyleClass::parse_single(rest).is_ok();
            return json!({"raw": raw, "variant": "hover", "ok": ok, "props": {}});
        }
        match StyleClass::parse_single(raw) {
            Ok(c) => {
                let mut props = Map::new();
                for (k, v) in class_props(&c, raw) {
                    props.insert(k, v);
                }
                json!({"raw": raw, "ok": true, "props": props})
            }
            Err(e) => json!({"raw": raw, "ok": false, "err": e}),
        }
    }

    #[test]
    fn style_parity_dump() {
        let Some(path) = locate_cases() else {
            println!("[style-parity-dump] SKIPPED — cases.json not found (set STYLE_PARITY_CASES)");
            return;
        };
        let text = std::fs::read_to_string(&path).expect("read cases.json");
        let cases: Value = serde_json::from_str(&text).expect("parse cases.json");
        let empty = Vec::new();
        let list = cases.get("cases").and_then(|c| c.as_array()).unwrap_or(&empty);
        println!("[style-parity-dump] BEGIN {} cases from {}", list.len(), path.display());
        for case in list {
            let id = case.get("id").and_then(|v| v.as_str()).unwrap_or("?").replace('"', "'");
            let classes = case.get("classes").and_then(|v| v.as_str()).unwrap_or("");
            let tokens: Vec<Value> = classes.split_whitespace().map(dump_token).collect();
            println!(
                "[style-parity-dump] {}",
                json!({"case": id, "tokens": tokens})
            );
        }
        println!("[style-parity-dump] END");
    }
}
