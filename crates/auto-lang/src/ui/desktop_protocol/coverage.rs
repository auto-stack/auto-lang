// Plan 500 步骤 3 —— 覆盖度探测（G4 / 设计 §1.3"覆盖度探测"）。
//
// [`Coverage`] 能力表（§1.3.1 爬坡目标集 = 单一事实源，与投影器实现
// （步骤 7）共同演进）vs App 视图清单（[`scan_view`] 装载期静态扫描）
// → [`Verdict`]。`auto` 裁决链消费（步骤 6 接线）：Covered → queue；
// NotCovered → 降级 independent + 宿主观测 `Log` 一行（缺项清单即载荷）。
// 未覆盖项显式列出——**禁止静默错绘**（G4）。
//
// 归一化：tag 小写；文本承载标签族（h1–h6/p/span/label）归一 kind
// "text"（与投影器 `is_text_tag` 同集）。样式类按前缀规则判定（布局/
// 盒模/装饰/排版子集——`ui/style` 词汇；hover: 交互态前缀可解析，queue
// 臂 v1 静态渲染忽略交互态）。

use std::collections::{BTreeMap, BTreeSet};

use crate::aura::{AuraEvent, AuraNode};

/// 归一化：折叠键（剥 `-`/`_` + 小写——aura.at 别名匹配策略同口径）；
/// 文本承载标签族（h1–h6/p/span/label）归一 kind "text"（与投影器
/// `is_text_tag` 同集）。
pub fn normalize_kind(tag: &str) -> String {
    let t: String = tag
        .chars()
        .filter(|c| *c != '-' && *c != '_')
        .collect::<String>()
        .to_ascii_lowercase();
    match t.as_str() {
        "text" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "p" | "span" | "label"
        // Plan 507 T6 —— typography 族（文本承载；投影器按 tag 缺省档
        // 分风格——归一仅覆盖判定用）。
        | "b" | "em" | "i" | "strong" | "small" | "code" | "pre" | "blockquote"
        | "quote" | "heading" | "codeblock" | "codepane" | "figcaption" => {
            "text".to_string()
        }
        _ => t,
    }
}

/// `AppProjector` 能力表：widget kind × prop × 事件 × 布局/构造 × 样式类
/// 前缀规则（§1.3.1 目标集；随投影器爬坡同步扩表）。
#[derive(Debug, Clone, Default)]
pub struct Coverage {
    /// 可投影 widget kind（归一名：text/button/input/image/a）。
    pub kinds: BTreeSet<String>,
    /// kind → 允许的非事件 prop 名（位置参数 sugar 落点 text/label 计入）。
    pub props: BTreeMap<String, BTreeSet<String>>,
    /// kind → 允许的事件名（零参 handler 才可投影，带参另列）。
    pub events: BTreeMap<String, BTreeSet<String>>,
    /// 布局容器/视图构造标签（center/col/row + if 条件块）。
    pub layouts: BTreeSet<String>,
    /// 支持的样式类前缀/裸类（tailwind 词汇子集；hover: 前缀整体放行）。
    pub style_prefixes: BTreeSet<String>,
}

impl Coverage {
    /// §1.3.1 爬坡目标集（001–005 实测清单）+ Plan 507 T3 display 族扩展
    /// ——当前投影器能力基线。
    pub fn target_set() -> Self {
        let kinds: BTreeSet<String> = [
            // 500 基线。
            "text", "button", "input", "image", "a",
            // Plan 507 T3 —— Tier1 display 族（归一折叠键）。
            "img", "icon", "badge", "avatar", "progress", "divider", "separator", "spacer",
            // Plan 507 T4 —— Tier1 form 族。
            "checkbox", "switch", "radio", "textarea",
        ]
        .into_iter()
        .map(String::from)
        .collect();
        let props: BTreeMap<String, BTreeSet<String>> = [
            ("text", vec!["text", "label", "style", "selectable", "class"]),
            ("button", vec!["text", "label", "style", "disabled"]),
            ("input", vec!["value", "placeholder", "type", "style", "disabled"]),
            ("image", vec!["src", "style", "alt"]),
            ("a", vec!["text", "label", "style"]),
            // Plan 507 T3 —— display 族（props = schema 声明面 + style/class）。
            ("img", vec!["src", "style", "alt"]),
            ("icon", vec!["name", "size", "style", "class"]),
            ("badge", vec!["text", "variant", "style", "class"]),
            ("avatar", vec!["src", "alt", "fallback", "style", "class"]),
            ("progress", vec!["value", "max", "style", "class"]),
            ("divider", vec!["direction", "style", "class"]),
            ("separator", vec!["orientation", "label", "style", "class"]),
            ("spacer", vec!["size", "style", "class"]),
            // Plan 507 T4 —— form 族。
            ("checkbox", vec!["checked", "disabled", "style", "class"]),
            ("switch", vec!["checked", "disabled", "style", "class"]),
            ("radio", vec!["checked", "disabled", "style", "class"]),
            ("textarea", vec!["value", "placeholder", "disabled", "rows", "style", "class"]),
        ]
        .into_iter()
        .map(|(k, ps)| (k.to_string(), ps.into_iter().map(String::from).collect()))
        .collect();
        let events: BTreeMap<String, BTreeSet<String>> = [
            ("button", vec!["onclick"]),
            ("input", vec!["oninput"]),
            // Plan 507 T4 —— Toggle 派发（register_toggle 认 onclick/onchange
            // 双键；013/024 真源 = onclick，schema 声明 = onchange）。
            ("checkbox", vec!["onclick", "onchange"]),
            ("switch", vec!["onclick", "onchange"]),
            ("radio", vec!["onclick", "onchange"]),
            ("textarea", vec!["oninput"]),
        ]
        .into_iter()
        .map(|(k, es)| (k.to_string(), es.into_iter().map(String::from).collect()))
        .collect();
        let layouts: BTreeSet<String> = [
            // 500 基线。
            "center", "col", "row", "if",
            // Plan 507 T3 —— Tier1 布局容器（catch-all 容器臂本就渲染，
            // 此处登记 = auto 探测放行）。
            "container", "scroll",
            // Plan 507 T5 —— grid（cols 等宽网格臂）+ card 族（表面缺省
            // 档容器；kebab/underscore 折叠键同归）。
            "grid", "griditem", "card", "cardaction", "cardcontent",
            "carddescription", "cardfooter", "cardheader", "cardtitle",
            // Plan 507 T6 —— 语义容器（块流纵排；列表标记不载——保真
            // 边界随注）。
            "article", "aside", "footer", "header", "main", "nav", "section",
            "figure", "details", "summary", "ul", "ol", "li", "dl", "dt", "dd",
        ]
        .into_iter()
        .map(String::from)
        .collect();
        // 样式前缀规则：盒模（p/m 含负值与轴缩写）/间距 gap/尺寸 w/h/max-w/
        // min-/flex-1/对齐 items-/justify-/文本对齐 text-center 等（text- 前
        // 缀同时覆盖尺寸/颜色/对齐三族）/排版 font- leading- underline/装饰
        // bg- border rounded shadow from- to-（渐变端点）/溢出 overflow-/
        // mx-auto/hover: 交互态。
        let style_prefixes: BTreeSet<String> = [
            "p-", "px-", "py-", "pt-", "pb-", "pl-", "pr-",
            "m-", "mx-", "my-", "mt-", "mb-", "ml-", "mr-", "-m",
            "gap-", "w-", "h-", "max-w-", "min-w-", "min-h-", "flex-1",
            "items-", "justify-", "overflow-", "mx-auto",
            "text-", "font-", "leading-", "underline",
            "bg-", "border", "rounded", "shadow", "from-", "to-",
            "hover:",
        ]
        .into_iter()
        .map(String::from)
        .collect();
        Self { kinds, props, events, layouts, style_prefixes }
    }

    /// 样式类是否在支持子集（前缀或裸类匹配）。
    pub fn style_token_supported(&self, token: &str) -> bool {
        if token.is_empty() {
            return true;
        }
        self.style_prefixes.iter().any(|p| token.starts_with(p.as_str()))
    }
}

/// 装载期静态扫描结果：App 视图清单（标签/prop/事件/样式类 + 带参
/// handler 现场）。标签**不预分类**（widget 还是布局容器由能力表 judge）。
#[derive(Debug, Clone, Default)]
pub struct ViewScan {
    /// 元素标签全集（归一：文本族 → "text"）+ 视图构造名（"if"/"for"/
    /// "component:<name>"/"outlet"/"link"）。
    pub tags: BTreeSet<String>,
    /// "tag.prop" 全集。
    pub tag_props: BTreeSet<String>,
    /// "tag.event" 全集。
    pub tag_events: BTreeSet<String>,
    /// 样式类全集（去重；仅收集 `style:` prop 的串）。
    pub style_tokens: BTreeSet<String>,
    /// 带参 handler 的 "tag.event(pattern)"（不可投影——显式缺项）。
    pub param_handlers: Vec<String>,
}

/// 可行性判定：缺项清单为空 = Covered。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    Covered,
    /// 缺项清单（tag:\<name\> / tag.prop / tag.event! / style:token /
    /// 带参 handler 条目）——auto 降级时的观测 Log 载荷。
    NotCovered(Vec<String>),
}

impl Verdict {
    pub fn is_covered(&self) -> bool {
        matches!(self, Self::Covered)
    }
}

/// 扫描视图模板（`DynamicComponent::view_template()` 的 AuraNode 树）。
pub fn scan_view(template: &AuraNode) -> ViewScan {
    let mut scan = ViewScan::default();
    scan_node(template, &mut scan);
    scan
}

fn scan_node(node: &AuraNode, scan: &mut ViewScan) {
    match node {
        AuraNode::Element { tag, props, events, children, .. } => {
            let kind = normalize_kind(tag);
            scan.tags.insert(kind.clone());
            for (key, value) in props {
                scan.tag_props.insert(format!("{kind}.{key}"));
                // 样式类仅来自 `style:` prop 的字符串字面量。
                if key == "style" {
                    if let crate::aura::AuraPropValue::Expr(expr) = value {
                        collect_style_tokens(expr, scan);
                    }
                }
            }
            for (key, event) in events {
                scan.tag_events.insert(format!("{kind}.{key}"));
                if is_param_handler(event) {
                    scan.param_handlers
                        .push(format!("{kind}.{key}({})", event.handler));
                }
            }
            for child in children {
                scan_node(child, scan);
            }
        }
        AuraNode::Text(_) => {
            scan.tags.insert("text".into());
        }
        AuraNode::Conditional { then_body, else_body, .. } => {
            scan.tags.insert("if".into());
            for child in then_body {
                scan_node(child, scan);
            }
            if let Some(else_body) = else_body {
                for child in else_body {
                    scan_node(child, scan);
                }
            }
        }
        // 未入目标集的构造：以构造名入清单（for/Component/Outlet/Link）。
        AuraNode::ForLoop { .. } => {
            scan.tags.insert("for".into());
        }
        AuraNode::Component { name, .. } => {
            scan.tags.insert(format!("component:{name}"));
        }
        AuraNode::Outlet => {
            scan.tags.insert("outlet".into());
        }
        AuraNode::Link { .. } => {
            scan.tags.insert("link".into());
        }
    }
}

/// 带参 handler（事件参数显式声明或 handler 模式带 `(`——与投影器
/// `handler_token` 的取舍规则同口径）。
fn is_param_handler(event: &AuraEvent) -> bool {
    !event.params.is_empty() || event.handler.contains('(')
}

/// 从 prop 表达式收集 style 串的样式类（`style: "p-8 gap-4"` 之类）。
fn collect_style_tokens(expr: &crate::ast::Expr, scan: &mut ViewScan) {
    if let crate::ast::Expr::Str(s) = expr {
        for token in s.split_whitespace() {
            scan.style_tokens.insert(token.to_string());
        }
    }
}

/// 能力表 vs 扫描清单 → 判定（缺项按字典序稳定输出）。标签分类在此：
/// 命中 `kinds` = widget（逐项查 prop/事件）；命中 `layouts` = 布局/
/// 构造；两者皆未命中 = 整体缺项 `tag:<name>`（widget/布局不做猜测——
/// 缺项文本保持中性）。
pub fn judge(scan: &ViewScan, coverage: &Coverage) -> Verdict {
    let mut missing: Vec<String> = Vec::new();

    for tag in &scan.tags {
        let as_widget = coverage.kinds.contains(tag);
        let as_layout = coverage.layouts.contains(tag);
        if !as_widget && !as_layout {
            missing.push(format!("tag:{tag}"));
            continue; // 整体未覆盖：其 prop/事件不再逐项列。
        }
        if as_widget {
            for kp in &scan.tag_props {
                if let Some((k, prop)) = kp.split_once('.') {
                    if k == tag
                        && !coverage.props.get(tag).is_some_and(|set| set.contains(prop))
                    {
                        missing.push(kp.clone());
                    }
                }
            }
            for ke in &scan.tag_events {
                if let Some((k, event)) = ke.split_once('.') {
                    if k == tag
                        && !coverage.events.get(tag).is_some_and(|set| set.contains(event))
                    {
                        missing.push(format!("{ke}!"));
                    }
                }
            }
        }
    }

    for token in &scan.style_tokens {
        if !coverage.style_token_supported(token) {
            missing.push(format!("style:{token}"));
        }
    }

    for ph in &scan.param_handlers {
        missing.push(format!("param-handler:{ph}"));
    }

    if missing.is_empty() {
        Verdict::Covered
    } else {
        missing.sort();
        missing.dedup();
        Verdict::NotCovered(missing)
    }
}

// ---------------------------------------------------------------------------
// 三态渲染开关（Plan 500 步骤 6：裁决链 spawn 参数 > pac.at > auto 探测）
// ---------------------------------------------------------------------------

/// per-App 三态渲染声明（pac.at `desktop_render:` / spawn `--render=`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderMode {
    /// 装载期覆盖度探测：Covered → queue；NotCovered → 降级 independent
    /// （宿主观测 Log 一行留痕）。
    #[default]
    Auto,
    /// 命令帧（DrawList → 宿主栅格化）。
    Queue,
    /// 像素帧（child 自带 iced 自渲染 → shm RGBA）。
    Independent,
}

impl RenderMode {
    /// 声明串解析（pac.at 字段 / spawn 参数共用）；未知值 = None（调用方
    /// 按来源报错或回退 Auto）。
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "queue" => Some(Self::Queue),
            "independent" => Some(Self::Independent),
            _ => None,
        }
    }

    /// 裁决链第一二环：spawn 参数 > manifest 声明 > Auto 缺省。
    /// （`adjudicate()` 的入口三步裁决是**进程形态**维度——Client/Broker/
    /// Standalone；本链是同形态内的**帧载荷**维度，挂 cmd_autodesk 消费。）
    pub fn resolve(spawn_arg: Option<&str>, manifest: Option<&str>) -> Self {
        if let Some(arg) = spawn_arg.and_then(Self::parse) {
            return arg;
        }
        if let Some(m) = manifest.and_then(Self::parse) {
            return m;
        }
        Self::Auto
    }
}

/// 三态 → 二态帧模式 + 降级观测行（auto 探测：装载期扫描 vs 能力表）。
/// 返回 (frame_mode, Option<降级日志行>)——`Some` = auto 降级 independent
/// 的宿主观测留痕（child 经孵化记录把降级标记带给宿主打印/ui_console）。
pub fn effective_frame_mode(
    mode: RenderMode,
    component: &crate::ui::dynamic::DynamicComponent,
) -> (super::message::FrameMode, Option<String>) {
    match mode {
        RenderMode::Queue => (super::message::FrameMode::Commands, None),
        RenderMode::Independent => (super::message::FrameMode::Pixels, None),
        RenderMode::Auto => {
            let scan = scan_view(component.view_template());
            match judge(&scan, &Coverage::target_set()) {
                Verdict::Covered => (super::message::FrameMode::Commands, None),
                Verdict::NotCovered(missing) => (
                    super::message::FrameMode::Pixels,
                    Some(format!(
                        "[render] auto -> independent downgrade ({} not covered: {})",
                        component.widget_name(),
                        missing.join(", ")
                    )),
                ),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// T2 单测：能力表 vs 视图清单判定（覆盖/不覆盖/降级载荷）
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// 001–005 五示例源文件（真源扫描——G4 覆盖表与实例清单的一致性钉）。
    const EXAMPLES: [&str; 5] = [
        "001-helloworld",
        "002-counter",
        "003-converter",
        "004-profile-card",
        "005-login",
    ];

    fn scan_example(dir: &str) -> ViewScan {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/ui/",
            "PLACEHOLDER/src/front/app.at"
        )
        .replace("PLACEHOLDER", dir);
        let src = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {path}: {e}"));
        let component =
            crate::build_dynamic_component(&src, None).unwrap_or_else(|e| panic!("build {dir}: {e}"));
        scan_view(component.view_template())
    }

    /// T2 主体：001–005 五示例在目标能力表下全 Covered（auto → queue 不降级）。
    #[test]
    fn examples_001_005_all_covered() {
        let coverage = Coverage::target_set();
        for dir in EXAMPLES {
            let scan = scan_example(dir);
            let verdict = judge(&scan, &coverage);
            assert!(verdict.is_covered(), "{dir} 应 Covered: {verdict:?}");
        }
    }

    /// 扫描面抽查：005 的标签/事件/样式清单与源码结构一致。
    #[test]
    fn scan_inventory_matches_source_shape() {
        let scan = scan_example("005-login");
        // 标签全集（h2 归一 text；布局容器 col/row + 构造 if 同列）。
        // Plan 512 S6：fit 迁移拆掉 center 居中外壳（根 col Shrink 包裹）。
        for tag in ["text", "button", "input", "a", "col", "row", "if"] {
            assert!(scan.tags.contains(tag), "005 应含 {tag}: {:?}", scan.tags);
        }
        // 事件：button.onclick + input.oninput（零参 msg 路径）。
        assert!(scan.tag_events.contains("button.onclick"));
        assert!(scan.tag_events.contains("input.oninput"));
        assert!(scan.param_handlers.is_empty(), "005 无带参 handler");
        // 样式类抽样（仅来自 style prop——位置参数文本不入样式清单）。
        // Plan 512 S6：卡片宽 max-w-md+w-full → 固定 w-112（Shrink 根
        // 下 w-full 塌缩；rem 任意值 iced 端不支持，用 Tailwind 刻度）。
        assert!(scan.style_tokens.contains("w-full"));
        assert!(scan.style_tokens.contains("w-112"));
        assert!(
            !scan.style_tokens.contains("Sign") && !scan.style_tokens.contains("In"),
            "非 style prop 的字符串不入样式清单: {:?}",
            scan.style_tokens
        );
    }

    /// 不覆盖 → NotCovered 显式缺项（禁止静默）。
    #[test]
    fn uncovered_widget_and_layout_listed() {
        let src = r#"widget T {
    view {
        col {
            select (value: .mode) { onchange: .Pick }
            svg { path {} }
        }
    }
}
"#;
        let component = crate::build_dynamic_component(src, None).expect("build");
        let scan = scan_view(component.view_template());
        let verdict = judge(&scan, &Coverage::target_set());
        let Verdict::NotCovered(missing) = verdict else {
            panic!("checkbox/scrollable 应 NotCovered");
        };
        assert!(missing.iter().any(|m| m == "tag:select"), "缺项列 select: {missing:?}");
        // 整体缺项（tag 未入表）不逐项列 prop/事件。
        assert!(
            !missing.iter().any(|m| m.starts_with("select.")),
            "整体缺项不逐项展开: {missing:?}"
        );
        assert!(
            missing.iter().any(|m| m == "tag:svg"),
            "缺项列 svg: {missing:?}"
        );
    }

    /// 带参 handler → 显式缺项（与投影器 handler_token 取舍同口径）。
    #[test]
    fn param_handler_listed() {
        let src = r#"widget T {
    model { var list str = "" }
    view { button "x" { onclick: .Delete(list.id) } }
}
"#;
        let component = crate::build_dynamic_component(src, None).expect("build");
        let scan = scan_view(component.view_template());
        let verdict = judge(&scan, &Coverage::target_set());
        let Verdict::NotCovered(missing) = verdict else {
            panic!("带参 handler 应 NotCovered");
        };
        assert!(
            missing.iter().any(|m| m.starts_with("param-handler:button.onclick")),
            "缺项列带参 handler: {missing:?}"
        );
    }

    /// 未支持样式类 → 显式缺项（style 子集外不静默丢弃）。
    #[test]
    fn unsupported_style_token_listed() {
        let coverage = Coverage::target_set();
        assert!(coverage.style_token_supported("p-8"));
        assert!(coverage.style_token_supported("-mt-10"));
        assert!(coverage.style_token_supported("hover:bg-blue-600"));
        assert!(coverage.style_token_supported("bg-gradient-to-r"));
        assert!(!coverage.style_token_supported("animate-pulse"), "动画类不在 v1 子集");
        assert!(!coverage.style_token_supported("backdrop-blur"), "滤镜类不在 v1 子集");
    }

    /// 空视图（纯文本节点）与空扫描 → Covered。
    #[test]
    fn empty_scan_covered() {
        let scan = ViewScan::default();
        assert_eq!(judge(&scan, &Coverage::target_set()), Verdict::Covered);
    }

    /// 三态裁决链：spawn 参数 > manifest > Auto 缺省；未知值不炸（回退
    /// 下一环）。
    #[test]
    fn render_mode_resolution_chain() {
        use super::RenderMode as RM;
        assert_eq!(RM::resolve(None, None), RM::Auto, "缺省 Auto");
        assert_eq!(RM::resolve(Some("queue"), None), RM::Queue);
        assert_eq!(RM::resolve(Some("independent"), Some("queue")), RM::Independent, "spawn 覆盖 manifest");
        assert_eq!(RM::resolve(None, Some("queue")), RM::Queue, "manifest 档");
        assert_eq!(RM::resolve(Some("bogus"), Some("independent")), RM::Independent, "未知 spawn 值回退 manifest");
        assert_eq!(RM::resolve(Some("bogus"), Some("bogus")), RM::Auto, "双未知回退 Auto");
        assert_eq!(RM::resolve(Some(" Queue "), None), RM::Queue, "空白宽容");
    }

    /// auto 探测：覆盖视图 → Commands 无降级行；未覆盖视图 → Pixels +
    /// 降级观测行（缺项清单随行）；显式档不走探测。
    #[test]
    fn effective_mode_probe_and_downgrade() {
        use crate::ui::desktop_protocol::message::FrameMode;
        use super::RenderMode as RM;

        // 覆盖视图（002 计数器形态）。
        let covered = crate::build_dynamic_component(
            "widget C { model { var count int = 0 } view { center { text `n: ${.count}` button \"+\" { onclick: () => {.count += 1} } } } }",
            None,
        ).expect("build");
        assert_eq!(effective_frame_mode(RM::Auto, &covered), (FrameMode::Commands, None));
        assert_eq!(effective_frame_mode(RM::Queue, &covered), (FrameMode::Commands, None));
        assert_eq!(effective_frame_mode(RM::Independent, &covered), (FrameMode::Pixels, None), "显式 independent 不探测");

        // 未覆盖视图（select 弹层族——Plan 507 T4 后 checkbox 已覆盖）→
        // Pixels + 降级行。
        let uncovered = crate::build_dynamic_component(
            "widget U { view { select (value: .mode) { onchange: .Pick } } }",
            None,
        ).expect("build");
        let (mode, downgrade) = effective_frame_mode(RM::Auto, &uncovered);
        assert_eq!(mode, FrameMode::Pixels, "auto 未覆盖降级 independent");
        let line = downgrade.expect("降级观测行");
        assert!(line.contains("auto -> independent"), "{line}");
        assert!(line.contains("select"), "缺项清单随行: {line}");
    }

    /// Plan 507 T2/T3 一致性钉：元素登记表的 covered 条目必须落在
    /// `Coverage::target_set()` 可投影集内（归一折叠后 kinds ∪ layouts）
    /// ——登记与能力表双向不脱钩（漂移围栏的运行时侧互补）。
    #[test]
    fn covered_elements_within_target_set() {
        let coverage = Coverage::target_set();
        let projectable = |tag: &str| {
            let kind = normalize_kind(tag);
            coverage.kinds.contains(&kind) || coverage.layouts.contains(&kind)
        };
        for (tag, status) in crate::aura::element_coverage::element_table() {
            if matches!(status, crate::aura::element_coverage::QueueStatus::Covered) {
                assert!(
                    projectable(tag),
                    "登记 covered 但 target_set 不可投影: {tag}（能力表/投影器臂缺失）"
                );
            }
        }
    }

    /// Plan 507 T7 —— Tier3 not-yet 族 auto 降级链路复核：代表族逐一
    /// 构造 → Pixels + 观测行（缺项清单即载荷——禁止静默错绘的机制证）。
    #[test]
    fn t7_not_yet_families_auto_downgrade() {
        use super::RenderMode as RM;
        use crate::ui::desktop_protocol::message::FrameMode;
        // (族, 源, 缺项证词)
        let cases: &[(&str, &str, &str)] = &[
            ("overlay 弹层", "widget O { view { select (value: .m) { onchange: .P } } }", "tag:select"),
            ("chart/diagram", "widget C { view { svg { path {} } } }", "tag:svg"),
            ("复合编辑器", "widget E { view { markdown (content: .doc) } }", "tag:markdown"),
            ("nav 系", "widget N { view { nav-item (label: \"x\") { onclick: .Go } } }", "tag:navitem"),
            ("表格族", "widget T { view { table { text \"r\" } } }", "tag:table"),
            ("瞬态浮层", "widget F { view { toaster {} } }", "tag:toaster"),
        ];
        for (family, src, evidence) in cases {
            let component = crate::build_dynamic_component(src, None)
                .unwrap_or_else(|e| panic!("{family} build: {e}"));
            let (mode, downgrade) = effective_frame_mode(RM::Auto, &component);
            assert_eq!(mode, FrameMode::Pixels, "{family} 应降级 independent");
            let line = downgrade.unwrap_or_else(|| panic!("{family} 应有降级观测行"));
            assert!(line.contains("auto -> independent"), "{family}: {line}");
            assert!(
                line.contains(&evidence.trim_start_matches("tag:")),
                "{family} 缺项清单随行: {line}"
            );
        }
    }

    /// Plan 507 T3：display 族 auto 探测放行（折叠键 + prop 声明面）。
    #[test]
    fn tier1_display_family_auto_eligible() {
        let coverage = Coverage::target_set();
        let src = r#"widget D {
    model { var pct double = 0.6 }
    view {
        col {
            icon (name: "star") { style: "w-6 h-6" }
            badge "New" { style: "text-xs" }
            avatar (fallback: "Jane Cooper") { style: "w-10 h-10" }
            progress (value: .pct, max: 1.0) { style: "w-full" }
            divider { style: "w-full" }
            separator { style: "w-full" }
            spacer { style: "h-4" }
            container { style: "p-2 bg-slate-800" }
            scroll { text "inner" }
        }
    }
}
"#;
        let component = crate::build_dynamic_component(src, None).expect("build");
        let scan = scan_view(component.view_template());
        let verdict = judge(&scan, &coverage);
        assert!(verdict.is_covered(), "display 族应 Covered: {verdict:?}");
        assert_eq!(
            effective_frame_mode(RenderMode::Auto, &component),
            (crate::ui::desktop_protocol::message::FrameMode::Commands, None)
        );
    }
}
