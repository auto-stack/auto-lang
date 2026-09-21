// Theme state + semantic color resolution — backend-neutral.
//
// Extracted from iced_adapter (Plan 413/418 follow-up): `style/class.rs`
// needs dark-mode-aware semantic alpha blending even when only the
// `code-editor` feature is on (no iced backend compiled), so the pure
// theme logic lives here with zero iced dependencies. iced_adapter
// re-exports everything for call-site compatibility.

use super::Color;

/// PLAN-593（Design 29 Phase 1）：语义 token 值单一事实源——zinc/scaffold
/// （CSS 面）与 stella（VM 面）三套色板 + accent 表 + 封闭词表。
/// 实体在无门的 `crate::design_tokens`（ui_gen 消费方不受 feature="ui" 门，
/// 见该模块头注）；此处 re-export 保持 `theme::registry` 路径稳定。
pub use crate::design_tokens::registry;

// PLAN-601 T-04/T-03：活动主题槽——命名主题（内置名或 theme{} 合成主题）
// 热切换。epoch 失效回路与 dark_mode 共用（THEME_EPOCH）；mode 仍由
// DARK_MODE 承载（主题对内 light/dark 选择器，`dark:` 门控经 dark_mode()
// 读值不变=零回归泛化）。
pub enum ActiveSpec {
    Builtin(&'static registry::ThemeSpec),
    Composed(std::sync::Arc<crate::design_tokens::decl::ComposedTheme>),
}

impl ActiveSpec {
    fn name(&self) -> String {
        match self {
            ActiveSpec::Builtin(t) => t.name.to_string(),
            ActiveSpec::Composed(t) => t.name.clone(),
        }
    }
    fn resolve(&self, token: registry::TokenName, is_dark: bool) -> Option<(u8, u8, u8)> {
        match self {
            ActiveSpec::Builtin(t) => registry::resolve_rgb(t, token, is_dark),
            ActiveSpec::Composed(t) => t.resolve_rgb(token, is_dark),
        }
    }
}

thread_local! {
    // PLAN-619 T-03（用户裁决）：缺省活动主题 = **scaffold**，与 auto-man 为
    // scaffold 类应用生成的 Vue `index.css`（shadcn 缺省 `.dark` 表）同源。
    // 此前缺省 "stella" 让任何**未声明 `theme{}`** 的应用在 VM 端系统性偏色
    // （同一语义 token 两端两套色板 = GOAL-007 跨端视觉一致的正面违例）。
    // 桌面宿主（stella 观感的唯一持有者）改为在 config 里显式声明
    // `theme_name: "stella"`——规则成文为「宿主 = stella、pac 应用 = scaffold」。
    static ACTIVE_THEME: std::cell::RefCell<ActiveSpec> =
        std::cell::RefCell::new(ActiveSpec::Builtin(
            registry::builtin("scaffold").expect("scaffold 恒在")
        ));
}

/// 当前活动主题名（内置名或合成主题名；缺省 "scaffold" = Vue 侧 scaffold
/// 色板同源，见 ACTIVE_THEME 注）。
pub fn theme_name() -> String {
    ACTIVE_THEME.with(|t| t.borrow().name())
}

/// PLAN-601 T-10：按语义 token 查活动主题双面值（内置表/合成体统一入口）。
/// code_editor 编辑器色域派生消费（bg/fg 从活动主题取，切主题即翻转）。
pub fn active_theme_rgb(token: registry::TokenName, is_dark: bool) -> Option<(u8, u8, u8)> {
    active_theme().resolve(token, is_dark)
}

/// PLAN-601 T-10：活动合成主题（editor syntax 系统首次构建时把它烘焙进
/// 主题集——合成主题 boot 后即固定，先于任何编辑器创建）。
pub fn active_composed() -> Option<std::sync::Arc<crate::design_tokens::decl::ComposedTheme>> {
    ACTIVE_THEME.with(|t| match &*t.borrow() {
        ActiveSpec::Composed(c) => Some(c.clone()),
        ActiveSpec::Builtin(_) => None,
    })
}

/// 切换活动主题为内置名（未知名返回 false 且零变化）。变化时
/// THEME_EPOCH 自增——既有失效回路（view 重建→重解析）随之生效。
pub fn set_theme(name: &str) -> bool {
    let Some(spec) = registry::builtin(name) else {
        return false;
    };
    set_active(ActiveSpec::Builtin(spec))
}

/// PLAN-601 T-03：应用 theme{} 合成主题（decl::compose 产物）。
/// 返回是否发生变化（同名同值不触发 epoch）。
pub fn set_theme_composed(theme: std::sync::Arc<crate::design_tokens::decl::ComposedTheme>) -> bool {
    set_active(ActiveSpec::Composed(theme))
}

fn set_active(spec: ActiveSpec) -> bool {
    let changed = ACTIVE_THEME.with(|t| {
        let mut t = t.borrow_mut();
        if t.name() != spec.name() {
            *t = spec;
            true
        } else {
            false
        }
    });
    if changed {
        THEME_EPOCH.with(|e| e.set(e.get().wrapping_add(1)));
    }
    changed
}

/// 活动主题 spec（解析入口共用）。
fn active_theme() -> ActiveSpec {
    ACTIVE_THEME.with(|t| match &*t.borrow() {
        ActiveSpec::Builtin(spec) => ActiveSpec::Builtin(spec),
        ActiveSpec::Composed(c) => ActiveSpec::Composed(c.clone()),
    })
}

// Plan 370 D-GAP-2/D-GAP-5: thread-local theme state for dark mode + accent.
// Set by the renderer before each render pass from VmBridge state.
// Plan 408: default to true because the iced window theme is hardcoded to
// Theme::Dark (renderer.rs ~line 4540). Apps that declare a `dark_mode`
// state var can override this; apps that don't (like widgets-gallery) get
// the correct dark palette by default, matching vue's <html class="dark">.
thread_local! {
    static DARK_MODE: std::cell::Cell<bool> = std::cell::Cell::new(true);
    static ACCENT_NAME: std::cell::RefCell<String> = std::cell::RefCell::new("indigo".to_string());
    /// PLAN-053 T12（051-候选修复，转介单①收回自修）：主题代数——
    /// `dark_mode` 每次值变化自增。内容寻址视图缓存
    /// （autodown_render::StreamCache 的结构键无主题维度）构建期记录
    /// 代数、与本值不符即全量重建——否则翻转帧 clone 旧块，fence 静态
    /// 档滞留构建时主题（首帧 D-GAP 同步前取档与翻转不重建双根因）。
    static THEME_EPOCH: std::cell::Cell<u32> = std::cell::Cell::new(0);
}

/// Set the global dark mode flag (called by renderer before rendering).
pub fn set_dark_mode(dark: bool) {
    let changed = DARK_MODE.with(|d| d.replace(dark)) != dark;
    if changed {
        THEME_EPOCH.with(|e| e.set(e.get().wrapping_add(1)));
    }
}

/// Read the global dark mode flag (Plan 413: code editor theme bridge).
pub fn dark_mode() -> bool {
    DARK_MODE.with(|d| d.get())
}

/// Current theme epoch: bump-on-change counter over `dark_mode` writes.
/// Content-keyed view caches compare their build epoch against this to
/// invalidate theme-dependent chrome (StreamCache 消费).
pub fn theme_epoch() -> u32 {
    THEME_EPOCH.with(|e| e.get())
}

/// Read the current accent name (Plan 413: code editor theme bridge).
pub fn accent_name() -> String {
    ACCENT_NAME.with(|n| n.borrow().clone())
}

/// Set the global accent color name (called by renderer before rendering).
pub fn set_accent_name(name: &str) {
    ACCENT_NAME.with(|n| *n.borrow_mut() = name.to_string());
}

// Plan 409 §10 续 11: 窗口宽度,供 VM builder 做响应式布局(如 category-section
// 的 grid 列数)。renderer 在 view() 前设值(同 set_dark_mode);window_resized
// 时 mark view_dirty 触发重建,让列数随窗口宽度更新。
thread_local! {
    static WINDOW_WIDTH: std::cell::Cell<f32> = std::cell::Cell::new(1024.0);
}
/// Set the current window width (called by renderer before rendering).
pub fn set_window_width(w: f32) {
    WINDOW_WIDTH.with(|c| c.set(w));
}
/// Read the current window width (for responsive layout in view builder).
pub fn window_width() -> f32 {
    WINDOW_WIDTH.with(|c| c.get())
}

/// PLAN-020 T-00b: 窗口高度(逻辑 px),与 window_width 同规约——renderer
/// 在 view() 前设值。窗口尺寸面(分屏矩形投影 px 类几何的标定源)。
thread_local! {
    static WINDOW_HEIGHT: std::cell::Cell<f32> = std::cell::Cell::new(768.0);
}
/// Set the current window height (called by renderer before rendering).
pub fn set_window_height(h: f32) {
    WINDOW_HEIGHT.with(|c| c.set(h));
}
/// Read the current window height.
pub fn window_height() -> f32 {
    WINDOW_HEIGHT.with(|c| c.get())
}

/// HSL → RGB conversion (for accent palettes).
fn hsl_to_rgb(h: u16, s: u8, l: u8) -> (u8, u8, u8) {
    let h = h as f64 / 360.0;
    let s = s as f64 / 100.0;
    let l = l as f64 / 100.0;
    if s == 0.0 {
        let v = (l * 255.0) as u8;
        return (v, v, v);
    }
    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    let hue_to_rgb = |p: f64, q: f64, mut t: f64| -> f64 {
        if t < 0.0 { t += 1.0; }
        if t > 1.0 { t -= 1.0; }
        if t < 1.0 / 6.0 { return p + (q - p) * 6.0 * t; }
        if t < 1.0 / 2.0 { return q; }
        if t < 2.0 / 3.0 { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
        p
    };
    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);
    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

/// Accent palette —— PLAN-593 后单源在 registry（vue TS ACCENT_PALETTES 值
/// 与 code_editor 派生同源互锁）；本地 `accent_hsl` 副本已删。

// Plan 458: theme preference + accent preset single source. The CLI
// (`auto run --theme/--accent`), pac.at parsing, VM env injection and the
// vue index.html generator all validate against / read from here.
// Plan 672 条目 6 继承链: "system" = 跟随 OS prefers-color-scheme
// （standalone 进程的宿主即操作系统；index.html 生成器据此前缀内联
// matchMedia 解析脚本，见 system_theme_bootstrap_js）。
pub const THEME_PREFS: [&str; 3] = ["dark", "light", "system"];
/// PLAN-593：预设名单值源自 registry（单一事实源）。
pub const ACCENT_PRESETS: [&str; 5] = registry::ACCENT_NAMES;

/// Effective theme preference injected by `auto run` (AUTO_UI_THEME env,
/// validated), or the built-in default "dark". Read by the vue/tauri
/// index.html generators so all scaffolding paths agree on the theme.
/// "system" 原样返回（调用方据此不发静态 dark class、改发内联解析脚本）。
pub fn theme_pref_from_env() -> &'static str {
    match std::env::var("AUTO_UI_THEME").as_deref() {
        Ok(t) if THEME_PREFS.contains(&t) => match t {
            "light" => "light",
            "system" => "system",
            _ => "dark",
        },
        _ => "dark",
    }
}

/// Plan 672 条目 6 继承链: theme="system" 的 index.html 内联解析脚本——
/// matchMedia 读取 OS prefers-color-scheme，把解析结果写入
/// `__AUTO_UI_THEME__` 并同步 `<html>` 的 dark class，且实时跟随 OS 切换。
/// `indent` 对齐各生成器的缩进（auto-man 4 空格 / auto cmd 2 空格）。
pub fn system_theme_bootstrap_js(indent: &str) -> String {
    let mut s = String::new();
    s.push_str(&format!("{indent}(function () {{\n"));
    s.push_str(&format!(
        "{indent}  var md = window.matchMedia('(prefers-color-scheme: dark)');\n"
    ));
    s.push_str(&format!("{indent}  var apply = function () {{\n"));
    s.push_str(&format!("{indent}    var t = md.matches ? 'dark' : 'light';\n"));
    s.push_str(&format!("{indent}    window.__AUTO_UI_THEME__ = t;\n"));
    s.push_str(&format!(
        "{indent}    document.documentElement.classList.toggle('dark', t === 'dark');\n"
    ));
    s.push_str(&format!("{indent}  }};\n"));
    s.push_str(&format!("{indent}  apply();\n"));
    s.push_str(&format!(
        "{indent}  if (md.addEventListener) md.addEventListener('change', apply);\n"
    ));
    s.push_str(&format!("{indent}}})();\n"));
    s
}

/// Effective accent preset injected by `auto run` (AUTO_UI_ACCENT env,
/// validated), or None (generators keep their stylesheet default = indigo).
pub fn accent_pref_from_env() -> Option<&'static str> {
    match std::env::var("AUTO_UI_ACCENT").as_deref() {
        Ok(a) if ACCENT_PRESETS.contains(&a) => Some(match a {
            "coral" => "coral",
            "ocean" => "ocean",
            "sage" => "sage",
            "amber" => "amber",
            _ => "indigo",
        }),
        _ => None,
    }
}

/// `--primary` shadcn token as an HSL triplet string ("H S% L%") for the
/// given accent preset + theme, or None for unknown preset names. Dark mode
/// gets the same L-boost as `resolve_semantic_rgb` (aligning to the
/// generated index.css `.dark --primary`). Consumers: vue index.html inline
/// bootstrap (Plan 458), code editors, etc.
pub fn accent_primary_hsl(name: &str, dark: bool) -> Option<String> {
    let (h, s, l) = registry::accent_hsl(name)?;
    let l = if dark { (l + 10).min(85) } else { l };
    Some(format!("{} {}% {}%", h, s, l))
}

/// Same as `accent_primary_hsl` but as an RGB tuple for native renderers
/// (iced window palette). Falls back to the caller on None (unknown name).
pub fn accent_primary_rgb(name: &str, dark: bool) -> Option<(u8, u8, u8)> {
    let (h, s, l) = registry::accent_hsl(name)?;
    let l = if dark { (l + 10).min(85) } else { l };
    Some(hsl_to_rgb(h, s, l))
}

// Plan 527 T5: font-sans/serif/mono 字体栈契约 —— Tailwind fontFamily 默认栈的
// 跨平台族名表。iced 端按 generic Family(SansSerif/Serif/Monospace) 交给
// cosmic-text 平台解析,此表是「栈语义」的成文契约与未来自定义字体回退链
// (fontFamily.config) 的锚点;docs/style-coverage.md 文本族行引用。
pub fn font_stack(kind: &str) -> &'static [&'static str] {
    match kind {
        "sans" => &[
            "ui-sans-serif", "system-ui", "-apple-system", "Segoe UI", "Roboto",
            "Helvetica Neue", "Arial", "Noto Sans", "sans-serif",
        ],
        "serif" => &[
            "ui-serif", "Georgia", "Cambria", "Times New Roman", "Times",
            "Noto Serif", "serif",
        ],
        "mono" => &[
            "ui-monospace", "SFMono-Regular", "Cascadia Mono", "Consolas",
            "Menlo", "Monaco", "DejaVu Sans Mono", "monospace",
        ],
        _ => &[],
    }
}

/// PLAN-593（Design 29 §5.4）：`Color` 枚举（VM 侧词表子集）→ registry
/// TokenName 的投影契约——card/popover/surface 三键在 VM 投影收敛为 Card
/// （词表全集以 registry 为准）。投影完备性由 plan593 T-c 测试钉死。
fn color_token(color: &Color) -> Option<registry::TokenName> {
    use registry::TokenName as T;
    Some(match color {
        Color::Secondary => T::Secondary,
        Color::Background => T::Background,
        Color::Surface => T::Card,
        Color::Muted => T::Muted,
        Color::Error => T::Error,
        Color::Warning => T::Warning,
        Color::Success => T::Success,
        Color::Info => T::Info,
        Color::OnPrimary => T::PrimaryForeground,
        Color::OnSecondary => T::SecondaryForeground,
        // PLAN-601 T-08（P593-D1）：accent 独立投影。
        Color::Accent => T::Accent,
        Color::OnAccent => T::AccentForeground,
        Color::OnDestructive => T::DestructiveForeground,
        Color::OnBackground => T::Foreground,
        Color::OnSurface => T::MutedForeground,
        Color::Border => T::Border,
        // Primary 保持运行时 accent 驱动（accent 降维为主题覆盖层属 Phase 2）；
        // 调色板/字面量色不在语义域。
        _ => return None,
    })
}

/// Resolve a semantic color to RGB, considering dark mode and accent.
/// PLAN-593 V2：静态语义色查 registry（stella 单源），零字面 RGB 臂。
pub fn resolve_semantic_rgb(color: &Color) -> Option<(u8, u8, u8)> {
    let is_dark = DARK_MODE.with(|d| d.get());
    if let Color::Primary = color {
        // Accent-driven: look up current accent name
        let name = ACCENT_NAME.with(|n| n.borrow().clone());
        let (h, s, l) = registry::accent_hsl(&name).unwrap_or(registry::ACCENT_DEFAULT);
        // Dark mode: align to vue index.css .dark `--primary: 239 84% 77%` (L=77%)
        let l_adjusted = if is_dark { (l + 10).min(85) } else { l };
        return Some(hsl_to_rgb(h, s, l_adjusted));
    }
    let token = color_token(color)?;
    active_theme().resolve(token, is_dark)
}

/// Plan 411 P2-A④: vue `border-border` 语义色(shadcn --border 变量)——
/// PLAN-593 V2：值改查 registry（stella 表），零字面 RGB。
/// （历史校准记录：Plan 518 暖灰 light #e3ddd1 / 蓝黑 dark #283146。）
pub fn resolve_border_rgb() -> (u8, u8, u8) {
    let is_dark = DARK_MODE.with(|d| d.get());
    active_theme()
        .resolve(registry::TokenName::Border, is_dark)
        .expect("活动主题 Border 槽：builtin 由 themes_core_complete 钉死；composed 含基座全表")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Plan 672 条目 6 继承链: "system" 预设全链路——env 校验通过且原样
    /// 透传；resolver 脚本含 matchMedia/实时跟随/回填三要素。
    #[test]
    fn theme_pref_system_passthrough_and_bootstrap_js() {
        // THEME_PREFS 名单校验（不 set env，只测名单包含性）
        assert!(THEME_PREFS.contains(&"system"));
        let js = system_theme_bootstrap_js("    ");
        assert!(js.contains("matchMedia('(prefers-color-scheme: dark)')"), "{js}");
        assert!(js.contains("__AUTO_UI_THEME__ = t"), "{js}");
        assert!(js.contains("classList.toggle('dark'"), "{js}");
        assert!(js.contains("addEventListener('change', apply)"), "{js}");
        // env 透传（串行竞争面小：仅此测设 system，结束即移除）
        std::env::set_var("AUTO_UI_THEME", "system");
        assert_eq!(theme_pref_from_env(), "system");
        std::env::remove_var("AUTO_UI_THEME");
        assert_eq!(theme_pref_from_env(), "dark");
    }

    /// PLAN-020 T-00b:窗口尺寸面金样——height 与 width 同规约(线程局部
    /// 全局,renderer 每帧 set;分屏矩形投影 px 类几何的标定源)。防回退:
    /// 默认槽位值 + setter 往返。
    #[test]
    fn plan020_window_size_face_set_get_roundtrip() {
        // thread_local 缺省槽位(与既有 width 缺省 1024 同代的合理值)。
        let h0 = window_height();
        assert!(h0 > 0.0, "window_height 缺省必须为正,实测 {h0}");
        set_window_height(482.0);
        assert_eq!(window_height(), 482.0);
        set_window_height(h0);
        assert_eq!(window_height(), h0);
        // width 面既有行为锚定(防 T-00b 改动波及)。
        let w0 = window_width();
        set_window_width(802.0);
        assert_eq!(window_width(), 802.0);
        set_window_width(w0);
    }

    fn rgb(color: Color) -> (u8, u8, u8) {
        resolve_semantic_rgb(&color).expect("semantic color must resolve")
    }

    /// PLAN-619 T-03：轨缺省主题已从 stella 改为 scaffold，故凡断言 stella
    /// 值的用例必须显式钉住 stella——本文件多数用例的被测面就是 stella 色板。
    fn pin_stella() {
        assert!(set_theme("stella"), "stella 恒在");
    }

    /// PLAN-619 T-03（用户裁决）：轨缺省主题 = scaffold——语义 token 与
    /// auto-man 为 scaffold 类应用生成的 Vue `index.css` 同源。修前缺省
    /// stella，任何未声明 `theme{}` 的应用在 VM 端面板底色整体偏色（P1）。
    /// 桌面宿主的 stella 观感改由 `DesktopConfig::default().theme_name` 显式持有。
    #[test]
    fn default_theme_is_scaffold_and_matches_vue_css_tokens() {
        // 同进程其它用例可能已改过主题（thread-local）→ 幂等回缺省名。
        // 注：`set_theme` 返回「是否发生变化」，同名时为 false，故不能以它断言。
        if theme_name() != "scaffold" {
            set_theme("scaffold");
        }
        assert_eq!(theme_name(), "scaffold", "轨缺省主题");
        set_dark_mode(true);
        // scaffold `.dark` 表 = Vue index.css `.dark` 块（--background 222.2 47% 7%
        // → (9,14,26)；--card 222.2 47% 10% → (13,20,37)，浏览器侧为 (14,21,37)，差 1）。
        assert_eq!(rgb(Color::Background), (9, 14, 26), "页面/编辑区底色");
        assert_eq!(rgb(Color::Surface), (13, 20, 37), "卡片/侧栏底色");
        set_dark_mode(true);
    }

    /// Plan 518 T1: 双主题语义值表——light 暖纸 / dark 精修蓝黑(stella 对齐)。
    /// PLAN-601 T-04：活动主题热切换——resolve 全翻转 + epoch 自增 +
    /// 未知名拒绝 + 还原防污染。
    #[test]
    fn theme_switch_flips_resolution_and_bumps_epoch() {
        // PLAN-619 T-03：轨缺省 = scaffold（与 scaffold 类应用的 Vue
        // index.css 同源）；本用例的被测面是切换语义，基线钉 stella。
        assert_eq!(super::theme_name(), "scaffold");
        pin_stella();
        super::set_dark_mode(false); // 先定 mode，再取 epoch 基线（dark 翻转也自增）
        let e0 = super::theme_epoch();
        assert!(!super::set_theme("nonsense"), "未知名拒绝");
        assert_eq!(super::theme_name(), "stella");
        assert_eq!(super::theme_epoch(), e0);
        assert!(super::set_theme("zinc"));
        assert_eq!(super::theme_epoch(), e0.wrapping_add(1));
        super::set_dark_mode(false);
        assert_eq!(
            super::resolve_semantic_rgb(&Color::Background),
            Some((255, 255, 255)),
            "zinc light background = 纯白"
        );
        assert!(super::set_theme("stella"));
        assert_eq!(
            super::resolve_semantic_rgb(&Color::Background),
            Some((245, 241, 232)),
            "stella 暖纸恢复"
        );
        assert_eq!(super::theme_epoch(), e0.wrapping_add(2), "两次主题切换各 +1（无 dark 翻转）");
        super::set_dark_mode(true); // 还原默认档，防污染其他用例
    }

    #[test]
    fn stella_light_palette() {
        pin_stella();
        set_dark_mode(false);
        assert_eq!(rgb(Color::Background), (245, 241, 232)); // #f5f1e8 暖纸
        assert_eq!(rgb(Color::Surface), (251, 248, 242)); // #fbf8f2 卡片微浮
        assert_eq!(rgb(Color::OnBackground), (42, 39, 35)); // 墨色 #2a2723
        assert_eq!(rgb(Color::Border), (227, 221, 209)); // 暖灰 #e3ddd1
        assert_eq!(rgb(Color::Muted), (240, 235, 226)); // 暖 muted #f0ebe2
        // PLAN-571: secondary 分档（≠muted）——light 暖灰一档深 #e3ddd1。
        assert_eq!(rgb(Color::Secondary), (227, 221, 209));
        assert_eq!(rgb(Color::OnSurface), (125, 119, 109)); // 暖次级文本 #7d776d
        assert_eq!(resolve_border_rgb(), (227, 221, 209));
    }

    #[test]
    fn stella_dark_palette() {
        pin_stella();
        set_dark_mode(true);
        assert_eq!(rgb(Color::Background), (20, 26, 41)); // #141a29 深蓝黑
        assert_eq!(rgb(Color::Surface), (26, 34, 53)); // #1a2235 面板
        assert_eq!(rgb(Color::Border), (40, 49, 70)); // 低对比 #283146
        assert_eq!(resolve_border_rgb(), (40, 49, 70));
        // PLAN-571: secondary 分档（≠muted）——dark slate-700 #334155。
        assert_eq!(rgb(Color::Secondary), (51, 65, 85));
        assert_ne!(rgb(Color::Secondary), rgb(Color::Muted));
    }

    /// 待澄清②裁定:stella 玫瑰粉 = 既有 coral 预设(503 已校准 light
    /// hsl(4,43%,59%) ≈ #c4706a,权威图实测 #C96B62),不新增 rose 预设。
    #[test]
    fn coral_matches_stella_rose_accent() {
        pin_stella();
        set_dark_mode(false);
        set_accent_name("coral");
        assert_eq!(rgb(Color::Primary), (195, 111, 105)); // hsl(4,43%,59%) 截断值
        set_dark_mode(true);
        assert_eq!(rgb(Color::Primary), (209, 146, 141)); // dark L+10 → 69%
        set_accent_name("indigo"); // 还原默认,防污染其他用例
    }

    #[test]
    fn border_resolver_consistent_with_border_token() {
        pin_stella();
        for dark in [false, true] {
            set_dark_mode(dark);
            assert_eq!(resolve_border_rgb(), rgb(Color::Border));
        }
        set_dark_mode(true); // 还原默认
    }

    /// Plan 527 T5: 字体栈契约 —— 三栈齐备且平台主流族名在册。
    #[test]
    fn font_stacks_cover_three_families() {
        for kind in ["sans", "serif", "mono"] {
            let stack = font_stack(kind);
            assert!(!stack.is_empty(), "{kind} 栈非空");
            assert!(
                stack.iter().any(|f| f.to_ascii_lowercase().contains(kind))
                    || stack.contains(&"Segoe UI")
                    || stack.contains(&"Georgia")
                    || stack.contains(&"Consolas"),
                "{kind} 栈应含 generic 兜底或平台主流族名"
            );
        }
        assert!(font_stack("unknown").is_empty(), "未知族返回空栈");
    }
}
