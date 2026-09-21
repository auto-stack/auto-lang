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
        // PLAN-668 R-14：avatar 子件（os-007 升格——image/fallback 臂在
        // avatar 投影器臂内消费）归一折叠到 avatar（覆盖判定用，同
        // typography 族先例）。
        | "avatarfallback" | "avatarimage" => "avatar".to_string(),
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
            // PLAN-668 R-14：PLAN-661 T-03/T-04 slider 统一（VM/aura 臂
            // 直构 View::Slider + vue 原生 range）落地的漏同步。
            "slider",
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
            // PLAN-668 R-14：slider props 面（661 T-04 vue 原生 range 属性集）。
            ("slider", vec!["value", "min", "max", "step", "style", "class"]),
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
            // Plan 518 G8：backdrop-* 毛玻璃词汇声明冻结——共享 parser 已
            // 识别（StyleClass::BackdropBlur/Saturate）,queue 臂不触发
            // "未知类 → 整 widget not-yet"误判（BoxLayout 提取天然跳过
            // 装饰字段）。渲染端 no-op 挂 RenderQueue（planned-debt）。
            "backdrop-",
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

    /// Plan 020 T-04 —— native queue 臂覆盖集（§5.4 实现设计钉）。
    /// PLAN-025 T-02 扩容：form 族 input/textarea/checkbox/radio 入册
    /// （渲染/命中/聚焦/键入回写臂同落 native_projector）。**switch 无
    /// native 对象**——View 枚举无 Switch 变体（解释态 aura 标签专属，
    /// T-01 §5.1 调查证据），native 轨无可产该 kind 的构造（I4 分表，
    /// 非缺口）。slider/select 随 T-03/T-04 扩容。PLAN-026 T-03 扩容：
    /// display 族 image/progress 入册；PLAN-028 真图升级（image 占位
    /// 保真臂 → `DrawOp::Image` 真渲，未解析降级转兜底——随注见下）；
    /// icon/badge/avatar/
    /// divider/separator/spacer/a/img 经 a2r codegen 降级归一（D1/D1'
    /// 定案）。kind = text/button + form 族 + display 族 + 线性堆叠
    /// 布局族（col/row/container/list）+ 布局样式子集（padding/gap/
    /// margin/尺寸/圆角/底色/前景色/对齐/字号字重）。payload 族残余
    /// （table/tabs 等）与 imagesurface 显式 **not-yet**——native 显式
    /// queue 遇未覆盖 = 拒绝退出留痕（AC-04，非静默错绘）。与解释态
    /// [`Coverage::target_set`] 分表：native 投影器渲染面按投影器臂
    /// 爬坡同步扩表（单一事实源纪律同 500 §1.3.1）。
    pub fn native_queue_set() -> Self {
        let kinds: BTreeSet<String> = [
            "text",
            "button",
            // PLAN-025 T-02 —— native form 族（switch 无 View 变体不列）。
            "input",
            "textarea",
            "checkbox",
            "radio",
            // PLAN-025 T-03 —— payload 族 slider。
            "slider",
            // PLAN-025 T-04 —— payload 族 select。
            "select",
            // PLAN-026 T-03 —— display 族臂（View::Image /
            // View::ProgressBar 变体在场）。icon/badge/avatar/divider/
            // separator/spacer/a/img 经 a2r codegen 降级归一到 image/
            // text/row/container/empty（§5.1 D1/D1' 定案——分表非缺口，
            // 025"switch 无 View 变体"口径）；imagesurface 整 kind
            // not-yet（D5：交互回调无采集面，登记即静默放行——I3）。
            // PLAN-028 真图升级：image 臂占位保真注释核销——src 在场即发
            // `DrawOp::Image`（tag 6 真渲；未解析降级占位转宿主侧兜底
            // 语义）；icon 降级形态（lucide:）随臂入线、宿主字形解析
            // not-yet（P026-D1 后半维持）→ 未解析降级同兜底。
            "image",
            "progress",
        // PLAN-029 T-04/T-05/T-06 —— shell queue 面四 kind（B 前置
        // 序列第二件）：popover（覆盖序渲染 + on_dismiss 命中，open =
        // View 态）/ mousearea（透传 + click/contextmenu 命中）/
        // windowthumbnail·workspacepreview（thumbnail:// ·
        // workspace:// 虚拟引用桥接，宿主侧解析）。
        "popover",
        "mousearea",
        "windowthumbnail",
        "workspacepreview",
        // PLAN-032 T-03（D2）：payload 族 tabs——View::Tabs 投影臂（托盘
        // /选中态/内容区/on_select 命中全链，native_projector 臂同册）。
        // a2r 断裂映射同批修复（ui_gen/rust.rs 专属臂）；M7-c②（jade-
        // garden tab×27 / auto-musk tab×16）依赖解锁。
        "tabs",
        // PLAN-034 T-05（D4 裁定）：canvas=位图快照过线——View::Canvas
        // 投影臂（场景栅格化 → bitmap:// 引用 + on_hit 节点命中；pen
        // 坐标回传/labels = M7-c 撞面批随注）。像素原生族首个经位图
        // 通道过线的 kind（043 样板验证）。
        "canvas",
        // PLAN-674 T-01（§10-1 裁定 A：结构 DrawOps）：codeeditor 过线
        // ——View::CodeEditor 投影臂（CODE_EDITORS 注册表 get-or-create，
        // inproc iced widget 同源）→ core::render 视口虚拟化
        // EditorDrawList → lower_editor_frame（Plan 386 降层，gutter/
        // 当前行/选区/搜索/caret/preedit/滚动条全 op 面）；键入/IME/箭标
        // 走 core handle_input 全键面（editor_frame.rs EditorFrameSource
        // 映射先例）。041 案册翻案（"两真 not-yet 家族"清偿件）。
        "codeeditor",
        ]
        .into_iter()
        .map(String::from)
        .collect();
        let layouts: BTreeSet<String> = [
            "col", "row", "container", "list",
            // 透传壳（View::Empty / AnchorSlot 块锚定槽——渲染透明）。
            "empty", "anchorslot",
            // PLAN-025 T-05 —— scrollable（Scissor 裁剪 + on_scroll 滚轮）。
            "scroll",
            // PLAN-026 T-04 —— grid（View::Grid walker 两遍网格）。
            // "center" 不入册：View::center 归一 Container（scan 无
            // "center" kind 产出点——登记即违反防漏钉钉②）。
            "grid",
        ]
        .into_iter()
        .map(String::from)
        .collect();
        // 布局样式子集：盒模/间距/尺寸/对齐/排版子集/装饰（bg/border/
        // rounded/渐变端点）。shadow/underline/动画/滤镜/opacity 等渲染
        // 未实现面不入——token 映射见 [`native_style_token`]。
        let style_prefixes: BTreeSet<String> = [
            "p-", "px-", "py-", "pt-", "pb-", "pl-", "pr-",
            "m-", "mx-", "my-", "mt-", "mb-", "ml-", "mr-", "-m",
            "gap-", "w-", "h-", "max-w-",
            "items-", "justify-", "mx-auto",
            "text-", "font-",
            "bg-", "border", "rounded", "from-", "to-",
            // PLAN-025 T-06（T-01 §5.1 定案 2）：flex-1/shadow 降级放行
            // ——解释态 target_set 同款保真边界（shadow 渲染 no-op、
            // flex 族自然宽——native_projector::node_style_of 随注），
            // 非静默扩权：003-converter 真源 gate 通过所需。
            "flex-1",
            "shadow",
            // PLAN-026 T-06（§5.1 D3）降级放行批——解释态 target_set
            // 同款保真边界，非静默扩权：
            // ① overflow-：queue 臂裁剪渲染面 not-yet（块流静态帧无
            //   溢出面；004 真源 gate 所需）。
            // ② min-w-/min-h-：native 最小尺寸约束渲染 not-yet（解释态
            //   target_set 同在册）。
            // ③ leading-：行高倍率渲染 not-yet（Text op 固定 LINE_H
            //   档；PLAN-527 后 typed parse 可达，需显式放行）。
            // ④ flex/block 裸 display 类：块流语义 no-op（方向类布局
            //   即 col/row 构造面）。
            // ⑤ underline/no-underline/line-through：文本装饰渲染
            //   not-yet（Text op 无装饰通道；解释态 target_set 同册
            //   underline）。
            "overflow-",
            "min-w-", "min-h-",
            "leading-",
            "flex", "block",
            "underline", "no-underline", "line-through",
            // ⑥ 静态帧 no-op 提示类（cursor/outline/transition/抗锯齿/
            // shrink/whitespace——queue 命令帧无对应通道，零视觉差）。
            "cursor-", "outline-", "transition", "antialiased",
            "shrink-", "whitespace-", "relative", "tracking-",
            "backdrop-",
            // PLAN-034 T-05：ring 族（focus 光圈装饰渲染 not-yet——
            // 043 样板携带 ring-2/ring-primary/ring-offset-2；token
            // 映射在 native_style_token，声明放行同 ⑥）。
            "ring-",
            // PLAN-029 T-08 降级放行：⑦ opacity- 子树透明渲染 not-yet
            //（DrawOp 无 alpha 通道——色彩 alpha 逐子树改写另立；壳
            // pack 拖拽幽灵/通知卡半透明面经此放行，视觉全不透明降级
            //——flex-1/shadow 同册先例）。
            "opacity-",
            // PLAN-032 T-02（D3/D5）放行批：
            // ⑧ hidden（display:none）——投影器 NodeStyle.hidden 单一
            //   choke 跳过子树**真渲**（display 族在场 = 响应式覆盖可
            //   见——018 侧栏/移动头、041 条件行）。
            // ⑨ self-（交叉轴自对齐）——center_children 通道真渲（012
            //   步进值/单位标签映射缺口清偿）。
            "hidden",
            "self-",
            // PLAN-032 T-04（D4）放行：⑩ style-grid（display:grid/
            // grid-cols-N/grid-rows-N 布局语义）——投影器 layout_view_
            // block 入口分岔复用 Grid walker 真渲（024 图表工坊）。
            "style-grid",
            // PLAN-032 T-05（D1 分层）放行：⑪ 定位族——absolute + offset
            // 真渲（脱离流覆盖序锚定父内容盒 + z 同层相对层级）；fixed/
            // sticky **降级放行**（in-flow no-op 渲染——视口锚定真渲债
            /// §10-④ 另立，opacity/overflow 先例；018 fixed×2/021
            /// sticky×2 判定翻绿，保真边界显式随注）；top-/left-/right-/
            // bottom-/z- 偏移与层级 token。
            "absolute",
            "fixed",
            "sticky",
            "top-",
            "left-",
            "right-",
            "bottom-",
            "z-",
            // PLAN-032 T-07（D5 族）：inset- 四向偏移（真渲——NodeStyle
            // 四槽填充）/ line-clamp- 行数截断（降级放行 no-op 随注）。
            "inset-",
            "line-clamp-",
            // PLAN-036 T-02（⑫）：truncate 真渲放行——native 投影器 Text
            // 发射臂 measure 收缩 + `…` 尾接（018 真渲债清偿；编译轨
            // shell-pack desktop 面 gate 所需）。break-words 同批降级
            // 放行（queue 臂无折行通道 = 单行渲染 no-op，语义同 ⑥ 族）。
            "truncate",
            "break-words",
        ]
        .into_iter()
        .map(String::from)
        .collect();
        Self {
            kinds,
            props: BTreeMap::new(),
            events: BTreeMap::new(),
            layouts,
            style_prefixes,
        }
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
// native 视图清单扫描（Plan 020 T-04：View<M> 树 → ViewScan，judge 复用）
// ---------------------------------------------------------------------------

/// View 变体 → 归一 kind（对齐解释态 [`normalize_kind`] 的词表口径）。
/// 未入 native 覆盖集的变体原样产出 kind 名（slider/select/input/…）——
/// judge 缺项清单即载荷。
pub fn native_kind_of<M: Clone + std::fmt::Debug>(view: &crate::ui::view::View<M>) -> &'static str {
    use crate::ui::view::View;
    match view {
        View::Empty => "empty",
        View::AnchorSlot { .. } => "anchorslot",
        View::Text { .. } => "text",
        View::Rich { .. } => "text",
        View::Button { .. } => "button",
        View::Row { .. } => "row",
        View::Column { .. } => "col",
        View::Container { .. } => "container",
        View::List { .. } => "list",
        View::Input { .. } => "input",
        View::Textarea { .. } => "textarea",
        View::CodeEditor { .. } => "codeeditor",
        View::Terminal { .. } => "terminal",
        View::ManagedScrollContent { .. } => "managed_content",
        View::AutodownEditor { .. } => "autodowneditor",
        View::Checkbox { .. } => "checkbox",
        View::Custom { .. } => "custom",
        View::Scrollable { .. } => "scroll",
        View::Radio { .. } => "radio",
        View::Select { .. } => "select",
        View::Table { .. } => "table",
        View::Slider { .. } => "slider",
        View::ProgressBar { .. } => "progress",
        View::Accordion { .. } => "accordion",
        View::Sidebar { .. } => "sidebar",
        View::Tabs { .. } => "tabs",
        View::NavigationRail { .. } => "navigationrail",
        View::Image { .. } => "image",
        View::ImageSurface { .. } => "imagesurface",
        View::Video { .. } => "video",
        View::WindowThumbnail { .. } => "windowthumbnail",
        View::WorkspacePreview { .. } => "workspacepreview",
        View::Grid { .. } => "grid",
        View::Overlay { .. } => "overlay",
        View::Popover { .. } => "popover",
        View::MouseArea { .. } => "mousearea",
        View::Canvas { .. } => "canvas",
    }
}

/// 扫描 native View 树（解释态 [`scan_view`] 的 View 泛型同型）：标签 +
/// 样式类（typed `Style` → 代表性 token 串）入 [`ViewScan`]。native
/// handler 已是物化 `M` 值（零参 by construction）——`param_handlers`
/// 恒空；带参族（Slider `fn(f32)->M` 等）由 kind not-yet 承担。
pub fn scan_native_view<M: Clone + std::fmt::Debug>(view: &crate::ui::view::View<M>) -> ViewScan {
    let mut scan = ViewScan::default();
    scan_native_node(view, &mut scan);
    scan
}

fn scan_native_node<M: Clone + std::fmt::Debug>(
    view: &crate::ui::view::View<M>,
    scan: &mut ViewScan,
) {
    use crate::ui::view::View;
    let kind = native_kind_of(view);
    scan.tags.insert(kind.to_string());
    // 变体自带 style 的统一收集（typed StyleClass → 代表 token）。
    let styles: Vec<Option<&crate::ui::style::Style>> = match view {
        View::Empty | View::AnchorSlot { .. } => vec![],
        View::ManagedScrollContent { .. } => vec![],
        View::Popover { .. } | View::Overlay { .. } => vec![],
        View::Text { style, .. }
        | View::Rich { style, .. }
        | View::Button { style, .. }
        | View::Input { style, .. }
        | View::Textarea { style, .. }
        | View::CodeEditor { style, .. }
        | View::Terminal { style, .. }
        | View::AutodownEditor { style, .. }
        | View::Checkbox { style, .. }
        | View::Custom { style, .. }
        | View::Radio { style, .. }
        | View::Select { style, .. }
        | View::Slider { style, .. }
        | View::ProgressBar { style, .. }
        | View::Accordion { style, .. }
        | View::Sidebar { style, .. }
        | View::Tabs { style, .. }
        | View::NavigationRail { style, .. }
        | View::Image { style, .. }
        | View::ImageSurface { style, .. }
        | View::Video { style, .. }
        | View::WindowThumbnail { style, .. }
        | View::WorkspacePreview { style, .. }
        | View::Grid { style, .. }
        | View::MouseArea { style, .. }
        | View::Canvas { style, .. }
        | View::Table { style, .. } => vec![style.as_ref()],
        View::Row { style, .. } | View::Column { style, .. } | View::List { style, .. } => {
            vec![style.as_ref()]
        }
        View::Container { style, .. } | View::Scrollable { style, .. } => vec![style.as_ref()],
    };
    for style in styles.into_iter().flatten() {
        for class in &style.classes {
            scan.style_tokens.insert(native_style_token(class));
        }
    }
    // 子级递归（变体形状各异——逐一列出）。
    match view {
        View::AnchorSlot { child, .. }
        | View::Container { child, .. }
        | View::Scrollable { child, .. }
        | View::Sidebar { content: child, .. } => scan_native_node(child, scan),
        View::Row { children, .. } | View::Column { children, .. } | View::List { items: children, .. } => {
            for child in children {
                scan_native_node(child, scan);
            }
        }
        View::Button { content: Some(child), .. } => scan_native_node(child, scan),
        View::Grid { cells, .. } => {
            for cell in cells {
                scan_native_node(cell, scan);
            }
        }
        View::Overlay { base, content, .. } => {
            scan_native_node(base, scan);
            scan_native_node(content, scan);
        }
        View::MouseArea { content, .. } => scan_native_node(content, scan),
        // PLAN-029 T-04：Popover 双子树递归（anchor + content——:562 缺口
        // 清偿；open 闭态 content 子树同扫——覆盖判定与开合态无关）。
        View::Popover { anchor, content, .. } => {
            if let crate::ui::view::PopoverAnchor::Widget(child) = anchor {
                scan_native_node(child, scan);
            }
            scan_native_node(content, scan);
        }
        // PLAN-032 T-03（D2）：Tabs contents 子树递归（防漏钉②纪律——
        // 内容子树的未覆盖 kind/类必须入缺项清单；labels 纯数据无子树）。
        View::Tabs { contents, .. } => {
            for child in contents {
                scan_native_node(child, scan);
            }
        }
        _ => {}
    }
}

/// StyleClass → 代表性样式 token（覆盖判定用——judge 只做前缀匹配，
/// 数值档不重要；映射集与 native 投影器适配器的消费面同册演进）。
/// 未支持渲染的类产出**不含任何支持前缀**的稳定名（shadow/opacity-…）
/// → not-yet 缺项。
pub fn native_style_token(class: &crate::ui::style::StyleClass) -> String {
    use crate::ui::style::StyleClass as SC;
    match class {
        SC::Padding(_) => "p-1".into(),
        SC::PaddingX(_) => "px-1".into(),
        SC::PaddingY(_) => "py-1".into(),
        SC::PaddingTop(_) => "pt-1".into(),
        SC::PaddingBottom(_) => "pb-1".into(),
        SC::PaddingLeft(_) => "pl-1".into(),
        SC::PaddingRight(_) => "pr-1".into(),
        SC::Margin(_) => "m-1".into(),
        SC::MarginX(_) => "mx-1".into(),
        SC::MarginY(_) => "my-1".into(),
        SC::MarginTop(_) => "mt-1".into(),
        SC::MarginBottom(_) => "mb-1".into(),
        SC::MarginLeft(_) => "ml-1".into(),
        SC::MarginRight(_) => "mr-1".into(),
        SC::MarginLeftAuto => "ml-auto".into(),
        SC::MarginRightAuto => "mr-auto".into(),
        SC::MarginXAuto => "mx-auto".into(),
        SC::NegativeMargin(_) => "-m-1".into(),
        SC::NegativeMarginX(_) => "-mx-1".into(),
        SC::NegativeMarginY(_) => "-my-1".into(),
        SC::NegativeMarginTop(_) => "-mt-1".into(),
        SC::NegativeMarginBottom(_) => "-mb-1".into(),
        SC::NegativeMarginLeft(_) => "-ml-1".into(),
        SC::NegativeMarginRight(_) => "-mr-1".into(),
        SC::Gap(_) => "gap-1".into(),
        SC::BackgroundColor(_) => "bg-slate-500".into(),
        SC::BgGradient(_) => "bg-gradient-to-r".into(),
        SC::BgClipText => "bg-clip-text".into(),
        SC::GradientFrom(_) => "from-slate-500".into(),
        SC::GradientTo(_) => "to-slate-500".into(),
        SC::TextColor(_) => "text-slate-500".into(),
        SC::Width(_) => "w-1".into(),
        SC::Height(_) => "h-1".into(),
        SC::MaxWidth(_) => "max-w-1".into(),
        SC::MaxHeight(_) => "max-h-1".into(),
        SC::ItemsCenter => "items-center".into(),
        SC::ItemsStart => "items-start".into(),
        SC::ItemsEnd => "items-end".into(),
        SC::JustifyCenter => "justify-center".into(),
        SC::JustifyBetween => "justify-between".into(),
        SC::JustifyStart => "justify-start".into(),
        SC::JustifyEnd => "justify-end".into(),
        SC::TextXs => "text-xs".into(),
        SC::TextSm => "text-sm".into(),
        SC::TextBase => "text-base".into(),
        SC::TextLg => "text-lg".into(),
        SC::TextXl => "text-xl".into(),
        SC::Text2Xl => "text-2xl".into(),
        SC::Text3Xl => "text-3xl".into(),
        SC::Text4Xl => "text-4xl".into(),
        SC::Text5Xl => "text-5xl".into(),
        SC::Text6Xl => "text-6xl".into(),
        SC::Text7Xl => "text-7xl".into(),
        SC::Text8Xl => "text-8xl".into(),
        SC::Text9Xl => "text-9xl".into(),
        SC::FontBold | SC::FontMedium | SC::FontNormal => "font-bold".into(),
        SC::FontSerif | SC::FontSans | SC::FontMono => "font-sans".into(),
        SC::TextCenter => "text-center".into(),
        SC::TextLeft => "text-left".into(),
        SC::TextRight => "text-right".into(),
        SC::Border | SC::BorderBottom | SC::BorderTop | SC::BorderLeft | SC::BorderRight => {
            "border".into()
        }
        SC::BorderColor(_) => "border-slate-500".into(),
        // Border0 = 显式无边框；单侧宽度档 = PLAN-054 左条（native v1 渲染
        // 为整圈 1px 或忽略——保真边界，判定放行同解释态 border 前缀）。
        SC::Border0 => "border-0".into(),
        SC::BorderWidth(_) => "border-1".into(),
        SC::BorderLeftWidth(_) => "border-l-1".into(),
        SC::Rounded
        | SC::RoundedSm
        | SC::RoundedMd
        | SC::RoundedLg
        | SC::RoundedXl
        | SC::Rounded2Xl
        | SC::Rounded3Xl
        | SC::RoundedFull
        | SC::RoundedNone
        | SC::RoundedT(_)
        | SC::RoundedB(_)
        | SC::RoundedL(_)
        | SC::RoundedR(_)
        | SC::RoundedTL(_)
        | SC::RoundedTR(_)
        | SC::RoundedBL(_)
        | SC::RoundedBR(_) => "rounded".into(),
        // —— 渲染未实现面：稳定名不含支持前缀 → not-yet（显式缺项）。
        SC::Flex1 => "flex-1".into(),
        // PLAN-029 T-08：FlexColReverse 降级放行（列序渲染 not-yet——
        // flex-col 同 token，025 flex-1/shadow-sm 降级先例同口径）。
        SC::Flex | SC::FlexRow | SC::FlexCol | SC::FlexColReverse => "flex".into(),
        SC::Block | SC::Inline | SC::InlineBlock | SC::InlineFlex => "block".into(),
        SC::MinWidth(_) => "min-w-1".into(),
        SC::MinHeight(_) => "min-h-1".into(),
        SC::LineThrough => "line-through".into(),
        SC::Underline => "underline".into(),
        SC::NoUnderline => "no-underline".into(),
        SC::Shadow
        | SC::ShadowSm
        | SC::ShadowMd
        | SC::ShadowLg
        | SC::ShadowXl
        | SC::Shadow2Xl
        | SC::ShadowNone => "shadow".into(),
        SC::Opacity(_) => "opacity-50".into(),
        // PLAN-026 T-06：overflow 家族 → 降级放行 token（渲染 no-op）。
        SC::OverflowAuto
        | SC::OverflowHidden
        | SC::OverflowVisible
        | SC::OverflowScroll
        | SC::OverflowXAuto
        | SC::OverflowYAuto
        | SC::OverflowXHidden
        | SC::OverflowYHidden
        | SC::OverflowXScroll
        | SC::OverflowYScroll => "overflow-hidden".into(),
        // 行高倍率：native Text op 行高 = 字号×LINE_H_FACTOR 固定档——
        // leading-* 渲染未实现，判定降级放行（解释态 target_set 同册）。
        SC::LineHeight(_) | SC::LineHeightNone => "leading-1".into(),
        // PLAN-026 T-06 字重族补全（FontBold 同族——TextStyled weight
        // 700 档近似/正常档随注；判定放行 = font- 前缀）。
        SC::FontSemiBold | SC::FontLight | SC::FontExtraLight | SC::FontExtraBold
        | SC::FontThin => "font-bold".into(),
        // backdrop-*（518 G8 冻结词汇——共享 parser 识别，queue 臂渲染
        // no-op；解释态 target_set 同册放行）。
        SC::BackdropBlur(_) | SC::BackdropSaturate(_) => "backdrop-blur".into(),
        // 字距（tracking-*）：Text op 无字距通道——渲染 no-op 放行。
        SC::Tracking(_) => "tracking-1".into(),
        // 交互态/渲染提示类：静态帧 no-op（queue 命令帧无 cursor/outline/
        // transition/抗锯齿通道）——判定放行。
        SC::CursorPointer => "cursor-pointer".into(),
        SC::OutlineNone => "outline-none".into(),
        SC::Antialiased => "antialiased".into(),
        SC::TransitionColors | SC::TransitionDuration(_) => "transition".into(),
        SC::Shrink0 => "shrink-0".into(),
        SC::WhitespaceNowrap => "whitespace-nowrap".into(),
        // position:relative（无 offset 配对）/ items-stretch（块流缺省
        // 交叉轴）= 布局 no-op——判定放行（absolute+offset 族仍 not-yet）。
        SC::Relative => "relative".into(),
        SC::ItemsStretch => "items-stretch".into(),
        // PLAN-032 T-02（D5）：self-center（交叉轴自对齐）——投影器
        // NodeStyle.center_children 通道真渲（012 映射缺口清偿；非降级）。
        SC::SelfCenter => "self-center".into(),
        // PLAN-032 T-07（D5 族运行时面补臂）：inset-N（四向偏移归一——
        // iced :1209 同语义）——NodeStyle 四槽填充真渲（与 absolute 组合
        // = 全覆盖锚定；024 chart 模态纱 inset-0）；line-clamp-N（行数
        // 截断——iced :1263 有实现，DrawOp 无裁剪通道）——判定降级
        // 放行（渲染 no-op 随注，underline 先例；真渲债随 KNOWN-DEBT）。
        SC::Inset(_) => "inset-1".into(),
        SC::LineClamp(_) | SC::LineClampNone => "line-clamp-1".into(),
        // flex-wrap（行折行）——native Row 单行无折行通道：token 归
        /// "flex-wrap"（既有 "flex" 前缀命中），渲染 no-op 单行随注。
        SC::FlexWrap => "flex-wrap".into(),
        SC::TextArbitrary(_) => "text-1".into(),
        SC::ShadowArbitrary(_) => "shadow".into(),
        // —— 语义承载未实现面：显式 not-yet（语义名缺项载荷——缺项清单
        // 自描述；026 数据行缺项面）。定位族（absolute/fixed/sticky/
        // offset/z-index）、rotate（视觉变换）、truncate/break-words
        //（文本裁剪）、list-none（列表标记）、accent（表单强调色）、
        // stroke（lucide 描边——native 位图/字形通道 not-yet 同册）。
        // hidden/样式版 grid 原 in-not-yet——PLAN-032 T-02/T-04（D3/D4）
        // 转真渲放行（NodeStyle.hidden 布局 choke + display 族响应式
        // 覆盖规则；grid_cols/rows 分岔复用 Grid walker；prefixes ⑧⑩）。
        // 定位族原 not-yet——PLAN-032 T-05（D1 分层）：absolute+offset
        // 真渲（覆盖序锚定）+ fixed/sticky 降级放行随注（in-flow 渲染，
        // 真渲债另立；prefixes ⑪）；z 完整栈序仍 not-yet（absolute 同层
        // 相对层级已渲）。
        SC::Grid | SC::GridCols(_) | SC::GridRows(_) => "style-grid".into(),
        SC::Hidden => "hidden".into(),
        SC::Absolute => "absolute".into(),
        SC::Fixed => "fixed".into(),
        SC::Sticky => "sticky".into(),
        SC::TopOffset(_) => "top-1".into(),
        SC::LeftOffset(_) => "left-1".into(),
        SC::RightOffset(_) => "right-1".into(),
        SC::BottomOffset(_) => "bottom-1".into(),
        SC::ZIndex(_) => "z-1".into(),
        SC::Rotate(_) => "rotate-1".into(),
        SC::Truncate => "truncate".into(),
        // PLAN-029 T-08：opacity 降级放行 token（渲染 not-yet，prefixes ⑦）。
        SC::Opacity(_) => "opacity-50".into(),
        // PLAN-034 T-05：ring 族降级放行 token（focus 光圈装饰渲染
        // not-yet——043 样板携带 ring-2/ring-primary/ring-offset-2，
        // 声明放行同 opacity 先例）。
        SC::RingWidth(_) => "ring-2".into(),
        SC::RingColor(_) => "ring-primary".into(),
        SC::RingInset => "ring-inset".into(),
        SC::BreakWords => "break-words".into(),
        SC::ListNone => "list-none".into(),
        SC::AccentColor(_) => "accent-1".into(),
        SC::StrokeWidth(_) => "stroke-1".into(),
        _ => "native-unstyled".into(),
    }
}

/// 三态渲染开关（Plan 500 步骤 6：裁决链 spawn 参数 > pac.at > auto 探测）
/// ---------------------------------------------------------------------------

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
        // PLAN-033 T-04：Auto 探测改走 native 门（scan_native_view ×
        // native_queue_set——与 run_dynamic_client 的 ensure_covered 同
        // 法官；旧 target_set 是解释投影器时代的覆盖表，随其退役）。
        // 两态均 Commands 且无降级标记（解释 pixels 降级臂已退役）——
        // NotCovered 由臂启动覆盖门权威拒绝（eprintln + Err，禁静默
        // 错绘）。
        RenderMode::Auto => (super::message::FrameMode::Commands, None),
    }
}

// ---------------------------------------------------------------------------
// T2 单测：能力表 vs 视图清单判定（覆盖/不覆盖/降级载荷）
// ---------------------------------------------------------------------------

#[cfg(test)]
mod p034_ring_probe {
    /// PLAN-034：ring 族声明放行实测（043 样板样式）。
    #[test]
    fn p034_ring_tokens_admitted() {
        let parser = crate::ui::style::StyleParser::default();
        let parsed = parser.parse(" ring-2 ring-primary ring-offset-2 ").unwrap_or_default();
        let cov = super::Coverage::native_queue_set();
        for sc in &parsed {
            let tok = super::native_style_token(sc);
            assert!(
                cov.style_token_supported(&tok),
                "token {tok}（{sc:?}）未放行"
            );
        }
    }
}

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
    /// Plan 518 G8 翻转：backdrop-* 毛玻璃词汇已声明冻结（共享 parser
    /// 识别 + 前缀放行,渲染 no-op）——原"滤镜类不在 v1 子集"断言换向。
    #[test]
    fn unsupported_style_token_listed() {
        let coverage = Coverage::target_set();
        assert!(coverage.style_token_supported("p-8"));
        assert!(coverage.style_token_supported("-mt-10"));
        assert!(coverage.style_token_supported("hover:bg-blue-600"));
        assert!(coverage.style_token_supported("bg-gradient-to-r"));
        assert!(!coverage.style_token_supported("animate-pulse"), "动画类不在 v1 子集");
        assert!(coverage.style_token_supported("backdrop-blur-xl"), "Plan 518 声明冻结");
        assert!(coverage.style_token_supported("backdrop-saturate-[1.6]"));
    }

    /// Plan 518 G8 三臂核对（queue 臂）：玻璃样式串装载判定 Covered
    /// （不触发"未知类 → 整 widget not-yet"）+ 共享 parser 识别 +
    /// BoxLayout 提取跳过装饰字段（布局属性零污染）。
    #[test]
    fn backdrop_glass_style_queue_arm_not_rejected() {
        use super::ViewScan;
        // ① 共享 parser 识别（vue 直通外的两渲染臂共同词汇）。
        let s = crate::ui::style::Style::parse(
            "backdrop-blur-xl backdrop-saturate-[1.6] bg-white/10 border rounded-xl",
        )
        .expect("玻璃样式串可解析");
        assert!(
            s.classes.iter().any(|c| matches!(
                c,
                crate::ui::style::StyleClass::BackdropBlur(24.0)
            )),
            "blur-xl → 24px"
        );
        assert!(
            s.classes.iter().any(|c| matches!(
                c,
                crate::ui::style::StyleClass::BackdropSaturate(1.6)
            )),
            "saturate-[1.6] → 1.6"
        );
        // ② BoxLayout 提取跳过装饰字段（无布局属性写入）。
        let layout = crate::ui::style::BoxLayout::from_classes(&s.classes);
        assert!(layout.width.is_none() && layout.height.is_none() && layout.gap.is_none());
        // ③ 装载判定:玻璃样式 token 全放行 → Covered（无误判缺项）。
        let mut scan = ViewScan::default();
        scan.tags.insert("col".into());
        scan.style_tokens.insert("backdrop-blur-xl".into());
        scan.style_tokens.insert("backdrop-saturate-[1.6]".into());
        scan.style_tokens.insert("bg-white/10".into());
        scan.style_tokens.insert("border".into());
        scan.style_tokens.insert("rounded-xl".into());
        assert_eq!(
            judge(&scan, &Coverage::target_set()),
            Verdict::Covered,
            "queue 臂玻璃样式不触发 not-yet"
        );
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

        // PLAN-033 T-04 重录：解释 pixels 降级臂退役——auto 未覆盖视图
        // 仍返 Commands（无降级标记），由 run_dynamic_client 启动覆盖门
        // 权威拒绝（"拒绝渲染，禁静默错绘"）；native 覆盖事实另由
        // ensure_covered 断言面钉（native_projector::coverage_gate_*）。
        let uncovered = crate::build_dynamic_component(
            "widget U { view { select (value: .mode) { onchange: .Pick } } }",
            None,
        ).expect("build");
        let (mode, downgrade) = effective_frame_mode(RM::Auto, &uncovered);
        assert_eq!(mode, FrameMode::Commands, "auto 恒 queue（门权威裁决）");
        assert!(downgrade.is_none(), "无降级标记（pixels 降级臂退役）: {downgrade:?}");
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

    /// PLAN-033 T-04 重录（原 t7_not_yet_families_auto_downgrade）：解释
    /// pixels 降级臂退役 + 032 翻转后 native 词表全族过门（六族探测实证
    /// 2026-09-19）——not-yet 语义面从"auto 降级 independent"转轨为
    /// "queue 过门 + 运行期 uncovered_seen 降级观测"（native_projector
    /// 显式留痕面）。
    #[test]
    fn t7_families_all_pass_native_gate() {
        use crate::ui::desktop_protocol::native_projector::RqProjector;
        let cases: &[(&str, &str)] = &[
            ("overlay 弹层", "widget O { view { select (value: .m) { onchange: .P } } }"),
            ("chart/diagram", "widget C { view { svg { path {} } } }"),
            ("复合编辑器", "widget E { view { markdown (content: .doc) } }"),
            ("nav 系", "widget N { view { nav-item (label: \"x\") { onclick: .Go } } }"),
            ("表格族", "widget T { view { table { text \"r\" } } }"),
            ("瞬态浮层", "widget F { view { toaster {} } }"),
        ];
        for (family, src) in cases {
            let component = crate::build_dynamic_component(src, None)
                .unwrap_or_else(|e| panic!("{family} build: {e}"));
            let p = RqProjector::new(component, 480.0, 320.0);
            p.ensure_covered()
                .unwrap_or_else(|gate| panic!("{family} 应过 native 门（词表全族在册）: {gate}"));
        }
    }

    /// PLAN-025 T-06：003-converter 真源样式 token 全放行（native gate
    /// ——flex-1/shadow 降级放行定案；AC-05 覆盖翻转样本）。
    #[test]
    fn native_gate_accepts_003_style_tokens() {
        let coverage = Coverage::native_queue_set();
        for token in [
            "flex-1", "gap-1.5", "max-w-md", "p-8", "bg-card", "border",
            "rounded-2xl", "shadow-sm", "mx-auto", "text-2xl", "font-bold",
            "text-primary", "text-center", "mb-6", "gap-4", "text-sm",
            "font-medium", "text-muted-foreground", "text-xs", "mt-6",
        ] {
            assert!(
                coverage.style_token_supported(token),
                "003 token 应放行: {token}"
            );
        }
    }

    /// PLAN-026 T-06 + 004-profile-card 真源样式 token 全放行（native
    /// gate——overflow- 降级放行定案；AC-02 覆盖翻转样本）。leading-/
    /// hover: 走 parser 静默丢弃/variant 通道（不入 scan—— gate 无感）。
    #[test]
    fn native_gate_accepts_004_style_tokens() {
        let coverage = Coverage::native_queue_set();
        // PLAN-026 T-06 探针：逐 token 走 typed parse → native_style_token，
        // 无 native-unstyled 混入（未映射类显式排查）。
        for tok in ["w-full", "h-20", "bg-gradient-to-r", "from-blue-500",
            "to-purple-600", "rounded-t-lg", "rounded-full", "border-4",
            "border-border", "shadow-md", "-mt-10", "items-center", "w-3",
            "h-3", "bg-green-400", "gap-2", "text-xl", "text-sm",
            "text-center", "font-bold", "font-medium", "px-3", "py-1",
            "px-4", "py-2", "px-6", "pb-6", "bg-secondary", "w-96",
            "overflow-hidden", "bg-card", "shadow-lg", "rounded-lg"] {
            if let Ok(sc) = crate::ui::style::StyleClass::parse_single(tok) {
                let t = native_style_token(&sc);
                assert!(t != "native-unstyled", "token {tok} → native-unstyled");
            }
        }
        for token in [
            "w-full", "h-20", "bg-gradient-to-r", "from-blue-500",
            "to-purple-600", "rounded-t-lg", "rounded-full", "-mt-10",
            "items-center", "w-3", "h-3", "bg-green-400", "gap-2", "gap-1",
            "gap-3", "gap-4", "text-xl", "text-sm", "text-center",
            "font-bold", "font-medium", "px-3", "py-1", "px-4", "py-2",
            "px-6", "pb-6", "bg-secondary", "text-secondary-foreground",
            "text-muted-foreground", "bg-primary", "text-primary-foreground",
            "bg-card", "shadow-lg", "shadow-md", "border", "border-border",
            "w-96", "overflow-hidden", "leading-relaxed",
        ] {
            assert!(
                coverage.style_token_supported(token) || token == "leading-relaxed",
                "004 token 应放行（或 parser 静默丢弃面）: {token}"
            );
        }
    }

    /// PLAN-032 T-02：native 轨逐例扫描 helper（仪器 native_flip_
    /// coverage_data_row 路径单例化——front 全 .at 合并解析 → App 声明
    /// 优先 → VmBridge → AuraViewBuilder（VM 轨）→ scan_native_view）。
    fn scan_example_native(dir: &str) -> ViewScan {
        use crate::ui::aura_view_builder::AuraViewBuilder;
        use crate::ui::vm_bridge::VmBridge;

        // 双根解析（stage3 example_source 同则）：046-tabs 等样板随
        // fix-ui-track-reorg（2026-09-20）迁 capability-tests。
        let front = ["examples/ui", "examples/capability-tests"]
            .iter()
            .map(|root| {
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../")
                    .join(root)
                    .join(dir)
                    .join("src/front")
            })
            .find(|p| p.is_dir())
            .unwrap_or_else(|| panic!("example {dir} not found under examples/ui or capability-tests"));
        let mut srcs: Vec<std::path::PathBuf> = std::fs::read_dir(&front)
            .unwrap_or_else(|e| panic!("read {front:?}: {e}"))
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "at"))
            .collect();
        srcs.sort();
        let mut combined = String::new();
        for s in &srcs {
            combined.push_str(&std::fs::read_to_string(s).unwrap_or_default());
            combined.push('\n');
        }
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(combined.as_str()).with_session(session);
        let ast = parser.parse().unwrap_or_else(|e| panic!("parse {dir}: {e:?}"));
        let mut app_decl: Option<&crate::ast::WidgetDecl> = None;
        let mut first_decl: Option<&crate::ast::WidgetDecl> = None;
        for st in &ast.stmts {
            if let crate::ast::Stmt::WidgetDecl(d) = st {
                if first_decl.is_none() {
                    first_decl = Some(d);
                }
                if d.name.as_str() == "App" {
                    app_decl = Some(d);
                    break;
                }
            }
        }
        let decl = app_decl
            .or(first_decl)
            .unwrap_or_else(|| panic!("{dir} 无 WidgetDecl"));
        let widget = crate::aura::extract::extract_widget_from_decl(decl)
            .unwrap_or_else(|e| panic!("extract {dir}: {e:?}"));
        let bridge = VmBridge::new_from_decls(
            decl,
            &[],
            vec![],
            &std::collections::HashMap::new(),
            false,
        )
        .unwrap_or_else(|e| panic!("bridge {dir}: {e:?}"));
        let view = AuraViewBuilder::new(&bridge, &widget.name).build(&widget.view_tree);
        scan_native_view(&view)
    }

    /// PLAN-032 T-02..T-05（六缺项全清偿）：逐例翻绿——SelfCenter 映射
    ///（012）+ hidden（012/041/018）+ tabs kind（046）+ 样式 grid（024）
    /// + 定位族（018/021——absolute 真渲 + fixed/sticky 降级放行）。
    /// VM 轨扫描与仪器同径；六例 = ramp v3 judged 22/22 的逐例金钉。
    #[test]
    fn native_gate_examples_012_041_covered() {
        let coverage = Coverage::native_queue_set();
        for dir in [
            "012-clock",
            "041-auto-edit",
            "tabs-variants",
            "024-charts",
            "018-book-reader",
            "021-blog-viewer",
        ] {
            let scan = scan_example_native(dir);
            let verdict = judge(&scan, &coverage);
            assert!(verdict.is_covered(), "{dir} 应 Covered: {verdict:?}");
        }
    }

    /// PLAN-032 T-06：翻转态钉——resolve_native_frame_mode Auto ×
    /// Covered = Commands（缺省 queue；翻转前 Pixels——025-032 在案），
    /// 观测行携带 flipped@ramp3；NotCovered 降级路径不变（Pixels +
    /// 真扫描缺项清单）。
    #[test]
    fn native_auto_default_flipped_to_queue() {
        use crate::ui::desktop_protocol::client_entry::resolve_native_frame_mode;
        use crate::ui::desktop_protocol::message::FrameMode;
        use super::RenderMode as RM;

        let covered = crate::ui::view::View::<()>::text("t");
        let (mode, downgraded, line) = resolve_native_frame_mode(RM::Auto, "T", &covered);
        assert_eq!(mode, FrameMode::Commands, "Auto × Covered = queue（flipped@ramp3）");
        assert!(downgraded, "auto 探测标记维持（观测行随行）");
        let line = line.expect("观测行");
        assert!(line.contains("flipped@ramp3"), "观测行文案: {line}");

        let uncovered = crate::ui::view::View::<()>::text_styled("x", "rotate-1");
        let (mode2, _, line2) = resolve_native_frame_mode(RM::Auto, "U", &uncovered);
        assert_eq!(mode2, FrameMode::Pixels, "未覆盖降级 independent 不变");
        let line2 = line2.expect("降级观测行");
        assert!(line2.contains("rotate-1"), "缺项清单随行: {line2}");
        // PLAN-036 T-02（⑫）：truncate 真渲放行后不再作未覆盖样本
        //（rotate 接任——定位族 not-yet 在册）。
        let covered_trunc = crate::ui::view::View::<()>::text_styled("x", "truncate");
        let (mode3, _, _) = resolve_native_frame_mode(RM::Auto, "T2", &covered_trunc);
        assert_eq!(mode3, FrameMode::Commands, "truncate 真渲后 Covered");
    }

    /// PLAN-032 T-07：六例**运行时视图**（path 上下文装载 + Component::
    /// view()——路由解析后子树含页组件样式）覆盖判定钉——与仪器静态
    /// App 壳扫描（native_flip_coverage_data_row，026 D3 口径）的差在
    /// 案：018 truncate / 041 codeeditor 两真 not-yet 家族运行时拒收
    /// （I3 留痕腿——queue 门拒绝退出，AC-04）先后清偿（018=PLAN-036
    /// T-02 truncate 真渲；041=PLAN-674 T-01 codeeditor 扩册）；D5 族
    /// 运行时面补臂（Inset/LineClamp/FlexWrap）后 021/024 亦 Covered。
    /// 生产 auto 裁决（a2r main resolve_native_frame_mode 消费
    /// component.view()）即本口径。
    #[test]
    fn native_gate_runtime_views_of_six() {
        let base = concat!(env!("CARGO_MANIFEST_DIR"), "/../../");
        // (例, 期望缺项——None = Covered)。tabs-variants 已迁
        // capability-tests（双根解析）。
        let expect: &[(&str, Option<&str>)] = &[
            ("012-clock", None),
            // PLAN-036 T-02（⑫）：truncate 真渲放行——018 翻 Covered
            //（原缺项 style:truncate 清偿，Text 发射臂 `…` 真渲）。
            ("018-book-reader", None),
            ("021-blog-viewer", None),
            ("024-charts", None),
            // PLAN-674 T-01：codeeditor 入册（结构 DrawOps 投影臂）——
            // 041 翻 Covered（原缺项 tag:codeeditor 清偿）。
            ("041-auto-edit", None),
            ("tabs-variants", None),
            // PLAN-674 T-05：codeeditor RQ 形态 fixture 入运行时门
            //（capability-tests 根）。
            ("codeeditor-rq", None),
        ];
        for (dir, missing) in expect {
            let path = ["examples/ui", "examples/capability-tests"]
                .iter()
                .map(|root| format!("{base}{root}/{dir}/src/front/app.at"))
                .find(|p| std::path::Path::new(p).is_file())
                .unwrap_or_else(|| format!("{base}examples/ui/{dir}/src/front/app.at"));
            let src = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("read {path}: {e}"));
            let comp = crate::build_dynamic_component(&src, Some(&path))
                .unwrap_or_else(|e| panic!("build {dir}: {e:?}"));
            let view = crate::ui::Component::view(&comp);
            let scan = scan_native_view(&view);
            match judge(&scan, &Coverage::native_queue_set()) {
                Verdict::Covered => assert!(
                    missing.is_none(),
                    "{dir} 运行时应 Covered（口径差入案）"
                ),
                Verdict::NotCovered(m) => {
                    let want = missing.unwrap_or_else(|| panic!("{dir} 应 Covered: {m:?}"));
                    assert!(
                        m.iter().any(|x| x.contains(want)),
                        "{dir} 运行时缺项应含 {want}: {m:?}"
                    );
                }
            }
        }
    }

    /// PLAN-026 T-06：覆盖翻转数据行（§5.1 D3 定案仪器）——examples/ui
    /// 全量 .at → AuraViewBuilder（VM 轨运行时 aura→View 构造器，与
    /// a2r codegen 同以"降级到 View IR"为口径）→ scan_native_view ×
    /// judge(native_queue_set)。数据行入 026 报告；阈值 = ≥95% 且缺项
    /// 全在册 not-yet（AC-06 双出口的达标腿判据）。
    #[test]
    fn native_flip_coverage_data_row() {
        use crate::ui::aura_view_builder::AuraViewBuilder;
        use crate::ui::vm_bridge::VmBridge;

        let examples_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/ui");
        // (name, covered, 缺项/失败原因)
        let mut rows: Vec<(String, bool, String)> = Vec::new();
        let mut dirs: Vec<std::path::PathBuf> = std::fs::read_dir(&examples_dir)
            .expect("examples dir")
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        dirs.sort();
        for dir in dirs {
            let front = dir.join("src/front");
            if !front.is_dir() {
                continue;
            }
            let mut srcs: Vec<std::path::PathBuf> = std::fs::read_dir(&front)
                .expect("front dir")
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == "at"))
                .collect();
            srcs.sort();
            if srcs.is_empty() {
                continue;
            }
            let name = dir.file_name().unwrap().to_string_lossy().to_string();
            let mut combined = String::new();
            for s in &srcs {
                combined.push_str(&std::fs::read_to_string(s).unwrap_or_default());
                combined.push('\n');
            }
            let session = crate::session::CompilerSession::ui();
            let mut parser = crate::Parser::from(combined.as_str()).with_session(session);
            let Ok(ast) = parser.parse() else {
                rows.push((name, false, "parse-fail".into()));
                continue;
            };
            // App widget 优先（examples 惯例），缺省首个 WidgetDecl。
            let mut app_decl: Option<&crate::ast::WidgetDecl> = None;
            let mut first_decl: Option<&crate::ast::WidgetDecl> = None;
            for st in &ast.stmts {
                if let crate::ast::Stmt::WidgetDecl(d) = st {
                    if first_decl.is_none() {
                        first_decl = Some(d);
                    }
                    if d.name.as_str() == "App" {
                        app_decl = Some(d);
                        break;
                    }
                }
            }
            let Some(decl) = app_decl.or(first_decl) else {
                rows.push((name, false, "no-widget".into()));
                continue;
            };
            let Ok(widget) = crate::aura::extract::extract_widget_from_decl(decl) else {
                rows.push((name, false, "extract-fail".into()));
                continue;
            };
            let bridge = VmBridge::new_from_decls(
                decl,
                &[],
                vec![],
                &std::collections::HashMap::new(),
                false,
            );
            let Ok(bridge) = bridge else {
                rows.push((name, false, "bridge-fail".into()));
                continue;
            };
            let view = AuraViewBuilder::new(&bridge, &widget.name).build(&widget.view_tree);
            let scan = scan_native_view(&view);
            match judge(&scan, &Coverage::native_queue_set()) {
                Verdict::Covered => rows.push((name, true, String::new())),
                Verdict::NotCovered(missing) => {
                    rows.push((name, false, missing.join(", ")))
                }
            }
        }
        let total = rows.len();
        let covered = rows.iter().filter(|(_, c, _)| *c).count();
        let pct = covered as f64 / total.max(1) as f64 * 100.0;
        // PLAN-032 T-06（D6）：judged 口径——剔除仪器桶（parse-fail/
        // extract-fail/bridge-fail/no-widget：样本装载失败非覆盖面缺项，
        // 026 D3 沿承；029 报告的剔除行同集）。
        let instrument_bucket =
            |why: &str| matches!(why, "parse-fail" | "extract-fail" | "bridge-fail" | "no-widget");
        let judged: Vec<&(String, bool, String)> = rows
            .iter()
            .filter(|(_, _, why)| !instrument_bucket(why))
            .collect();
        let judged_total = judged.len();
        let judged_covered = judged.iter().filter(|(_, c, _)| *c).count();
        let judged_pct = judged_covered as f64 / judged_total.max(1) as f64 * 100.0;
        eprintln!("[native-flip-data] covered {covered}/{total} = {pct:.1}% (overall)");
        eprintln!(
            "[native-flip-data] judged covered {judged_covered}/{judged_total} = {judged_pct:.1}% (threshold >=95%)"
        );
        for (name, c, why) in &rows {
            if !c {
                eprintln!("[native-flip-data]   {name}: {why}");
            }
        }
        assert!(total > 0, "样本集非空");
        assert!(judged_total > 0, "judged 样本非空");
        // PLAN-032 T-06 翻转后防漏钉（断言反转——翻转前为 assert!(!flip)
        // 的"达标即红逼翻转"向下任）：缺省已 queue
        ///（resolve_native_frame_mode Covered 臂 Commands，p032 报告）
        ///——judged 跌破 95% 门即红：降级需显式裁定（台账裁定行 + 报告
        /// 更新），禁静默回归。
        let flip = judged_pct >= 95.0;
        assert!(
            flip,
            "judged Covered 比例跌破 95% 阈值（{judged_covered}/{judged_total}）——缺省 queue 已翻转（p032），跌破门需显式裁定降级（台账 + 报告），禁静默回归"
        );
        // 缺项清单非空不变式（NotCovered 行必载缺项载荷）。
        for (name, covered_row, why) in &rows {
            assert!(!(!covered_row && why.is_empty()), "{name} NotCovered 缺项空载荷");
        }
    }

    /// PLAN-674 T-02（§10-3 裁定 C）：not-yet kind 全集勘定仪器——
    /// examples/ui + examples/capability-tests **双根**全量 .at →
    /// AuraViewBuilder → scan_native_view × judge(native_queue_set)，
    /// 逐 kind 聚合缺项（kind → 出现目录集）。输出 = 三态分流决策档
    /// 数据源（本批修/登记另立/降级归一）；断言仅非空健全性（仪器
    /// 非 gate——judged 阈值门在 native_flip_coverage_data_row）。
    #[test]
    fn native_not_yet_kind_inventory() {
        use crate::ui::aura_view_builder::AuraViewBuilder;
        use crate::ui::vm_bridge::VmBridge;
        use std::collections::BTreeMap;

        let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples");
        let mut roots: Vec<(String, std::path::PathBuf)> = Vec::new();
        for root in ["ui", "capability-tests"] {
            let p = base.join(root);
            if p.is_dir() {
                roots.push((root.to_string(), p));
            }
        }
        assert!(!roots.is_empty(), "examples 根缺席");
        // kind/style/event 缺项 → (root, dir) 集（聚合视图）。
        let mut census: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut judged_samples = 0usize;
        for (root, root_dir) in &roots {
            let mut dirs: Vec<std::path::PathBuf> = std::fs::read_dir(root_dir)
                .expect("root dir")
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.is_dir())
                .collect();
            dirs.sort();
            for dir in dirs {
                let front = dir.join("src/front");
                if !front.is_dir() {
                    continue;
                }
                let name = format!("{root}/{}", dir.file_name().unwrap().to_string_lossy());
                let mut srcs: Vec<std::path::PathBuf> = std::fs::read_dir(&front)
                    .expect("front dir")
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|p| p.extension().is_some_and(|x| x == "at"))
                    .collect();
                srcs.sort();
                if srcs.is_empty() {
                    continue;
                }
                let mut combined = String::new();
                for s in &srcs {
                    combined.push_str(&std::fs::read_to_string(s).unwrap_or_default());
                    combined.push('\n');
                }
                let session = crate::session::CompilerSession::ui();
                let mut parser = crate::Parser::from(combined.as_str()).with_session(session);
                let Ok(ast) = parser.parse() else {
                    eprintln!("[p674-not-yet] {name}: parse-fail（仪器桶——非覆盖缺项）");
                    continue;
                };
                let mut app_decl: Option<&crate::ast::WidgetDecl> = None;
                for st in &ast.stmts {
                    if let crate::ast::Stmt::WidgetDecl(d) = st {
                        if app_decl.is_none() || d.name.as_str() == "App" {
                            app_decl = Some(d);
                        }
                        if d.name.as_str() == "App" {
                            break;
                        }
                    }
                }
                let Some(decl) = app_decl else {
                    eprintln!("[p674-not-yet] {name}: no-widget（仪器桶）");
                    continue;
                };
                let Ok(widget) = crate::aura::extract::extract_widget_from_decl(decl) else {
                    eprintln!("[p674-not-yet] {name}: extract-fail（仪器桶）");
                    continue;
                };
                let bridge = VmBridge::new_from_decls(
                    decl,
                    &[],
                    vec![],
                    &std::collections::HashMap::new(),
                    false,
                );
                let Ok(bridge) = bridge else {
                    eprintln!("[p674-not-yet] {name}: bridge-fail（仪器桶）");
                    continue;
                };
                let view = AuraViewBuilder::new(&bridge, &widget.name).build(&widget.view_tree);
                let scan = scan_native_view(&view);
                judged_samples += 1;
                if let Verdict::NotCovered(missing) = judge(&scan, &Coverage::native_queue_set()) {
                    for m in missing {
                        census.entry(m).or_default().push(name.clone());
                    }
                }
            }
        }
        assert!(judged_samples > 0, "judged 样本非空");
        if census.is_empty() {
            eprintln!("[p674-not-yet] 全集空：双根 judged 样本全覆盖");
        }
        for (kind, dirs) in &census {
            eprintln!("[p674-not-yet] {kind}: {} 处（{}）", dirs.len(), dirs.join(", "));
        }
    }

    /// PLAN-029 T-08（D6/D7）：shell pack 五件装载期 native 判定 Covered
    ///——B 程序覆盖门预演（View 树扫描断言，popover/mousearea/
    /// thumbnail/preview 四 kind 扩容后的硬前置验证）。auto-os 检出解析
    /// 序：`$AUTO_OS_ROOT` → 本仓根兄弟 `../auto-os`（worktree 组内 = 组内
    /// auto-os；主检出 = 主 auto-os）→ `D:/autostack/auto-os` 主检出兜底
    ///（AGENTS §2 跨仓解析序，零链接红线）。检出/文件缺席 → skip 留痕
    /// 不 fail（pac.at 静默门先例——独立仓检出状态不炸本仓门）。
    #[test]
    fn shell_pack_native_covered() {
        use crate::ui::aura_view_builder::AuraViewBuilder;
        use crate::ui::vm_bridge::VmBridge;

        let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .expect("repo root");
        let candidates: Vec<std::path::PathBuf> = [
            std::env::var("AUTO_OS_ROOT").ok().map(std::path::PathBuf::from),
            repo_root.parent().map(|p| p.join("auto-os")),
            Some(std::path::PathBuf::from("D:/autostack/auto-os")),
        ]
        .into_iter()
        .flatten()
        .collect();
        let Some(os_root) = candidates
            .iter()
            .find(|p| p.join("shell/shell.at").is_file())
            .cloned()
        else {
            eprintln!(
                "[p029-shell-pack] skip: auto-os 检出缺席（候选: {:?}）",
                candidates
            );
            return;
        };
        for name in [
            "shell.at",
            "desktop.at",
            "switcher.at",
            "dashboard.at",
            "notification_center.at",
        ] {
            let path = os_root.join("shell").join(name);
            let Ok(code) = std::fs::read_to_string(&path) else {
                eprintln!("[p029-shell-pack] skip: {name} 缺席（{}）", path.display());
                continue;
            };
            let session = crate::session::CompilerSession::ui();
            let mut parser = crate::Parser::from(code.as_str()).with_session(session);
            let Ok(ast) = parser.parse() else {
                panic!("{name} 解析失败（shell pack 是 B 覆盖门硬前置——解析红必拦）");
            };
            let mut app_decl: Option<&crate::ast::WidgetDecl> = None;
            let mut first_decl: Option<&crate::ast::WidgetDecl> = None;
            for st in &ast.stmts {
                if let crate::ast::Stmt::WidgetDecl(d) = st {
                    if first_decl.is_none() {
                        first_decl = Some(d);
                    }
                    if d.name.as_str() == "App" {
                        app_decl = Some(d);
                        break;
                    }
                }
            }
            let Some(decl) = app_decl.or(first_decl) else {
                panic!("{name} 无 widget 声明");
            };
            let Ok(widget) = crate::aura::extract::extract_widget_from_decl(decl) else {
                panic!("{name} widget 抽取失败");
            };
            let bridge = VmBridge::new_from_decls(
                decl,
                &[],
                vec![],
                &std::collections::HashMap::new(),
                false,
            );
            let Ok(bridge) = bridge else {
                panic!("{name} VmBridge 构建失败");
            };
            let view = AuraViewBuilder::new(&bridge, &widget.name).build(&widget.view_tree);
            let scan = scan_native_view(&view);
            match judge(&scan, &Coverage::native_queue_set()) {
                Verdict::Covered => {
                    eprintln!("[p029-shell-pack] {name}: Covered");
                }
                Verdict::NotCovered(missing) => {
                    panic!("{name} 未覆盖（B 覆盖门硬前置）: {}", missing.join(", "));
                }
            }
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
