//! Plan 435 P5 / P8-1(D13)—— schema 驱动的组件文档生成器(库侧)。
//!
//! 从 schema/aura.at 生成两份产物:
//! - `generate_core_reference()`:核心组件参考 Markdown
//!   (tier ∈ {builtin_widget, native_html};tier/后端徽章 + props 表 + 别名/家族);
//! - `generate_kitchen_sink()`:kitchen-sink demo 页 `.at`
//!   (builtin_widget 层含可字面化 props 的元素,每节默认形态 + 变体)。
//!
//! 调用方:`auto docs gen` CLI(crates/auto)与 tests/docs_gen.rs 同步围栏。
//! 围栏(文档覆盖/对拍)留在测试侧 —— 生成是库能力,纪律是仓库纪律。

use crate::aura::schema::PropType;

/// (canonical, tier, category, description, aliases, sub_widgets, props, backends)
pub struct ElemInfo {
    pub canonical: String,
    pub tier: String,
    pub category: String,
    pub description: String,
    pub aliases: Vec<String>,
    pub sub_widgets: Vec<String>,
    pub props: Vec<(String, String, String, String)>, // (name, type, default, desc)
    pub web: String,
    pub iced: String,
    /// P7-2/D9:vue 组件 import 路径 —— 非 @/components/ui/* 的是 app 本地
    /// 命令式外壳(CodeEditor/AutoDownEditor/ChatMessage),props 不按
    /// 字面 prop 绑定,kitchen-sink 变体发射须跳过。
    pub vue_import: Option<String>,
}

pub fn prop_type_str(t: &PropType) -> String {
    match t {
        PropType::String => "string".into(),
        PropType::Int => "int".into(),
        PropType::Float => "float".into(),
        PropType::Bool => "bool".into(),
        PropType::Color => "color".into(),
        PropType::StateRef => "state_ref".into(),
        PropType::MsgRef => "msg_ref".into(),
        PropType::Expr => "expr".into(),
        PropType::Closure => "closure".into(),
        PropType::StyleBinding => "class_binding".into(),
        PropType::Interpolated => "interpolated".into(),
        PropType::Union(v) => {
            let inner: Vec<String> = v.iter().map(prop_type_str).collect();
            format!("union: {}", inner.join("|"))
        }
        PropType::OneOf(v) => format!("one_of: {}", v.join("|")),
    }
}

pub fn load_elements() -> Vec<ElemInfo> {
    let schema = crate::aura::default_schema_cached().expect("schema");
    let mut out = Vec::new();
    for (tag, def) in &schema.elements {
        let meta = schema.meta.get(*tag);
        let tier = meta
            .map(|m| m.tier.as_str())
            .unwrap_or("unclassified")
            .to_string();
        let web = meta.map(|m| m.backends.web.clone()).unwrap_or_default();
        let iced = meta.map(|m| m.backends.iced.clone()).unwrap_or_default();
        out.push(ElemInfo {
            canonical: tag.to_string(),
            tier,
            category: format!("{:?}", def.category).to_lowercase(),
            description: def.description.to_string(),
            aliases: meta.map(|m| m.aliases.iter().map(|a| a.to_string()).collect())
                .unwrap_or_default(),
            sub_widgets: meta
                .map(|m| m.sub_widgets.iter().map(|s| s.to_string()).collect())
                .unwrap_or_default(),
            props: def
                .props
                .iter()
                .map(|p| {
                    (
                        p.name.to_string(),
                        prop_type_str(&p.type_),
                        p.default.unwrap_or("—").to_string(),
                        p.description.to_string(),
                    )
                })
                .collect(),
            web,
            iced,
            vue_import: meta.and_then(|m| m.vue.as_ref()).and_then(|v| v.import.clone()),
        });
    }
    out
}

/// Plan 435 P8-7(D13):gallery 页面 stem 折叠键表(fold → 页名 stem)。
/// `root` = 仓库根(core.md 的 demo 链接以 widgets-gallery 页路由为目标)。
pub fn gallery_page_stems(root: &std::path::Path) -> std::collections::BTreeMap<String, String> {
    let dir = root.join("examples/widgets-gallery/src/front/pages");
    let mut out = std::collections::BTreeMap::new();
    if let Ok(rd) = std::fs::read_dir(&dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().map_or(false, |x| x == "at") {
                if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                    if stem != "index" {
                        out.insert(fold_str(stem), stem.to_string());
                    }
                }
            }
        }
    }
    out
}

fn fold_str(s: &str) -> String {
    s.chars()
        .filter(|c| *c != '-' && *c != '_')
        .collect::<String>()
        .to_lowercase()
}

/// 核心组件参考(docs/components/core.md)生成。
/// `root` = 仓库根(P8-7:demo 链接需查 gallery 页面表)。
pub fn generate_core_reference(root: &std::path::Path) -> String {
    let demo_pages = gallery_page_stems(root);
    let mut elems = load_elements();
    elems.retain(|e| e.tier == "builtin_widget" || e.tier == "native_html");
    elems.sort_by(|a, b| {
        let ra = if a.tier == "builtin_widget" { 0 } else { 1 };
        let rb = if b.tier == "builtin_widget" { 0 } else { 1 };
        ra.cmp(&rb).then(a.canonical.cmp(&b.canonical))
    });

    let mut md = String::new();
    md.push_str("---\ntitle: 核心组件参考(Core Components)\n---\n\n");
    md.push_str(
        "> 本页由 schema/aura.at 生成(Plan 435 P5)—— **勿手改**;再生成:\n\
         > `auto docs gen`(推荐;测试内等价 `DOCS_GEN_UPDATE=1 cargo test -p auto-lang --test docs_gen`)\n\
         > tier 语义:`builtin_widget`=桌面有实现;`native_html`=Web 原生直通。\n\
         > shadcn 家族组件的活文档/Demo 见 widgets-gallery(本页仅收核心层)。\n\n",
    );

    for tier in ["builtin_widget", "native_html"] {
        md.push_str(&format!("## {}\n\n", if tier == "builtin_widget" {
            "内置组件(builtin_widget)"
        } else {
            "原生直通(native_html)"
        }));
        for e in elems.iter().filter(|e| e.tier == tier) {
            md.push_str(&format!("### `{}`\n\n", e.canonical));
            md.push_str(&format!(
                "`{}` · `{}` · web: `{}` · iced: `{}` · category: `{}`\n\n",
                e.tier, e.canonical, e.web, e.iced, e.category
            ));
            // P8-7:有 gallery 页的元素加 demo 链接(VitePress → gallery 互链)
            if let Some(stem) = demo_pages.get(&fold_str(&e.canonical)) {
                md.push_str(&format!(
                    "[demo →](/examples/widgets-gallery/{stem})

"
                ));
            }
            if !e.description.is_empty() {
                md.push_str(&format!("{}\n\n", e.description));
            }
            if !e.aliases.is_empty() {
                md.push_str(&format!(
                    "别名:{}\n\n",
                    e.aliases.iter().map(|a| format!("`{}`", a)).collect::<Vec<_>>().join(" ")
                ));
            }
            if e.props.is_empty() {
                md.push_str("_props 待声明_\n\n");
            } else {
                md.push_str("| Prop | Type | Default | Description |\n|------|------|---------|-------------|\n");
                for (n, ty, d, desc) in &e.props {
                    md.push_str(&format!(
                        "| `{}` | `{}` | {} | {} |\n",
                        n, ty, d, desc
                    ));
                }
                md.push('\n');
            }
            if !e.sub_widgets.is_empty() {
                md.push_str(&format!(
                    "子件:{}\n\n",
                    e.sub_widgets.iter().map(|s| format!("`{}`", s)).collect::<Vec<_>>().join(" ")
                ));
            }
            md.push_str("---\n\n");
        }
    }
    md
}

fn literal_prop_variants(prop: &(String, String, String, String)) -> Vec<String> {
    // 返回该 prop 的示例取值(字面可写的);空 = 跳过
    let (name, ty, _default, _desc) = prop;
    if name == "class" || name == "style" || name.starts_with("on") {
        return Vec::new();
    }
    if let Some(opts) = ty.strip_prefix("one_of:") {
        // prop_type_str 用 | 连接 OneOf 取值
        return opts.split('|').take(3).map(|v| v.trim().to_string()).collect();
    }
    match ty.as_str() {
        "string" => vec!["sample".to_string()],
        "bool" => vec!["true".to_string()],
        "int" | "float" | "color" => vec!["1".to_string()],
        _ => Vec::new(), // state_ref/msg_ref/expr/closure/union —— 需状态,跳过
    }
}

/// kitchen-sink demo 页(examples/widgets-gallery/src/front/pages/kitchen-sink.at)生成。
pub fn generate_kitchen_sink() -> String {
    let mut elems = load_elements();
    elems.retain(|e| {
        e.tier == "builtin_widget" && e.props.iter().any(|pr| !literal_prop_variants(pr).is_empty())
    });
    elems.sort_by(|a, b| a.canonical.cmp(&b.canonical));

    let mut at = String::new();
    at.push_str("// Plan 435 P5b —— kitchen-sink demo 页(schema/aura.at 生成,**勿手改**)\n");
    at.push_str("// 再生成:auto docs gen(测试内等价 KITCHEN_SINK_UPDATE=1 cargo test -p auto-lang --test docs_gen)\n");
    at.push_str("// 覆盖:builtin_widget 层含可字面化 props 的全部元素(当前 ");
    at.push_str(&format!("{}", elems.len()));
    at.push_str(" 个)。\n\n");
    at.push_str("widget KitchenSinkPage {\n");
    at.push_str("    msg Msg { Go }\n");
    at.push_str("    model { dummy int = 0 }\n");
    at.push_str("    on { .Go -> { } }\n");
    at.push_str("    view {\n");
    at.push_str("        col (style: \"p-6 space-y-8\") {\n");
    at.push_str("            h1 \"Kitchen Sink\"\n");
    at.push_str("            text \"核心组件全量示例 —— schema 生成页,展示每个组件的 props 取值。\" { style: \"text-muted-foreground\" }\n");
    for e in &elems {
        at.push_str(&format!("\n            h2 \"{}\"\n", e.canonical));
        at.push_str("            row (style: \"gap-2 flex-wrap items-center\") {\n");
        // 默认形态(裸标签或 text 简写)
        if e.props.iter().any(|pr| pr.0 == "text") {
            at.push_str(&format!("                {} \"sample\" {{}}\n", e.canonical));
        } else {
            at.push_str(&format!("                {} {{}}\n", e.canonical));
        }
        // 变体:每 prop 至多 2 个取值,总变体至多 4。
        // P7-2/D9:app 本地命令式外壳组件(import 非 @/components/ui/*)的
        // props 不按字面绑定(如 code_editor 的 key/content 在真实组件上
        // 不存在),只发射默认形态。
        let shell_component = e
            .vue_import
            .as_deref()
            .map_or(false, |p| !p.starts_with("@/components/ui/"));
        let mut variants = 0;
        for pr in &e.props {
            if shell_component || variants >= 4 { break; }
            for v in literal_prop_variants(pr).into_iter().take(2) {
                if variants >= 4 { break; }
                if pr.0 == "text" { continue; } // text 已用简写
                at.push_str(&format!(
                    "                {} ({}: {}) {{}}\n",
                    e.canonical, pr.0, quote_if_str(&v, &pr.1)
                ));
                variants += 1;
            }
        }
        at.push_str("            }\n");
    }
    at.push_str("        }\n");
    at.push_str("    }\n");
    at.push_str("}\n");
    at
}

fn quote_if_str(v: &str, ty: &str) -> String {
    if ty.starts_with("one_of:") || ty == "string" || ty == "color" {
        format!("\"{}\"", v)
    } else {
        v.to_string()
    }
}
