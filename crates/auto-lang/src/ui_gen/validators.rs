// SFC 生成后校验（Plan 361）
//
// 在 VueGenerator::generate_sfc() 返回前对生成的 SFC 字符串做一组纯文本/正则级检查，
// 发现违反"生成器契约"的情况（同名组件 key 冲突、store 用了没 import、handler 引用了没定义等）。
//
// 设计目标：
//   - 纯文本分析，不做完整 JS/TS 解析（避免引入 tree-sitter 等重依赖）
//   - 不阻塞生成，只打印警告；但可通过 ValidationContext.strict 让 auto build 失败
//   - 规则可单元测试：每条规则一个 fn，输入 SFC 字符串，输出 Vec<ValidationWarning>
//
// 与 generate_component_from_file（Plan 361 §3）的关系：
//   校验在 generate_sfc 末尾自动运行，也会在 generate_component_from_file 的产物上再跑一次。

use std::collections::HashMap;

// ============================================================================
// 类型定义
// ============================================================================

/// 校验规则的严重度。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    /// 生成产物几乎肯定无法正常工作。strict 模式下 `auto build` 会失败。
    Error,
    /// 生成产物能跑，但有已知陷阱模式或可疑代码。建议人工 review。
    Warning,
    /// 可能是问题，也可能是合理的写法。仅信息性提示。
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Error => write!(f, "ERROR"),
            Severity::Warning => write!(f, "WARNING"),
            Severity::Info => write!(f, "INFO"),
        }
    }
}

/// 单条校验警告。
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    /// 规则 ID，如 "R001"。用于 stable 引用和测试断言。
    pub rule: &'static str,
    pub severity: Severity,
    /// 所在 widget 名（SFC 名）。
    pub widget: String,
    /// 人类可读的说明，包含足够上下文让开发者定位问题。
    pub message: String,
    /// 建议的修复方向（可选）。
    pub fix_hint: Option<String>,
}

impl ValidationWarning {
    /// Create a warning (pub since Plan 012 Batch A: generators outside this
    /// module raise codegen warnings through the same channel).
    pub fn new(rule: &'static str, severity: Severity, widget: &str, message: impl Into<String>) -> Self {
        Self {
            rule,
            severity,
            widget: widget.to_string(),
            message: message.into(),
            fix_hint: None,
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.fix_hint = Some(hint.into());
        self
    }
}

/// 校验时的上下文信息。由调用方（VueGenerator）提供。
///
/// 生成器本身比纯 SFC 字符串知道更多信息（如 store_deps 是否声明、handler 是否为空），
/// 这些信息通过 ctx 传给校验规则，让它们能做更精确的判断。
#[derive(Debug, Default, Clone)]
pub struct ValidationContext {
    /// 这个 SFC 声明了哪些 store 依赖（`use store: X` 提取出来的）。
    /// 用于 R002：store 用了但没 import。
    pub store_deps: Vec<String>,
    /// 项目是否依赖 @autodown/editor（来自 pac.at 的 npm_deps）。
    /// 用于 R003：用了 AutoDownEditor 但 main.ts 没导入 CSS。
    pub uses_autodown: bool,
    /// 生成器检测到的、模板里引用的 handler 名集合（不含前导点）。
    /// 用于 R004：模板引用了 handler 但 script 里没定义。
    pub used_handlers: Vec<String>,
    /// 是否为 strict 模式（有 ERROR 时让 build 失败）。
    pub strict: bool,
}

// ============================================================================
// Plan 012 Batch A: unified warning channel — strict mode + print-once
// ============================================================================

/// Process-wide strict flag. When set, any non-Info validation warning makes
/// the build fail (see `generate_component_from_file`). Set by
/// `auto build --strict`; read by every codegen path that goes through the
/// unified entry point.
static STRICT_MODE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Enable/disable strict mode (warning escalation to build failure).
pub fn set_strict(strict: bool) {
    STRICT_MODE.store(strict, std::sync::atomic::Ordering::Relaxed);
}

/// Whether strict mode is active.
pub fn strict_enabled() -> bool {
    STRICT_MODE.load(std::sync::atomic::Ordering::Relaxed)
}

/// True when the warning list contains anything that should fail a strict
/// build (Warning or Error severity; Info is advisory only).
pub fn has_blocking_warnings(warnings: &[ValidationWarning]) -> bool {
    warnings.iter().any(|w| w.severity != Severity::Info)
}

/// Print warnings to stderr, deduplicated process-wide per
/// (file, rule, widget, message). Used by build drivers whose pipeline calls
/// the generator more than once for the same file (e.g. auto-man's
/// sub-widget pre-scan + real pass) so each distinct warning prints once per
/// process instead of twice.
pub fn print_warnings_once(file_tag: &str, warnings: &[ValidationWarning]) {
    use std::sync::{Mutex, OnceLock};
    static SEEN: OnceLock<Mutex<std::collections::HashSet<String>>> = OnceLock::new();
    let seen = SEEN.get_or_init(|| Mutex::new(std::collections::HashSet::new()));
    let mut fresh: Vec<ValidationWarning> = Vec::new();
    {
        let mut seen = seen.lock().unwrap();
        for w in warnings {
            let key = format!("{}|{}|{}|{}", file_tag, w.rule, w.widget, w.message);
            if seen.insert(key) {
                fresh.push(w.clone());
            }
        }
    }
    if !fresh.is_empty() {
        eprintln!("{}", format_warnings(&fresh));
    }
}

// ============================================================================
// 入口：对单个 SFC 跑所有规则
// ============================================================================

/// 对生成的 SFC 跑所有校验规则。
///
/// `sfc` 是完整的 .vue 文件内容。`widget_name` 是组件名（如 "EditorPanel"）。
/// `ctx` 提供生成器知道的额外上下文。
pub fn validate_sfc(sfc: &str, widget_name: &str, ctx: &ValidationContext) -> Vec<ValidationWarning> {
    let mut warnings = Vec::new();
    warnings.extend(r001_duplicate_component_key(sfc, widget_name));
    warnings.extend(r002_store_usage_without_import(sfc, widget_name, ctx));
    warnings.extend(r003_autodown_css_missing(sfc, widget_name, ctx));
    warnings.extend(r004_undefined_handler(sfc, widget_name, ctx));
    warnings.extend(r005_emit_without_declaration(sfc, widget_name));
    warnings.extend(r006_v_for_without_key(sfc, widget_name));
    warnings.extend(r007_autodown_dual_instance(sfc, widget_name));
    warnings.extend(r009_define_expose_undefined(sfc, widget_name));
    warnings
}

/// 便捷方法：把警告格式化成人类可读的多行字符串（用于打印到 stderr）。
pub fn format_warnings(warnings: &[ValidationWarning]) -> String {
    if warnings.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    for w in warnings {
        out.push_str(&format!(
            "  [{} {}] {}\n    {}\n",
            w.rule, w.severity, w.widget, w.message
        ));
        if let Some(ref hint) = w.fix_hint {
            out.push_str(&format!("    Fix: {}\n", hint));
        }
    }
    out
}

// ============================================================================
// R001: 同名组件 key 冲突
// ============================================================================

/// R001: 模板内同名组件的 `:key` 必须互不相同。
///
/// 本次会话最痛的问题：两个 `<AutoDownEditor>` 在不同 v-if 分支，都拿到固定 key，
/// Vue patch 而非 remount → Tiptap 初始化失败 → 编辑框空白。
fn r001_duplicate_component_key(sfc: &str, widget: &str) -> Vec<ValidationWarning> {
    let template = extract_template(sfc);
    let component_keys = collect_component_keys(&template);
    let mut warnings = Vec::new();

    // 按组件名分组，找同名组件里 key 重复或缺失的
    let mut by_tag: HashMap<String, Vec<Option<String>>> = HashMap::new();
    for (tag, key) in &component_keys {
        // 只关注 PascalCase 标签（Vue 组件），跳过原生 HTML
        if tag.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
            by_tag.entry(tag.clone()).or_default().push(key.clone());
        }
    }

    for (tag, keys) in &by_tag {
        // 只有一个实例的不会冲突
        if keys.len() < 2 {
            continue;
        }
        // 检查是否有重复的 key 值
        let mut seen: HashMap<String, usize> = HashMap::new();
        for k in keys {
            match k {
                Some(key_val) => *seen.entry(key_val.clone()).or_insert(0) += 1,
                None => {} // 缺 key 由 R006/v-for 规则处理
            }
        }
        for (key_val, count) in &seen {
            if *count > 1 {
                warnings.push(
                    ValidationWarning::new(
                        "R001",
                        Severity::Error,
                        widget,
                        format!(
                            "Duplicate :key=\"{}\" on <{}> ({} instances share this key). \
                             Vue will patch in place instead of remounting when switching v-if branches, \
                             which breaks components that rely on fresh mount (e.g. Tiptap editor).",
                            key_val, tag, count
                        ),
                    )
                    .with_hint(format!(
                        "Give each <{}> instance a unique key, or restructure as a single \
                         instance whose props drive the mode switch.",
                        tag
                    )),
                );
            }
        }
    }

    warnings
}

/// 从 SFC 字符串里提取 `<template>...</template>` 内容。
fn extract_template(sfc: &str) -> String {
    let start = match sfc.find("<template>") {
        Some(i) => i + "<template>".len(),
        None => return String::new(),
    };
    let end = match sfc.rfind("</template>") {
        Some(i) => i,
        None => return sfc[start..].to_string(),
    };
    sfc[start..end].to_string()
}

/// 从模板内容里收集所有组件标签及其 `:key` 值（如果有）。
///
/// 返回 (tag_name, key_value_option) 列表。
/// 仅做正则级匹配，不做完整 HTML 解析。
fn collect_component_keys(template: &str) -> Vec<(String, Option<String>)> {
    let mut result = Vec::new();
    // 匹配 Vue 组件标签：首字母大写的标识符，支持自闭合和多行
    // 简化策略：找所有 `<TagName` 形式的 token，然后在同一个标签内找 `:key="..."`
    let tag_re = regex_lite(r"<([A-Z][A-Za-z0-9]*)\b([^>]*)");

    for cap in tag_re.captures_iter(template) {
        let tag = cap.group(1).to_string();
        let attrs = cap.group(2);

        // 在 attrs 里找 :key="..." 或 :key='...'
        let key = find_attr_value(attrs, ":key")
            .or_else(|| find_attr_value(attrs, ":key".into())); // 容错
        result.push((tag, key));
    }
    result
}

/// 在属性字符串里找 `name="value"` 或 `name='value'` 的 value 部分。
fn find_attr_value(attrs: &str, name: &str) -> Option<String> {
    // 先找 name="..." 形式
    let patterns = [
        format!(r#"{}\s*=\s*"([^"]*)""#, regex_escape(name)),
        format!(r"{}\s*=\s*'([^']*)'", regex_escape(name)),
    ];
    for pat in &patterns {
        let re = regex_lite(pat);
        if let Some(cap) = re.captures(attrs) {
            return Some(cap.group(1).to_string());
        }
    }
    None
}

// ============================================================================
// R002: store 使用了但没 import
// ============================================================================

/// R002: script 引用了 `store.X` 但没有 `import { useXStore }`。
///
/// 本次会话 store_deps 丢失的症状：生成的 .vue 里直接 `store.notes` 但没 import store。
fn r002_store_usage_without_import(
    sfc: &str,
    widget: &str,
    _ctx: &ValidationContext,
) -> Vec<ValidationWarning> {
    // 在 script 段找 `store\.\w+` 引用
    let script = extract_script(sfc);
    if script.is_empty() {
        return vec![];
    }

    let store_usage_re = regex_lite(r"\bstore\.([a-zA-Z_]\w*)");
    let mut uses_store = false;
    for cap in store_usage_re.captures_iter(&script) {
        // 排除注释行（简化检查）
        let _field = cap.group(1);
        uses_store = true;
        break; // 只需知道是否有引用
    }

    if !uses_store {
        return vec![];
    }

    // 检查是否有 `import { useXxxStore }` 或 `const store = ...Store`
    let import_re = regex_lite(r"import\s*\{\s*use\w+Store\s*\}");
    let const_re = regex_lite(r"const\s+store\s*=");
    if import_re.is_match(&script) && const_re.is_match(&script) {
        return vec![];
    }

    // 检查是否是 store composable 本身（它用 `export function useXxxStore`，不会 import 自己）
    let is_store_def = regex_lite(r"export\s+function\s+use\w+Store").is_match(&script);
    if is_store_def {
        return vec![];
    }

    vec![ValidationWarning::new(
        "R002",
        Severity::Error,
        widget,
        "Script references `store.X` but has no `import { useXxxStore }` or \
         `const store = ...` declaration. The generated component will fail at \
         runtime with 'store is not defined'."
            .to_string(),
    )
    .with_hint(
        "Ensure the .at file declares `use store: XxxStore`, and that \
         generate_component_from_file is propagating store_deps (Plan 361 §3).",
    )]
}

/// 从 SFC 提取 `<script setup ...>...</script>` 内容。
fn extract_script(sfc: &str) -> String {
    // 匹配 <script setup ...> 或 <script>
    let start_re = regex_lite(r"<script[^>]*>");
    let start = match start_re.find(sfc) {
        Some(m) => m.end(),
        None => return String::new(),
    };
    let end = match sfc[start..].find("</script>") {
        Some(i) => start + i,
        None => return sfc[start..].to_string(),
    };
    sfc[start..end].to_string()
}

// ============================================================================
// R003: 用了 AutoDownEditor 但 main.ts 没导入 CSS
// ============================================================================

/// R003: 模板含 AutoDownEditor 但 main.ts 缺少 `@autodown/editor/style.css` 导入。
///
/// 本次会话症状：底部出现奇怪的 `+` 号，因为 AutoDownEditor 的 CSS 默认 opacity:0，
/// 没 import 样式表 → CSS 不生效 → `+` 一直可见。
///
/// 注意：这个规则需要跨文件信息（main.ts）。当 main_ts_content 为 None 时，
/// 若 ctx.uses_autodown 为 true 仍能发出警告（提示需要确保 main.ts 导入）。
fn r003_autodown_css_missing(
    sfc: &str,
    widget: &str,
    ctx: &ValidationContext,
) -> Vec<ValidationWarning> {
    let template = extract_template(sfc);
    if !template.contains("AutoDownEditor") {
        return vec![];
    }
    if !ctx.uses_autodown {
        return vec![];
    }
    // 生成器层：既然这个 SFC 用了 AutoDownEditor 且项目依赖了 @autodown/editor，
    // 唯一可能的失效点是 generate_main_ts 没有注入 CSS import。
    // 我们无法在单个 SFC 视角看到 main.ts，这里只做信息性提示：
    // 真正的跨文件检查由 generate_component_from_file 在工程层面做。
    vec![ValidationWarning::new(
        "R003",
        Severity::Info,
        widget,
        "Template uses <AutoDownEditor>. Make sure main.ts imports \
         '@autodown/editor/style.css' (auto-injected by generate_main_ts when \
         npm_deps includes @autodown/editor)."
            .to_string(),
    )]
}

// ============================================================================
// R004: 模板引用了 handler 但 script 没定义
// ============================================================================

/// R004: `@click="X"` 的 X 未在 script 里定义为函数。
///
/// 本次会话症状：Cancel 点击无反应，因为 handler 引用了但 on 块没定义。
/// 当 ctx.used_handlers 提供时，优先用它（更精确）；否则从模板里正则提取。
fn r004_undefined_handler(sfc: &str, widget: &str, ctx: &ValidationContext) -> Vec<ValidationWarning> {
    let template = extract_template(sfc);
    let script = extract_script(sfc);

    // 提取模板里所有 @xxx="Y" / @xxx="Y(args)" 引用的 handler 名
    // Only treat as a handler reference when the identifier is followed by `(`
    // (call) or the closing `"` (bare reference). This excludes inline
    // assignment expressions like `@click="foo = !foo"` or `@click="bar = 'x'"`
    // which toggle/set a ref directly and need no function definition —
    // otherwise R004 false-positives every Code/Tab toggle button.
    let handler_ref_re = regex_lite(r#"@\w+(?:\.\w+)*\s*=\s*"([a-zA-Z_]\w*)\s*(?:\(|")"#);
    let mut referenced: Vec<String> = Vec::new();
    for cap in handler_ref_re.captures_iter(&template) {
        referenced.push(cap.group(1).to_string());
    }
    if referenced.is_empty() {
        return vec![];
    }

    // 从 script 里找所有定义的 function 名
    let func_def_re = regex_lite(r"(?:async\s+)?function\s+([a-zA-Z_]\w*)");
    let mut defined: std::collections::HashSet<String> = std::collections::HashSet::new();
    for cap in func_def_re.captures_iter(&script) {
        defined.insert(cap.group(1).to_string());
    }
    // Vue 内置/隐式 handler：不报警
    let builtins: &[&str] = &["$event"];
    for b in builtins {
        defined.insert((*b).to_string());
    }

    // 同时记录 ctx.used_handlers（生成器已知）作为可信集合
    let generator_known: std::collections::HashSet<&str> =
        ctx.used_handlers.iter().map(|s| s.as_str()).collect();

    let mut warnings = Vec::new();
    let mut already_reported = std::collections::HashSet::new();
    for name in &referenced {
        if already_reported.contains(name) {
            continue;
        }
        // 生成器知道这个 handler 是 used 的 → 它一定定义了（可能函数体为空，那由 R007 管）
        if generator_known.contains(name.as_str()) {
            continue;
        }
        if !defined.contains(name) {
            already_reported.insert(name.clone());
            warnings.push(
                ValidationWarning::new(
                    "R004",
                    Severity::Warning,
                    widget,
                    format!(
                        "Template references @handler \"{}\" but no `function {}()` is defined \
                         in <script setup>. The generated stub will be empty and clicks will do nothing.",
                        name, name
                    ),
                )
                .with_hint(format!(
                    "Add `.{} -> {{ ... }}` to the `on {{}}` block in the .at file.",
                    name
                )),
            );
        }
    }
    warnings
}

// ============================================================================
// R005: emit('X') 但 defineEmits 没声明 X
// ============================================================================

/// R005: script 里调用了 `emit('X')` 但 defineEmits 里没声明 X。
fn r005_emit_without_declaration(sfc: &str, widget: &str) -> Vec<ValidationWarning> {
    let script = extract_script(sfc);
    if script.is_empty() {
        return vec![];
    }

    // 提取 emit('X') / emit("X") 的 X
    let emit_call_re = regex_lite(r#"\bemit\s*\(\s*['"]([^'"]+)['"]"#);
    let mut emitted: Vec<String> = Vec::new();
    for cap in emit_call_re.captures_iter(&script) {
        emitted.push(cap.group(1).to_string());
    }
    if emitted.is_empty() {
        return vec![];
    }

    // 提取 defineEmits<{ X: [...] }>() 里声明的 event 名
    let emit_decl_re = regex_lite(r"defineEmits\s*<\s*\{([^}]*)\}");
    let mut declared: std::collections::HashSet<String> = std::collections::HashSet::new();
    for cap in emit_decl_re.captures_iter(&script) {
        let body = cap.group(1);
        // body 形如 "X: []\n  'update:open': [boolean]"，取冒号前的名字。
        // Plan 015: quoted msg variants emit quoted keys — accept both the
        // bare-identifier form and the quoted form (R005 used to false-positive
        // on every quoted event).
        let name_re = regex_lite(r#"["']?([a-zA-Z_][\w:.-]*)["']?\s*:"#);
        for nc in name_re.captures_iter(body) {
            declared.insert(nc.group(1).to_string());
        }
    }

    let mut warnings = Vec::new();
    let mut reported = std::collections::HashSet::new();
    for name in &emitted {
        if reported.contains(name) {
            continue;
        }
        if !declared.contains(name) {
            reported.insert(name.clone());
            warnings.push(ValidationWarning::new(
                "R005",
                Severity::Warning,
                widget,
                format!(
                    "Script calls `emit('{}')` but '{}' is not declared in `defineEmits<{{...}}>()`. \
                     Vue will drop the event silently.",
                    name, name
                ),
            ));
        }
    }
    warnings
}

// ============================================================================
// R006: v-for 没有 key
// ============================================================================

/// R006: `<div v-for="...">` 缺少 `:key` 绑定。
///
/// Vue 会警告，但我们也在生成器层面提醒，确保 for 循环都有 key。
fn r006_v_for_without_key(sfc: &str, widget: &str) -> Vec<ValidationWarning> {
    let template = extract_template(sfc);
    let vfor_re = regex_lite(r"<(\w+)\s+([^>]*v-for[^>]*)");

    let mut warnings = Vec::new();
    let mut reported_tags = std::collections::HashSet::new();
    for cap in vfor_re.captures_iter(&template) {
        let tag = cap.group(1).to_string();
        let attrs = cap.group(2);
        if attrs.contains(":key") || attrs.contains("v-bind:key") {
            continue;
        }
        let dedup_key = tag.clone();
        if reported_tags.insert(dedup_key) {
            warnings.push(ValidationWarning::new(
                "R006",
                Severity::Warning,
                widget,
                format!(
                    "`<{} v-for=\"...\">` is missing a :key binding. Vue requires keys for \
                     correct list item identity (reorder/insert/delete may misbehave).",
                    tag
                ),
            ));
        }
    }
    warnings
}

// ============================================================================
// R007: 同一模板内出现 ≥2 个 AutoDownEditor（已知脆弱模式）
// ============================================================================

/// R007: 同一模板出现 ≥2 个 AutoDownEditor，通常意味着"双实例 v-if 切换"反模式。
///
/// 这是本次会话最典型的陷阱：读/写两个 editor 在两个 v-if 分支，切换时触发 Tiptap
/// 生命周期错误。生成器（Plan 360 已修）会给它们不同 key，但根本解决是单实例 + prop 切换。
fn r007_autodown_dual_instance(sfc: &str, widget: &str) -> Vec<ValidationWarning> {
    let template = extract_template(sfc);
    let count = template.matches("AutoDownEditor").count();
    // 一个 AutoDownEditor 标签会出现 2 次（开标签 + 可能的引用），我们数开标签
    let open_count = regex_lite(r"<AutoDownEditor\b").find_iter(&template).count();
    if open_count < 2 {
        return vec![];
    }
    vec![ValidationWarning::new(
        "R007",
        Severity::Info,
        widget,
        format!(
            "Template has {} <AutoDownEditor> instances. If these sit in different v-if branches \
             (read/edit mode switching), consider consolidating to a single instance with \
             `:content` and `:can-edit` props driven by editing state. This avoids Tiptap \
             mount/unmount lifecycle issues.",
            open_count
        ),
    )
    .with_hint(
        "See editor-integration pattern (Plan 363) for the single-instance approach.",
    )]
}

// ============================================================================
// R009: defineExpose references a name not defined in <script setup>
// ============================================================================

/// R009: `defineExpose({ X })` 中的 X 在 script 里没有对应定义。
///
/// Plan 012 Batch A (gap 45): an exposed `on` handler that was never generated
/// (e.g. parameterized handler not counted as used) leaves `defineExpose({ Open })`
/// pointing at nothing — the reference silently resolves to a GLOBAL at runtime
/// (`window.open`!), and vue-tsc does not flag it. This rule is the safety net:
/// every exposed name must have a `function X` / `const X` / `let X` definition
/// or appear in an import statement.
fn r009_define_expose_undefined(sfc: &str, widget: &str) -> Vec<ValidationWarning> {
    let script = extract_script(sfc);
    if script.is_empty() {
        return vec![];
    }

    // Find defineExpose({ ... }) — single or multi-line object literal.
    let expose_re = regex_lite(r"(?s)defineExpose\s*\(\s*\{(.*?)\}\s*\)");
    let Some(cap) = expose_re.captures(&script) else {
        return vec![];
    };
    let body = cap.group(1);

    // Exposed names: comma-separated identifiers (shorthand entries).
    let name_re = regex_lite(r"[a-zA-Z_$][\w$]*");
    let mut names: Vec<String> = Vec::new();
    for nc in name_re.captures_iter(body) {
        names.push(nc.group(0).to_string());
    }
    if names.is_empty() {
        return vec![];
    }

    let mut warnings = Vec::new();
    for name in names {
        // Defined as a function, a const/let, destructured from defineProps,
        // or brought in by an import statement?
        let def_re = regex_lite(&format!(
            r"(?:\bfunction\s+{0}\b)|(?:\bconst\s+(?:\{{\s*[^}}]*\b{0}\b|\b{0}\b))|(?:\blet\s+{0}\b)",
            regex_escape(&name)
        ));
        if def_re.is_match(&script) {
            continue;
        }
        let import_re = regex_lite(&format!(
            r"(?m)^\s*import\s+[^;]*\b{0}\b[^;]*from",
            regex_escape(&name)
        ));
        if import_re.is_match(&script) {
            continue;
        }
        warnings.push(
            ValidationWarning::new(
                "R009",
                Severity::Warning,
                widget,
                format!(
                    "defineExpose references '{}' but no `function {}`, `const {}`, or import \
                     defines it in <script setup>. At runtime the reference silently resolves \
                     to a global binding (e.g. `open` → window.open) — the parent calling this \
                     exposed member hits the wrong function or undefined.",
                    name, name, name
                ),
            )
            .with_hint(
                "Expose only members the widget actually defines: an `on` handler, a model \
                 var, a computed, a template ref, or a `use { fn: ... }` import.",
            ),
        );
    }
    warnings
}

// ============================================================================
// R016: view 元素与 parser hard keyword 撞名（Plan 015 P1#8）
// ============================================================================

/// R016: 扫 widget 的 view AST，发现与 hard keyword 撞名的元素用法。
///
/// Plan 015 P1#8（jade gaps 18/29/34/53，tmp/dsl-probes/auto-down 探针仲裁）：
/// `view`/`task` 是 lexer hard keyword（TokenKind::View/Task），在 view 块里
/// 被当作元素名时 parser 不报错、生成器静默降级为 `<div>`（探针
/// kw-elem-view/kw-elem-task）；`link` 不带 `to:` 时静默生成
/// `<router-link to="">`（kw-elem-link）。model 字段命名为 `view` 时，
/// `text .view` 里的 `.view` 会被 lex 成 View token，在 view 树里漏出一个
/// 垃圾 `view` 元素节点（kw-model-view-ref）——同样被本规则的 `view`
/// 元素检查捕获。
///
/// 探针确认无碰撞、不误报的场景：`path`（model/computed/局部/for 变量均
/// 正常）；prop 的 `type:`/`as:`/`to:` 块内与圆括号写法（Plan 012 P2 已
/// 上下文敏感化）。
///
/// 与 R001-R009 不同，本规则的输入是 aura view AST（不是 SFC 文本），由
/// generate_component_from_file 在 widget 提取后调用，strict 模式下经
/// has_blocking_warnings 让 build 失败。
///
/// 命名注记：plan 015 文档里本规则记作 "R014"，但 R014（v-html children
/// 忽略，vue.rs:4432）与 R015 均已被占用，故取下一空号 R016。
pub fn r016_keyword_collision(
    view_tree: &crate::aura::AuraNode,
    widget_name: &str,
) -> Vec<ValidationWarning> {
    let mut warnings = Vec::new();
    r016_walk(view_tree, widget_name, &mut warnings);
    warnings
}

fn r016_walk(
    node: &crate::aura::AuraNode,
    widget: &str,
    warnings: &mut Vec<ValidationWarning>,
) {
    use crate::aura::AuraNode;
    match node {
        AuraNode::Element { tag, children, .. } => {
            match tag.as_str() {
                "view" | "task" => warnings.push(
                    ValidationWarning::new(
                        "R016",
                        Severity::Error,
                        widget,
                        format!(
                            "Element `{}` collides with a parser hard keyword: it parses \
                             without error but is silently emitted as a plain `<div>` \
                             (jade gaps 18/29). The intended element is lost.",
                            tag
                        ),
                    )
                    .with_hint(format!(
                        "Rename the element (e.g. `panel`, `card`), or use a supported \
                         container like `col`/`row`. If this came from a model field \
                         named `{}` referenced as `text .{}`, rename the field.",
                        tag, tag
                    )),
                ),
                _ => {}
            }
            // `text .view`（model 字段名撞 hard keyword）：lexer 把 `.view`
            // 吐成一个整体 token，dot-primary 路径不触发，漏出 tag 形如
            // `.view ` 的垃圾元素节点（静默降级为 `<div/>`）。合法元素名
            // 永远不会以 `.` 开头。
            let trimmed = tag.trim();
            if trimmed.starts_with('.') {
                warnings.push(
                    ValidationWarning::new(
                        "R016",
                        Severity::Error,
                        widget,
                        format!(
                            "Element tag `{}` is not a valid element name — it looks like \
                             a model/computed reference (e.g. `text .view`) whose field \
                             name collides with a hard keyword, so the reference leaked \
                             into the view tree as a garbage node (emitted as `<div/>`).",
                            trimmed
                        ),
                    )
                    .with_hint(
                        "Rename the referenced field so it is not a hard keyword \
                         (e.g. `view` → `view_mode`, `task` → `task_item`).",
                    ),
                );
            }
            for child in children {
                r016_walk(child, widget, warnings);
            }
        }
        AuraNode::Link { to, href, children, .. } => {
            if to.is_empty() && href.is_empty() {
                warnings.push(
                    ValidationWarning::new(
                        "R016",
                        Severity::Error,
                        widget,
                        "`link` without a `to:` prop parses as a router link with an \
                         empty target — emitted as `<router-link to=\"\">`, navigating \
                         nowhere (jade gap 34/53)."
                            .to_string(),
                    )
                    .with_hint(
                        "Add `to: \"/path\"` for a router link, or rename the element \
                         if a plain anchor/container was intended.",
                    ),
                );
            }
            for child in children {
                r016_walk(child, widget, warnings);
            }
        }
        AuraNode::ForLoop { body, .. } => {
            for child in body {
                r016_walk(child, widget, warnings);
            }
        }
        AuraNode::Conditional { then_body, else_body, .. } => {
            for child in then_body {
                r016_walk(child, widget, warnings);
            }
            if let Some(else_nodes) = else_body {
                for child in else_nodes {
                    r016_walk(child, widget, warnings);
                }
            }
        }
        AuraNode::Component { children, .. } => {
            for child in children {
                r016_walk(child, widget, warnings);
            }
        }
        _ => {}
    }
}

// ============================================================================
// 极简正则工具
// ============================================================================

/// 编译一个硬编码的正则（编译期已确保 pattern 合法，所以 unwrap 安全）。
fn regex_lite(pat: &str) -> regex::Regex {
    regex::Regex::new(pat).expect("hardcoded regex must compile")
}

/// 转义字符串使其能作为正则的字面量。
fn regex_escape(s: &str) -> String {
    regex::escape(s)
}

/// 为 regex::Captures 提供便捷的 `.group(n)` 方法（取第 n 个捕获组，1-indexed）。
/// 原生 API 是 `caps.get(n).unwrap().as_str()`，太啰嗦。
trait CapturesExt<'a> {
    fn group(&self, n: usize) -> &'a str;
}
impl<'a> CapturesExt<'a> for regex::Captures<'a> {
    fn group(&self, n: usize) -> &'a str {
        self.get(n).map(|m| m.as_str()).unwrap_or("")
    }
}

// ============================================================================
// Plan 435 P2:schema 驱动的 view 校验
// ============================================================================

/// schema 校验的解析候选集:schema 元素 + 本项目 widget/子件名 + ext 组件名。
pub struct SchemaResolveScope {
    /// 折叠键(fold:剥 -/_ + 小写)集合,来自 widget 名与已知子件名
    local_fold: std::collections::HashSet<String>,
}

fn fold(s: &str) -> String {
    s.chars()
        .filter(|c| *c != '-' && *c != '_')
        .collect::<String>()
        .to_lowercase()
}

impl SchemaResolveScope {
    pub fn from_widgets(
        widgets: &[crate::aura::AuraWidget],
        known_sub_widgets: &[String],
    ) -> Self {
        let mut local_fold = std::collections::HashSet::new();
        for name in known_sub_widgets {
            local_fold.insert(fold(name));
        }
        for w in widgets {
            local_fold.insert(fold(&w.name));
            // ext 组件(`use { component: X from "..." }`)是合法 tag 来源
            for imp in &w.ext_imports {
                if matches!(imp.kind, crate::ast::ExtImportKind::Component) {
                    for s in &imp.symbols {
                        local_fold.insert(fold(&s.to_string()));
                    }
                }
            }
        }
        SchemaResolveScope { local_fold }
    }

    fn is_local(&self, tag: &str) -> bool {
        self.local_fold.contains(&fold(tag))
    }
}

/// 通用 prop(任何元素都可挂,不参与"未声明 prop"判定)。
const UNIVERSAL_PROPS: &[&str] = &["class", "style", "id", "key", "if", "for"];

/// Plan 435 P2:用 schema/aura.at 校验 widget 的 view 树。
/// - 未知 tag(schema 未声明、非本地 widget/子件/ext 组件)→ Warning + 拼写建议
/// - 已知元素(声明过 props)上出现未声明 prop → Info
/// advisory:仅 --strict(`auto build --strict`)下 Warning 及以上会阻断。
pub fn validate_aura_against_schema(
    widgets: &[crate::aura::AuraWidget],
    known_sub_widgets: &[String],
) -> Vec<ValidationWarning> {
    let schema = match crate::aura::load_default_schema() {
        Ok(s) => s,
        Err(_) => return Vec::new(), // schema 加载失败静默跳过(不阻塞构建)
    };
    let scope = SchemaResolveScope::from_widgets(widgets, known_sub_widgets);
    let mut out = Vec::new();

    for widget in widgets {
        walk_node(
            &widget.view_tree,
            &widget.name,
            &schema,
            &scope,
            &mut out,
        );
    }
    out
}

fn walk_node(
    node: &crate::aura::AuraNode,
    widget_name: &str,
    schema: &crate::aura::schema::AuraSchema,
    scope: &SchemaResolveScope,
    out: &mut Vec<ValidationWarning>,
) {
    use crate::aura::AuraNode;
    match node {
        AuraNode::Element { tag, props, children, .. } => {
            if let Some((_canon, def)) = schema.resolve_tag(tag) {
                // prop 校验:仅当元素声明过 props(空 props = P2 待补,跳过)
                if !def.props.is_empty() {
                    for p in props.keys() {
                        let universal = UNIVERSAL_PROPS.contains(&p.as_str())
                            || p.starts_with("on")
                            || p.ends_with("-if");
                        if !universal && def.get_prop(p).is_none() {
                            out.push(
                                ValidationWarning::new(
                                    "S001",
                                    Severity::Info,
                                    widget_name,
                                    format!(
                                        "prop `{}` not declared on `<{}>` (schema declares: {})",
                                        p,
                                        tag,
                                        def.props
                                            .iter()
                                            .map(|d| d.name)
                                            .collect::<Vec<_>>()
                                            .join(", ")
                                    ),
                                )
                                .with_hint(format!(
                                    "查 schema/aura.at 的 `{}` 定义;确属新 prop 请更新 schema(SCHEMA_DRIFT_GENERATE_AT=1)",
                                    _canon
                                )),
                            );
                        }
                    }
                }
            } else if !scope.is_local(tag)
                // PascalCase tag = 组件引用语义(内置 tag 全小写;跨文件子组件在
                // 单文件模式下不可见),交由生成层解析,不在此告警
                && !tag.chars().next().map_or(false, |c| c.is_uppercase())
            {
                // 未知 tag:给拼写建议(折叠匹配已失败,levenshtein 兜底)
                let suggestion = schema
                    .all_tags()
                    .into_iter()
                    .min_by_key(|t| {
                        levenshtein(fold(tag).as_bytes(), fold(t).as_bytes())
                    })
                    .filter(|t| {
                        levenshtein(fold(tag).as_bytes(), fold(t).as_bytes()) <= 3
                    });
                let mut w = ValidationWarning::new(
                    "S002",
                    Severity::Warning,
                    widget_name,
                    format!("unknown element `<{}>` — not in schema, not a local widget/component", tag),
                );
                if let Some(s) = suggestion {
                    w = w.with_hint(format!("did you mean `<{}>`? (schema/aura.at)", s));
                }
                out.push(w);
            }
            for c in children {
                walk_node(c, widget_name, schema, scope, out);
            }
        }
        AuraNode::ForLoop { body, .. } => {
            for c in body {
                walk_node(c, widget_name, schema, scope, out);
            }
        }
        AuraNode::Conditional { then_body, else_body, .. } => {
            for c in then_body {
                walk_node(c, widget_name, schema, scope, out);
            }
            if let Some(body) = else_body {
                for c in body {
                    walk_node(c, widget_name, schema, scope, out);
                }
            }
        }
        _ => {}
    }
}

/// 小型 levenshtein(字节级;仅做建议,不追求 Unicode 精确)。
fn levenshtein(a: &[u8], b: &[u8]) -> usize {
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for i in 1..=a.len() {
        cur[0] = i;
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sfc(template: &str, script: &str) -> String {
        format!(
            r#"<!-- Test -->
<script setup lang="ts">
{}
</script>

<template>
{}
</template>

<style></style>
"#,
            script, template
        )
    }

    // --- R001 duplicate-component-key ---

    #[test]
    fn r001_detects_duplicate_key() {
        let sfc = make_sfc(
            r#"<div>
              <AutoDownEditor :key="'AutoDownEditor'" />
              <div v-if="x"><AutoDownEditor :key="'AutoDownEditor'" /></div>
            </div>"#,
            "",
        );
        let ws = r001_duplicate_component_key(&sfc, "Test");
        assert_eq!(ws.len(), 1);
        assert_eq!(ws[0].rule, "R001");
        assert_eq!(ws[0].severity, Severity::Error);
        assert!(ws[0].message.contains("AutoDownEditor"));
    }

    #[test]
    fn r001_ok_with_distinct_keys() {
        let sfc = make_sfc(
            r#"<div>
              <AutoDownEditor :key="'AutoDownEditor-1'" />
              <AutoDownEditor :key="'AutoDownEditor-2'" />
            </div>"#,
            "",
        );
        let ws = r001_duplicate_component_key(&sfc, "Test");
        assert_eq!(ws.len(), 0, "distinct keys should not warn");
    }

    #[test]
    fn r001_ignores_single_instance() {
        let sfc = make_sfc(r#"<AutoDownEditor :key="'x'" />"#, "");
        let ws = r001_duplicate_component_key(&sfc, "Test");
        assert_eq!(ws.len(), 0);
    }

    #[test]
    fn r001_ignores_lowercase_html() {
        // 原生 HTML 标签不应触发（即使重复 key）
        let sfc = make_sfc(
            r#"<div :key="'a'"></div><div :key="'a'"></div>"#,
            "",
        );
        let ws = r001_duplicate_component_key(&sfc, "Test");
        assert_eq!(ws.len(), 0);
    }

    // --- R002 store-usage-without-import ---

    #[test]
    fn r002_detects_store_without_import() {
        let sfc = make_sfc(
            "",
            r#"function Foo() { store.notes = []; }
function Bar() { console.log(store.active_id); }"#,
        );
        let ctx = ValidationContext::default();
        let ws = r002_store_usage_without_import(&sfc, "Test", &ctx);
        assert_eq!(ws.len(), 1);
        assert_eq!(ws[0].rule, "R002");
        assert_eq!(ws[0].severity, Severity::Error);
    }

    #[test]
    fn r002_ok_with_import() {
        let sfc = make_sfc(
            "",
            r#"import { useFooStore } from '@/stores/useFooStore'
import { reactive } from 'vue'
const store = reactive(useFooStore())
function Foo() { store.notes = []; }"#,
        );
        let ctx = ValidationContext::default();
        let ws = r002_store_usage_without_import(&sfc, "Test", &ctx);
        assert_eq!(ws.len(), 0);
    }

    #[test]
    fn r002_ignores_store_composable_definition() {
        let sfc = make_sfc(
            "",
            r#"export function useFooStore() { return { ... } }"#,
        );
        let ctx = ValidationContext::default();
        let ws = r002_store_usage_without_import(&sfc, "Test", &ctx);
        assert_eq!(ws.len(), 0);
    }

    // --- R009 define-expose-undefined (Plan 012 Batch A, gap 45) ---

    #[test]
    fn r009_detects_undefined_exposed_name() {
        // `defineExpose({ Open })` with no `function Open` / import — the
        // runtime reference would silently resolve to a global (window.open).
        let sfc = make_sfc(
            "",
            r#"const msg = ref('')
defineExpose({ Open })"#,
        );
        let ws = r009_define_expose_undefined(&sfc, "Test");
        assert_eq!(ws.len(), 1);
        assert_eq!(ws[0].rule, "R009");
        assert_eq!(ws[0].severity, Severity::Warning);
        assert!(ws[0].message.contains("Open"));
    }

    #[test]
    fn r009_ok_when_exposed_name_is_defined() {
        let sfc = make_sfc(
            "",
            r#"function Open(entry: string): void { console.log(entry) }
defineExpose({ Open })"#,
        );
        let ws = r009_define_expose_undefined(&sfc, "Test");
        assert!(ws.is_empty(), "defined function must not warn: {ws:?}");
    }

    #[test]
    fn r009_ok_when_exposed_name_is_imported() {
        let sfc = make_sfc(
            "",
            r#"import { useClock } from './useClock'
const clock = useClock()
defineExpose({ useClock })"#,
        );
        let ws = r009_define_expose_undefined(&sfc, "Test");
        assert!(ws.is_empty(), "imported name must not warn: {ws:?}");
    }

    #[test]
    fn r009_ignores_sfc_without_define_expose() {
        let sfc = make_sfc("", r#"const msg = ref('')"#);
        let ws = r009_define_expose_undefined(&sfc, "Test");
        assert!(ws.is_empty());
    }

    // --- R003 autodown-css-missing ---

    #[test]
    fn r003_info_when_autodown_used() {
        let sfc = make_sfc(r#"<AutoDownEditor :content="x" />"#, "");
        let ctx = ValidationContext {
            uses_autodown: true,
            ..Default::default()
        };
        let ws = r003_autodown_css_missing(&sfc, "Test", &ctx);
        assert_eq!(ws.len(), 1);
        assert_eq!(ws[0].severity, Severity::Info);
    }

    #[test]
    fn r003_silent_without_autodown() {
        let sfc = make_sfc(r#"<AutoDownEditor />"#, "");
        let ctx = ValidationContext::default();
        let ws = r003_autodown_css_missing(&sfc, "Test", &ctx);
        assert_eq!(ws.len(), 0);
    }

    // --- R004 undefined-handler ---

    #[test]
    fn r004_detects_missing_handler() {
        let sfc = make_sfc(
            r#"<button @click="DoesNotExist">x</button>"#,
            r#"function Other() {}"#,
        );
        let ctx = ValidationContext::default();
        let ws = r004_undefined_handler(&sfc, "Test", &ctx);
        assert_eq!(ws.len(), 1);
        assert!(ws[0].message.contains("DoesNotExist"));
    }

    #[test]
    fn r004_ok_when_defined() {
        let sfc = make_sfc(
            r#"<button @click="Save">x</button>"#,
            r#"function Save() {}"#,
        );
        let ctx = ValidationContext::default();
        let ws = r004_undefined_handler(&sfc, "Test", &ctx);
        assert_eq!(ws.len(), 0);
    }

    #[test]
    fn r004_ignores_inline_toggle_assignment() {
        // `@click="foo = !foo"` toggles a ref inline — NOT a handler call.
        // Must NOT be flagged (regression guard for gallery Code/Tab buttons).
        let sfc = make_sfc(
            r#"<button @click="foo = !foo">x</button>"#,
            r#"const foo = ref(true)"#,
        );
        let ctx = ValidationContext::default();
        let ws = r004_undefined_handler(&sfc, "Test", &ctx);
        assert_eq!(ws.len(), 0);
    }

    #[test]
    fn r004_ignores_inline_setter_assignment() {
        // `@click="tab = 'auto'"` sets a ref inline — NOT a handler call.
        let sfc = make_sfc(
            r#"<button @click="tab = 'auto'">x</button>"#,
            r#"const tab = ref('auto')"#,
        );
        let ctx = ValidationContext::default();
        let ws = r004_undefined_handler(&sfc, "Test", &ctx);
        assert_eq!(ws.len(), 0);
    }

    #[test]
    fn r004_trusts_generator_known_handlers() {
        let sfc = make_sfc(
            r#"<button @click="Edit">x</button>"#,
            "", // script 里没定义，但生成器知道它 used
        );
        let ctx = ValidationContext {
            used_handlers: vec!["Edit".to_string()],
            ..Default::default()
        };
        let ws = r004_undefined_handler(&sfc, "Test", &ctx);
        assert_eq!(ws.len(), 0, "generator-known handlers are trusted");
    }

    // --- R005 emit-without-declaration ---

    #[test]
    fn r005_detects_undeclared_emit() {
        let sfc = make_sfc(
            "",
            r#"const emit = defineEmits<{ Save: [] }>()
function Foo() { emit('Save'); emit('Cancel'); }"#,
        );
        let ws = r005_emit_without_declaration(&sfc, "Test");
        assert_eq!(ws.len(), 1);
        assert!(ws[0].message.contains("Cancel"));
    }

    #[test]
    fn r005_ok_when_all_declared() {
        let sfc = make_sfc(
            "",
            r#"const emit = defineEmits<{ Save: []; Cancel: [] }>()
function Foo() { emit('Save'); emit('Cancel'); }"#,
        );
        let ws = r005_emit_without_declaration(&sfc, "Test");
        assert_eq!(ws.len(), 0);
    }

    // --- R006 v-for-without-key ---

    #[test]
    fn r006_detects_missing_key() {
        let sfc = make_sfc(
            r#"<div v-for="item in items">{{ item }}</div>"#,
            "",
        );
        let ws = r006_v_for_without_key(&sfc, "Test");
        assert_eq!(ws.len(), 1);
    }

    #[test]
    fn r006_ok_with_key() {
        let sfc = make_sfc(
            r#"<div v-for="item in items" :key="item.id">{{ item }}</div>"#,
            "",
        );
        let ws = r006_v_for_without_key(&sfc, "Test");
        assert_eq!(ws.len(), 0);
    }

    // --- R007 autodown-dual-instance ---

    #[test]
    fn r007_detects_dual_editor() {
        let sfc = make_sfc(
            r#"<div v-if="a"><AutoDownEditor /></div>
              <div v-if="b"><AutoDownEditor /></div>"#,
            "",
        );
        let ws = r007_autodown_dual_instance(&sfc, "Test");
        assert_eq!(ws.len(), 1);
        assert_eq!(ws[0].severity, Severity::Info);
    }

    #[test]
    fn r007_ok_with_single_editor() {
        let sfc = make_sfc(r#"<AutoDownEditor />"#, "");
        let ws = r007_autodown_dual_instance(&sfc, "Test");
        assert_eq!(ws.len(), 0);
    }

    // --- R016 keyword-collision (Plan 015 P1#8) ---

    fn make_element(tag: &str, children: Vec<crate::aura::AuraNode>) -> crate::aura::AuraNode {
        crate::aura::AuraNode::Element {
            tag: tag.to_string(),
            props: Default::default(),
            events: Default::default(),
            children,
            span: None,
            debug_id: None,
        }
    }

    fn make_link(to: &str, children: Vec<crate::aura::AuraNode>) -> crate::aura::AuraNode {
        crate::aura::AuraNode::Link {
            to: to.to_string(),
            text: String::new(),
            href: String::new(),
            children,
            span: None,
            debug_id: None,
        }
    }

    #[test]
    fn r016_flags_view_element() {
        let tree = make_element("col", vec![make_element("view", vec![])]);
        let ws = r016_keyword_collision(&tree, "Test");
        assert_eq!(ws.len(), 1);
        assert_eq!(ws[0].rule, "R016");
        assert_eq!(ws[0].severity, Severity::Error);
        assert!(ws[0].message.contains("view"));
    }

    #[test]
    fn r016_flags_task_element() {
        let tree = make_element("col", vec![make_element("task", vec![])]);
        let ws = r016_keyword_collision(&tree, "Test");
        assert_eq!(ws.len(), 1);
        assert_eq!(ws[0].rule, "R016");
        assert!(ws[0].message.contains("task"));
    }

    #[test]
    fn r016_flags_link_without_to() {
        let tree = make_element("col", vec![make_link("", vec![])]);
        let ws = r016_keyword_collision(&tree, "Test");
        assert_eq!(ws.len(), 1);
        assert_eq!(ws[0].rule, "R016");
        assert!(ws[0].message.contains("router-link"));
    }

    #[test]
    fn r016_ok_link_with_to() {
        let tree = make_element("col", vec![make_link("/home", vec![])]);
        let ws = r016_keyword_collision(&tree, "Test");
        assert!(ws.is_empty(), "router link with to: must not warn: {ws:?}");
    }

    #[test]
    fn r016_ok_normal_elements() {
        // col/div/text and a nested loop/conditional tree — no keywords collide.
        let tree = make_element(
            "col",
            vec![
                make_element("div", vec![]),
                make_element("text", vec![]),
                crate::aura::AuraNode::Conditional {
                    condition: ".flag".to_string(),
                    then_body: vec![make_element("button", vec![])],
                    else_body: None,
                    span: None,
                    debug_id: None,
                },
            ],
        );
        let ws = r016_keyword_collision(&tree, "Test");
        assert!(ws.is_empty(), "normal elements must not warn: {ws:?}");
    }

    // --- 入口测试 ---

    #[test]
    fn validate_sfc_aggregates_all_rules() {
        // 一个有多个问题的 SFC
        let sfc = make_sfc(
            r#"<AutoDownEditor :key="'a'" />
              <AutoDownEditor :key="'a'" />
              <button @click="Missing">x</button>"#,
            r#"store.notes = []"#,
        );
        let ctx = ValidationContext {
            uses_autodown: true,
            ..Default::default()
        };
        let ws = validate_sfc(&sfc, "Test", &ctx);
        let rules: Vec<&str> = ws.iter().map(|w| w.rule).collect();
        assert!(rules.contains(&"R001"), "should catch dup key");
        assert!(rules.contains(&"R002"), "should catch store w/o import");
        assert!(rules.contains(&"R003"), "should catch autodown info");
        assert!(rules.contains(&"R007"), "should catch dual editor");
    }

    #[test]
    fn schema_validation_catches_unknown_tag_and_prop() {
        // Plan 435 P2:未知 tag → S002(Warning,带建议);未声明 prop → S001(Info)。
        // view { button (tex: "x") {} btton {} } —— button 有声明 props,
        // tex 未声明;btton 无处可解析。
        let code = r#"widget T {
    msg M { Go }
    model { n int = 0 }
    on { .Go -> { } }
    view {
        col {
            button (tex: "hi") {}
            btton "click" {}
        }
    }
}"#;
        let session = crate::session::CompilerSession::new(crate::session::Scenario::UI);
        let mut parser = crate::Parser::from(code);
        parser = parser.with_session(session);
        let ast = parser.parse().expect("parse");
        let mut widgets = Vec::new();
        for stmt in &ast.stmts {
            if let crate::ast::Stmt::WidgetDecl(d) = stmt {
                if let Ok(w) = crate::aura::extract_widget_from_decl(d) {
                    widgets.push(w);
                }
            }
        }
        assert_eq!(widgets.len(), 1);
        let ws = validate_aura_against_schema(&widgets, &[]);
        let unknown_tags: Vec<&ValidationWarning> =
            ws.iter().filter(|w| w.rule == "S002").collect();
        let unknown_props: Vec<&ValidationWarning> =
            ws.iter().filter(|w| w.rule == "S001").collect();
        assert_eq!(unknown_tags.len(), 1, "btton 应触发 S002: {ws:?}");
        assert!(unknown_tags[0].message.contains("btton"));
        assert!(
            unknown_tags[0]
                .fix_hint
                .as_deref()
                .unwrap_or("")
                .contains("button"),
            "S002 应建议 button: {:?}",
            unknown_tags[0].fix_hint
        );
        assert_eq!(unknown_props.len(), 1, "tex 应触发 S001: {ws:?}");
        assert!(unknown_props[0].message.contains("tex"));
    }

    #[test]
    fn schema_validation_accepts_local_widgets_and_fold_aliases() {
        // 本地 widget 名(CopyButton)与折叠别名(alert-dialog ≡ alert_dialog)
        // 都不应触发 S002。
        let code = r#"widget Demo {
    msg M { Go }
    model { n int = 0 }
    on { .Go -> { } }
    view {
        col {
            copy-button {}
            alert-dialog-action {}
        }
    }
}
widget CopyButton {
    msg M { Go }
    model { n int = 0 }
    on { .Go -> { } }
    view { button "copy" {} }
}"#;
        let session = crate::session::CompilerSession::new(crate::session::Scenario::UI);
        let mut parser = crate::Parser::from(code);
        parser = parser.with_session(session);
        let ast = parser.parse().expect("parse");
        let mut widgets = Vec::new();
        for stmt in &ast.stmts {
            if let crate::ast::Stmt::WidgetDecl(d) = stmt {
                if let Ok(w) = crate::aura::extract_widget_from_decl(d) {
                    widgets.push(w);
                }
            }
        }
        let ws = validate_aura_against_schema(&widgets, &[]);
        let s002: Vec<&ValidationWarning> =
            ws.iter().filter(|w| w.rule == "S002").collect();
        assert!(s002.is_empty(), "本地 widget 与折叠别名不应告警: {s002:?}");
    }

    #[test]
    fn format_warnings_produces_readable_output() {
        let ws = vec![ValidationWarning::new(
            "R001",
            Severity::Error,
            "Test",
            "Something is wrong",
        )
        .with_hint("Do X")];
        let out = format_warnings(&ws);
        assert!(out.contains("R001"));
        assert!(out.contains("ERROR"));
        assert!(out.contains("Something is wrong"));
        assert!(out.contains("Fix: Do X"));
    }
}
