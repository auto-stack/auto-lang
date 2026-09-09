// design_tokens/registry.rs —— 语义 token 值的单一事实源（PLAN-593，
// Design 29 Phase 1）。S1 对账（见 plan 复审记录）确立的三套现行色板在此
// 各持一份、逐字迁移（零漂移金样/期望表钉死，tests: plan593_theme_registry_tests
// + auto-man plan593_index_css_golden_tests）：
//
//   zinc     —— ui_gen::vue::generate_base_css 原手写模板（E2，仅测试消费的
//               遗留面；Phase 2 裁定退役或对齐 scaffold）。
//   scaffold —— auto-man::vue::generate_index_css 原手写模板（E3，真实 Vue
//               脚手架路径：shadcn zinc + 烤入 indigo primary + sidebar 族；
//               dark card 10%/muted 15% 与 zinc 分叉为现状，归一属 Phase 2）。
//   stella   —— VM/Iced 臂 resolve_semantic_rgb 原 match 臂（E1，Plan 518
//               暖纸 light/蓝黑 dark 结构化 RGB；primary 槽保持运行时 accent
//               驱动——accent 表同在本文件单源）。
//
// Phase 1 形态约定：CSS 面持逐字块（含排版注释——结构化渲染需排版元数据，
// 复杂度不成比例，待 Phase 2 theme.at 解析落地时自然结构化）；VM 面持结构化
// RGB 表。此后任何视觉校准 = 改本文件一处。

/// 语义 token 封闭词表（PLAN-593：shadcn 全集 19 键 + sidebar 族 8 键 +
/// AutoUI 扩展 4 键 = 31 键）。未知名一律编译期不可表示（枚举闭集）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenName {
    // ── shadcn 基础 19 ──
    Background, Foreground,
    Card, CardForeground, Popover, PopoverForeground,
    Primary, PrimaryForeground, Secondary, SecondaryForeground,
    Muted, MutedForeground, Accent, AccentForeground,
    Destructive, DestructiveForeground, Border, Input, Ring,
    // ── sidebar 族 8（scaffold 消费）──
    SidebarBackground, SidebarForeground, SidebarPrimary, SidebarPrimaryForeground,
    SidebarAccent, SidebarAccentForeground, SidebarBorder, SidebarRing,
    // ── AutoUI 扩展 4（模式不变功能色）──
    Success, Warning, Info, Error,
}

impl TokenName {
    /// CSS 变量名（`--card-foreground` 形态）。
    pub fn css_var(self) -> &'static str {
        match self {
            TokenName::Background => "background",
            TokenName::Foreground => "foreground",
            TokenName::Card => "card",
            TokenName::CardForeground => "card-foreground",
            TokenName::Popover => "popover",
            TokenName::PopoverForeground => "popover-foreground",
            TokenName::Primary => "primary",
            TokenName::PrimaryForeground => "primary-foreground",
            TokenName::Secondary => "secondary",
            TokenName::SecondaryForeground => "secondary-foreground",
            TokenName::Muted => "muted",
            TokenName::MutedForeground => "muted-foreground",
            TokenName::Accent => "accent",
            TokenName::AccentForeground => "accent-foreground",
            TokenName::Destructive => "destructive",
            TokenName::DestructiveForeground => "destructive-foreground",
            TokenName::Border => "border",
            TokenName::Input => "input",
            TokenName::Ring => "ring",
            TokenName::SidebarBackground => "sidebar-background",
            TokenName::SidebarForeground => "sidebar-foreground",
            TokenName::SidebarPrimary => "sidebar-primary",
            TokenName::SidebarPrimaryForeground => "sidebar-primary-foreground",
            TokenName::SidebarAccent => "sidebar-accent",
            TokenName::SidebarAccentForeground => "sidebar-accent-foreground",
            TokenName::SidebarBorder => "sidebar-border",
            TokenName::SidebarRing => "sidebar-ring",
            TokenName::Success => "success",
            TokenName::Warning => "warning",
            TokenName::Info => "info",
            TokenName::Error => "error",
        }
    }
}

/// accent 预设表（E4/E5/E6 三处置收敛后的单源：vue 脚手架 TS
/// ACCENT_PALETTES 值互锁、VM resolve_semantic_rgb Primary 臂、code_editor
/// 编辑器主题派生）。dark 提亮差值不在表内——vue TS 臂 +4 / Rust 臂 +10
/// 为既有双端分叉（S1 对账②，Phase 2 归一裁定）。
pub fn accent_hsl(name: &str) -> Option<(u16, u8, u8)> {
    match name {
        "indigo" => Some((239, 84, 67)),
        // Plan 503: coral 校准至 stella-os 玫瑰粉 light #c4706a = hsl(4,43%,59%)。
        "coral"  => Some((4, 43, 59)),
        "ocean"  => Some((217, 91, 60)),
        "sage"   => Some((160, 84, 39)),
        "amber"  => Some((38, 92, 50)),
        _ => None,
    }
}

/// 未知 accent 名的回退基值（= indigo）——消费方回退臂引用此常量，
/// 函数体零字面色值（PLAN-593 R2）。
pub const ACCENT_DEFAULT: (u16, u8, u8) = (239, 84, 67);

/// accent 预设名全表（theme::ACCENT_PRESETS 的值源）。
pub const ACCENT_NAMES: [&str; 5] = ["indigo", "coral", "ocean", "sage", "amber"];

// ── CSS 面（V1 消费：ui_gen / auto-man 模板插入块）────────────────────

/// 一个主题一个 mode 的 CSS 变量块（逐字持有：含排版注释/空行）。
pub struct CssPalette {
    /// 核心色变量块（background..ring，含注释排版）。
    pub core: &'static str,
    /// 后置色块（scaffold 的 sidebar 族；无则空串）。
    pub extra: &'static str,
}

pub struct CssTheme {
    pub name: &'static str,
    pub light: CssPalette,
    pub dark: CssPalette,
}

/// zinc（E2 原 ui_gen 手写模板逐字迁移）。
pub const ZINC: CssTheme = CssTheme {
    name: "zinc",
    light: CssPalette { core: ZINC_LIGHT, extra: "" },
    dark: CssPalette { core: ZINC_DARK, extra: "" },
};

/// scaffold（E3 原 auto-man 手写模板逐字迁移——真实 Vue 脚手架路径）。
pub const SCAFFOLD: CssTheme = CssTheme {
    name: "scaffold",
    light: CssPalette { core: SCAFFOLD_LIGHT, extra: SCAFFOLD_LIGHT_SIDEBAR },
    dark: CssPalette { core: SCAFFOLD_DARK, extra: SCAFFOLD_DARK_SIDEBAR },
};

pub fn css_builtin(name: &str) -> Option<&'static CssTheme> {
    match name {
        "zinc" => Some(&ZINC),
        "scaffold" => Some(&SCAFFOLD),
        _ => None,
    }
}

const ZINC_LIGHT: &str = r##"    --background: 0 0% 100%;
    --foreground: 222.2 84% 4.9%;
    --card: 0 0% 100%;
    --card-foreground: 222.2 84% 4.9%;
    --popover: 0 0% 100%;
    --popover-foreground: 222.2 84% 4.9%;
    --primary: 222.2 47.4% 11.2%;
    --primary-foreground: 210 40% 98%;
    /* PLAN-571: secondary 与 muted 分档（≠210 40% 96.1% 暖纸 muted）——暖灰一档深 #e3ddd1，
       与 Rust 侧 theme.rs Color::Secondary 互锁（改任一须同步）。 */
    --secondary: 40 24% 85.5%;
    --secondary-foreground: 222.2 47.4% 11.2%;
    --muted: 210 40% 96.1%;
    --muted-foreground: 215.4 16.3% 46.9%;
    --accent: 210 40% 96.1%;
    --accent-foreground: 222.2 47.4% 11.2%;
    --destructive: 0 84.2% 60.2%;
    --destructive-foreground: 210 40% 98%;
    --border: 214.3 31.8% 91.4%;
    --input: 214.3 31.8% 91.4%;
    --ring: 222.2 84% 4.9%;
"##;
const ZINC_DARK: &str = r##"    --background: 222.2 47% 7%;
    --foreground: 210 40% 98%;
    --card: 222.2 47% 11%;
    --card-foreground: 210 40% 98%;
    --popover: 222.2 47% 11%;
    --popover-foreground: 210 40% 98%;
    --primary: 210 40% 98%;
    --primary-foreground: 222.2 47.4% 11.2%;
    /* PLAN-571: secondary 分档——slate-700 #334155（muted 保持 217.2 32.6% 17.5% 不动）。 */
    --secondary: 215 25% 27%;
    --secondary-foreground: 210 40% 98%;
    --muted: 217.2 32.6% 17.5%;
    --muted-foreground: 215 20.2% 65.1%;
    --accent: 217.2 32.6% 17.5%;
    --accent-foreground: 210 40% 98%;
    --destructive: 0 62.8% 30.6%;
    --destructive-foreground: 210 40% 98%;
    --border: 217.2 32.6% 17.5%;
    --input: 217.2 32.6% 17.5%;
    --ring: 212.7 26.8% 83.9%;
"##;
const SCAFFOLD_LIGHT: &str = r##"    --background: 0 0% 100%;
    --foreground: 222.2 84% 4.9%;

    --card: 0 0% 100%;
    --card-foreground: 222.2 84% 4.9%;

    --popover: 0 0% 100%;
    --popover-foreground: 222.2 84% 4.9%;

    --primary: 239 84% 67%;
    --primary-foreground: 210 40% 98%;

    /* PLAN-571: secondary 与 muted 分档（≠--muted 210 40% 96.1%）——暖灰一档深 #e3ddd1，
       与 theme.rs / ui_gen 互锁（40 24% 85.5% ≈ #e3ddd1）。 */
    --secondary: 40 24% 85.5%;
    --secondary-foreground: 222.2 47.4% 11.2%;

    --muted: 210 40% 96.1%;
    --muted-foreground: 215.4 16.3% 46.9%;

    --accent: 210 40% 96.1%;
    --accent-foreground: 222.2 47.4% 11.2%;

    --destructive: 0 84.2% 60.2%;
    --destructive-foreground: 210 40% 98%;

    --border: 214.3 31.8% 91.4%;
    --input: 214.3 31.8% 91.4%;
    --ring: 239 84% 67%;
"##;
const SCAFFOLD_LIGHT_SIDEBAR: &str = r##"    --sidebar-background: 0 0% 98%;
    --sidebar-foreground: 222.2 47.4% 11.2%;
    --sidebar-primary: 239 84% 67%;
    --sidebar-primary-foreground: 210 40% 98%;
    --sidebar-accent: 210 40% 96.1%;
    --sidebar-accent-foreground: 222.2 47.4% 11.2%;
    --sidebar-border: 214.3 31.8% 91.4%;
    --sidebar-ring: 239 84% 67%;
"##;
const SCAFFOLD_DARK: &str = r##"    --background: 222.2 47% 7%;
    --foreground: 210 40% 98%;

    --card: 222.2 47% 10%;
    --card-foreground: 210 40% 98%;

    --popover: 222.2 47% 10%;
    --popover-foreground: 210 40% 98%;

    --primary: 239 84% 77%;
    --primary-foreground: 222.2 47.4% 11.2%;

    /* PLAN-571: secondary 分档——slate-700 #334155（--muted 保持 217.2 32.6% 17.5%）。 */
    --secondary: 215 25% 27%;
    --secondary-foreground: 210 40% 98%;

    --muted: 217.2 32.6% 15%;
    --muted-foreground: 215 20.2% 65.1%;

    --accent: 217.2 32.6% 17.5%;
    --accent-foreground: 210 40% 98%;

    --destructive: 0 62.8% 30.6%;
    --destructive-foreground: 210 40% 98%;

    --border: 217.2 32.6% 17.5%;
    --input: 217.2 32.6% 17.5%;
    --ring: 239 84% 77%;
"##;
const SCAFFOLD_DARK_SIDEBAR: &str = r##"    --sidebar-background: 222.2 47% 10%;
    --sidebar-foreground: 210 40% 98%;
    --sidebar-primary: 239 84% 77%;
    --sidebar-primary-foreground: 222.2 47.4% 11.2%;
    --sidebar-accent: 217.2 32.6% 17.5%;
    --sidebar-accent-foreground: 210 40% 98%;
    --sidebar-border: 217.2 32.6% 17.5%;
    --sidebar-ring: 239 84% 77%;
"##;

// ── VM 面（V2 消费：resolve_semantic_rgb 查表）───────────────────────

/// stella 结构化表（E1 原 match 臂 RGB 逐值迁移；只含 VM 投影实际消费的
/// 键——Primary 运行时 accent 驱动、popover/ring 等无 VM 消费面故缺）。
pub struct RgbTheme {
    pub name: &'static str,
    pub light: &'static [(TokenName, (u8, u8, u8))],
    pub dark: &'static [(TokenName, (u8, u8, u8))],
}

pub const STELLA: RgbTheme = RgbTheme {
    name: "stella",
    light: &[
        (TokenName::Background, (245, 241, 232)),          // #f5f1e8 暖纸
        (TokenName::Card, (251, 248, 242)),                // #fbf8f2 卡片微浮
        (TokenName::Secondary, (227, 221, 209)),           // #e3ddd1 暖灰一档深
        (TokenName::Muted, (240, 235, 226)),               // #f0ebe2 暖 muted
        (TokenName::Foreground, (42, 39, 35)),             // 墨色 #2a2723
        (TokenName::MutedForeground, (125, 119, 109)),     // 暖次级 #7d776d
        (TokenName::Border, (227, 221, 209)),              // 暖灰
        (TokenName::PrimaryForeground, (248, 250, 252)),   // 白字
        (TokenName::SecondaryForeground, (42, 39, 35)),
        (TokenName::DestructiveForeground, (248, 250, 252)),
        (TokenName::Success, (34, 197, 94)),
        (TokenName::Warning, (234, 179, 8)),
        (TokenName::Info, (59, 130, 246)),
        (TokenName::Error, (239, 68, 68)),
    ],
    dark: &[
        (TokenName::Background, (20, 26, 41)),             // #141a29 深蓝黑
        (TokenName::Card, (26, 34, 53)),                   // #1a2235 面板
        (TokenName::Secondary, (51, 65, 85)),              // #334155 slate-700
        (TokenName::Muted, (30, 41, 59)),                  // slate-800
        (TokenName::Foreground, (248, 250, 252)),
        (TokenName::MutedForeground, (151, 163, 181)),
        (TokenName::Border, (40, 49, 70)),                 // #283146
        (TokenName::PrimaryForeground, (15, 23, 42)),      // #0f172a 黑字
        (TokenName::SecondaryForeground, (248, 250, 252)),
        (TokenName::DestructiveForeground, (248, 250, 252)),
        (TokenName::Success, (34, 197, 94)),
        (TokenName::Warning, (234, 179, 8)),
        (TokenName::Info, (59, 130, 246)),
        (TokenName::Error, (239, 68, 68)),
    ],
};

pub fn rgb_builtin(name: &str) -> Option<&'static RgbTheme> {
    match name {
        "stella" => Some(&STELLA),
        _ => None,
    }
}

/// 查主题（按 mode 取表）。VM 语义色解析入口（resolve_semantic_rgb 委托）。
pub fn resolve_rgb(theme: &RgbTheme, token: TokenName, is_dark: bool) -> Option<(u8, u8, u8)> {
    let palette = if is_dark { theme.dark } else { theme.light };
    palette
        .iter()
        .find(|(t, _)| *t == token)
        .map(|(_, rgb)| *rgb)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 块完整性指纹（迁移装配正确性的第一道闸——逐字节级由 S5 金样钉）。
    #[test]
    fn css_blocks_fingerprint() {
        assert!(ZINC.light.core.contains("--background: 0 0% 100%;"));
        assert!(ZINC.dark.core.contains("--primary: 210 40% 98%;"));
        assert!(ZINC.light.extra.is_empty() && ZINC.dark.extra.is_empty());
        assert!(SCAFFOLD.light.core.contains("--primary: 239 84% 67%;"));
        assert!(SCAFFOLD.dark.core.contains("--muted: 217.2 32.6% 15%;"));
        assert!(SCAFFOLD.light.extra.contains("--sidebar-background: 0 0% 98%;"));
        assert!(SCAFFOLD.dark.extra.contains("--sidebar-ring: 239 84% 77%;"));
        for (name, t) in [("zinc", &ZINC), ("scaffold", &SCAFFOLD)] {
            for (mode, p) in [("light", &t.light), ("dark", &t.dark)] {
                assert!(p.core.ends_with('\n'), "{name}.{mode}.core 末尾换行");
            }
        }
    }

    /// accent 表与 STELLA 表基本指纹。
    #[test]
    fn tables_fingerprint() {
        assert_eq!(accent_hsl("indigo"), Some((239, 84, 67)));
        assert_eq!(accent_hsl("nonsense"), None);
        assert_eq!(ACCENT_NAMES.len(), 5);
        assert_eq!(
            resolve_rgb(&STELLA, TokenName::Background, false),
            Some((245, 241, 232))
        );
        assert_eq!(
            resolve_rgb(&STELLA, TokenName::Background, true),
            Some((20, 26, 41))
        );
        assert_eq!(resolve_rgb(&STELLA, TokenName::Primary, false), None, "primary=accent 驱动,不在静态表");
    }
}
