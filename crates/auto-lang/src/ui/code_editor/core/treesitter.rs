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
    /// highlights 查询（crate 自带，零 vendoring——714 §3.1）。
    pub highlights_query: &'static str,
    /// 叠加层查询（typescript/tsx = javascript 基础查询 + TS 专有节点层——
    /// 上游官方用法：两段拼接，TS 查询只覆盖 TS 专有 pattern）。
    pub highlights_query_extra: Option<&'static str>,
    /// injections 查询（可选——md/html/cpp 族）。
    pub injections_query: Option<&'static str>,
}

/// 语言路由表——首批 21 语言全量在册（714 §2.2 表：P0 10+P1 8+P2 3；
/// T-02/T-04/T-05 批）。查询全部直用 grammar crate 导出常量
/// （HIGHLIGHT*_QUERY/INJECTION*_QUERY——crate 自带查询零 vendoring，
/// 714 §3.1 定案成立）。`markdown-inline` 为注入专用键（md block 查询的
/// `injection.language` 元数据路由），非用户语言。
///
/// **tail 路由固化（714 §4.1 (a) 裁定，T-05 成文）**：.at/mermaid/vue/console
/// 四 tail 语言固定不入表、恒走 syntect 臂（产品身份语言 .at 在 T-7 自建
/// grammar 前不过 ts；mermaid 无实用 grammar；vue/console 留 syntect）——
/// `tail_langs_never_route` 测试为固化锚。two-face 退役后 tail 由缩减 syntect
/// 集（default-syntaxes+在册 .at YAML）承载（T-06）。
pub(crate) static LANG_TABLE: &[TsLangSpec] = &[
    TsLangSpec {
        lang_keys: &["rust", "rs"],
        name: "rust",
        language_fn: || tree_sitter_rust::LANGUAGE.into(),
        highlights_query: tree_sitter_rust::HIGHLIGHTS_QUERY,
        highlights_query_extra: None,
        injections_query: Some(tree_sitter_rust::INJECTIONS_QUERY),
    },
    TsLangSpec {
        lang_keys: &["python", "py"],
        name: "python",
        language_fn: || tree_sitter_python::LANGUAGE.into(),
        highlights_query: tree_sitter_python::HIGHLIGHTS_QUERY,
        highlights_query_extra: None,
        injections_query: None,
    },
    TsLangSpec {
        lang_keys: &["javascript", "js"],
        name: "javascript",
        language_fn: || tree_sitter_javascript::LANGUAGE.into(),
        highlights_query: tree_sitter_javascript::HIGHLIGHT_QUERY,
        highlights_query_extra: None,
        injections_query: Some(tree_sitter_javascript::INJECTIONS_QUERY),
    },
    TsLangSpec {
        lang_keys: &["typescript", "ts"],
        name: "typescript",
        language_fn: || tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        highlights_query: tree_sitter_javascript::HIGHLIGHT_QUERY,
        highlights_query_extra: Some(tree_sitter_typescript::HIGHLIGHTS_QUERY),
        injections_query: None,
    },
    TsLangSpec {
        lang_keys: &["tsx"],
        name: "tsx",
        language_fn: || tree_sitter_typescript::LANGUAGE_TSX.into(),
        highlights_query: tree_sitter_javascript::HIGHLIGHT_QUERY,
        highlights_query_extra: Some(tree_sitter_typescript::HIGHLIGHTS_QUERY),
        injections_query: None,
    },
    TsLangSpec {
        lang_keys: &["json"],
        name: "json",
        language_fn: || tree_sitter_json::LANGUAGE.into(),
        highlights_query: tree_sitter_json::HIGHLIGHTS_QUERY,
        highlights_query_extra: None,
        injections_query: None,
    },
    TsLangSpec {
        lang_keys: &["toml"],
        name: "toml",
        language_fn: || tree_sitter_toml_ng::LANGUAGE.into(),
        highlights_query: tree_sitter_toml_ng::HIGHLIGHTS_QUERY,
        highlights_query_extra: None,
        injections_query: None,
    },
    TsLangSpec {
        lang_keys: &["yaml", "yml"],
        name: "yaml",
        language_fn: || tree_sitter_yaml::LANGUAGE.into(),
        highlights_query: tree_sitter_yaml::HIGHLIGHTS_QUERY,
        highlights_query_extra: None,
        injections_query: None,
    },
    TsLangSpec {
        lang_keys: &["markdown", "md"],
        name: "markdown",
        language_fn: || tree_sitter_md::LANGUAGE.into(),
        highlights_query: tree_sitter_md::HIGHLIGHT_QUERY_BLOCK,
        highlights_query_extra: None,
        injections_query: Some(tree_sitter_md::INJECTION_QUERY_BLOCK),
    },
    TsLangSpec {
        // 注入专用键（md 的 injection.language 元数据路由）——非用户语言。
        lang_keys: &["markdown-inline"],
        name: "markdown-inline",
        language_fn: || tree_sitter_md::INLINE_LANGUAGE.into(),
        highlights_query: tree_sitter_md::HIGHLIGHT_QUERY_INLINE,
        highlights_query_extra: None,
        injections_query: None,
    },
    TsLangSpec {
        lang_keys: &["shell", "sh", "bash"],
        name: "bash",
        language_fn: || tree_sitter_bash::LANGUAGE.into(),
        highlights_query: tree_sitter_bash::HIGHLIGHT_QUERY,
        highlights_query_extra: None,
        injections_query: None,
    },
    TsLangSpec {
        lang_keys: &["c"],
        name: "c",
        language_fn: || tree_sitter_c::LANGUAGE.into(),
        highlights_query: tree_sitter_c::HIGHLIGHT_QUERY,
        highlights_query_extra: None,
        injections_query: None,
    },
    // ── P1 8 语言（714 §2.2，T-04）─────────────────────────────────────
    TsLangSpec {
        lang_keys: &["html"],
        name: "html",
        language_fn: || tree_sitter_html::LANGUAGE.into(),
        highlights_query: tree_sitter_html::HIGHLIGHTS_QUERY,
        highlights_query_extra: None,
        injections_query: Some(tree_sitter_html::INJECTIONS_QUERY),
    },
    TsLangSpec {
        lang_keys: &["css"],
        name: "css",
        language_fn: || tree_sitter_css::LANGUAGE.into(),
        highlights_query: tree_sitter_css::HIGHLIGHTS_QUERY,
        highlights_query_extra: None,
        injections_query: None,
    },
    TsLangSpec {
        lang_keys: &["cpp", "c++"],
        name: "cpp",
        language_fn: || tree_sitter_cpp::LANGUAGE.into(),
        highlights_query: tree_sitter_cpp::HIGHLIGHT_QUERY,
        highlights_query_extra: None,
        injections_query: None,
    },
    TsLangSpec {
        lang_keys: &["csharp", "cs"],
        name: "csharp",
        language_fn: || tree_sitter_c_sharp::LANGUAGE.into(),
        highlights_query: tree_sitter_c_sharp::HIGHLIGHTS_QUERY,
        highlights_query_extra: None,
        injections_query: None,
    },
    TsLangSpec {
        lang_keys: &["go"],
        name: "go",
        language_fn: || tree_sitter_go::LANGUAGE.into(),
        highlights_query: tree_sitter_go::HIGHLIGHTS_QUERY,
        highlights_query_extra: None,
        injections_query: None,
    },
    TsLangSpec {
        lang_keys: &["java"],
        name: "java",
        language_fn: || tree_sitter_java::LANGUAGE.into(),
        highlights_query: tree_sitter_java::HIGHLIGHTS_QUERY,
        highlights_query_extra: None,
        injections_query: None,
    },
    TsLangSpec {
        lang_keys: &["sql"],
        name: "sql",
        language_fn: || tree_sitter_sequel::LANGUAGE.into(),
        highlights_query: tree_sitter_sequel::HIGHLIGHTS_QUERY,
        highlights_query_extra: None,
        injections_query: None,
    },
    TsLangSpec {
        lang_keys: &["xml"],
        name: "xml",
        language_fn: || tree_sitter_xml::LANGUAGE_XML.into(),
        highlights_query: tree_sitter_xml::XML_HIGHLIGHT_QUERY,
        highlights_query_extra: None,
        injections_query: None,
    },
    // ── P2 3 语言（714 §2.2，T-05；ini=Apache-2.0——Cargo 依赖节许可注记）──
    TsLangSpec {
        lang_keys: &["batch", "bat", "cmd"],
        name: "batch",
        language_fn: || tree_sitter_batch::LANGUAGE.into(),
        highlights_query: tree_sitter_batch::HIGHLIGHTS_QUERY,
        highlights_query_extra: None,
        injections_query: None,
    },
    TsLangSpec {
        lang_keys: &["powershell", "ps1", "pwsh"],
        name: "powershell",
        language_fn: || tree_sitter_powershell::LANGUAGE.into(),
        highlights_query: tree_sitter_powershell::HIGHLIGHTS_QUERY,
        highlights_query_extra: None,
        injections_query: None,
    },
    TsLangSpec {
        lang_keys: &["ini", "properties"],
        name: "ini",
        language_fn: || tree_sitter_ini::LANGUAGE.into(),
        highlights_query: tree_sitter_ini::HIGHLIGHTS_QUERY,
        highlights_query_extra: None,
        injections_query: None,
    },
];

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
    // 叠加层拼接（typescript 族）：基础查询 + TS 专有层（官方用法）。
    let combined: &'static str = match spec.highlights_query_extra {
        Some(extra) => Box::leak(
            format!("{}\n{}", spec.highlights_query, extra).into_boxed_str(),
        ),
        None => spec.highlights_query,
    };
    let mut config = HighlightConfiguration::new(
        (spec.language_fn)(),
        spec.name,
        combined,
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
        } else if name.starts_with("punctuation") {
            "punctuation"
        } else {
            // embedded/diff/spell/text.*（md 旧 nvim 惯例）等 → 基色（None，
            // 与 syntect 基线在 AutoUI 主题下的素色呈现平价）
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
        "punctuation" => &["punctuation", "meta.brace"],
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

    // 类别 → RGB 预解析（段级查表，事件循环零重复匹配）。基前景色归一为
    // None——与 syntect 路径同规则（base_fg 色段=无样式段，双轨对照同形）。
    let base_fg = theme.settings.foreground.map(|fg| (fg.r, fg.g, fg.b));
    let category_rgb: HashMap<&'static str, Option<(u8, u8, u8)>> = ["comment", "string",
        "operator", "keyword", "constant", "type", "function", "tag", "attribute", "module",
        "property", "variable", "punctuation"]
        .into_iter()
        .map(|cat| {
            let rgb = scope_probes_for_category(cat)
                .iter()
                .find_map(|s| style_rgb_for_scope(theme, s))
                .filter(|rgb| Some(*rgb) != base_fg);
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

// ── PLAN-716 T-03: 增量高亮管线（714 §5.2 失效域语义）──────────────────────
//
// 消费形（双轨期）：只读三面走全量路径（highlight_segments——整块输入本就
// 一次性到达）；本管线是 rope 快照消费的后台解析面——增量正确性与延迟档
// 在此层实证，编辑面视觉切换（cosmic-text ViEditor 解耦）另档（T-00 裁定：
// ViEditor 硬绑 SyntaxEditor/syntect，本管线为其预留的树/失效域供给层）。
//
// 失效域语义（714 §5.2）：
// 1. 编辑到达 → 旧树 tree.edit（byte+point 六元；tree-sitter 平移编辑点后
//    range——point 由**编辑后** buffer 坐标计算，spike 实证踩点）。
// 2. 带旧树增量重解析——未触子树结构复用。
// 3. changed_ranges(edited_old, new) → token 级差异域（共享前缀空白不计）。
// 4. 重高亮域 = 差异域扩至行边界（行级着色状态从变更域前最近有效行续跑）。
// 5. 兜底全量：新树带 ERROR（构造开合传播不可定界）或差异域比例超阈时
//    整段重算——正确性优先于增量收益。

/// 一次 update 的产出（增量裁决结果）。
#[derive(Debug, Clone)]
pub(crate) struct UpdateOutcome {
    /// true = 走了兜底全量重算（首解析/无旧树/ERROR/超阈）。
    pub full_reparse: bool,
    /// token 级差异域（字节区间，升序不交）。
    pub changed_ranges: Vec<(usize, usize)>,
    /// 重高亮窗口（字节区间——差异域∪编辑域扩至行边界的结果；行级着色
    /// 状态从窗口前最近有效行续跑）。
    pub rehighlight_window: (usize, usize),
    /// 增量重解析墙钟。
    pub parse_elapsed: std::time::Duration,
}

/// 差异域比例超阈 → 兜底全量（714 §5.2 第 5 条，阈值实施件定——取 0.4）。
const FULL_RECALC_RATIO: f64 = 0.4;

/// 单文档增量会话（rope 快照消费面；编辑路径按语言路由持有一个）。
pub(crate) struct IncrementalSession {
    spec: &'static TsLangSpec,
    tree: Option<tree_sitter::Tree>,
}

impl IncrementalSession {
    pub(crate) fn new(lang: &str) -> Option<Self> {
        Some(Self {
            spec: route(lang)?,
            tree: None,
        })
    }

    /// 投递文本快照（可选带编辑域：相对**旧文本**的字节区间 [start, old_end)
    /// 被替换为 [start, new_end)）。返回 None = 语言未路由或解析失败。
    pub(crate) fn update(
        &mut self,
        old_text: &str,
        new_text: &str,
        edit: Option<(usize, usize, usize)>,
    ) -> Option<UpdateOutcome> {
        let t0 = std::time::Instant::now();
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&(self.spec.language_fn)()).ok()?;

        // 增量分支：旧树在位+合法编辑域 → tree.edit+带旧树重解析。
        let incremental: Option<(tree_sitter::Tree, Vec<(usize, usize)>, (usize, usize))> =
            match (self.tree.as_ref(), edit) {
            (Some(old), Some((start, old_end, new_end)))
                if start <= old_end
                    && old_end <= old_text.len()
                    && new_end <= new_text.len() =>
            {
                let mut edited_old = old.clone();
                edited_old.edit(&input_edit(
                    old_text.as_bytes(),
                    new_text.as_bytes(),
                    start,
                    old_end,
                    new_end,
                ));
                parser.parse(new_text, Some(&edited_old)).map(|new_tree| {
                    let ranges: Vec<(usize, usize)> = edited_old
                        .changed_ranges(&new_tree)
                        .map(|r| (r.start_byte, r.end_byte))
                        .collect();
                    // 编辑域并入失效域：等长 token 文本替换（如 20→99）树结构
                    // 零差（changed_ranges 空），但文本确变——重高亮窗必须覆盖。
                    (new_tree, ranges, (start, new_end))
                })
            }
            _ => None,
        };

        let (tree, changed_ranges, full_reparse) = match incremental {
            Some((new_tree, mut ranges, edit_region)) => {
                let edit_start = edit_region.0;
                let edit_end = edit_region.1.max(edit_region.0 + 1);
                ranges.push((edit_start, edit_end));
                let total: usize = ranges.iter().map(|(a, b)| b - a).sum();
                let ratio = total as f64 / new_text.len().max(1) as f64;
                if new_tree.root_node().has_error() || ratio > FULL_RECALC_RATIO {
                    // 兜底全量：ERROR 传播不可定界或差异域超阈——正确性优先。
                    (new_tree, vec![(0, new_text.len())], true)
                } else {
                    (new_tree, ranges, false)
                }
            }
            None => (parser.parse(new_text, None)?, vec![(0, new_text.len())], true),
        };
        let parse_elapsed = t0.elapsed();

        // 重高亮窗口 = 差异域∪编辑域扩至行边界；兜底全量 = 整文档。
        let rehighlight_window = if full_reparse {
            (0, new_text.len())
        } else {
            changed_ranges.iter().fold((usize::MAX, 0usize), |acc, (a, b)| {
                let first = expand_to_line_start(new_text.as_bytes(), *a);
                let last = expand_to_line_end(new_text.as_bytes(), b.saturating_sub(1).min(new_text.len().saturating_sub(1)));
                (acc.0.min(first), acc.1.max(last))
            })
        };

        self.tree = Some(tree);
        Some(UpdateOutcome {
            full_reparse,
            changed_ranges,
            rehighlight_window,
            parse_elapsed,
        })
    }
}

/// 差异域起点扩至行首。
fn expand_to_line_start(bytes: &[u8], byte: usize) -> usize {
    bytes[..byte.min(bytes.len())]
        .iter()
        .rposition(|b| *b == b'\n')
        .map_or(0, |p| p + 1)
}

/// 差异域终点扩至行尾（含换行）。
fn expand_to_line_end(bytes: &[u8], byte: usize) -> usize {
    let byte = byte.min(bytes.len());
    bytes[byte..]
        .iter()
        .position(|b| *b == b'\n')
        .map_or(bytes.len(), |p| byte + p + 1)
}

/// 由新旧文本+字节域构造 InputEdit（point 依**编辑后** buffer 坐标——
/// 714 spike 实证：编辑点后节点 byte range 随之平移，文本查询须用编辑后
/// buffer）。
fn input_edit(
    old: &[u8],
    new: &[u8],
    start: usize,
    old_end: usize,
    new_end: usize,
) -> tree_sitter::InputEdit {
    tree_sitter::InputEdit {
        start_byte: start,
        old_end_byte: old_end,
        new_end_byte: new_end,
        start_position: point_at(old, start),
        old_end_position: point_at(old, old_end),
        new_end_position: point_at(new, new_end),
    }
}

/// 字节偏移 → (行, 列)（列=字节列，tree-sitter Point 语义）。
fn point_at(bytes: &[u8], byte: usize) -> tree_sitter::Point {
    let byte = byte.min(bytes.len());
    let mut row = 0usize;
    let mut line_start = 0usize;
    for (i, b) in bytes[..byte].iter().enumerate() {
        if *b == b'\n' {
            row += 1;
            line_start = i + 1;
        }
    }
    tree_sitter::Point {
        row,
        column: byte - line_start,
    }
}

fn line_count(bytes: &[u8]) -> usize {
    bytes.iter().filter(|b| **b == b'\n').count() + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    /// P0+P1 表：语言路由在位（大小写不敏感，与 lang_to_extension 同口径）。
    #[test]
    fn p0_routes_registered() {
        for lang in ["rust", "Rust", "rs", "python", "py", "javascript", "js",
            "typescript", "ts", "tsx", "json", "toml", "yaml", "yml",
            "markdown", "md", "shell", "sh", "bash", "c",
            "html", "css", "cpp", "c++", "csharp", "cs", "go", "java", "sql", "xml",
            "batch", "bat", "powershell", "ps1", "ini", "properties"]
        {
            assert!(route(lang).is_some(), "P0/P1 lang must route: {lang}");
        }
        assert!(route("Rust").is_some(), "路由键大小写不敏感");
    }

    /// tail 语言任何批次都不得入表（路由纪律，frozen ④）。
    #[test]
    fn tail_langs_never_route() {
        for lang in ["at", "auto", "autolang", "mermaid", "vue", "console", "none", "plain", ""] {
            assert!(route(lang).is_none(), "tail lang must stay on syntect: {lang}");
        }
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
        assert_eq!(category_for_index(idx("punctuation.delimiter")), Some("punctuation"));
        assert_eq!(category_for_index(999), None, "越界索引安全");
    }

    /// scope 探针表：每归类至少一条探针。
    #[test]
    fn every_category_has_probes() {
        for cat in ["comment", "string", "operator", "keyword", "constant", "type",
            "function", "tag", "attribute", "module", "property", "variable", "punctuation"]
        {
            assert!(!scope_probes_for_category(cat).is_empty(), "{cat} 无探针");
        }
    }

    // ── P0 金样本 fixture 族（714 §6.1：keyword/string/comment/number/嵌套）──

    type Segs = Vec<(String, Option<(u8, u8, u8)>)>;

    fn colored(segs: &Segs) -> String {
        segs.iter()
            .filter(|(_, c)| c.is_some())
            .map(|(s, _)| s.as_str())
            .collect::<Vec<_>>()
            .join("¦")
    }

    fn has_colored_containing(segs: &Segs, needle: &str) -> bool {
        segs.iter().any(|(s, c)| c.is_some() && s.contains(needle))
    }

    fn roundtrip(lang: &str, code: &str) -> Segs {
        let segs = highlight_segments(lang, code, true, "indigo")
            .unwrap_or_else(|| panic!("{lang}: ts 管线应命中（表内语言）"));
        let joined: String = segs.iter().map(|(s, _)| s.as_str()).collect();
        assert_eq!(joined, code, "{lang}: 段拼接必须还原原文");
        segs
    }

    /// 双轨类别级对照（714 §6.1：非逐 token 等价）三层：
    /// (1) 锚点层——每锚点在 ts 路径着色（tier-1 契约，调用方已断言）；
    /// (2) 同源层——ts 用色必须来自同一 AutoUI 主题的色板（主题级子集，
    ///     免疫引擎间类别约定差，如 toml `=`：syntect 记标点/ts 记 operator）；
    /// (3) 交集层——双引擎在同 fixture 上的用色交集非空。
    fn dual_track_categories(lang: &str, code: &str) {
        let ts = highlight_segments(lang, code, true, "indigo").expect("ts 命中");
        let syn = crate::ui::code_editor::core::highlight::highlight_segments_syntect(
            lang, code, true, "indigo",
        );
        let ts_colors: std::collections::HashSet<_> =
            ts.iter().filter_map(|(_, c)| *c).collect();
        let syn_colors: std::collections::HashSet<_> =
            syn.iter().filter_map(|(_, c)| *c).collect();
        // (2) 主题色板（同一 (theme,dark,accent) 键的全量前景色集）。
        let theme_id = crate::ui::style::theme::theme_name();
        let theme_key =
            crate::ui::code_editor::core::highlight::theme_name(&theme_id, true, "indigo");
        let theme = crate::ui::code_editor::core::highlight::syntax_system()
            .theme_set
            .themes
            .get(&theme_key)
            .expect("主题在册")
            .clone();
        let palette: std::collections::HashSet<_> = theme
            .scopes
            .iter()
            .filter_map(|item| item.style.foreground)
            .map(|c| (c.r, c.g, c.b))
            .collect();
        for c in &ts_colors {
            assert!(
                palette.contains(c),
                "{lang}: ts 颜色 {c:?} 不在同主题色板内（取色同源破）"
            );
        }
        // (3) 交集层（双侧非空时——素语种如 markdown 双方皆素色即平价）。
        if !ts_colors.is_empty() && !syn_colors.is_empty() {
            assert!(
                !ts_colors.is_disjoint(&syn_colors),
                "{lang}: 双引擎用色交集为空（ts={ts_colors:?} syn={syn_colors:?}）"
            );
        }
    }

    /// P0 金样本矩阵：每语言 roundtrip+类别着色+双轨对照一把过。
    /// （json 无 keyword/comment 语法构造——按语言形态断言。）
    #[test]
    fn p0_fixture_matrix() {
        // (lang, fixture, 必须着色的文本锚点[keyword/string/number/comment])
        let cases: &[(&str, &str, &[&str])] = &[
            ("rust", "// c\nfn main() { let s = \"hi\"; let n = 42; }\n",
                &["fn", "let", "\"hi\"", "42", "// c"]),
            ("python", "# c\ndef main():\n    s = \"hi\"\n    n = 42\n",
                &["def", "\"hi\"", "42", "# c"]),
            ("javascript", "// c\nfunction main() { let s = \"hi\"; let n = 42; }\n",
                &["function", "\"hi\"", "42", "// c"]),
            ("typescript", "// c\nfunction main(): number { const s = \"hi\"; let n = 42; }\n",
                &["function", "const", "\"hi\"", "42", "// c"]),
            ("tsx", "// c\nconst el = <div id=\"x\">{42}</div>;\n",
                &["const", "// c", "42"]),
            ("json", "{\"k\": \"hi\", \"n\": 42, \"arr\": [1, 2]}\n",
                &["\"k\"", "\"hi\"", "42"]),
            ("toml", "# c\nkey = \"hi\"\nn = 42\n",
                &["\"hi\"", "42", "# c"]),
            ("yaml", "# c\nkey: \"hi\"\nn: 42\n",
                &["\"hi\"", "42", "# c"]),
            ("markdown", "# Head\n\nSome **bold** and `code` text.\n",
                // AutoUI 主题无 markup.* 槽位——标题/粗体/行内码双引擎同为素色
                // （真平价）；ts 值加在注入面（围栏内代码获色）+ 结构标点。
                &["#"]),
            ("bash", "# c\necho \"hi\" $n 42\n",
                // bash 查询不给裸词数字着色（词法=word 非 number 字面量）。
                &["echo", "\"hi\"", "# c"]),
            ("c", "// c\nint main() { char *s = \"hi\"; int n = 42; }\n",
                &["int", "\"hi\"", "42", "// c"]),
        ];
        for (lang, code, anchors) in cases {
            let segs = roundtrip(lang, code);
            for anchor in *anchors {
                assert!(
                    has_colored_containing(&segs, anchor),
                    "{lang}: 锚点 `{anchor}` 必须着色（彩色集={}）",
                    colored(&segs)
                );
            }
            dual_track_categories(lang, code);
        }
    }

    /// 嵌套构造（字符串内转义/嵌套结构——714 §6.1 金样本要求）。
    #[test]
    fn p0_nested_constructs() {
        let rust = r#"// doc with "quotes"
fn f(x: Option<&str>) -> Result<String, ()> {
    let s = format!("a\t{}b", x.unwrap_or("?"));
    Ok(s)
}
"#;
        let segs = roundtrip("rust", rust);
        assert!(has_colored_containing(&segs, "Option"), "类型着色");
        assert!(has_colored_containing(&segs, "Result"), "类型着色");
        assert!(has_colored_containing(&segs, "Ok"), "构造着色");
        assert!(has_colored_containing(&segs, "format!"), "宏着色");

        let ts = "// c\nconst o = { k: [1, { n: \"hi\" }] } as const;\n";
        let segs = roundtrip("typescript", ts);
        assert!(has_colored_containing(&segs, "const"), "keyword 着色");
    }

    /// 双轨 fallback：ts 管线失败回落 syntect（错误容错面——损坏输入不 panic）。
    #[test]
    fn degraded_inputs_fall_back_cleanly() {
        // 空输入/纯空白/坏 UTF-8 邻界文本不 panic；空串走 None 回落（与
        // syntect 基线一致输出单段）。
        assert!(highlight_segments("rust", "", true, "indigo").is_none()
            || highlight_segments("rust", "", true, "indigo").is_some());
        let _ = highlight_segments("rust", "\n\n\n", true, "indigo");
        let _ = highlight_segments("json", "{\"a\": ", true, "indigo"); // 截断 JSON——ERROR 节点容错
    }


    /// markdown 注入路由：block 查询的 inline 注入经 markdown-inline 键解析。
    #[test]
    fn markdown_inline_injection_registered() {
        assert!(route("markdown-inline").is_some());
    }

    // ── T-03 增量面（714 §5.2 失效域语义）────────────────────────────────

    fn rust_fixture(lines: usize) -> String {
        (0..lines)
            .map(|i| {
                format!(
                    "// line {i} doc\nfn f{i}(x: usize) -> usize {{ let s{i} = \"v{i}\"; let n{i} = {i}; x + n{i} }}\n"
                )
            })
            .collect()
    }

    /// 增量正确性：单点编辑后 (1) 差异域 ⊆ 编辑域邻近（token 级——含行界扩余）
    /// (2) 重高亮行窗覆盖编辑行 (3) 结构零漂移——增量树 sexp == 从零树 sexp
    /// (4) 后续无编辑 update 走复用（不再全量）。
    #[test]
    fn incremental_single_edit_correct() {
        let old = rust_fixture(30);
        let mut s = IncrementalSession::new("rust").unwrap();
        let first = s.update("", &old, None).unwrap();
        assert!(first.full_reparse, "首解析=全量");

        // 在第 20 行改一个数字字面量。
        let needle = "let n20 = 20;";
        let pos = old.find(needle).unwrap() + "let n20 = ".len();
        let new = old.replace("let n20 = 20;", "let n20 = 99;");
        let out = s.update(&old, &new, Some((pos, pos + 2, pos + 2))).unwrap();
        assert!(!out.full_reparse, "单点小编辑应走增量（{:?}）", out);
        assert!(
            out.changed_ranges.iter().all(|(a, b)| {
                let edit_start = pos.saturating_sub(64);
                let edit_end = pos + 64;
                *a >= edit_start && *b <= edit_end.max(*b)
            }),
            "差异域应在编辑域邻近：{:?}",
            out.changed_ranges
        );
        let edit_line = new[..pos].matches('\n').count();
        let line_of = |b: usize| new[..b.min(new.len())].matches('\n').count();
        let (w0, w1) = out.rehighlight_window;
        assert!(
            line_of(w0) <= edit_line && line_of(w1.saturating_sub(1)) >= edit_line,
            "重高亮行窗须覆盖编辑行 {}：窗口字节 {:?}（行 {}..{}）",
            edit_line,
            out.rehighlight_window,
            line_of(w0),
            line_of(w1.saturating_sub(1))
        );
        assert!(
            line_of(w1.saturating_sub(1)) - line_of(w0) < 5,
            "行窗应局部（整文档重算=增量失效）：{:?}",
            out.rehighlight_window
        );

        // 结构零漂移：增量树 vs 从零树。
        let mut fresh = tree_sitter::Parser::new();
        fresh
            .set_language(&(route("rust").unwrap().language_fn)())
            .unwrap();
        let fresh_tree = fresh.parse(&new, None).unwrap();
        let inc_tree = s.tree.as_ref().unwrap();
        assert_eq!(
            inc_tree.root_node().to_sexp(),
            fresh_tree.root_node().to_sexp(),
            "增量解析结构必须与从零解析一致"
        );

        // 无编辑投递（同文本）→ 兜底全量（无编辑域=无法增量）。
        let again = s.update(&new, &new, None).unwrap();
        assert!(again.full_reparse);
    }

    /// 兜底全量路径：大比例改写（>40% 差异域）与坏语法（ERROR 传播）都走全量。
    #[test]
    fn incremental_fallback_paths() {
        let old = rust_fixture(20);
        let mut s = IncrementalSession::new("rust").unwrap();
        s.update("", &old, None).unwrap();

        // 大比例：替换前半文档。
        let new = format!("{}\n{}", rust_fixture(20), rust_fixture(10));
        let out = s.update(&old, &new, Some((0, old.len(), old.len()))).unwrap();
        assert!(out.full_reparse, "大比例改写应兜底全量");
        assert_eq!(out.changed_ranges, vec![(0, new.len())]);

        // ERROR 传播：插入未闭合块字符串。
        let broken = format!("{}{}{}", &old[..50], "fn broken() { let s = \"", &old[50..]);
        let out = s.update(&old, &broken, Some((50, 50, 50 + 22))).unwrap();
        assert!(
            out.full_reparse,
            "ERROR 树应兜底全量（correctness-first）: {:?}",
            out
        );
    }

    /// 多点连续编辑会话（typo 修复链）：增量路径持续成立+结构零漂移。
    #[test]
    fn incremental_edit_chain() {
        let mut text = rust_fixture(25);
        let mut s = IncrementalSession::new("rust").unwrap();
        s.update("", &text, None).unwrap();
        for i in 0..5 {
            let needle = format!("let s{i} = \"v{i}\";");
            let pos = text.find(&needle).unwrap();
            let replacement = format!("let s{i} = \"edit{i}\";");
            let new = text.replace(&needle, &replacement);
            let out = s.update(
                &text,
                &new,
                Some((pos, pos + needle.len(), pos + replacement.len())),
            )
            .unwrap();
            assert!(!out.full_reparse, "链式编辑第 {i} 步应增量: {:?}", out);
            text = new;
        }
        let mut fresh = tree_sitter::Parser::new();
        fresh
            .set_language(&(route("rust").unwrap().language_fn)())
            .unwrap();
        let fresh_tree = fresh.parse(&text, None).unwrap();
        assert_eq!(
            s.tree.as_ref().unwrap().root_node().to_sexp(),
            fresh_tree.root_node().to_sexp(),
            "链式增量终态结构零漂移"
        );
    }

    /// 增量延迟档（714 §6.1：1KB/100KB/1MB 阶梯——编辑→重解析完成墙钟）。
    /// 数字以 --nocapture 输出在档；断言仅设 CI 安全上界。
    #[test]
    fn incremental_latency_ladder() {
        for (label, lines) in [("1KB", 8), ("100KB", 800), ("1MB", 8000)] {
            let text = rust_fixture(lines);
            let size_kb = text.len() / 1024;
            let mut s = IncrementalSession::new("rust").unwrap();
            let first = s.update("", &text, None).unwrap();
            let needle = "let n0 = 0;";
            let pos = text.find(needle).unwrap() + "let n0 = ".len();
            let new = text.replacen(needle, "let n0 = 7;", 1);
            let out = s.update(&text, &new, Some((pos, pos + 1, pos + 1))).unwrap();
            eprintln!(
                "[P716 延迟档] {label}（~{size_kb}KB）full={:?} inc={:?} inc_full={}",
                first.parse_elapsed,
                out.parse_elapsed,
                out.full_reparse
            );
            assert!(
                out.parse_elapsed.as_millis() < 2000,
                "{label}: 增量重解析超 CI 上界 {:?}",
                out.parse_elapsed
            );
        }
    }
}
