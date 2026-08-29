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

// P8-1(D13):生成器本体在库侧(ui_gen::docs_gen);本测试只保留
// 仓库纪律 —— 覆盖围栏、对拍一致性、生成物同步。CLI 入口:auto docs gen。
use auto_lang::ui_gen::docs_gen::{
    generate_core_reference, generate_kitchen_sink, load_elements,
};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fold(s: &str) -> String {
    s.chars()
        .filter(|c| *c != '-' && *c != '_')
        .collect::<String>()
        .to_lowercase()
}

// ============================================================================
// 2) 文档覆盖围栏
// ============================================================================

/// 白名单:他处已文档化(fold 键)。
const DOC_EXCLUDE: &[&str] = &[
    // 语义 HTML/Layout 组页面(alignment/scroll/position/...)文档化
    "article", "aside", "footer", "header", "main", "nav", "section",
    "p", "span", "h1", "h2", "h3",
    // Plan 019 Phase 4:文本格式化/标题尾族随 aura.at 重排进入
    // builtin_widget 层——语义与 h1-h3/p/span 同族，随 Layout 组文档化
    // （层级 h4-h6 与行内 b/i/em/strong/small/link 不另设页面）。
    "b", "em", "h4", "h5", "h6", "i", "link", "small", "strong",
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
    // Plan 435 P8-3(D11)批量归类配套:家族件随家族页文档化(父族在
    // cmd_vue.rs 安装表或官方包;tier 已归 web_component,文档面随页)。
    "autocomplete", "autocompleteempty", "autocompleteinput", // combobox 页
    "autocompleteitem", "autocompletelist",
    "buttongroup",      // button 页(变体分组)
    "combobox", "comboboxanchor", "comboboxempty", "comboboxgroup", // combobox 页
    "comboboxinput", "comboboxitem", "comboboxlist", "comboboxtrigger",
    "command", "commandempty", "commandgroup", "commandinput", // command 页
    "commanditem", "commandlist", "commandseparator", "commandshortcut",
    "contextmenushortcut", "contextmenucheckboxitem", "contextmenulabel", // contextmenu 页
    "contextmenuradiogroup", "contextmenuradioitem", "contextmenuseparator",
    "contextmenusub", "contextmenusubcontent", "contextmenusubtrigger",
    "drawerclose",      // drawer 页
    "dropdown", "dropdowncontent", "dropdownitem", "dropdownlabel", // dropdownmenu 页
    "dropdownseparator", "dropdowntrigger",
    "field", "formitem", // form 页
    "inputgroup", "inputotp", "kbd", // input 页 / kbd(无页,键帽原语)
    "loading",           // skeleton 页(同义)
    "menubarlabel", "menubarseparator", // menubar 页
    "nativeselect",     // select 页
    "numberfield", "numberfielddecrement", "numberfieldincrement", // number-field 家族(无独立页,input 页提及)
    "numberfieldinput", "numberinput",
    "paginationfirst", "paginationlast", "paginationprev", // pagination 页
    "pininput", "pininputgroup", "pininputseparator", "pininputslot", // pin-input 家族(无独立页)
    "resizable", "resizablehandle", "resizablepanel", // resizable 家族(无独立页)
    "scrollview",       // scrollarea 页
    "selectseparator", "selectscrollbutton", // select 页
    "sidebargroup", "sidebargroupcontent", "sidebargrouplabel", // sidebar 页
    "sidebarprovider", "sidebartrigger",
    "stepper", "stepperdescription", "stepperindicator", "stepperitem", // stepper 家族(无独立页)
    "stepperseparator", "steppertitle", "steppertrigger",
    "tagsinput", "tagsinputdelete", "tagsinputfield", "tagsinputitem", // tags-input 家族(无独立页)
    "togglegroup", "togglegroupitem", // togglegroup 页
    "toaster",           // toast 页(宿主)
    "embed", "query",    // iframe 嵌入与数据查询原语
];

/// 基线:已知文档债(fold 键;冻结,新增即红)。
/// P8-4:已覆盖条目 = 红(见 docs_coverage_fence 尾部断言;首批已裁
/// areachart/barchart/donutchart —— area-chart/bar-chart/donut-chart 页)。
const DOC_TODO_BASELINE: &[&str] = &[
    "autodowneditor", "box", "chart", "chatmessage",
    "chip", "container", "date", "datetime", "datetimeinput", "divider",
    "griditem", "icon", "image", "img", "list", "markdown",
    "mermaid", "modal", "navmenu", "radioitem", "range", "spacer", "square",
    "stack", "svg", "swiper", "tag", "toolbar", "listitem",
    "navdestination",
    // Plan 463 desktop shell 新增(taskbar),文档化跟随该计划的文档批次。
    "taskbar",
    // Plan 473(桌面 dock 线)并入的 virtual_window——该线只跑了 --lib 门禁,
    // docs_gen 围栏在此暴露缺口;文档化归 473 后续批次(Plan 482 裁定旁置;
    // 481 独立同因修复,merge 去重保留单条)。
    "virtualwindow",
    // Plan 481:slot 升 builtin_widget(plan050 视图管线已消费)但属语言级
    // slot 机制(Plan 476 实现),非画廊组件——文档化另行批次。
    "slot",
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

    // Plan 435 P8-4(D12):基线条目已被 gallery 页覆盖 = 红(冻结不腐化;
    // 原 println 提示从未触发裁剪,33 条只增不减)。覆盖判定 = 折叠键相同。
    let mut covered: Vec<String> = Vec::new();
    for b in &baseline {
        if pages.contains(b) {
            covered.push(b.clone());
        }
    }
    assert!(
        covered.is_empty(),
        "Plan 435 P8-4:DOC_TODO_BASELINE 条目已有 gallery 页,请裁剪:\n  {}\n\
         (文档债清偿后基线必须收缩,冻结不等于免检)",
        covered.join(", ")
    );
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
// 4) kitchen-sink demo 页生成(gallery pages/kitchen-sink.at)
//    schema 驱动的核心组件示例:每个「builtin_widget 且含可字面化 props」的
//    元素一节(默认 + 每 one_of 取值一个变体,至多 3)。
// ============================================================================

// ============================================================================
// 生成物同步围栏(core.md 与 schema 同步)
// ============================================================================

#[test]
fn core_reference_in_sync() {
    let path = repo_root().join("docs/components/core.md");
    let generated = generate_core_reference(&repo_root());
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

#[test]
fn kitchen_sink_page_in_sync() {
    let path = repo_root().join("examples/widgets-gallery/src/front/pages/kitchen-sink.at");
    let generated = generate_kitchen_sink();
    if std::env::var("KITCHEN_SINK_UPDATE").is_ok() {
        fs::write(&path, &generated).expect("write kitchen-sink.at");
        panic!(
            "kitchen-sink.at 已重写 —— 复核 diff 后重跑(不带环境变量)确认绿(golden 需同步重采样)"
        );
    }
    let committed = fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!(
            "kitchen-sink.at 缺失 —— 先生成:
KITCHEN_SINK_UPDATE=1 cargo test -p auto-lang --test docs_gen"
        )
    });
    assert_eq!(
        committed, generated,
        "kitchen-sink.at 与 schema 不同步 —— 再生成:
KITCHEN_SINK_UPDATE=1 cargo test -p auto-lang --test docs_gen"
    );
}
