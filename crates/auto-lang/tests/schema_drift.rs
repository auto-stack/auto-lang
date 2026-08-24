//! Plan 435 P0 —— 内置组件四表漂移围栏(schema drift fence)。
//!
//! 来源:2026-08-23 漂移审计(scratch/schema_drift_audit.py,Plan 435 §1.1)的
//! Rust 化,常驻测试套。比对内置组件的全部事实表:
//!   1. `src/aura/schema.rs`                 验证声明(elements.insert)
//!   2. `src/ui_gen/vue.rs`                  Web 路径(components.insert + match tag 表)
//!   3. `src/ui/aura_view_builder.rs`        桌面路径(两张 match tag 派发表)
//!   4. `src/ui/render_support.rs`           iced 支持级(单张 match tag 表)
//!   5. `src/parser.rs`                      tag 特判(get_primary_prop 归类表)
//!   6. `src/a2ui/export.rs`                 tag → A2UI 互操作映射
//!   7. `src/a2ui/import.rs`                 A2UI → tag 反向映射(tag: "..." 字面量)
//! 另带 `schema/aura.at`(Plan 098 冻结孤儿,P1 重建)的对照维度。
//!
//! 围栏语义:**只拦新增漂移**。审计当日已知漂移冻结在
//! `tests/fixtures/schema_drift_baseline.txt`;出现 baseline 之外的新孤立项即红;
//! 漂移消除只打印提示(请顺手裁剪 baseline)。master 因此始终绿,P1-P3 逐项收编。
//! 同表重复 insert(如审计发现的 popover 两处)无 baseline 豁免,直接红。
//!
//! 生成/更新 baseline(提交前人工复核 diff,逐维度写明理由):
//! ```text
//! SCHEMA_DRIFT_UPDATE_BASELINE=1 cargo test -p auto-lang --test schema_drift
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

/// view_builder 里的语句级 `match tag {` 派发表数量(tracked + untracked)。
/// `let x = match tag {` 的小表(默认样式等)刻意不收。
const EXPECTED_VB_TABLES: usize = 2;
/// render_support 的 `get_support` 派发表数量。
const EXPECTED_RENDER_TABLES: usize = 1;
/// vue.rs 的语句级 `match tag {` 表数量(map_tag 原生映射 + PascalCase 归一)。
/// `match tag_lower.as_str() {` 这类小表刻意不收。
const EXPECTED_VUE_TABLES: usize = 2;
/// parser.rs 的语句级 `match tag {` 表数量(get_primary_prop 的 tag 归类)。
const EXPECTED_PARSER_TABLES: usize = 1;
/// a2ui/export.rs 的语句级 `match tag {` 表数量(tag → A2UIComponentBody)。
const EXPECTED_A2UI_EXPORT_TABLES: usize = 1;

fn repo_file(rel: &str) -> PathBuf {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    assert!(
        p.exists(),
        "源文件不存在: {} —— 本测试必须在仓库布局内运行",
        p.display()
    );
    p
}

fn read(rel: &str) -> String {
    fs::read_to_string(repo_file(rel)).expect("read source file")
}

fn is_ident_char(b: u8) -> bool {
    b == b'_' || b.is_ascii_alphanumeric()
}

/// 提取 `map.insert("tag", ...)` 的 tag 序表(保序,供重复检测)。
/// map 名要求精确匹配(前一字符不是标识符字符),排除
/// `shadcn_components_used.insert` 这类前缀撞名。
fn scan_insert_tags(src: &str, map_name: &str) -> Vec<String> {
    let needle = format!("{}.insert(\"", map_name);
    let mut out = Vec::new();
    for line in src.lines() {
        if let Some(pos) = line.find(&needle) {
            if pos > 0 && is_ident_char(line.as_bytes()[pos - 1]) {
                continue;
            }
            let rest = &line[pos + needle.len()..];
            if let Some(end) = rest.find('"') {
                out.push(rest[..end].to_string());
            }
        }
    }
    out
}

/// 行骨架:去掉行注释与字符串字面量**内容**(保留引号对占位),只留结构字符。
/// 括号深度与 `=>` 判定都用骨架,注释/文案里的 `{}`、`=>` 不再干扰。
fn skeleton(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();
    let mut in_str = false;
    while let Some(c) = chars.next() {
        if in_str {
            match c {
                '\\' => {
                    chars.next();
                }
                '"' => {
                    in_str = false;
                    out.push('"');
                }
                _ => {}
            }
            continue;
        }
        match c {
            '"' => {
                in_str = true;
                out.push('"');
            }
            '/' if chars.peek() == Some(&'/') => break,
            _ => out.push(c),
        }
    }
    out
}

/// 提取文件中所有行 trim 后恰为 `match tag {` 的派发表,返回每张表的 tag 集
/// (臂头的全部字符串字面量,含别名)。逐行扫描,用花括号/圆括号深度区分
/// 臂头与臂体:臂体里的字符串(shadcn 模块路径、说明文案)不会被误收。
fn scan_match_tables(src: &str) -> Vec<BTreeSet<String>> {
    let lines: Vec<&str> = src.lines().collect();
    let mut tables = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].trim() != "match tag {" {
            i += 1;
            continue;
        }
        let mut tags = BTreeSet::new();
        let mut depth: i32 = 1;
        let mut paren: i32 = 0;
        let mut head: Vec<&str> = Vec::new();
        let mut in_body = false;
        i += 1;
        while i < lines.len() && depth > 0 {
            let line = lines[i];
            let skel = skeleton(line);
            let opens = skel.matches('{').count() as i32;
            let closes = skel.matches('}').count() as i32;
            let parens =
                skel.matches('(').count() as i32 - skel.matches(')').count() as i32;
            let trimmed = skel.trim();
            if !in_body {
                if line.trim().starts_with("//") {
                    i += 1;
                    continue;
                }
                depth += opens - closes;
                paren += parens;
                head.push(line);
                if skel.contains("=>") {
                    for h in &head {
                        // 臂头只可能是 `"tag" | "tag"` 模式;取首个 `=>` 前的原始文本
                        let before = h.split("=>").next().unwrap_or("");
                        for tag in quoted_strings(before) {
                            tags.insert(tag);
                        }
                    }
                    head.clear();
                    in_body = true;
                    if depth == 1 && paren == 0 && trimmed.ends_with(',') {
                        in_body = false; // 单行表达式臂:`"row" => self.f(...),`
                    }
                }
            } else {
                depth += opens - closes;
                paren += parens;
                let arm_done = depth == 1
                    && paren == 0
                    && (trimmed.ends_with(',') || trimmed.ends_with('}'));
                if arm_done {
                    in_body = false;
                }
            }
            i += 1;
        }
        tables.push(tags);
    }
    tables
}

/// 提取一段文本里的双引号字符串字面量(tag 不会含 `//`,不考虑转义)。
fn quoted_strings(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let mut j = 0;
    while j < bytes.len() {
        if bytes[j] == b'"' {
            if let Some(end) = text[j + 1..].find('"') {
                out.push(text[j + 1..j + 1 + end].to_string());
                j = j + 1 + end + 1;
            } else {
                break;
            }
        } else {
            j += 1;
        }
    }
    out
}

/// 提取 `tag: "xxx"` 字面量(a2ui/import.rs 的 enum → tag 反向映射形态)。
fn scan_tag_literals(src: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for line in src.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("tag: \"") {
            if let Some(end) = rest.find('"') {
                out.insert(rest[..end].to_string());
            }
        }
    }
    out
}

/// 提取 `schema/aura.at` 的 `element NAME {` 声明名表。
fn scan_aura_at(src: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for line in src.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("element ") {
            if let Some(name) = rest.strip_suffix('{') {
                out.insert(name.trim().to_string());
            }
        }
    }
    out
}

/// 提取 `schema/aura.at` 各元素 `aliases: ["a", "b"]` 行的全部别名。
fn scan_aura_at_aliases(src: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for line in src.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("aliases: [") {
            if let Some(inner) = rest.strip_suffix(']') {
                for tag in quoted_strings(inner) {
                    out.insert(tag);
                }
            }
        }
    }
    out
}

type Drift = BTreeMap<&'static str, BTreeSet<String>>;

// ============================================================================
// P1:schema/aura.at 生成器(生产代码提取,一次性基准 + 可再生)
// 触发:SCHEMA_DRIFT_GENERATE_AT=1 cargo test -p auto-lang --test schema_drift
// 生成后 panic 强制人工复核 diff;围栏常驻断言"生产 ⊆ schema(tags ∪ aliases)"。
// ============================================================================

/// schema.rs 的 PropDef 概况(生成 .at props 用)。
struct RsProp {
    name: String,
    type_str: String,
    required: bool,
    default: Option<String>,
    description: String,
}

/// schema.rs 的 ElementDef 概况。
struct RsElement {
    tag: String,
    category: String,
    props: Vec<RsProp>,
    allows_children: bool,
    description: String,
}

/// 提取 `elements.insert("tag", ElementDef { ... });` 的结构化内容。
/// schema.rs 是目前唯一带类型 props 的来源;解析不求全,识别既有写法即可
/// (未识别的行静默跳过,围栏另有四表 tag 级断言兜底)。
fn extract_rs_elements(src: &str) -> BTreeMap<String, RsElement> {
    let mut out = BTreeMap::new();
    let lines: Vec<&str> = src.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if let Some(pos) = line.find("elements.insert(\"") {
            let rest = &line[pos + "elements.insert(\"".len()..];
            let Some(q1) = rest.find('"') else { i += 1; continue };
            let tag = rest[..q1].to_string();
            // 花括号配对收集块体(基于骨架,免疫注释/字符串干扰)
            let mut depth = 0i32;
            let mut body: Vec<&str> = Vec::new();
            let mut j = i;
            while j < lines.len() {
                let skel = skeleton(lines[j]);
                if j == i {
                    depth += skel.matches('{').count() as i32
                        - skel.matches('}').count() as i32;
                    body.push(lines[j]);
                    j += 1;
                    continue;
                }
                depth += skel.matches('{').count() as i32
                    - skel.matches('}').count() as i32;
                if depth <= 0 {
                    break;
                }
                body.push(lines[j]);
                j += 1;
            }
            out.insert(tag.clone(), parse_rs_element_body(&tag, &body));
            i = j;
        } else {
            i += 1;
        }
    }
    out
}

fn parse_rs_element_body(tag: &str, body: &[&str]) -> RsElement {
    let mut category = String::new();
    let mut description = String::new();
    let mut allows_children = true;
    let mut props = Vec::new();
    for line in body {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("category: ElementCategory::") {
            category = rest.trim_end_matches(',').to_string();
        } else if t.starts_with("description:") {
            description = quoted_first(t).unwrap_or_default();
        } else if let Some(v) = t.strip_prefix("allows_children:") {
            allows_children = v.trim().starts_with("true");
        } else if t.trim_start().starts_with("PropDef {") {
            if let Some(p) = parse_rs_prop(t) {
                props.push(p);
            }
        }
    }
    RsElement {
        tag: tag.to_string(),
        category,
        props,
        allows_children,
        description,
    }
}

/// 解析单行 `PropDef { name: "x", type_: ..., required: false, default: Some("..")/None, description: ".." }`。
fn parse_rs_prop(line: &str) -> Option<RsProp> {
    let name = extract_field_str(line, "name:")?;
    let type_str = line
        .find("type_: PropType::")
        .map(|p| {
            // 取到平衡括号的表达式(Union(vec![..]) 内含逗号,不能按首个逗号截断)
            let rest = &line[p + "type_: PropType::".len()..];
            let expr = take_balanced_expr(rest);
            map_proptype(expr)
        })
        .unwrap_or_else(|| "unknown".to_string());
    let required = line.contains("required: true");
    let default = if let Some(p) = line.find("default: Some(") {
        let rest = &line[p + "default: Some(".len()..];
        quoted_first(rest)
    } else {
        None
    };
    let description = extract_field_str(line, "description:").unwrap_or_default();
    Some(RsProp { name, type_str, required, default, description })
}

/// 从 `rest` 起取一个平衡括号的表达式:裸标识符到首个逗号;
/// `Union(vec![...])`/`OneOf(vec![...])` 取到配对的右括号(方括号同计)。
fn take_balanced_expr(rest: &str) -> &str {
    let bytes = rest.as_bytes();
    let mut depth = 0i32;
    let mut end = bytes.len();
    for (i, b) in bytes.iter().enumerate() {
        match b {
            b'(' | b'[' => depth += 1,
            b')' | b']' => {
                depth -= 1;
                if depth == 0 {
                    end = i + 1;
                    break;
                }
            }
            b',' if depth == 0 => {
                end = i;
                break;
            }
            _ => {}
        }
    }
    rest[..end].trim()
}

/// `PropType` 源码形态 → .at schema 类型串。
/// Union(vec![PropType::String, PropType::StyleBinding]) → "union:string,class_binding"
/// OneOf(vec!["a", "b"]) → "one_of:a,b"
fn map_proptype(expr: &str) -> String {
    if let Some(inner) = expr
        .strip_prefix("Union(vec![")
        .and_then(|s| s.strip_suffix("])"))
    {
        let names = inner
            .split("PropType::")
            .filter_map(|t| {
                let n = t.trim().trim_end_matches(',');
                if n.is_empty() { None } else { Some(map_type_name(n)) }
            })
            .collect::<Vec<_>>()
            .join(",");
        format!("union:{}", names)
    } else if let Some(inner) = expr
        .strip_prefix("OneOf(vec![")
        .and_then(|s| s.strip_suffix("])"))
    {
        let vals = quoted_strings(inner).join(",");
        format!("one_of:{}", vals)
    } else {
        map_type_name(expr.trim_end_matches(',')).to_string()
    }
}

fn map_type_name(n: &str) -> &'static str {
    match n {
        "String" | "Str" => "string",
        "Int" => "int",
        "Float" => "float",
        "Bool" => "bool",
        "Color" => "color",
        "StateRef" => "state_ref",
        "MsgRef" => "msg_ref",
        "Expr" => "expr",
        "Closure" => "closure",
        "StyleBinding" => "class_binding",
        "Interpolated" => "interpolated",
        _ => "unknown",
    }
}

fn extract_field_str(line: &str, key: &str) -> Option<String> {
    let p = line.find(key)?;
    let rest = &line[p + key.len()..];
    // 跳过空白,取首个引号串
    let rest = rest.trim_start();
    if rest.starts_with('"') {
        quoted_first(rest)
    } else {
        None
    }
}

fn quoted_first(s: &str) -> Option<String> {
    let start = s.find('"')?;
    let rest = &s[start + 1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

/// 提取所有 `match tag` 表的**臂组**(同臂全部 tag = 行为等价 → 别名组)。
/// 复用 scan_match_tables 的骨架/深度判定,但按臂返回分组。
fn scan_match_arm_groups(src: &str) -> Vec<Vec<String>> {
    let lines: Vec<&str> = src.lines().collect();
    let mut groups = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].trim() != "match tag {" {
            i += 1;
            continue;
        }
        let mut depth: i32 = 1;
        let mut paren: i32 = 0;
        let mut head: Vec<&str> = Vec::new();
        let mut in_body = false;
        i += 1;
        while i < lines.len() && depth > 0 {
            let line = lines[i];
            let skel = skeleton(line);
            let opens = skel.matches('{').count() as i32;
            let closes = skel.matches('}').count() as i32;
            let parens =
                skel.matches('(').count() as i32 - skel.matches(')').count() as i32;
            let trimmed = skel.trim();
            if !in_body {
                if line.trim().starts_with("//") {
                    i += 1;
                    continue;
                }
                depth += opens - closes;
                paren += parens;
                head.push(line);
                if skel.contains("=>") {
                    let mut tags: Vec<String> = Vec::new();
                    for h in &head {
                        let before = h.split("=>").next().unwrap_or("");
                        for tag in quoted_strings(before) {
                            if !tags.contains(&tag) {
                                tags.push(tag);
                            }
                        }
                    }
                    if tags.len() > 1 {
                        groups.push(tags);
                    }
                    head.clear();
                    in_body = true;
                    if depth == 1 && paren == 0 && trimmed.ends_with(',') {
                        in_body = false;
                    }
                }
            } else {
                depth += opens - closes;
                paren += parens;
                let arm_done = depth == 1
                    && paren == 0
                    && (trimmed.ends_with(',') || trimmed.ends_with('}'));
                if arm_done {
                    in_body = false;
                }
            }
            i += 1;
        }
    }
    groups
}

/// scan_match_arm_groups 的伴生:同时返回单 tag 臂,供 render 级别提取用。
/// 返回 (臂 tags, 级别串) 列表;级别取臂行 `TagSupport::xxx` 首个匹配,
/// 无匹配(struct 字面量等)记 "custom"。
fn scan_match_arm_groups_plus_singles(
    src: &str,
) -> Vec<(Vec<String>, String)> {
    let lines: Vec<&str> = src.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].trim() != "match tag {" {
            i += 1;
            continue;
        }
        let mut depth: i32 = 1;
        let mut paren: i32 = 0;
        let mut head: Vec<&str> = Vec::new();
        let mut in_body = false;
        i += 1;
        while i < lines.len() && depth > 0 {
            let line = lines[i];
            let skel = skeleton(line);
            let opens = skel.matches('{').count() as i32;
            let closes = skel.matches('}').count() as i32;
            let parens =
                skel.matches('(').count() as i32 - skel.matches(')').count() as i32;
            let trimmed = skel.trim();
            if !in_body {
                if line.trim().starts_with("//") {
                    i += 1;
                    continue;
                }
                depth += opens - closes;
                paren += parens;
                head.push(line);
                if skel.contains("=>") {
                    let mut tags: Vec<String> = Vec::new();
                    let mut level = String::new();
                    for h in &head {
                        let (before, after) = h.split_once("=>").unwrap_or((h, ""));
                        for tag in quoted_strings(before) {
                            if !tags.contains(&tag) {
                                tags.push(tag);
                            }
                        }
                        for lv in ["full", "partial", "fallback", "unsupported"] {
                            if after.contains(&format!("TagSupport::{}", lv)) {
                                level = lv.to_string();
                            }
                        }
                    }
                    if !tags.is_empty() {
                        out.push((tags, level));
                    }
                    head.clear();
                    in_body = true;
                    if depth == 1 && paren == 0 && trimmed.ends_with(',') {
                        in_body = false;
                    }
                }
            } else {
                depth += opens - closes;
                paren += parens;
                let arm_done = depth == 1
                    && paren == 0
                    && (trimmed.ends_with(',') || trimmed.ends_with('}'));
                if arm_done {
                    in_body = false;
                }
            }
            i += 1;
        }
    }
    out
}

/// vue.rs `components.insert("tag",\n ("@/components/ui/xxx", vec![...]))`
/// 的 tag → import 路径(从 insert 位置向后 300 字符窗口找首个 "@/" 串)。
fn scan_vue_imports(src: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let bytes = src.as_bytes();
    let needle = b"components.insert(\"";
    let mut i = 0;
    while i + needle.len() <= bytes.len() {
        if &bytes[i..i + needle.len()] == needle
            && (i == 0 || !is_ident_char(bytes[i - 1]))
        {
            let rest = &src[i + needle.len()..];
            if let Some(q1) = rest.find('"') {
                let tag = rest[..q1].to_string();
                let window = &rest[..rest.len().min(300)];
                if let Some(at) = window.find("\"@/") {
                    let after = &window[at + 1..];
                    if let Some(end) = after.find('"') {
                        out.insert(tag, after[..end].to_string());
                    }
                }
            }
        }
        i += 1;
    }
    out
}

/// 扫描 widgets-gallery 的 .at 源(第 10 个提取源,唯一**消费侧**来源):
/// 收集元素位置的候选 tag(行首缩进 + 名字 + `{`/`(`/`"`/`|`),
/// 排除语言关键字与本地 `widget` 声明(那是 Local 组件,归 P4 Registry)。
/// 这些 tag 无任何生产表登记(走 vue fallback / ext 组件路径),按
/// "生产代码 + examples 是事实源"原则以 unclassified 入册。
fn scan_gallery_tags() -> BTreeSet<String> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/widgets-gallery/src");
    let mut local: BTreeSet<String> = BTreeSet::new();
    let mut files: Vec<PathBuf> = Vec::new();
    if let Ok(rd) = fs::read_dir(&dir) {
        for e in rd.flatten() {
            collect_at_files(&e.path(), &mut files);
        }
    }
    for f in &files {
        let Ok(text) = fs::read_to_string(f) else { continue };
        for m in text.lines().filter_map(|l| {
            let t = l.trim_start();
            t.strip_prefix("widget ").and_then(|r| r.split_whitespace().next())
        }) {
            local.insert(m.to_string());
        }
    }
    const KW: &[&str] = &[
        "widget", "model", "view", "msg", "on", "computed", "style", "use",
        "fn", "let", "state", "store", "route", "routes", "config", "handler",
        "effect", "prop", "props", "emits", "head", "script", "template",
        "import", "export", "const", "type", "struct", "enum", "impl", "if",
        "else", "for", "while", "return", "match", "block", "ui_config",
        "menus", "actions", "f",
    ];
    let mut out = BTreeSet::new();
    for f in &files {
        let Ok(text) = fs::read_to_string(f) else { continue };
        for line in text.lines() {
            let t = line.trim_start();
            let Some(first) = t.chars().next() else { continue };
            if !first.is_ascii_alphabetic() {
                continue;
            }
            let name: String = t
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
                .collect();
            if name.is_empty() || KW.contains(&name.as_str()) || local.contains(&name) {
                continue;
            }
            let rest = t[name.len()..].trim_start();
            if rest.starts_with('{')
                || rest.starts_with('(')
                || rest.starts_with('"')
                || rest.starts_with('|')
            {
                if name == name.to_lowercase() || !local.contains(&name.to_lowercase()) {
                    out.insert(name);
                }
            }
        }
    }
    out
}

fn collect_at_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(rd) = fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                collect_at_files(&p, out);
            } else if p.extension().map_or(false, |x| x == "at") {
                out.push(p);
            }
        }
    }
}

/// 生成器的全部输入(测试里一次采集,生成与断言共用)。
struct AtGenInput {
    rs_elements: BTreeMap<String, RsElement>,
    /// 全部生产表 tag 集(schema.rs/vue 两表/vb 两表/render/parser/a2ui×2)
    prod_union: BTreeSet<String>,
    /// 各表成员资格(canonical 级 tier/backends 推导用)
    in_vb: BTreeSet<String>,
    in_vue_shadcn: BTreeSet<String>,
    in_vue_native: BTreeSet<String>,
    render_levels: BTreeMap<String, String>,
    vue_imports: BTreeMap<String, String>,
    /// widgets-gallery 使用但无生产表登记的 tag(消费侧来源,unclassified 入册)
    gallery_only: BTreeSet<String>,
    /// 臂组别名并查结果:tag → canonical
    canonical_of: BTreeMap<String, String>,
}

/// 折叠键:剥分隔符 + 小写("Card"≡"card","AlertDialog"≡"alert-dialog")。
fn fold_key(s: &str) -> String {
    s.chars()
        .filter(|c| *c != '-' && *c != '_')
        .collect::<String>()
        .to_lowercase()
}

/// 别名归并:臂组(行为等价)+ 分隔符拼写变体。
/// 仅用于**行为派发表**(vb 两表/vue map_tag+Pascal/a2ui export)——
/// parser(get_primary_prop)与 render_support 是"归类表",同臂≠同一组件,不参与。
/// 守卫:臂内 ≥2 个 rs 有意声明的元素时不合并(如 text|h1|p|span 共用转换器)。
fn build_alias_groups(
    arm_groups: &[Vec<String>],
    all_tags: &BTreeSet<String>,
    rs_tags: &BTreeSet<String>,
) -> BTreeMap<String, String> {
    // 并查集:parent 映射,root 取组内字典序最小 tag
    let mut parent: BTreeMap<String, String> = BTreeMap::new();
    for t in all_tags {
        parent.insert(t.clone(), t.clone());
    }
    fn find(parent: &mut BTreeMap<String, String>, x: &str) -> String {
        let p = parent[x].clone();
        if p == x {
            return x.to_string();
        }
        let root = find(parent, &p);
        parent.insert(x.to_string(), root.clone());
        root
    }
    let mut union = |parent: &mut BTreeMap<String, String>, a: &str, b: &str| {
        let ra = find(parent, a);
        let rb = find(parent, b);
        if ra != rb {
            let (w, l) = if ra < rb { (ra, rb) } else { (rb, ra) };
            parent.insert(l, w);
        }
    };
    for group in arm_groups {
        if group.len() > 4 {
            continue; // 家族性大臂(svg 图元/HTML5 语义标签共用转换器),各自成元素
        }
        let rs_count = group.iter().filter(|t| rs_tags.contains(*t)).count();
        if rs_count > 1 {
            continue; // 多个 rs 有意声明的元素共用臂(泛化转换器),不并别名
        }
        for w in group.iter().skip(1) {
            union(&mut parent, &group[0], w);
        }
    }
    // 折叠变体:按 fold_key 分桶(分隔符 + 大小写),同桶且不同时为 rs 元素时归并
    let mut buckets: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for t in all_tags {
        buckets.entry(fold_key(t)).or_default().push(t.clone());
    }
    for (_, mut members) in buckets {
        if members.len() < 2 {
            continue;
        }
        members.sort();
        let rs_count = members.iter().filter(|t| rs_tags.contains(*t)).count();
        if rs_count > 1 {
            continue; // 两个 rs 有意声明,不合并
        }
        for m in members.iter().skip(1) {
            union(&mut parent, &members[0], m);
        }
    }
    // 输出 tag → 组代表(字典序最小)
    let mut out: BTreeMap<String, String> = BTreeMap::new();
    for t in all_tags {
        let root = find(&mut parent, t);
        out.insert(t.clone(), root);
    }
    out
}

/// P2 人工归类表(生成器 override):表成员资格推不出的 tier 在此声明。
/// 依据:
/// - builtin_widget:vb fallback 显式分支(nav-link→链接按钮 / toast-provider→
///   View::Empty)或 VNodeKind 运行时词汇(list/list_item)
/// - native_html:HTML 词汇直通(a/audio/video/canvas/tfoot/li/summary/code)
/// - web_component:shadcn 家族子件(gallery 实际使用的 dialog-*/dropdown-menu-*/
///   tooltip-*/avatar-*/skeleton/navigation-menu/radioitem 等)
/// 未列出的 unclassified 仍是"待定词汇"(parser/a2ui 词汇无实现,P3 normalize_tag
/// 收编或实现后归位)——保留 unclassified 比错误归类更诚实。
const TIER_OVERRIDES: &[(&str, &str)] = &[
    // builtin_widget(桌面有实现,机制在派发表之外)
    ("nav-link", "builtin_widget"),
    ("nav_link", "builtin_widget"),
    ("toast-provider", "builtin_widget"),
    ("toast_provider", "builtin_widget"),
    ("list", "builtin_widget"),
    ("list_item", "builtin_widget"),
    // native_html(HTML 词汇)
    ("a", "native_html"),
    ("audio", "native_html"),
    ("video", "native_html"),
    ("canvas", "native_html"),
    ("tfoot", "native_html"),
    ("li", "native_html"),
    ("summary", "native_html"),
    ("code", "native_html"),
    // web_component(shadcn 家族,gallery 在用)
    ("avatar-fallback", "web_component"),
    ("avatar-image", "web_component"),
    ("card-action", "web_component"),
    ("date-picker-trigger", "web_component"),
    ("dialog", "web_component"),
    ("dialog-close", "web_component"),
    ("dialog-content", "web_component"),
    ("dialog-description", "web_component"),
    ("dialog-footer", "web_component"),
    ("dialog-header", "web_component"),
    ("dialog-title", "web_component"),
    ("dialog-trigger", "web_component"),
    ("dropdown-menu-content", "web_component"),
    ("dropdown-menu-item", "web_component"),
    ("dropdown-menu-separator", "web_component"),
    ("dropdown-menu-trigger", "web_component"),
    ("navigation-menu", "web_component"),
    ("radioitem", "web_component"),
    ("sheet-description", "web_component"),
    ("skeleton", "web_component"),
    ("switch", "web_component"),
    ("tooltip-content", "web_component"),
    ("tooltip-provider", "web_component"),
    ("tooltip-trigger", "web_component"),
    ("TabTrigger", "web_component"),
    // badge 家族词汇(parser 归为 text-bearing badge 变体)
    ("chip", "web_component"),
    ("tag", "web_component"),
    ("range", "web_component"),
    ("date", "web_component"),
    ("datetime", "web_component"),
    ("datetimeinput", "web_component"),
];

/// tier 推导(组级):桌面实现 > web 组件 > 原生直通 > 未分类;override 优先。
fn derive_tier(group: &BTreeSet<String>, inp: &AtGenInput) -> &'static str {
    for (tag, tier) in TIER_OVERRIDES {
        if group.contains(*tag) {
            return tier;
        }
    }
    if group.iter().any(|t| inp.in_vb.contains(t)) {
        "builtin_widget"
    } else if group.iter().any(|t| inp.in_vue_shadcn.contains(t)) {
        "web_component"
    } else if group.iter().any(|t| inp.in_vue_native.contains(t)) {
        "native_html"
    } else {
        "unclassified"
    }
}

fn generate_aura_at(inp: &AtGenInput) -> String {
    // 组:canonical → members
    let mut groups: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (tag, canonical) in &inp.canonical_of {
        groups.entry(canonical.clone()).or_default().insert(tag.clone());
    }
    // rs 声明优先;否则偏好 kebab-case(命名规范),再短、字典序
    let mut final_groups: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut final_of_root: BTreeMap<String, String> = BTreeMap::new();
    for (root, members) in &groups {
        let canonical = members
            .iter()
            .find(|t| inp.rs_elements.contains_key(*t))
            .cloned()
            .or_else(|| {
                members
                    .iter()
                    .filter(|t| **t == t.to_lowercase())
                    .min_by_key(|t| (!t.contains('-'), t.len(), t.to_string()))
                    .cloned()
            })
            .unwrap_or_else(|| root.clone());
        final_of_root.insert(root.clone(), canonical.clone());
        let entry = final_groups.entry(canonical).or_default();
        for m in members {
            entry.insert(m.clone());
        }
    }

    let mut out = String::new();
    out.push_str(
        "// AURA Schema —— Plan 435 P1:从生产代码提取生成(单一声明源)\n\
         //\n\
         // 本文件是 AutoUI 内置组件的唯一声明:tag/category/props 来自 schema.rs\n\
         // (唯一带类型 props 的来源);aliases 来自派发表臂组(行为等价)与分隔符\n\
         // 拼写变体;tier/backends 来自各生产表的成员资格(见围栏测试)。\n\
         // 官方/第三方 .at 组件(widget)不在此声明——它们由 ComponentRegistry 注册\n\
         // (P4);sub_widgets 字段为组件家族预留(P4 填充,web 家族已按 vue import\n\
         // 路径预填)。\n\
         //\n\
         // tier 语义:native_html=Web 原生直通(类比 HTML_TAGS);builtin_widget=\n\
         //   桌面(iced)有实现;web_component=shadcn 家族生成映射;\n\
         //   unclassified=P1 提取时尚无归属(待人工归类,P2 消除)。\n\
         // backends 语义:web=native|component|none;iced=full|partial|fallback|\n\
         //   unsupported|unknown(unknown=有实现但支持级未登记)|none;\n\
         //   gpui=unknown(P1 未盘点)。\n\
         //\n\
         // 再生成(修改生产表后):\n\
         //   SCHEMA_DRIFT_GENERATE_AT=1 cargo test -p auto-lang --test schema_drift\n\
         // 围栏会强制:生产表 tag ⊆ 本文件(tags ∪ aliases),新增组件必须重生成。\n\
         //\n\
         // 别名匹配策略(P2 校验器实现):tag 与 aliases 均按折叠键匹配\n\
         // (剥 `-`/`_` + 小写),如 alert-dialog-action ≡ alert_dialog_action。\n\
         // 依据:widgets-gallery 实际使用 kebab 形态,生产表登记 underscore/concat\n\
         // 形态,管线靠隐式归一弥合 —— schema 将该归一显式化为策略。\n\n",
    );

    // 分 tier 输出,组内按 category、tag 排序
    let mut ordered: Vec<(&String, &BTreeSet<String>)> = final_groups.iter().collect();
    ordered.sort_by(|a, b| {
        let ta = derive_tier(a.1, inp);
        let tb = derive_tier(b.1, inp);
        ta.cmp(tb).then_with(|| a.0.cmp(b.0))
    });

    let mut current_tier = "";
    for (canonical, members) in ordered {
        let tier = derive_tier(members, inp);
        if tier != current_tier {
            out.push_str(&format!("\n// ============ tier: {} ============\n\n", tier));
            current_tier = tier;
        }
        let rs_def = inp.rs_elements.get(canonical);
        let category = rs_def
            .map(|d| d.category.to_lowercase())
            .filter(|c| !c.is_empty())
            .unwrap_or_else(|| "unknown".to_string());
        let aliases: Vec<String> = members
            .iter()
            .filter(|t| *t != canonical)
            .cloned()
            .collect();
        // web:组件映射优先于原生直通(map_tag 解析顺序)
        let web = if members.iter().any(|t| inp.in_vue_shadcn.contains(t)) {
            "component"
        } else if members.iter().any(|t| inp.in_vue_native.contains(t)) {
            "native"
        } else {
            "none"
        };
        let iced = members
            .iter()
            .find_map(|t| inp.render_levels.get(t).cloned())
            .filter(|l| !l.is_empty())
            .or_else(|| {
                if members.iter().any(|t| inp.in_vb.contains(t)) {
                    Some("unknown".to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "none".to_string());
        // sub_widgets:web 家族 = 同 vue import 路径、且名字以本元素为前缀的其他
        // canonical(select-* 归 select;divider 与 separator 同路径但非前缀,不收)
        let mut sub_widgets: BTreeSet<String> = BTreeSet::new();
        if let Some(own_path) = members
            .iter()
            .find_map(|t| inp.vue_imports.get(t).cloned())
        {
            let own_fold = fold_key(canonical);
            for (t, path) in &inp.vue_imports {
                if path != &own_path || members.contains(t) {
                    continue;
                }
                if let Some(c) = inp.canonical_of.get(t) {
                    // 并查集 root → 最终 canonical(rs 偏好可能改键)
                    let final_c = final_of_root.get(c).unwrap_or(c);
                    if final_c != canonical && fold_key(final_c).starts_with(&own_fold) {
                        sub_widgets.insert(final_c.clone());
                    }
                }
            }
        }
        let description = rs_def
            .map(|d| d.description.clone())
            .filter(|d| !d.is_empty())
            .or_else(|| {
                if inp.gallery_only.contains(canonical) {
                    Some(
                        "P1: used in widgets-gallery but unregistered in any \
                         production table (implicit fallback/ext path); P2 review"
                            .to_string(),
                    )
                } else {
                    None
                }
            })
            .unwrap_or_else(|| {
                "P1 extracted from production tables; props TBD".to_string()
            });

        out.push_str(&format!("element {} {{\n", canonical));
        out.push_str(&format!("    tag: \"{}\"\n", canonical));
        out.push_str(&format!("    category: \"{}\"\n", category));
        out.push_str(&format!("    tier: \"{}\"\n", tier));
        if !aliases.is_empty() {
            let list = aliases
                .iter()
                .map(|a| format!("\"{}\"", a))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!("    aliases: [{}]\n", list));
        }
        out.push_str(&format!(
            "    backends: {{ web: \"{}\", iced: \"{}\", gpui: \"unknown\" }}\n",
            web, iced
        ));
        if !sub_widgets.is_empty() {
            let list = sub_widgets
                .iter()
                .map(|a| format!("\"{}\"", a))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!("    sub_widgets: [{}]\n", list));
        }
        match rs_def {
            Some(d) if !d.props.is_empty() => {
                out.push_str("    props: [\n");
                for p in &d.props {
                    let mut entry = format!(
                        "        {{ name: \"{}\", type: \"{}\"",
                        p.name, p.type_str
                    );
                    if p.required {
                        entry.push_str(", required: true");
                    }
                    if let Some(def) = &p.default {
                        entry.push_str(&format!(", default: \"{}\"", def));
                    }
                    if !p.description.is_empty() {
                        entry.push_str(&format!(", description: \"{}\"", escape_at(&p.description)));
                    }
                    entry.push_str(" }");
                    out.push_str(&entry);
                    out.push('\n');
                }
                out.push_str("    ]\n");
            }
            _ => out.push_str("    props: []\n"),
        }
        out.push_str(&format!("    allows_children: {}\n", rs_def.map(|d| d.allows_children).unwrap_or(true)));
        out.push_str(&format!("    description: \"{}\"\n", escape_at(&description)));
        out.push_str("}\n\n");
    }
    out
}

fn escape_at(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn add(drift: &mut Drift, dim: &'static str, items: BTreeSet<String>) {
    if !items.is_empty() {
        drift.entry(dim).or_default().extend(items);
    }
}

fn only_in(a: &BTreeSet<String>, b: &BTreeSet<String>) -> BTreeSet<String> {
    a.difference(b).cloned().collect()
}

#[test]
fn schema_drift_fence() {
    let schema_rs = read("src/aura/schema.rs");
    let vue_rs = read("src/ui_gen/vue.rs");
    let vb_rs = read("src/ui/aura_view_builder.rs");
    let render_rs = read("src/ui/render_support.rs");
    let aura_at = read("../../schema/aura.at");

    // ---- 提取四表 + 结构断言 ----
    let rs_seq = scan_insert_tags(&schema_rs, "elements");
    let mut rs: BTreeSet<String> = BTreeSet::new();
    let mut rs_dup: BTreeSet<String> = BTreeSet::new();
    for tag in &rs_seq {
        if !rs.insert(tag.clone()) {
            rs_dup.insert(tag.clone());
        }
    }
    let at = scan_aura_at(&aura_at);
    // vue.rs 的重复 insert 与 schema.rs 同罪:HashMap 后写覆盖,前者是死代码。
    let vue_seq = scan_insert_tags(&vue_rs, "components");
    let mut vue_shadcn: BTreeSet<String> = BTreeSet::new();
    let mut vue_dup: BTreeSet<String> = BTreeSet::new();
    for tag in &vue_seq {
        if !vue_shadcn.insert(tag.clone()) {
            vue_dup.insert(tag.clone());
        }
    }

    let vb_tables = scan_match_tables(&vb_rs);
    assert_eq!(
        vb_tables.len(),
        EXPECTED_VB_TABLES,
        "view_builder 语句级 `match tag {{` 派发表数量变化({} -> {}):新表若属组件派发,\
         请把它的维度加进本测试并更新 baseline;小表请改名避开 `match tag {{`",
        EXPECTED_VB_TABLES,
        vb_tables.len()
    );
    let (vb0, vb1) = (&vb_tables[0], &vb_tables[1]);
    let vb_union: BTreeSet<String> = vb0.union(vb1).cloned().collect();

    let render_tables = scan_match_tables(&render_rs);
    assert_eq!(
        render_tables.len(),
        EXPECTED_RENDER_TABLES,
        "render_support `match tag {{` 表数量变化({} -> {})",
        EXPECTED_RENDER_TABLES,
        render_tables.len()
    );
    let render = &render_tables[0];

    let vue_tables = scan_match_tables(&vue_rs);
    assert_eq!(
        vue_tables.len(),
        EXPECTED_VUE_TABLES,
        "vue.rs 语句级 `match tag {{` 表数量变化({} -> {}):新表若属 tag 归属决策,\
         请加维度并更新 baseline",
        EXPECTED_VUE_TABLES,
        vue_tables.len()
    );
    let (vue_mt0, vue_mt1) = (&vue_tables[0], &vue_tables[1]);

    let parser_rs = read("src/parser.rs");
    let parser_tables = scan_match_tables(&parser_rs);
    assert_eq!(
        parser_tables.len(),
        EXPECTED_PARSER_TABLES,
        "parser.rs 语句级 `match tag {{` 表数量变化({} -> {}):新表若属 tag 知识,\
         请把它的维度加进本测试并更新 baseline;小表请改名避开 `match tag {{`",
        EXPECTED_PARSER_TABLES,
        parser_tables.len()
    );
    let parser_tags = &parser_tables[0];

    let a2ui_export_rs = read("src/a2ui/export.rs");
    let a2ui_export_tables = scan_match_tables(&a2ui_export_rs);
    assert_eq!(
        a2ui_export_tables.len(),
        EXPECTED_A2UI_EXPORT_TABLES,
        "a2ui/export.rs `match tag {{` 表数量变化({} -> {})",
        EXPECTED_A2UI_EXPORT_TABLES,
        a2ui_export_tables.len()
    );
    let a2ui_export = &a2ui_export_tables[0];
    let a2ui_import = scan_tag_literals(&read("src/a2ui/import.rs"));

    // ---- P1:别名归并 + 生成输入 ----
    let rs_elements = extract_rs_elements(&schema_rs);
    let mut render_levels: BTreeMap<String, String> = BTreeMap::new();
    for (tags, level) in scan_match_arm_groups_plus_singles(&render_rs) {
        if !level.is_empty() {
            for t in tags {
                render_levels.insert(t, level.clone());
            }
        }
    }
    let vue_imports = scan_vue_imports(&vue_rs);
    let mut prod_union: BTreeSet<String> = BTreeSet::new();
    for set in [
        &rs,
        &vue_shadcn,
        vue_mt0,
        vue_mt1,
        &vb_union,
        render,
        parser_tags,
        a2ui_export,
        &a2ui_import,
    ] {
        prod_union.extend(set.iter().cloned());
    }
    let mut arm_groups: Vec<Vec<String>> = Vec::new();
    // 只收渲染行为派发表(vb 两表/vue 两表);parser/render/a2ui 是归类或序列化
    // 等价表(同臂≠同一组件,如 a2ui "select"|"dropdown" 只是 A2UI 体相同)
    for src in [&vb_rs, &vue_rs] {
        arm_groups.extend(scan_match_arm_groups(src));
    }
    let canonical_of = build_alias_groups(&arm_groups, &prod_union, &rs);
    // 第 10 源:widgets-gallery 消费侧 tag —— 折叠匹配后仍无生产表登记的,
    // 各自成组入册(unclassified;隐式 fallback / ext 组件路径,P2 告警收编)
    let gallery_tags = scan_gallery_tags();
    let prod_fold: BTreeSet<String> =
        prod_union.iter().map(|t| fold_key(t)).collect();
    let mut canonical_of = canonical_of;
    let mut gallery_only: BTreeSet<String> = BTreeSet::new();
    for g in &gallery_tags {
        if !prod_fold.contains(&fold_key(g)) {
            canonical_of.insert(g.clone(), g.clone());
            gallery_only.insert(g.clone());
        }
    }
    let inp = AtGenInput {
        rs_elements,
        prod_union: prod_union.clone(),
        in_vb: vb_union.clone(),
        in_vue_shadcn: vue_shadcn.clone(),
        in_vue_native: vue_mt0.clone(),
        render_levels,
        vue_imports,
        gallery_only,
        canonical_of,
    };

    // 生成 gate(复核流程同 baseline)
    if std::env::var("SCHEMA_DRIFT_GENERATE_AT").is_ok() {
        let text = generate_aura_at(&inp);
        let at_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../schema/aura.at");
        fs::write(&at_path, &text).expect("write schema/aura.at");
        panic!(
            "schema/aura.at 已重生成({} bytes,{} 元素)—— 复核 diff 后重跑(不带环境变量)确认绿",
            text.len(),
            inp.canonical_of.values().collect::<BTreeSet<_>>().len()
        );
    }

    println!("四表规模: schema.rs={} aura.at={} vue.shadcn={} vb0={} vb1={} render={}",
        rs.len(), at.len(), vue_shadcn.len(), vb0.len(), vb1.len(), render.len());

    // ---- P1 覆盖断言(硬闸,无 baseline 豁免)----
    // 1) 生产表 tag ⊆ aura.at(tags ∪ aliases):新增组件必须重生成 schema。
    // 2) aura.at(tags ∪ aliases) ⊆ 生产表 ∪ schema.rs:不允许幻影声明。
    let at_aliases = scan_aura_at_aliases(&aura_at);
    let at_all: BTreeSet<String> = at.union(&at_aliases).cloned().collect();
    let mut missing: Vec<(&str, Vec<String>)> = Vec::new();
    for (name, set) in [
        ("schema.rs", &rs),
        ("vue.components", &vue_shadcn),
        ("vue.map_tag", vue_mt0),
        ("vue.pascal", vue_mt1),
        ("view_builder", &vb_union),
        ("render_support", render),
        ("parser", parser_tags),
        ("a2ui.export", a2ui_export),
        ("a2ui.import", &a2ui_import),
    ] {
        let m: Vec<String> = only_in(set, &at_all).into_iter().collect();
        if !m.is_empty() {
            missing.push((name, m));
        }
    }
    if !missing.is_empty() {
        let mut msg = String::from(
            "Plan 435 P1:生产表的 tag 未被 schema/aura.at 覆盖(tags ∪ aliases)。\n\
             修复:改完生产表后重生成 schema:\n\
             SCHEMA_DRIFT_GENERATE_AT=1 cargo test -p auto-lang --test schema_drift\n\
             (复核 diff 后不带环境变量重跑确认绿)\n\n",
        );
        for (name, tags) in &missing {
            msg.push_str(&format!("  [{}] {}\n", name, tags.join(", ")));
        }
        panic!("{}", msg);
    }
    let phantom: Vec<String> = only_in(&at_all, &prod_union)
        .into_iter()
        .filter(|t| !inp.gallery_only.contains(t))
        .collect();
    assert!(
        phantom.is_empty(),
        "Plan 435 P1:schema/aura.at 存在生产表与 schema.rs 之外的幻影条目(手改漂移?):\n  {}",
        phantom.join(", ")
    );

    // P1 附带验收:aura.at 必须能被 schema_loader 解析(P2 接线的前置条件)。
    {
        use auto_lang::aura::schema_loader::SchemaLoader;
        let mut loader = SchemaLoader::new();
        let parsed = loader
            .load(&aura_at)
            .expect("schema_loader 应能解析 schema/aura.at(生成格式回归?)");
        assert!(
            parsed.get_element("button").is_some(),
            "aura.at 解析结果缺 button"
        );
    }
    // ---- 维度语义(新增漂移时随报错一起打印) ----
    let mut drift: Drift = BTreeMap::new();
    add(&mut drift, "rs_duplicate_insert", rs_dup);
    add(&mut drift, "vue_duplicate_insert", vue_dup);
    add(&mut drift, "vue_not_in_rs", only_in(&vue_shadcn, &rs));
    add(&mut drift, "rs_not_in_vue", only_in(&rs, &vue_shadcn));
    add(&mut drift, "vb_not_in_rs", only_in(&vb_union, &rs));
    add(&mut drift, "rs_not_in_vb", only_in(&rs, &vb_union));
    add(&mut drift, "vb0_not_in_vb1", only_in(vb0, vb1));
    add(&mut drift, "vb1_not_in_vb0", only_in(vb1, vb0));
    add(&mut drift, "render_not_in_rs", only_in(render, &rs));
    add(&mut drift, "rs_not_in_render", only_in(&rs, render));
    add(&mut drift, "vb_not_in_render", only_in(&vb_union, render));
    add(&mut drift, "render_not_in_vb", only_in(render, &vb_union));
    add(&mut drift, "vue_mt0_not_in_rs", only_in(vue_mt0, &rs));
    add(&mut drift, "vue_mt1_not_in_rs", only_in(vue_mt1, &rs));
    add(&mut drift, "parser_not_in_rs", only_in(parser_tags, &rs));
    add(&mut drift, "a2ui_export_not_in_rs", only_in(a2ui_export, &rs));
    add(&mut drift, "a2ui_import_not_in_rs", only_in(&a2ui_import, &rs));

    // ---- baseline:只拦新增 ----
    let baseline_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/schema_drift_baseline.txt");
    if std::env::var("SCHEMA_DRIFT_UPDATE_BASELINE").is_ok() {
        let mut text = String::from(
            "# Plan 435 P0 —— schema 漂移围栏 baseline(已知漂移白名单)\n\
             # 格式: 维度<TAB>tag,按字母序。围栏只拦 baseline 之外的新增漂移;\n\
             # 漂移消除后请顺手裁剪本文件。更新方式:\n\
             # SCHEMA_DRIFT_UPDATE_BASELINE=1 cargo test -p auto-lang --test schema_drift\n\
             # 约定:新增 baseline 条目必须在提交信息里写明理由(重生成会覆盖手动注释,\n\
             # 所以理由落在 commit message,不在本文件)。\n\n",
        );
        for (dim, tags) in &drift {
            for tag in tags {
                text.push_str(&format!("{}\t{}\n", dim, tag));
            }
        }
        fs::write(&baseline_path, text).expect("write baseline");
        panic!(
            "baseline 已重写 {} —— 复核 diff 后重跑(不带环境变量)确认绿",
            baseline_path.display()
        );
    }
    let baseline_text = fs::read_to_string(&baseline_path).unwrap_or_else(|_| {
        panic!(
            "baseline 缺失: {} —— 先跑一次:\n\
             SCHEMA_DRIFT_UPDATE_BASELINE=1 cargo test -p auto-lang --test schema_drift",
            baseline_path.display()
        )
    });
    let mut baseline: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for line in baseline_text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((dim, tag)) = line.split_once('\t') {
            baseline.entry(dim.to_string()).or_default().insert(tag.to_string());
        }
    }

    // 已消除的漂移:提示裁剪,不拦。
    let known_dims: BTreeSet<&str> = drift.keys().copied().collect();
    for (dim, tags) in &baseline {
        if !known_dims.contains(dim.as_str()) {
            println!("[schema-drift] baseline 维度 `{dim}` 已不在围栏中(计划推进后移除?),请裁剪");
            continue;
        }
        let current = &drift[dim.as_str()];
        for resolved in tags.difference(current) {
            println!("[schema-drift] 漂移已消除,请裁剪 baseline: {dim}\t{resolved}");
        }
    }

    // 新增漂移:红。
    let mut fresh: Vec<(String, Vec<String>)> = Vec::new();
    for (dim, current) in &drift {
        let allowed = baseline.get(*dim);
        let new_tags: Vec<String> = current
            .iter()
            .filter(|t| allowed.map_or(true, |a| !a.contains(*t)))
            .cloned()
            .collect();
        if !new_tags.is_empty() {
            fresh.push((dim.to_string(), new_tags));
        }
    }
    if !fresh.is_empty() {
        let mut msg = String::from(
            "Plan 435 P0 漂移围栏:发现 baseline 之外的新增漂移(四表不同步)。\n\
             维度语义:\n\
             - rs/vue_duplicate_insert: schema.rs / vue.rs 同一 tag insert 两次(HashMap 后写覆盖,前者是死代码)\n\
             - *_not_in_rs / rs_not_in_*: 声明表(schema.rs)与各实现表的孤儿 tag\n\
             - parser_not_in_rs: parser.rs tag 特判表里的孤儿(含 PascalCase 变体)\n\
             - a2ui_export/import_not_in_rs: A2UI 互操作映射的孤儿 tag\n\
             - vb0/vb1_not_in_vb1/vb0: view_builder 两张派发表不镜像(D-GAP 纪律)\n\
             - vb/render_not_in_render/vb: 桌面实现与 iced 支持级表不同步\n\
             - vue_mt0/1_not_in_rs: vue.rs map_tag 原生映射 / Pascal 归一表的孤儿\n\
             修复二选一:同步四表(推荐);或确属有意漂移,更新 baseline 并写明理由:\n\
             SCHEMA_DRIFT_UPDATE_BASELINE=1 cargo test -p auto-lang --test schema_drift\n\n",
        );
        for (dim, tags) in &fresh {
            msg.push_str(&format!("  [{}] {}\n", dim, tags.join(", ")));
        }
        panic!("{}", msg);
    }
}
