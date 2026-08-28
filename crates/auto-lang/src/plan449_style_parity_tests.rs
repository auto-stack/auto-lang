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
        ("border-r", "gap", "单侧边框无解析臂——iced 边框均匀,降级项"),
        ("border-border", "ok", "边框语义色"),
        ("gap-3", "ok", "12px"),
        ("px-3", "ok", "12px"),
        ("pb-3", "ok", ""),
        ("pt-0", "ok", ""),
        ("h-full", "ok", ""),
        ("flex", "ok", ""),
        ("items-baseline", "gap", "items 仅 center/start/end/stretch——baseline 丢弃"),
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
        ("px-2.5", "gap", "p/m 族分数值无 0.5 步进臂(gap 有)——D3 候选"),
        ("py-2.5", "gap", "同 px-2.5"),
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
        ("underline", "gap", "text-decoration 无臂——VM 无法表达下划线"),
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
        ("tracking-wide", "gap", "letter-spacing 无臂"),
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
