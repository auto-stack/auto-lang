//! Plan 435 P5 —— schema 驱动的组件文档系统(生成器 + 三道围栏)。
//!
//! 1. **核心组件参考生成**:tier ∈ {builtin_widget, native_html} 的元素生成
//!    `docs/components/core.md`(tier/后端徽章 + props 表 + 别名/家族),
//!    website 经 prepare-content(DOCS_INCLUDE 含 components)自动收录。
//!    再生成:`DOCS_GEN_UPDATE=1 cargo test -p auto-lang --test docs_gen`
//! 2. **文档覆盖围栏**:主组件(非 subwidget、tier ∈ {builtin_widget,
//!    web_component})必须有 gallery 页面;白名单=他处文档化;基线=已知
//!    文档债(冻结,新增未文档化组件即红)。
//! 3. **Properties 对拍一致性**:gallery 页面手写 Properties 表不得与
//!    schema 矛盾(声明了 props 的元素;手写 prop 必须 ∈ schema)。

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fold(s: &str) -> String {
    s.chars()
        .filter(|c| *c != '-' && *c != '_')
        .collect::<String>()
        .to_lowercase()
}

/// (canonical, tier, category, description, aliases, sub_widgets, props, backends)
struct ElemInfo {
    canonical: String,
    tier: String,
    category: String,
    description: String,
    aliases: Vec<String>,
    sub_widgets: Vec<String>,
    props: Vec<(String, String, String, String)>, // (name, type, default, desc)
    web: String,
    iced: String,
}

fn prop_type_str(t: &auto_lang::aura::schema::PropType) -> String {
    use auto_lang::aura::schema::PropType;
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

fn load_elements() -> Vec<ElemInfo> {
    let schema = auto_lang::aura::default_schema_cached().expect("schema");
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
        });
    }
    out
}

// ============================================================================
// 1) 核心组件参考生成(docs/components/core.md)
// ============================================================================

fn generate_core_reference() -> String {
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
         > `DOCS_GEN_UPDATE=1 cargo test -p auto-lang --test docs_gen`\n\
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

// ============================================================================
// 2) 文档覆盖围栏
// ============================================================================

/// 白名单:他处已文档化(fold 键)。
const DOC_EXCLUDE: &[&str] = &[
    // 语义 HTML/Layout 组页面(alignment/scroll/position/...)文档化
    "article", "aside", "footer", "header", "main", "nav", "section",
    "p", "span", "h1", "h2", "h3",
    // 基础文本原语,Layout 组 demo 内文档化
    "text",
    // form 家族件(form 页文档化)
    "formcontrol", "formdescription", "formfield", "formlabel", "formmessage",
    // 家族子件随家族页文档化
    "cardaction",        // card 页
    "datepickertrigger", // datepicker 页
    "dropdownmenuseparator", // dropdownmenu 页
    "toastprovider",     // toast 页
    "tabtrigger",        // tabs 页(Pascal 拼写变体)
];

/// 基线:已知文档债(fold 键;冻结,新增即红)。
const DOC_TODO_BASELINE: &[&str] = &[
    "areachart", "autodown", "barchart", "box", "chart", "chatmessage",
    "chip", "container", "date", "datetime", "datetimeinput", "divider",
    "donutchart", "griditem", "icon", "image", "img", "list", "markdown",
    "mermaid", "modal", "navmenu", "radioitem", "range", "spacer", "square",
    "stack", "svg", "swiper", "tag", "toolbar", "listitem",
    "navdestination",
];

fn gallery_page_folds() -> BTreeSet<String> {
    let dir = repo_root().join("examples/widgets-gallery/src/front/pages");
    let mut out = BTreeSet::new();
    if let Ok(rd) = fs::read_dir(&dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().map_or(false, |x| x == "at") {
                if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                    if stem != "index" {
                        out.insert(fold(stem));
                    }
                }
            }
        }
    }
    out
}

#[test]
fn docs_coverage_fence() {
    let elems = load_elements();
    let mut subwidget_folds: BTreeSet<String> = BTreeSet::new();
    for e in &elems {
        for s in &e.sub_widgets {
            subwidget_folds.insert(fold(s));
        }
    }
    let pages = gallery_page_folds();
    let excluded: BTreeSet<String> = DOC_EXCLUDE.iter().map(|s| s.to_string()).collect();
    let baseline: BTreeSet<String> = DOC_TODO_BASELINE.iter().map(|s| s.to_string()).collect();

    let mut fresh: Vec<String> = Vec::new();
    for e in &elems {
        if e.tier != "builtin_widget" && e.tier != "web_component" {
            continue;
        }
        let f = fold(&e.canonical);
        if subwidget_folds.contains(&f)
            || pages.contains(&f)
            || excluded.contains(&f)
            || baseline.contains(&f)
        {
            continue;
        }
        fresh.push(format!("{}({})", e.canonical, e.tier));
    }
    assert!(
        fresh.is_empty(),
        "Plan 435 P5 文档覆盖围栏:新增主组件未文档化(widgets-gallery 页/\
         白名单/基线三选一):\n  {}\n修复:补 gallery 页(推荐),或加入 \
         docs_gen.rs 的 DOC_EXCLUDE(注明何处文档化);确属文档债才进 \
         DOC_TODO_BASELINE。",
        fresh.join(", ")
    );

    // 基线漂移提示(已覆盖的基线条目请裁剪)
    for b in &baseline {
        if pages.contains(b) {
            println!("[docs] 基线条目已覆盖,请裁剪: {b}");
        }
    }
}

// ============================================================================
// 3) gallery Properties 表 vs schema 对拍一致性
// ============================================================================

/// 从页面 .at 源提取 Properties 表的 prop 名序列(tr 的首个 td)。
fn page_prop_names(src: &str) -> Vec<String> {
    let mut props = Vec::new();
    let mut in_table = false;
    for line in src.lines() {
        let t = line.trim();
        if t.starts_with("table") && t.ends_with('{') {
            in_table = true;
            continue;
        }
        if in_table && t == "}" {
            break;
        }
        if in_table && t.starts_with("td \"") {
            // 每行一个 td;表头行(Property...)之后的首列即 prop 名
            let val = t.trim_start_matches("td \"");
            if let Some(end) = val.find('"') {
                props.push(val[..end].to_string());
            }
        }
    }
    // 去表头:第一组以 "Property" 开头 → 移除首个 "Property"
    if props.first().map_or(false, |p| p == "Property") {
        props.remove(0);
    }
    props
}

#[test]
fn gallery_properties_conform_to_schema() {
    use auto_lang::aura::default_schema_cached;
    let schema = default_schema_cached().expect("schema");
    let dir = repo_root().join("examples/widgets-gallery/src/front/pages");
    let mut violations: Vec<String> = Vec::new();

    for e in fs::read_dir(&dir).expect("pages dir").flatten() {
        let p = e.path();
        if !p.extension().map_or(false, |x| x == "at") {
            continue;
        }
        let stem = p.file_stem().unwrap().to_string_lossy().to_string();
        let Ok(src) = fs::read_to_string(&p) else { continue };
        let props = page_prop_names(&src);
        if props.is_empty() {
            continue;
        }
        // 页面对应元素(schema 三级解析)
        let Some((canon, def)) = schema.resolve_tag(&stem) else {
            violations.push(format!("{stem}: 页面无对应 schema 元素"));
            continue;
        };
        // 空 props 元素跳过(schema 未声明,无从对拍;P5 后续以文档表回填)
        if def.props.is_empty() {
            continue;
        }
        for name in props {
            let universal = name == "class" || name == "style" || name.starts_with("on");
            if !universal && def.get_prop(&name).is_none() {
                violations.push(format!(
                    "{stem}(={canon}): 手写表含 `{name}`,schema 未声明 —— 文档/声明漂移",
                ));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "Plan 435 P5 对拍:gallery Properties 表与 schema 矛盾:\n  {}",
        violations.join("\n  ")
    );
}

// ============================================================================
// 生成物同步围栏(core.md 与 schema 同步)
// ============================================================================

#[test]
fn core_reference_in_sync() {
    let path = repo_root().join("docs/components/core.md");
    let generated = generate_core_reference();
    if std::env::var("DOCS_GEN_UPDATE").is_ok() {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, &generated).expect("write core.md");
        panic!(
            "docs/components/core.md 已重写 —— 复核 diff 后重跑(不带环境变量)确认绿"
        );
    }
    let committed =
        fs::read_to_string(&path).unwrap_or_else(|_| {
            panic!(
                "core.md 缺失: {} —— 先生成:\n\
                 DOCS_GEN_UPDATE=1 cargo test -p auto-lang --test docs_gen",
                path.display()
            )
        });
    assert_eq!(
        committed, generated,
        "docs/components/core.md 与 schema 不同步 —— 再生成:\n\
         DOCS_GEN_UPDATE=1 cargo test -p auto-lang --test docs_gen"
    );
}
