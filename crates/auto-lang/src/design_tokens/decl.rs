//! theme{} 声明解析与合成（PLAN-601 T-03，Design 29 §4.2/§5.2）。
//!
//! 声明形态（pac.at，对象需冒号形态——`back: { project }` 同构）：
//! ```auto
//! theme: {
//!     extends: "stella"        // 内置主题名或已声明的具名主题；缺省按轨
//!     mode: "auto"             // light|dark|auto（缺省沿 extends 链继承）
//!     colors: {
//!         primary: "#8b5cf6"   // partial——未知键=compose 期错误
//!     }
//! }
//! ```
//! 兼容既有标量 `theme: "dark"|"light"`（仅 mode 语义，走既有 458 链路）。
//!
//! 合成规则：extends 链（深度 ≤4，防环）——基座为链顶 builtin，双 mode
//! css 值串起步；链上声明 colors 逐层覆盖（**本声明最后胜**）；mode 取链上
//! 最近声明（自身优先，祖先次之）。值接受 `#rrggbb`（规范化为 shadcn HSL
//! 串）或 `"h s% l%"`。

use super::registry::{self, TokenName};
use std::collections::BTreeMap;

/// 解析/合成错误（负用例枚举，测试钉死）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeclError {
    UnknownToken(String),
    UnknownExtends(String),
    ChainTooDeep,
    ExtendsCycle(String),
    InvalidValue(String),
    InvalidMode(String),
}

impl std::fmt::Display for DeclError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeclError::UnknownToken(t) => write!(f, "未知主题 token `{t}`（封闭词表 31 键外）"),
            DeclError::UnknownExtends(n) => write!(f, "extends 目标 `{n}` 不是内置主题也不是已声明主题"),
            DeclError::ChainTooDeep => write!(f, "extends 链深度超过 4"),
            DeclError::ExtendsCycle(n) => write!(f, "extends 成环：`{n}`"),
            DeclError::InvalidValue(v) => write!(f, "色值 `{v}` 既非 #rrggbb 也非 h s% l% 形态"),
            DeclError::InvalidMode(m) => write!(f, "mode `{m}` 非 light|dark|auto"),
        }
    }
}

impl std::error::Error for DeclError {}

/// 声明态（pac.at theme{} 块解析结果；extends 目标可为具名声明）。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ThemeDecl {
    /// 声明名（具名主题时用于被 extends 引用；无名声明不参与链）。
    pub name: Option<String>,
    pub extends: Option<String>,
    /// light | dark | auto（None = 沿链继承/调用方轨缺省）。
    pub mode: Option<String>,
    /// 覆盖键值（token css 名 → 值串），按声明序保留。
    pub colors: Vec<(String, String)>,
}

/// `#rrggbb` → RGB；非 hex 返回 None。
fn hex_to_rgb(v: &str) -> Option<(u8, u8, u8)> {
    let h = v.strip_prefix('#')?;
    if h.len() != 6 || !h.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let n = u32::from_str_radix(h, 16).ok()?;
    Some(((n >> 16) as u8, ((n >> 8) & 0xff) as u8, (n & 0xff) as u8))
}

/// 值规范化：hex → shadcn HSL 串；合法 HSL 串原样；否则 InvalidValue。
pub fn normalize_value(v: &str) -> Result<String, DeclError> {
    let v = v.trim();
    if let Some(rgb) = hex_to_rgb(v) {
        return Ok(registry::rgb_to_hsl_str(rgb));
    }
    if registry::hsl_str_to_rgb(v).is_some() {
        return Ok(v.to_string());
    }
    Err(DeclError::InvalidValue(v.to_string()))
}

/// 合成态主题：css 值串双 mode 表（渲染/求值与 builtin 同口径——
/// hsl_str_to_rgb 求值、canonical 键序渲染）。
#[derive(Debug, Clone, PartialEq)]
pub struct ComposedTheme {
    pub name: String,
    /// 链上最近声明的缺省 mode（None = 调用方轨缺省；auto = 跟随系统）。
    pub default_dark: Option<bool>,
    pub default_auto: bool,
    pub light: Vec<(TokenName, String)>,
    pub dark: Vec<(TokenName, String)>,
}

impl ComposedTheme {
    fn palette(&self, dark: bool) -> &[(TokenName, String)] {
        if dark { &self.dark } else { &self.light }
    }

    /// VM 面：按 mode 求值（HSL 串解析）。
    pub fn resolve_rgb(&self, token: TokenName, is_dark: bool) -> Option<(u8, u8, u8)> {
        self.palette(is_dark)
            .iter()
            .find(|(t, _)| *t == token)
            .and_then(|(_, v)| registry::hsl_str_to_rgb(v))
    }

    /// CSS 面：canonical 核心块（键序与 builtin render_core 一致）。
    pub fn render_core(&self, is_dark: bool) -> String {
        render_tokens(self.palette(is_dark), &registry::CORE_ORDER)
    }

    /// CSS 面：canonical sidebar 块。
    pub fn render_sidebar(&self, is_dark: bool) -> String {
        render_tokens(self.palette(is_dark), &registry::SIDEBAR_ORDER)
    }

    /// JS 运行时对象字面量（vue 脚手架 `window.__AUTO_COMPOSED_THEME__`
    /// 注入体，PLAN-601 T-06）——与 registry::render_theme_pairs_js 同形，
    /// 声明合成体经此并入运行时 THEME_PALETTES。
    pub fn render_pairs_js(&self) -> String {
        let mode = |dark: bool| -> String {
            let pairs: Vec<String> = self
                .palette(dark)
                .iter()
                .map(|(t, v)| format!("    '{}': '{}'", t.css_var(), v))
                .collect();
            format!("{{\n{}\n  }}", pairs.join(",\n"))
        };
        format!("{{ light: {}, dark: {} }}", mode(false), mode(true))
    }
}

fn render_tokens(pal: &[(TokenName, String)], order: &[TokenName]) -> String {
    let mut out = String::new();
    for t in order {
        if let Some((_, v)) = pal.iter().find(|(k, _)| k == t) {
            out.push_str(&format!("    --{}: {};\n", t.css_var(), v));
        }
    }
    out
}

/// extends 最大链深（基座 builtin 之上最多再叠 4 层声明）。
const MAX_CHAIN: usize = 4;

/// 合成：`decl`（可无名）extends 具名声明集 `declared` 与内置主题。
pub fn compose(
    decl: &ThemeDecl,
    declared: &BTreeMap<String, ThemeDecl>,
) -> Result<ComposedTheme, DeclError> {
    // 1) 解链：自 decl 沿 extends 上溯（chain[0]=本声明），基座必须 builtin。
    let mut chain: Vec<ThemeDecl> = Vec::new();
    let mut cur = decl.clone();
    let mut visited: Vec<String> = Vec::new();
    loop {
        match cur.extends.as_deref() {
            None => {
                chain.push(cur);
                break;
            }
            Some(base) => {
                if let Some(next) = declared.get(base) {
                    if visited.iter().any(|v| v == base) {
                        return Err(DeclError::ExtendsCycle(base.to_string()));
                    }
                    visited.push(base.to_string());
                    if visited.len() > MAX_CHAIN {
                        return Err(DeclError::ChainTooDeep);
                    }
                    chain.push(cur);
                    cur = next.clone();
                } else if registry::builtin(base).is_some() {
                    chain.push(cur);
                    break;
                } else {
                    return Err(DeclError::UnknownExtends(base.to_string()));
                }
            }
        }
    }
    if chain.len() > MAX_CHAIN + 1 {
        return Err(DeclError::ChainTooDeep);
    }

    // 2) 基座 palette：链顶声明的 extends 目标 builtin（无 extends → stella）。
    let root_builtin = chain
        .last()
        .and_then(|d| d.extends.as_deref())
        .and_then(registry::builtin)
        .or_else(|| registry::builtin("stella"))
        .expect("stella 恒在");
    let mut light: Vec<(TokenName, String)> = root_builtin
        .light
        .iter()
        .map(|(t, lit)| (*t, lit.css_str()))
        .collect();
    let mut dark: Vec<(TokenName, String)> = root_builtin
        .dark
        .iter()
        .map(|(t, lit)| (*t, lit.css_str()))
        .collect();

    // 3) 声明覆盖：祖先先铺、本声明最后胜（chain[0]=本声明 → rev 尾为其上）。
    for d in chain.iter().rev() {
        apply_colors(&mut light, &d.colors)?;
        apply_colors(&mut dark, &d.colors)?;
    }

    // 4) mode：链上最近声明胜（chain[0]=自身优先）；全链校验。
    for d in &chain {
        if let Some(m) = d.mode.as_deref() {
            if !matches!(m, "light" | "dark" | "auto") {
                return Err(DeclError::InvalidMode(m.to_string()));
            }
        }
    }
    let nearest_mode = chain.first().and_then(|d| d.mode.clone())
        .or_else(|| chain.iter().skip(1).find_map(|d| d.mode.clone()));
    let (default_dark, default_auto) = match nearest_mode.as_deref() {
        Some("light") => (Some(false), false),
        Some("dark") => (Some(true), false),
        Some("auto") => (None, true),
        _ => (None, false),
    };

    Ok(ComposedTheme {
        name: decl.name.clone().unwrap_or_else(|| root_builtin.name.to_string()),
        default_dark,
        default_auto,
        light,
        dark,
    })
}

fn apply_colors(pal: &mut Vec<(TokenName, String)>, colors: &[(String, String)]) -> Result<(), DeclError> {
    for (k, v) in colors {
        let token = TokenName::from_css_var(k).ok_or_else(|| DeclError::UnknownToken(k.clone()))?;
        let norm = normalize_value(v)?;
        if let Some(slot) = pal.iter_mut().find(|(t, _)| *t == token) {
            slot.1 = norm;
        } else {
            pal.push((token, norm));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decl(extends: Option<&str>, colors: &[(&str, &str)]) -> ThemeDecl {
        ThemeDecl {
            name: Some("app".into()),
            extends: extends.map(|s| s.to_string()),
            mode: None,
            colors: colors.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
        }
    }

    /// 合法：extends stella + hex 覆盖 primary + HSL 串覆盖 border；未覆盖键
    /// 继承基座；覆盖落双 mode。
    #[test]
    fn compose_overrides_and_normalizes() {
        let d = decl(Some("stella"), &[("primary", "#8b5cf6"), ("border", "40 24% 85%")]);
        let t = compose(&d, &BTreeMap::new()).unwrap();
        assert_eq!(t.name, "app");
        // T-06：JS 对字面量形态（与 CSS 面同值源）
        let js = t.render_pairs_js();
        assert!(js.contains("light: {") && js.contains("dark: {"), "{js}");
        assert!(js.contains(&format!("'primary': '{}'", normalize_value("#8b5cf6").unwrap())));
        let css_l = t.render_core(false);
        let css_d = t.render_core(true);
        let want_primary = format!("--primary: {};", normalize_value("#8b5cf6").unwrap());
        assert!(css_l.contains(&want_primary), "{css_l}");
        assert!(css_d.contains(&want_primary), "覆盖落双 mode");
        // 未覆盖键继承 stella 基座 css 值
        let st = registry::builtin("stella").unwrap();
        let bg = st.light.iter().find(|(t, _)| *t == TokenName::Background).unwrap().1.css_str();
        assert!(css_l.contains(&format!("--background: {bg};")));
        // VM 面可求值（hex 规范化后 HSL 解析）
        assert!(t.resolve_rgb(TokenName::Primary, false).is_some());
        assert_eq!(t.default_dark, None);
        assert!(!t.default_auto);
    }

    /// 负用例：未知 token / 未知 extends / 非法值 / 非法 mode。
    #[test]
    fn negative_cases() {
        let e = compose(&decl(Some("stella"), &[("nope", "#000000")]), &BTreeMap::new());
        assert_eq!(e.unwrap_err(), DeclError::UnknownToken("nope".into()));
        let e = compose(&decl(Some("nonsense"), &[]), &BTreeMap::new());
        assert_eq!(e.unwrap_err(), DeclError::UnknownExtends("nonsense".into()));
        let e = compose(&decl(Some("stella"), &[("primary", "blue")]), &BTreeMap::new());
        assert_eq!(e.unwrap_err(), DeclError::InvalidValue("blue".into()));
        let e = compose(&ThemeDecl { mode: Some("blue".into()), ..decl(None, &[]) }, &BTreeMap::new());
        assert_eq!(e.unwrap_err(), DeclError::InvalidMode("blue".into()));
        // 无 extends：缺省 stella 基座，合法
        assert!(compose(&decl(None, &[]), &BTreeMap::new()).is_ok());
    }

    /// 具名链：app extends brand extends stella——本声明覆盖胜 + mode 沿链
    /// 继承（brand 声明 dark，app 未声明）。
    #[test]
    fn named_chain_layering_and_mode_inheritance() {
        let mut declared = BTreeMap::new();
        declared.insert(
            "brand".to_string(),
            ThemeDecl {
                name: Some("brand".into()),
                extends: Some("stella".into()),
                mode: Some("dark".into()),
                colors: vec![("primary".into(), "#22c55e".into())],
            },
        );
        let d = ThemeDecl {
            name: Some("app".into()),
            extends: Some("brand".into()),
            mode: None,
            colors: vec![("primary".into(), "#ef4444".into())],
        };
        let t = compose(&d, &declared).unwrap();
        assert_eq!(t.default_dark, Some(true), "mode 继承自 brand");
        assert_eq!(t.default_auto, false);
        let red = normalize_value("#ef4444").unwrap();
        let green = normalize_value("#22c55e").unwrap();
        let css = t.render_core(false);
        assert!(css.contains(&format!("--primary: {red};")), "本声明最后胜");
        assert!(!css.contains(&format!("--primary: {green};")));
    }

    /// 链深上限与成环。
    #[test]
    fn depth_limit_and_cycle() {
        let mut declared = BTreeMap::new();
        for i in 0..6usize {
            let ext = if i == 0 { Some("stella".into()) } else { Some(format!("l{}", i - 1)) };
            declared.insert(format!("l{i}"), ThemeDecl {
                name: Some(format!("l{i}")),
                extends: ext,
                mode: None,
                colors: vec![],
            });
        }
        assert_eq!(
            compose(declared.get("l5").unwrap(), &declared).unwrap_err(),
            DeclError::ChainTooDeep
        );
        // 成环：a → b → a
        let mut cyc = BTreeMap::new();
        cyc.insert("a".to_string(), ThemeDecl { name: Some("a".into()), extends: Some("b".into()), mode: None, colors: vec![] });
        cyc.insert("b".to_string(), ThemeDecl { name: Some("b".into()), extends: Some("a".into()), mode: None, colors: vec![] });
        assert!(matches!(
            compose(cyc.get("a").unwrap(), &cyc),
            Err(DeclError::ExtendsCycle(_))
        ));
    }
}
