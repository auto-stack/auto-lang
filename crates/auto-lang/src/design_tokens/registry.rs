// design_tokens/registry.rs —— 语义 token 值的单一事实源（PLAN-593 立项，
// PLAN-601 T-02 双面统一：Phase 1 的「CSS 逐字块 + VM RGB 表」两形态合并为
// 单一结构化 ThemeSpec——每主题一份表同时服务 CSS 面（canonical 渲染）与
// VM 面（RGB 求值），CSS 值与 RGB 值同源。
//
// 内置主题五套（值来源=Phase 1 迁移值 + auto CLI 两模板现值）：
//   zinc     —— ui_gen base_css 原值（E2 遗留面；PLAN-601 T-09 随 E2 退役后
//               仅存此表作为可切换内置主题）。
//   scaffold —— auto-man generate_index_css 原值（真实 Vue 脚手架路径）。
//   stella   —— VM resolve_semantic_rgb 原值（VM 轨默认）；14 键 Rgb 真值
//               原样（VM 投影零漂移），CSS 面缺键按补全约定派生（见
//               STELLA_PAIR/FILL 对应注释——popover←card 对、accent←muted 对、
//               input←border、primary/ring=indigo 基值〔运行时 accent 覆盖层
//               接管〕、destructive 族=zinc 常量）。
//   tauri / cli-vue —— auto CLI cmd_tauri/cmd_vue 原值（PLAN-601 T-02 收编
//               入表；生成器本体 T-09 改装配。cli-vue 双 sidebar 块按 CSS
//               级联取后者胜的生效值）。
//
// 金样口径（PLAN-601 §5.1）：CSS 产物金样从 byte-pinned 升级 value-pinned
// （变量名→值对等），canonical 布局归一属合法再生（593-R 轮「块=产物、
// 值=契约」先例）。

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
    // ── sidebar 族 8（scaffold/cli-vue 消费）──
    SidebarBackground, SidebarForeground, SidebarPrimary, SidebarPrimaryForeground,
    SidebarAccent, SidebarAccentForeground, SidebarBorder, SidebarRing,
    // ── AutoUI 扩展 4（模式不变功能色）──
    Success, Warning, Info, Error,
}

impl TokenName {
    /// CSS 变量名反查（封闭词表解析入口——theme{} 声明键校验消费）。
    pub fn from_css_var(name: &str) -> Option<TokenName> {
        Some(match name {
            "background" => TokenName::Background,
            "foreground" => TokenName::Foreground,
            "card" => TokenName::Card,
            "card-foreground" => TokenName::CardForeground,
            "popover" => TokenName::Popover,
            "popover-foreground" => TokenName::PopoverForeground,
            "primary" => TokenName::Primary,
            "primary-foreground" => TokenName::PrimaryForeground,
            "secondary" => TokenName::Secondary,
            "secondary-foreground" => TokenName::SecondaryForeground,
            "muted" => TokenName::Muted,
            "muted-foreground" => TokenName::MutedForeground,
            "accent" => TokenName::Accent,
            "accent-foreground" => TokenName::AccentForeground,
            "destructive" => TokenName::Destructive,
            "destructive-foreground" => TokenName::DestructiveForeground,
            "border" => TokenName::Border,
            "input" => TokenName::Input,
            "ring" => TokenName::Ring,
            "sidebar-background" => TokenName::SidebarBackground,
            "sidebar-foreground" => TokenName::SidebarForeground,
            "sidebar-primary" => TokenName::SidebarPrimary,
            "sidebar-primary-foreground" => TokenName::SidebarPrimaryForeground,
            "sidebar-accent" => TokenName::SidebarAccent,
            "sidebar-accent-foreground" => TokenName::SidebarAccentForeground,
            "sidebar-border" => TokenName::SidebarBorder,
            "sidebar-ring" => TokenName::SidebarRing,
            "success" => TokenName::Success,
            "warning" => TokenName::Warning,
            "info" => TokenName::Info,
            "error" => TokenName::Error,
            _ => return None,
        })
    }

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

/// accent 预设表（单源：vue 脚手架 TS ACCENT_PALETTES 值互锁、VM
/// resolve_semantic_rgb Primary 臂、code_editor 编辑器主题派生）。
/// PLAN-601 D2 裁定：dark 提亮双端统一 **+10**（T-07 落地 vue TS 侧）。
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

/// 未知 accent 名的回退基值（= indigo）——消费方回退臂引用此常量。
pub const ACCENT_DEFAULT: (u16, u8, u8) = (239, 84, 67);

/// accent 预设名全表（theme::ACCENT_PRESETS 的值源）。
pub const ACCENT_NAMES: [&str; 5] = ["indigo", "coral", "ocean", "sage", "amber"];

// ── 双面值表示 ────────────────────────────────────────────────────────

/// 一个 token 的值：HSL 串（shadcn 形态 "h s% l%"，原文保真）或 RGB 真值
/// （stella VM 投影 14 键）。CSS 面与 VM 面经同一份值派生（css_str/rgb）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorLit {
    Hsl(&'static str),
    Rgb(u8, u8, u8),
}

/// "h s% l%" → RGB（u8 截断；s/l 接受小数如 85.5）。
pub fn hsl_str_to_rgb(s: &str) -> Option<(u8, u8, u8)> {
    let mut it = s.split_whitespace();
    let h: f32 = it.next()?.parse().ok()?;
    let sat: f32 = it.next()?.trim_end_matches('%').parse().ok()?;
    let lig: f32 = it.next()?.trim_end_matches('%').parse().ok()?;
    if it.next().is_some() { return None; }
    let (r, g, b) = hsl_to_rgb_f(h as u16, sat, lig);
    Some((r, g, b))
}

fn hsl_to_rgb_f(h: u16, s: f32, l: f32) -> (u8, u8, u8) {
    let h = h as f32 / 360.0;
    let s = s / 100.0;
    let l = l / 100.0;
    if s == 0.0 {
        let v = (l * 255.0) as u8;
        return (v, v, v);
    }
    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    let hue = |mut t: f32| -> f32 {
        if t < 0.0 { t += 1.0; }
        if t > 1.0 { t -= 1.0; }
        if t < 1.0 / 6.0 { return p + (q - p) * 6.0 * t; }
        if t < 0.5 { return q; }
        if t < 2.0 / 3.0 { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
        p
    };
    let f2u = |v: f32| (v * 255.0) as u8;
    (f2u(hue(h + 1.0 / 3.0)), f2u(hue(h)), f2u(hue(h - 1.0 / 3.0)))
}

/// RGB → "h s% l%"（整数分量；stella CSS 面派生用）。
pub fn rgb_to_hsl_str(rgb: (u8, u8, u8)) -> String {
    let (r, g, b) = (rgb.0 as f32 / 255.0, rgb.1 as f32 / 255.0, rgb.2 as f32 / 255.0);
    let (mx, mn) = (r.max(g).max(b), r.min(g).min(b));
    let l = (mx + mn) / 2.0;
    if mx == mn { return format!("0 0% {}%", (l * 100.0).round() as i32); }
    let d = mx - mn;
    let s = if l > 0.5 { d / (2.0 - mx - mn) } else { d / (mx + mn) };
    let h = if mx == r {
        ((g - b) / d + if g < b { 6.0 } else { 0.0 })
    } else if mx == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    } / 6.0;
    format!("{} {}% {}%", (h * 360.0).round() as i32, (s * 100.0).round() as i32, (l * 100.0).round() as i32)
}

impl ColorLit {
    /// CSS 面：Hsl 原文；Rgb 派生 HSL 串。
    pub fn css_str(&self) -> String {
        match self {
            ColorLit::Hsl(s) => (*s).to_string(),
            ColorLit::Rgb(r, g, b) => rgb_to_hsl_str((*r, *g, *b)),
        }
    }
    /// VM 面：Rgb 原值；Hsl 解析转换。
    pub fn rgb(&self) -> (u8, u8, u8) {
        match self {
            ColorLit::Hsl(s) => hsl_str_to_rgb(s).unwrap_or((0, 0, 0)),
            ColorLit::Rgb(r, g, b) => (*r, *g, *b),
        }
    }
}

// ── 主题表（五套，值机械迁移自 PLAN-593 registry + auto CLI 两模板）───

/// 内置主题：light/dark 双 mode 成对表（同构于运行期 theme{} 声明合成的
/// ThemeSpec——T-03 落地后同一类型承载用户声明）。
pub struct ThemeSpec {
    pub name: &'static str,
    pub light: &'static [(TokenName, ColorLit)],
    pub dark: &'static [(TokenName, ColorLit)],
}

pub const ZINC: ThemeSpec = ThemeSpec {
    name: "zinc",
    light: &[
        (TokenName::Background, ColorLit::Hsl("0 0% 100%")),
        (TokenName::Foreground, ColorLit::Hsl("222.2 84% 4.9%")),
        (TokenName::Card, ColorLit::Hsl("0 0% 100%")),
        (TokenName::CardForeground, ColorLit::Hsl("222.2 84% 4.9%")),
        (TokenName::Popover, ColorLit::Hsl("0 0% 100%")),
        (TokenName::PopoverForeground, ColorLit::Hsl("222.2 84% 4.9%")),
        (TokenName::Primary, ColorLit::Hsl("222.2 47.4% 11.2%")),
        (TokenName::PrimaryForeground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::Secondary, ColorLit::Hsl("40 24% 85.5%")),
        (TokenName::SecondaryForeground, ColorLit::Hsl("222.2 47.4% 11.2%")),
        (TokenName::Muted, ColorLit::Hsl("210 40% 96.1%")),
        (TokenName::MutedForeground, ColorLit::Hsl("215.4 16.3% 46.9%")),
        (TokenName::Accent, ColorLit::Hsl("210 40% 96.1%")),
        (TokenName::AccentForeground, ColorLit::Hsl("222.2 47.4% 11.2%")),
        (TokenName::Destructive, ColorLit::Hsl("0 84.2% 60.2%")),
        (TokenName::DestructiveForeground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::Border, ColorLit::Hsl("214.3 31.8% 91.4%")),
        (TokenName::Input, ColorLit::Hsl("214.3 31.8% 91.4%")),
        (TokenName::Ring, ColorLit::Hsl("222.2 84% 4.9%")),
    ],
    dark: &[
        (TokenName::Background, ColorLit::Hsl("222.2 47% 7%")),
        (TokenName::Foreground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::Card, ColorLit::Hsl("222.2 47% 11%")),
        (TokenName::CardForeground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::Popover, ColorLit::Hsl("222.2 47% 11%")),
        (TokenName::PopoverForeground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::Primary, ColorLit::Hsl("210 40% 98%")),
        (TokenName::PrimaryForeground, ColorLit::Hsl("222.2 47.4% 11.2%")),
        (TokenName::Secondary, ColorLit::Hsl("215 25% 27%")),
        (TokenName::SecondaryForeground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::Muted, ColorLit::Hsl("217.2 32.6% 17.5%")),
        (TokenName::MutedForeground, ColorLit::Hsl("215 20.2% 65.1%")),
        (TokenName::Accent, ColorLit::Hsl("217.2 32.6% 17.5%")),
        (TokenName::AccentForeground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::Destructive, ColorLit::Hsl("0 62.8% 30.6%")),
        (TokenName::DestructiveForeground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::Border, ColorLit::Hsl("217.2 32.6% 17.5%")),
        (TokenName::Input, ColorLit::Hsl("217.2 32.6% 17.5%")),
        (TokenName::Ring, ColorLit::Hsl("212.7 26.8% 83.9%")),
    ],
};

pub const SCAFFOLD: ThemeSpec = ThemeSpec {
    name: "scaffold",
    light: &[
        (TokenName::Background, ColorLit::Hsl("0 0% 100%")),
        (TokenName::Foreground, ColorLit::Hsl("222.2 84% 4.9%")),
        (TokenName::Card, ColorLit::Hsl("0 0% 100%")),
        (TokenName::CardForeground, ColorLit::Hsl("222.2 84% 4.9%")),
        (TokenName::Popover, ColorLit::Hsl("0 0% 100%")),
        (TokenName::PopoverForeground, ColorLit::Hsl("222.2 84% 4.9%")),
        (TokenName::Primary, ColorLit::Hsl("239 84% 67%")),
        (TokenName::PrimaryForeground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::Secondary, ColorLit::Hsl("40 24% 85.5%")),
        (TokenName::SecondaryForeground, ColorLit::Hsl("222.2 47.4% 11.2%")),
        (TokenName::Muted, ColorLit::Hsl("210 40% 96.1%")),
        (TokenName::MutedForeground, ColorLit::Hsl("215.4 16.3% 46.9%")),
        (TokenName::Accent, ColorLit::Hsl("210 40% 96.1%")),
        (TokenName::AccentForeground, ColorLit::Hsl("222.2 47.4% 11.2%")),
        (TokenName::Destructive, ColorLit::Hsl("0 84.2% 60.2%")),
        (TokenName::DestructiveForeground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::Border, ColorLit::Hsl("214.3 31.8% 91.4%")),
        (TokenName::Input, ColorLit::Hsl("214.3 31.8% 91.4%")),
        (TokenName::Ring, ColorLit::Hsl("239 84% 67%")),
        (TokenName::SidebarBackground, ColorLit::Hsl("0 0% 98%")),
        (TokenName::SidebarForeground, ColorLit::Hsl("222.2 47.4% 11.2%")),
        (TokenName::SidebarPrimary, ColorLit::Hsl("239 84% 67%")),
        (TokenName::SidebarPrimaryForeground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::SidebarAccent, ColorLit::Hsl("210 40% 96.1%")),
        (TokenName::SidebarAccentForeground, ColorLit::Hsl("222.2 47.4% 11.2%")),
        (TokenName::SidebarBorder, ColorLit::Hsl("214.3 31.8% 91.4%")),
        (TokenName::SidebarRing, ColorLit::Hsl("239 84% 67%")),
    ],
    dark: &[
        (TokenName::Background, ColorLit::Hsl("222.2 47% 7%")),
        (TokenName::Foreground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::Card, ColorLit::Hsl("222.2 47% 10%")),
        (TokenName::CardForeground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::Popover, ColorLit::Hsl("222.2 47% 10%")),
        (TokenName::PopoverForeground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::Primary, ColorLit::Hsl("239 84% 77%")),
        (TokenName::PrimaryForeground, ColorLit::Hsl("222.2 47.4% 11.2%")),
        (TokenName::Secondary, ColorLit::Hsl("215 25% 27%")),
        (TokenName::SecondaryForeground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::Muted, ColorLit::Hsl("217.2 32.6% 15%")),
        (TokenName::MutedForeground, ColorLit::Hsl("215 20.2% 65.1%")),
        (TokenName::Accent, ColorLit::Hsl("217.2 32.6% 17.5%")),
        (TokenName::AccentForeground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::Destructive, ColorLit::Hsl("0 62.8% 30.6%")),
        (TokenName::DestructiveForeground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::Border, ColorLit::Hsl("217.2 32.6% 17.5%")),
        (TokenName::Input, ColorLit::Hsl("217.2 32.6% 17.5%")),
        (TokenName::Ring, ColorLit::Hsl("239 84% 77%")),
        (TokenName::SidebarBackground, ColorLit::Hsl("222.2 47% 10%")),
        (TokenName::SidebarForeground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::SidebarPrimary, ColorLit::Hsl("239 84% 77%")),
        (TokenName::SidebarPrimaryForeground, ColorLit::Hsl("222.2 47.4% 11.2%")),
        (TokenName::SidebarAccent, ColorLit::Hsl("217.2 32.6% 17.5%")),
        (TokenName::SidebarAccentForeground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::SidebarBorder, ColorLit::Hsl("217.2 32.6% 17.5%")),
        (TokenName::SidebarRing, ColorLit::Hsl("239 84% 77%")),
    ],
};

pub const STELLA: ThemeSpec = ThemeSpec {
    name: "stella",
    light: &[
        (TokenName::Background, ColorLit::Rgb(245, 241, 232)),
        (TokenName::Foreground, ColorLit::Rgb(42, 39, 35)),
        (TokenName::Card, ColorLit::Rgb(251, 248, 242)),
        (TokenName::CardForeground, ColorLit::Hsl("34 9% 15%")),
        (TokenName::Popover, ColorLit::Hsl("40 53% 97%")),
        (TokenName::PopoverForeground, ColorLit::Hsl("34 9% 15%")),
        (TokenName::Primary, ColorLit::Hsl("239 84% 67%")),
        (TokenName::PrimaryForeground, ColorLit::Rgb(248, 250, 252)),
        (TokenName::Secondary, ColorLit::Rgb(227, 221, 209)),
        (TokenName::SecondaryForeground, ColorLit::Rgb(42, 39, 35)),
        (TokenName::Muted, ColorLit::Rgb(240, 235, 226)),
        (TokenName::MutedForeground, ColorLit::Rgb(125, 119, 109)),
        (TokenName::Accent, ColorLit::Hsl("39 32% 91%")),
        (TokenName::AccentForeground, ColorLit::Hsl("38 7% 46%")),
        (TokenName::Destructive, ColorLit::Hsl("0 84.2% 60.2%")),
        (TokenName::DestructiveForeground, ColorLit::Rgb(248, 250, 252)),
        (TokenName::Border, ColorLit::Rgb(227, 221, 209)),
        (TokenName::Input, ColorLit::Hsl("40 24% 85%")),
        (TokenName::Ring, ColorLit::Hsl("239 84% 67%")),
        (TokenName::Success, ColorLit::Rgb(34, 197, 94)),
        (TokenName::Warning, ColorLit::Rgb(234, 179, 8)),
        (TokenName::Info, ColorLit::Rgb(59, 130, 246)),
        (TokenName::Error, ColorLit::Rgb(239, 68, 68)),
    ],
    dark: &[
        (TokenName::Background, ColorLit::Rgb(20, 26, 41)),
        (TokenName::Foreground, ColorLit::Rgb(248, 250, 252)),
        (TokenName::Card, ColorLit::Rgb(26, 34, 53)),
        (TokenName::CardForeground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::Popover, ColorLit::Hsl("222 34% 15%")),
        (TokenName::PopoverForeground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::Primary, ColorLit::Hsl("239 84% 77%")),
        (TokenName::PrimaryForeground, ColorLit::Rgb(15, 23, 42)),
        (TokenName::Secondary, ColorLit::Rgb(51, 65, 85)),
        (TokenName::SecondaryForeground, ColorLit::Rgb(248, 250, 252)),
        (TokenName::Muted, ColorLit::Rgb(30, 41, 59)),
        (TokenName::MutedForeground, ColorLit::Rgb(151, 163, 181)),
        (TokenName::Accent, ColorLit::Hsl("217 33% 17%")),
        (TokenName::AccentForeground, ColorLit::Hsl("216 17% 65%")),
        (TokenName::Destructive, ColorLit::Hsl("0 62.8% 30.6%")),
        (TokenName::DestructiveForeground, ColorLit::Rgb(248, 250, 252)),
        (TokenName::Border, ColorLit::Rgb(40, 49, 70)),
        (TokenName::Input, ColorLit::Hsl("222 27% 22%")),
        (TokenName::Ring, ColorLit::Hsl("239 84% 77%")),
        (TokenName::Success, ColorLit::Rgb(34, 197, 94)),
        (TokenName::Warning, ColorLit::Rgb(234, 179, 8)),
        (TokenName::Info, ColorLit::Rgb(59, 130, 246)),
        (TokenName::Error, ColorLit::Rgb(239, 68, 68)),
    ],
};

pub const TAURI: ThemeSpec = ThemeSpec {
    name: "tauri",
    light: &[
        (TokenName::Background, ColorLit::Hsl("0 0% 100%")),
        (TokenName::Foreground, ColorLit::Hsl("222.2 84% 4.9%")),
        (TokenName::Card, ColorLit::Hsl("0 0% 100%")),
        (TokenName::CardForeground, ColorLit::Hsl("222.2 84% 4.9%")),
        (TokenName::Popover, ColorLit::Hsl("0 0% 100%")),
        (TokenName::PopoverForeground, ColorLit::Hsl("222.2 84% 4.9%")),
        (TokenName::Primary, ColorLit::Hsl("222.2 47.4% 11.2%")),
        (TokenName::PrimaryForeground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::Secondary, ColorLit::Hsl("40 24% 85.5%")),
        (TokenName::SecondaryForeground, ColorLit::Hsl("222.2 47.4% 11.2%")),
        (TokenName::Muted, ColorLit::Hsl("210 40% 96.1%")),
        (TokenName::MutedForeground, ColorLit::Hsl("215.4 16.3% 46.9%")),
        (TokenName::Accent, ColorLit::Hsl("210 40% 96.1%")),
        (TokenName::AccentForeground, ColorLit::Hsl("222.2 47.4% 11.2%")),
        (TokenName::Destructive, ColorLit::Hsl("0 84.2% 60.2%")),
        (TokenName::DestructiveForeground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::Border, ColorLit::Hsl("214.3 31.8% 91.4%")),
        (TokenName::Input, ColorLit::Hsl("214.3 31.8% 91.4%")),
        (TokenName::Ring, ColorLit::Hsl("222.2 84% 4.9%")),
    ],
    dark: &[
        (TokenName::Background, ColorLit::Hsl("222.2 84% 4.9%")),
        (TokenName::Foreground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::Card, ColorLit::Hsl("222.2 84% 4.9%")),
        (TokenName::CardForeground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::Popover, ColorLit::Hsl("222.2 84% 4.9%")),
        (TokenName::PopoverForeground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::Primary, ColorLit::Hsl("210 40% 98%")),
        (TokenName::PrimaryForeground, ColorLit::Hsl("222.2 47.4% 11.2%")),
        (TokenName::Secondary, ColorLit::Hsl("215 25% 27%")),
        (TokenName::SecondaryForeground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::Muted, ColorLit::Hsl("217.2 32.6% 17.5%")),
        (TokenName::MutedForeground, ColorLit::Hsl("215 20.2% 65.1%")),
        (TokenName::Accent, ColorLit::Hsl("217.2 32.6% 17.5%")),
        (TokenName::AccentForeground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::Destructive, ColorLit::Hsl("0 62.8% 30.6%")),
        (TokenName::DestructiveForeground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::Border, ColorLit::Hsl("217.2 32.6% 17.5%")),
        (TokenName::Input, ColorLit::Hsl("217.2 32.6% 17.5%")),
        (TokenName::Ring, ColorLit::Hsl("212.7 26.8% 83.9%")),
    ],
};

pub const CLI_VUE: ThemeSpec = ThemeSpec {
    name: "cli_vue",
    light: &[
        (TokenName::Background, ColorLit::Hsl("0 0% 100%")),
        (TokenName::Foreground, ColorLit::Hsl("222.2 84% 4.9%")),
        (TokenName::Card, ColorLit::Hsl("0 0% 100%")),
        (TokenName::CardForeground, ColorLit::Hsl("222.2 84% 4.9%")),
        (TokenName::Popover, ColorLit::Hsl("0 0% 100%")),
        (TokenName::PopoverForeground, ColorLit::Hsl("222.2 84% 4.9%")),
        (TokenName::Primary, ColorLit::Hsl("239 84% 67%")),
        (TokenName::PrimaryForeground, ColorLit::Hsl("0 0% 100%")),
        (TokenName::Secondary, ColorLit::Hsl("40 24% 85.5%")),
        (TokenName::SecondaryForeground, ColorLit::Hsl("222.2 47.4% 11.2%")),
        (TokenName::Muted, ColorLit::Hsl("210 40% 96.1%")),
        (TokenName::MutedForeground, ColorLit::Hsl("215.4 16.3% 46.9%")),
        (TokenName::Accent, ColorLit::Hsl("210 40% 96.1%")),
        (TokenName::AccentForeground, ColorLit::Hsl("222.2 47.4% 11.2%")),
        (TokenName::Destructive, ColorLit::Hsl("0 84.2% 60.2%")),
        (TokenName::DestructiveForeground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::Border, ColorLit::Hsl("214.3 31.8% 91.4%")),
        (TokenName::Input, ColorLit::Hsl("214.3 31.8% 91.4%")),
        (TokenName::Ring, ColorLit::Hsl("239 84% 67%")),
        (TokenName::SidebarBackground, ColorLit::Hsl("0 0% 98%")),
        (TokenName::SidebarForeground, ColorLit::Hsl("240 5.3% 26.1%")),
        (TokenName::SidebarPrimary, ColorLit::Hsl("239 84% 67%")),
        (TokenName::SidebarPrimaryForeground, ColorLit::Hsl("0 0% 100%")),
        (TokenName::SidebarAccent, ColorLit::Hsl("240 4.8% 95.9%")),
        (TokenName::SidebarAccentForeground, ColorLit::Hsl("240 5.9% 10%")),
        (TokenName::SidebarBorder, ColorLit::Hsl("220 13% 91%")),
        (TokenName::SidebarRing, ColorLit::Hsl("239 84% 67%")),
    ],
    dark: &[
        (TokenName::Background, ColorLit::Hsl("222 47% 11%")),
        (TokenName::Foreground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::Card, ColorLit::Hsl("222 47% 13%")),
        (TokenName::CardForeground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::Popover, ColorLit::Hsl("222 47% 13%")),
        (TokenName::PopoverForeground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::Primary, ColorLit::Hsl("239 84% 77%")),
        (TokenName::PrimaryForeground, ColorLit::Hsl("222 47% 11%")),
        (TokenName::Secondary, ColorLit::Hsl("215 25% 27%")),
        (TokenName::SecondaryForeground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::Muted, ColorLit::Hsl("217 33% 17%")),
        (TokenName::MutedForeground, ColorLit::Hsl("215 20.2% 65.1%")),
        (TokenName::Accent, ColorLit::Hsl("217 33% 17%")),
        (TokenName::AccentForeground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::Destructive, ColorLit::Hsl("0 62.8% 30.6%")),
        (TokenName::DestructiveForeground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::Border, ColorLit::Hsl("217 33% 20%")),
        (TokenName::Input, ColorLit::Hsl("217 33% 20%")),
        (TokenName::Ring, ColorLit::Hsl("239 84% 77%")),
        (TokenName::SidebarBackground, ColorLit::Hsl("222 47% 9%")),
        (TokenName::SidebarForeground, ColorLit::Hsl("210 40% 90%")),
        (TokenName::SidebarPrimary, ColorLit::Hsl("239 84% 77%")),
        (TokenName::SidebarPrimaryForeground, ColorLit::Hsl("222 47% 11%")),
        (TokenName::SidebarAccent, ColorLit::Hsl("217 33% 15%")),
        (TokenName::SidebarAccentForeground, ColorLit::Hsl("210 40% 98%")),
        (TokenName::SidebarBorder, ColorLit::Hsl("217 33% 18%")),
        (TokenName::SidebarRing, ColorLit::Hsl("239 84% 77%")),
    ],
};

// ── 查询/渲染 API ─────────────────────────────────────────────────────

pub fn builtin(name: &str) -> Option<&'static ThemeSpec> {
    match name {
        "zinc" => Some(&ZINC),
        "scaffold" => Some(&SCAFFOLD),
        "stella" => Some(&STELLA),
        "tauri" => Some(&TAURI),
        "cli-vue" => Some(&CLI_VUE),
        _ => None,
    }
}

/// 内置主题名全表（settings 选择器/校验消费）。
pub const BUILTIN_NAMES: [&str; 5] = ["zinc", "scaffold", "stella", "tauri", "cli-vue"];

fn palette<'a>(theme: &'a ThemeSpec, is_dark: bool) -> &'a [(TokenName, ColorLit)] {
    if is_dark { theme.dark } else { theme.light }
}

/// VM 面：按 mode 取 token 值（RGB）。resolve_semantic_rgb 委托入口。
pub fn resolve_rgb(theme: &ThemeSpec, token: TokenName, is_dark: bool) -> Option<(u8, u8, u8)> {
    token_lit(theme, token, is_dark).map(|lit| lit.rgb())
}

/// 取 token 的原始值表示（CSS/VM 双面共用）。
pub fn token_lit(theme: &ThemeSpec, token: TokenName, is_dark: bool) -> Option<ColorLit> {
    palette(theme, is_dark)
        .iter()
        .find(|(t, _)| *t == token)
        .map(|(_, lit)| *lit)
}

/// CSS 面：核心 19 键 canonical 块（固定键序、4 空格缩进、每行
/// `--name: value;`）。消费方模板自持非色脚手架（--radius 等）。
pub fn render_core(theme: &ThemeSpec, is_dark: bool) -> String {
    render_tokens(theme, is_dark, &CORE_ORDER)
}

/// CSS 面：sidebar 8 键 canonical 块（无 sidebar 值的主题输出空串）。
pub fn render_sidebar(theme: &ThemeSpec, is_dark: bool) -> String {
    render_tokens(theme, is_dark, &SIDEBAR_ORDER)
}

pub const CORE_ORDER: [TokenName; 19] = [
    TokenName::Background, TokenName::Foreground,
    TokenName::Card, TokenName::CardForeground,
    TokenName::Popover, TokenName::PopoverForeground,
    TokenName::Primary, TokenName::PrimaryForeground,
    TokenName::Secondary, TokenName::SecondaryForeground,
    TokenName::Muted, TokenName::MutedForeground,
    TokenName::Accent, TokenName::AccentForeground,
    TokenName::Destructive, TokenName::DestructiveForeground,
    TokenName::Border, TokenName::Input, TokenName::Ring,
];
pub const SIDEBAR_ORDER: [TokenName; 8] = [
    TokenName::SidebarBackground, TokenName::SidebarForeground,
    TokenName::SidebarPrimary, TokenName::SidebarPrimaryForeground,
    TokenName::SidebarAccent, TokenName::SidebarAccentForeground,
    TokenName::SidebarBorder, TokenName::SidebarRing,
];

fn render_tokens(theme: &ThemeSpec, is_dark: bool, order: &[TokenName]) -> String {
    let pal = palette(theme, is_dark);
    let mut out = String::new();
    for t in order {
        if let Some((_, lit)) = pal.iter().find(|(k, _)| k == t) {
            out.push_str(&format!("    --{}: {};\n", t.css_var(), lit.css_str()));
        }
    }
    out
}

// ── JS 运行时值源（PLAN-601 T-06）─────────────────────────────────────
// vue 脚手架 applyTheme 的 THEME_PALETTES 嵌入体。与 render_core/
// render_sidebar 同取 token_lit 事实源（双面一致性测试守护），键为裸
// token 名（写入时由 JS 侧补 `--` 前缀）。

/// 单主题 light/dark 双 mode JS 对象字面量。
pub fn render_theme_pairs_js(theme: &ThemeSpec) -> String {
    let mode = |dark: bool| -> String {
        let pairs: Vec<String> = palette(theme, dark)
            .iter()
            .map(|(tok, lit)| format!("    '{}': '{}'", tok.css_var(), lit.css_str()))
            .collect();
        format!("{{\n{}\n  }}", pairs.join(",\n"))
    };
    format!("{{ light: {}, dark: {} }}", mode(false), mode(true))
}

/// 五内置主题全集 JS 对象（`{ zinc: {light,dark}, ... }`）。
pub fn render_theme_palettes_js() -> String {
    let entries: Vec<String> = BUILTIN_NAMES
        .iter()
        .map(|name| {
            let t = builtin(name).expect("BUILTIN_NAMES 与表互锁");
            format!("  '{name}': {}", render_theme_pairs_js(t))
        })
        .collect();
    format!("{{\n{}\n}}", entries.join(",\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 完备性：五主题 × 双 mode 核心 19 键齐全；sidebar 8 键仅
    /// scaffold/cli-vue 持有（AC-05）。
    #[test]
    fn themes_core_complete() {
        for name in BUILTIN_NAMES {
            let t = builtin(name).expect(name);
            for dark in [false, true] {
                let pal = palette(t, dark);
                for tok in CORE_ORDER {
                    assert!(
                        pal.iter().any(|(k, _)| *k == tok),
                        "{name} @dark={dark} 缺核心键 {:?}",
                        tok.css_var()
                    );
                }
            }
            let sb = t.light.iter().filter(|(k, _)| SIDEBAR_ORDER.contains(k)).count();
            let expect_sb = matches!(name, "scaffold" | "cli-vue");
            assert_eq!(sb == 8, expect_sb, "{name} sidebar 键集不符: {sb}");
        }
    }

    /// 双面一致性：每主题每键值可解析（Hsl→RGB 成功）；stella Rgb 键
    /// css_str→rgb 往返误差 ≤2/255。
    #[test]
    fn dual_face_consistency() {
        for name in BUILTIN_NAMES {
            let t = builtin(name).unwrap();
            for dark in [false, true] {
                for (tok, lit) in palette(t, dark) {
                    match lit {
                        ColorLit::Hsl(s) => {
                            assert!(hsl_str_to_rgb(s).is_some(), "{name} {:?} 无法解析: {s}", tok.css_var());
                        }
                        ColorLit::Rgb(r, g, b) => {
                            let back = hsl_str_to_rgb(&lit.css_str()).expect("roundtrip");
                            let want = [*r, *g, *b];
                            for (a, b) in [back.0, back.1, back.2].into_iter().zip(want) {
                                assert!((a as i32 - b as i32).abs() <= 4, "{name} {:?} 往返漂移", tok.css_var()); // 容差 ±4：h/s/l 三分量各自整数化（±0.5）+ 通道 u8 截断的累积舍入；
                                // s/l 互换类真 bug 偏差 ≥30，仍必被捕获。
                            }
                        }
                    }
                }
            }
        }
    }

    /// canonical 渲染指纹：zinc light 首行、scaffold sidebar、stella dark
    /// 派生值（AC-05 抽样）。
    #[test]
    fn render_fingerprint() {
        let z = builtin("zinc").unwrap();
        assert!(render_core(z, false).starts_with("    --background: 0 0% 100%;\n"));
        let s = builtin("scaffold").unwrap();
        assert!(render_core(s, false).contains("--primary: 239 84% 67%;"));
        assert!(render_sidebar(s, false).contains("--sidebar-background: 0 0% 98%;"));
        assert!(render_sidebar(z, false).is_empty());
        let st = builtin("stella").unwrap();
        // VM 投影零漂移指纹（plan593 期望表同值）
        assert_eq!(resolve_rgb(st, TokenName::Background, false), Some((245, 241, 232)));
        assert_eq!(resolve_rgb(st, TokenName::Background, true), Some((20, 26, 41)));
        // CLI 两表值指纹（T-09 装配前的值保真）
        assert!(render_core(builtin("tauri").unwrap(), false).contains("--primary: 222.2 47.4% 11.2%;"));
        assert!(render_core(builtin("cli-vue").unwrap(), false).contains("--ring: 239 84% 67%;"));
    }

    /// T-06：JS 值源发射——五主题全集可解析回每键值；zinc 无 sidebar 键、
    /// scaffold 有；stella Rgb 键以 css_str 形态出现（与 CSS 面同源）。
    #[test]
    fn theme_palettes_js_shape() {
        let js = render_theme_palettes_js();
        for name in BUILTIN_NAMES {
            assert!(js.contains(&format!("'{name}': {{")), "{name} 缺席");
        }
        assert!(js.contains("'background': '0 0% 100%'"), "zinc light 背景指纹");
        // zinc 无 sidebar、scaffold 有（与完备性测试同口径）
        let zinc = render_theme_pairs_js(builtin("zinc").unwrap());
        assert!(!zinc.contains("sidebar"));
        let scaffold = render_theme_pairs_js(builtin("scaffold").unwrap());
        assert!(scaffold.contains("'sidebar-background':"));
    }

    /// accent 表基本指纹（Phase 1 承袭）。
    #[test]
    fn accent_table_fingerprint() {
        assert_eq!(accent_hsl("indigo"), Some((239, 84, 67)));
        assert_eq!(accent_hsl("nonsense"), None);
        assert_eq!(ACCENT_NAMES.len(), 5);
        assert_eq!(ACCENT_DEFAULT, (239, 84, 67));
    }
}
