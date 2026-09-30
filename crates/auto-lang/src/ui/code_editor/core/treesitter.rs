// PLAN-716 组A: tree-sitter 双轨语法高亮 runtime（714 契约 T-1..T-5 实施）。
//
// 路由纪律（frozen ①/④，全批次有效）:
// - plain/none/空 lang 旁路在本模块上游（highlight.rs `lang_to_extension` →
//   None，013 硬边界）——本模块只对"有扩展名的 lang"被征询，big 态零 parse。
// - 路由表未登记的语言返回 `None` → 调用方走 syntect 双轨基线（逐字节等价）；
//   tail 语言（.at/mermaid/vue/console）固定不登记（714 §4.1 (a) 裁定——
//   产品身份语言 .at 留 syntect 臂直至 T-7 自建 grammar）。
// - ts 管线失败（parse/查询异常）→ `None` 回落 syntect，正确性优先。
// - feature 关闭 = 本模块整体不参与编译（零语义，syntect 基线）。
//
// 配色纪律: capture 名按前缀归类后，用与 syntect 路径**同一个** AutoUI 主题
// （syntax_system().theme_set，(theme, dark, accent) 同键）做 scope 选择器
// 匹配取色——类别级对照（714 §6.1：非逐 token 等价）下双引擎同主题同源，
// accent/明暗态零分叉。
//
// 语言配置（HighlightConfiguration）进程级单例；Highlighter 线程本地复用
// （0.27 内嵌 Parser——714 §5.1 单例纪律继承）。

use std::collections::HashMap;
use std::cell::RefCell;
use std::sync::{Mutex, OnceLock};

use tree_sitter::Language;
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};

/// 路由表条目——一种语言一个登记位（714 §2.2，21 crate 批次接入）。
pub(crate) struct TsLangSpec {
    /// 路由键（DSL lang token，小写；如 ["rust", "rs"]）。
    pub lang_keys: &'static [&'static str],
    /// 诊断名（= a2r/日志/探针口径，如 "rust"）。
    pub name: &'static str,
    /// grammar language 构造（crate 顶层 fn，晚绑定避免 const 求值约束）。
    pub language_fn: fn() -> Language,
    /// highlights 查询（vendored 自 grammar crate，出处注记在文件头）。
    pub highlights_query: &'static str,
    /// injections 查询（可选——md/html/cpp 族）。
    pub injections_query: Option<&'static str>,
}

/// 语言路由表——T-01 骨架期空表（全量 syntect 基线）；T-02 起按批次登记。
/// tail 语言（.at/mermaid/vue/console）任何批次都不得入表（路由纪律）。
pub(crate) static LANG_TABLE: &[TsLangSpec] = &[];

/// lang → ts 规格路由。表外语言一律 None（syntect 基线）。
pub(crate) fn route(lang: &str) -> Option<&'static TsLangSpec> {
    let key = lang.to_ascii_lowercase();
    LANG_TABLE.iter().find(|s| s.lang_keys.contains(&key.as_str()))
}

/// 进程级配置单例（Query 编译一次；Box::leak 共享）。
fn config_for(spec: &TsLangSpec) -> Option<&'static HighlightConfiguration> {
    static CONFIGS: OnceLock<Mutex<HashMap<&'static str, &'static HighlightConfiguration>>> =
        OnceLock::new();
    let configs = CONFIGS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut map = configs.lock().ok()?;
    if let Some(cfg) = map.get(spec.name) {
        return Some(cfg);
    }
    let mut config = HighlightConfiguration::new(
        (spec.language_fn)(),
        spec.name,
        spec.highlights_query,
        spec.injections_query.unwrap_or(""),
        "",
    )
    .ok()?;
    config.configure(&HIGHLIGHT_NAMES);
    let leaked: &'static HighlightConfiguration = Box::leak(Box::new(config));
    map.insert(spec.name, leaked);
    Some(leaked)
}

/// configure 名单（tree-sitter 标准捕获名集，714 §3.2 烟测口径的全集超集）。
/// Highlight.0 索引即本表下标；表外捕获名被 tree-sitter-highlight 忽略。
const HIGHLIGHT_NAMES: &[&str] = &[
    "attribute",
    "boolean",
    "comment",
    "constant",
    "constant.builtin",
    "constant.character",
    "constant.character.escape",
    "constructor",
    "embedded",
    "function",
    "function.builtin",
    "function.call",
    "function.macro",
    "include",
    "keyword",
    "keyword.function",
    "keyword.operator",
    "keyword.return",
    "label",
    "method",
    "module",
    "number",
    "operator",
    "property",
    "punctuation.bracket",
    "punctuation.delimiter",
    "punctuation.special",
    "string",
    "string.escape",
    "string.special",
    "symbol",
    "tag",
    "type",
    "type.builtin",
    "variable",
    "variable.builtin",
    "variable.member",
    "variable.parameter",
];

/// capture 索引（Highlight.0 = configure 名单下标）→ 归类名。
fn category_for_index(idx: usize) -> Option<&'static str> {
    let name = HIGHLIGHT_NAMES.get(idx)?;
    Some(
        if name.starts_with("comment") {
            "comment"
        } else if name.starts_with("string") {
            "string"
        } else if name.starts_with("keyword.operator") || name.starts_with("operator") {
            "operator"
        } else if name.starts_with("keyword") {
            "keyword"
        } else if name.starts_with("number")
            || name.starts_with("constant")
            || name.starts_with("boolean")
        {
            "constant"
        } else if name.starts_with("type") {
            "type"
        } else if name.starts_with("function")
            || name.starts_with("method")
            || name.starts_with("constructor")
        {
            "function"
        } else if name.starts_with("tag") {
            "tag"
        } else if name.starts_with("attribute") {
            "attribute"
        } else if name.starts_with("include") || name.starts_with("module") || name.starts_with("label") {
            "module"
        } else if name.starts_with("property") || name.starts_with("variable.member") {
            "property"
        } else if name.starts_with("variable") || name.starts_with("symbol") {
            "variable"
        } else {
            // punctuation/embedded/diff/spell 等 → 基色（None）
            return None;
        },
    )
}

/// 归类名 → AutoUI 主题 scope 探针（优先级序，首个命中生效）——与 syntect
/// 路径同主题取色，类别级同源。
fn scope_probes_for_category(category: &str) -> &'static [&'static str] {
    match category {
        "comment" => &["comment"],
        "string" => &["string"],
        "operator" => &["keyword.operator"],
        "keyword" => &["keyword.control", "keyword"],
        "constant" => &["constant.numeric", "constant"],
        "type" => &["entity.name.type", "support.type"],
        "function" => &["entity.name.function", "support.function"],
        "tag" => &["entity.name.tag"],
        "attribute" => &["entity.other.attribute-name"],
        "module" => &["entity.name.namespace", "support.module"],
        "property" => &["variable.other.property", "support.variable.property", "variable.other.member"],
        "variable" => &["variable"],
        _ => &[],
    }
}

/// 在 syntect 主题上做 scope 匹配取前景色（顺序覆盖——与 syntect 选择器
/// 顺序应用语义一致；无命中 → None = 基色）。
fn style_rgb_for_scope(
    theme: &syntect::highlighting::Theme,
    scope: &str,
) -> Option<(u8, u8, u8)> {
    use syntect::parsing::{Scope, ScopeStack};
    let Ok(scope) = Scope::new(scope) else { return None };
    let mut stack = ScopeStack::new();
    stack.push(scope);
    let mut fg: Option<syntect::highlighting::Color> = None;
    for item in &theme.scopes {
        if item.scope.does_match(stack.as_slice()).is_some() {
            if let Some(f) = item.style.foreground {
                fg = Some(f);
            }
        }
    }
    fg.map(|c| (c.r, c.g, c.b))
}

/// ts 高亮主入口（只读三面消费：markdown code_block/autodown fence/渲染器）。
/// 返回 None = 语言未路由或管线失败（调用方回落 syntect 基线）。
/// 输出形与 highlight_segments 相同：文本连续段按色合并（`None` = 基色）。
pub(crate) fn highlight_segments(
    lang: &str,
    text: &str,
    dark: bool,
    accent: &str,
) -> Option<Vec<(String, Option<(u8, u8, u8)>)>> {
    let spec = route(lang)?;
    let config = config_for(spec)?;
    let theme_id = crate::ui::style::theme::theme_name();
    let theme_key = crate::ui::code_editor::core::highlight::theme_name(&theme_id, dark, accent);
    let theme = crate::ui::code_editor::core::highlight::syntax_system()
        .theme_set
        .themes
        .get(&theme_key)?;

    // 类别 → RGB 预解析（段级查表，事件循环零重复匹配）。
    let category_rgb: HashMap<&'static str, Option<(u8, u8, u8)>> = ["comment", "string",
        "operator", "keyword", "constant", "type", "function", "tag", "attribute", "module",
        "property", "variable"]
        .into_iter()
        .map(|cat| {
            let rgb = scope_probes_for_category(cat)
                .iter()
                .find_map(|s| style_rgb_for_scope(theme, s));
            (cat, rgb)
        })
        .collect();

    // Highlighter 线程本地复用；事件先物化（迭代器借用 highlighter）。
    let mut events: Vec<HighlightEvent> = Vec::new();
    HIGHLIGHTER.with_borrow_mut(|highlighter| {
        let Ok(iter) = highlighter.highlight(
            config,
            text.as_bytes(),
            None,
            None,
            |injected: &str| route(injected).and_then(config_for),
        ) else {
            return false;
        };
        for event in iter {
            match event {
                Ok(e) => events.push(e),
                Err(_) => return false,
            }
        }
        true
    })
    .then_some(())
    ?;

    let mut out: Vec<(String, Option<(u8, u8, u8)>)> = Vec::new();
    let mut current: Option<(u8, u8, u8)> = None;
    for event in &events {
        match *event {
            HighlightEvent::HighlightStart(h) => {
                let category = category_for_index(h.0);
                current = category.and_then(|c| category_rgb.get(c).copied().flatten());
            }
            HighlightEvent::HighlightEnd => current = None,
            HighlightEvent::Source { start, end } => {
                let bytes = text.as_bytes().get(start..end).unwrap_or(&[]);
                let seg = std::str::from_utf8(bytes).unwrap_or("");
                match out.last_mut() {
                    Some((prev, prev_color)) if *prev_color == current => prev.push_str(seg),
                    _ => out.push((seg.to_string(), current)),
                }
            }
        }
    }
    if out.is_empty() {
        return None;
    }
    Some(out)
}

thread_local! {
    /// 每线程复用 Highlighter（内嵌 Parser——714 §5.1 复用纪律）。
    static HIGHLIGHTER: RefCell<Highlighter> = RefCell::new(Highlighter::new());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// T-01 骨架期：路由表空——一切语言 syntect 基线（零行为断言）。
    #[test]
    fn empty_table_routes_nothing() {
        assert!(route("rust").is_none());
        assert!(route("python").is_none());
        assert!(highlight_segments("rust", "fn main() {}\n", true, "indigo").is_none());
    }

    /// tail 语言任何批次都不得入表（路由纪律，frozen ④）。
    #[test]
    fn tail_langs_never_route() {
        for lang in ["at", "auto", "autolang", "mermaid", "vue", "console", "none", "plain", ""] {
            assert!(route(lang).is_none(), "tail lang must stay on syntect: {lang}");
        }
    }

    /// 路由键大小写不敏感（与 lang_to_extension 同口径）。
    #[test]
    fn route_key_case_insensitive() {
        // 空表下断言的是查表路径本身：大写键不 panic 且同样返回 None。
        assert!(route("Rust").is_none());
    }

    /// capture 归类表（色映射前缀语义）。
    #[test]
    fn capture_categories() {
        let idx = |name: &str| HIGHLIGHT_NAMES.iter().position(|n| *n == name).unwrap();
        assert_eq!(category_for_index(0), Some("attribute"));
        assert_eq!(category_for_index(idx("comment")), Some("comment"));
        assert_eq!(category_for_index(idx("string.escape")), Some("string"));
        assert_eq!(category_for_index(idx("keyword.operator")), Some("operator"));
        assert_eq!(category_for_index(idx("keyword")), Some("keyword"));
        assert_eq!(category_for_index(idx("number")), Some("constant"));
        assert_eq!(category_for_index(idx("type.builtin")), Some("type"));
        assert_eq!(category_for_index(idx("function")), Some("function"));
        assert_eq!(category_for_index(idx("tag")), Some("tag"));
        assert_eq!(category_for_index(idx("variable")), Some("variable"));
        assert_eq!(category_for_index(idx("punctuation.delimiter")), None);
        assert_eq!(category_for_index(999), None, "越界索引安全");
    }

    /// scope 探针表：每归类至少一条探针。
    #[test]
    fn every_category_has_probes() {
        for cat in ["comment", "string", "operator", "keyword", "constant", "type",
            "function", "tag", "attribute", "module", "property", "variable"]
        {
            assert!(!scope_probes_for_category(cat).is_empty(), "{cat} 无探针");
        }
    }
}
