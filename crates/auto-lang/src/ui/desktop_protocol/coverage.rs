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

/// 文本承载标签 → 归一 kind "text"（投影器 `is_text_tag` 同集）。
pub fn normalize_kind(tag: &str) -> String {
    let t = tag.to_ascii_lowercase();
    match t.as_str() {
        "text" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "p" | "span" | "label" => {
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
    /// §1.3.1 爬坡目标集（001–005 实测清单）——当前投影器能力基线。
    pub fn target_set() -> Self {
        let kinds: BTreeSet<String> =
            ["text", "button", "input", "image", "a"].into_iter().map(String::from).collect();
        let props: BTreeMap<String, BTreeSet<String>> = [
            ("text", vec!["text", "label", "style", "selectable"]),
            ("button", vec!["text", "label", "style"]),
            ("input", vec!["value", "placeholder", "type", "style"]),
            ("image", vec!["src", "style"]),
            ("a", vec!["text", "label", "style"]),
        ]
        .into_iter()
        .map(|(k, ps)| (k.to_string(), ps.into_iter().map(String::from).collect()))
        .collect();
        let events: BTreeMap<String, BTreeSet<String>> = [
            ("button", vec!["onclick"]),
            ("input", vec!["oninput"]),
        ]
        .into_iter()
        .map(|(k, es)| (k.to_string(), es.into_iter().map(String::from).collect()))
        .collect();
        let layouts: BTreeSet<String> =
            ["center", "col", "row", "if"].into_iter().map(String::from).collect();
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
        // 标签全集（h2 归一 text；布局容器 center/col/row + 构造 if 同列）。
        for tag in ["text", "button", "input", "a", "center", "col", "row", "if"] {
            assert!(scan.tags.contains(tag), "005 应含 {tag}: {:?}", scan.tags);
        }
        // 事件：button.onclick + input.oninput（零参 msg 路径）。
        assert!(scan.tag_events.contains("button.onclick"));
        assert!(scan.tag_events.contains("input.oninput"));
        assert!(scan.param_handlers.is_empty(), "005 无带参 handler");
        // 样式类抽样（仅来自 style prop——位置参数文本不入样式清单）。
        assert!(scan.style_tokens.contains("w-full"));
        assert!(scan.style_tokens.contains("max-w-md"));
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
            checkbox (checked: .ok) { onchange: .Toggle }
            scrollable { text "x" }
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
        assert!(missing.iter().any(|m| m == "tag:checkbox"), "缺项列 checkbox: {missing:?}");
        // 整体缺项（tag 未入表）不逐项列 prop/事件。
        assert!(
            !missing.iter().any(|m| m.starts_with("checkbox.")),
            "整体缺项不逐项展开: {missing:?}"
        );
        assert!(
            missing.iter().any(|m| m == "tag:scrollable"),
            "缺项列 scrollable: {missing:?}"
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
}
