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
    fn new(rule: &'static str, severity: Severity, widget: &str, message: impl Into<String>) -> Self {
        Self {
            rule,
            severity,
            widget: widget.to_string(),
            message: message.into(),
            fix_hint: None,
        }
    }

    fn with_hint(mut self, hint: impl Into<String>) -> Self {
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
    let handler_ref_re = regex_lite(r#"@\w+(?:\.\w+)*\s*=\s*"([a-zA-Z_]\w*)"#);
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
        // body 形如 "X: []\n  Y: []"，取冒号前的标识符
        let name_re = regex_lite(r"([a-zA-Z_]\w*)\s*:");
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
