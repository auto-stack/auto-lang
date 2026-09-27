//! PLAN-045: VM 渲染组件级 memo（第一批：菜单族四件 + sidebar nav 块）。
//!
//! 机制（canonical 档 `auto-os docs/specs/shell/vm-render-memo.md`，T-01 决策
//! 注记为准）：`memo: true` 显式 opt-in 的组件节点按 **稳定 path 键** 缓存
//! `convert_*` 求值产物；命中条件 = 全局 `state_mutation_seq` 未动（快速
//! 路径）∧ 全局 episode 指纹同 ∧ 静态读槽重解析值指纹同（慢路径）。缺省/
//! `false` 走原始求值路径，行为逐字节一致。
//!
//! 正确性论证（为什么不需要 per-field 版本表，见 PLAN-045 T-01）：转换器是
//! 确定式函数 `f(AST 子树, prop 表达式解析值, 全局 episode 态)`。同一桥生命
//! 周期内 view 模板不可变（hot-reload 走新建 VmBridge，缓存随桥弃置），
//! AST 恒同；故只需证明 prop 表达式的解析值与全局态未变。慢路径在 check
//! 时经**同一求值通道**（`resolve_expr_to_value`）重解析全部读槽并指纹比对
//! ——读槽值的任何来源（state 直读/容器内容）都被值指纹覆盖，不依赖写点
//! 归因；全局 seq 只是免重解析的快速通道（桥写已补 bump，见 vm_bridge
//! PLAN-045 注记）。
//!
//! 降级（宁缺勿错，正确性下限 = memo 错误只允许"变慢"）：扫描到静态不可证
//! 形态（Block/Call/方法调用/FStr 插值/ForLoop/Conditional/Component/
//! Outlet/StyleBinding）→ 不建条目直接原始路径；bindings 非空或 widget 声明
//! computed → 该 builder 上下文整体不 memo。

use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};

use crate::ast::Expr;
use crate::aura::AuraNodeId;
use crate::aura::{AuraNode, AuraTextContent};
use auto_val::Value;

use crate::ui::debug::ProbeEntry;
use crate::ui::interpreter::DynamicMessage;
use crate::ui::view::View;

/// 指纹预算：单值指纹展开的节点数上限。超出 → `None`（降级，走原始路径）。
/// 首批消费者（菜单族/sidebar nav 块）的读槽都是标量/小容器，预算不咬合；
/// 大容器（如 datatable 723 行）超限降级保正确。
const FINGERPRINT_BUDGET: usize = 4096;

/// 每 builder（=每 VmBridge）缓存条目上限（LRU 逐出最旧）。PLAN-045 Q-03：
/// 64/组件初值；此为每桥全局口径（首批量级 ≪64），执行期按内存实测调。
const MEMO_CACHE_CAP: usize = 256;

// ─────────────────────────────────────────────────────────────────────
// 键与条目
// ─────────────────────────────────────────────────────────────────────

/// 缓存键。`ctx_state_obj` 区分子 widget 状态作用域（child 渲染时
/// `override_state_obj_id` 生效）；`skeleton_fp` 是子树静态骨架指纹
/// （tag/字面量 prop/文本/结构——纯 AST 漫步，不含表达式值；模板在桥
/// 生命周期内不可变 → 骨架同 ⇒ 结构同，表达式值差异由 dyn_fp 兜住）；
/// `site` 防不同 convert 家族骨架撞键；probe_on 进键（PLAN-045 §4：
/// probe-off 先填充、probe-on 后命中会吞 acceptance 事件索引——两态各存
/// 各的条目）。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemoKey {
    pub ctx_state_obj: u64,
    pub site: u8,
    pub skeleton_fp: u64,
    pub probe_on: bool,
}

/// memo 条目：fill 时的动态输入指纹 + 产物 + probe/id_map 重放面。
#[derive(Debug, Clone)]
pub struct MemoEntry {
    /// fill 时刻的全局状态突变序号（快速路径比对基准；慢路径命中后刷新）。
    pub seq_at_fill: u64,
    /// fill 时的全局 episode 指纹（theme_epoch/menubar_open/popover_open/
    /// action_config 身份）。
    pub globals_fp: u64,
    /// fill 时静态提取的读槽表达式（check 时经同一通道重解析）。
    pub read_exprs: Vec<Expr>,
    /// fill 时读槽解析值指纹；`None` = 无读槽（纯静态子树）。
    pub dyn_fp: Option<u64>,
    /// fill 产物（`View: Clone`，命中按值克隆返回）。
    pub product: View<DynamicMessage>,
    /// fill 期间 base path 前缀下新增的 probe 条目（命中重放，保 acceptance
    /// 事件索引——AC-06）。
    pub probe_replay: Vec<(Vec<u16>, ProbeEntry)>,
    /// fill 期间新增的 id_map 条目（dialog/popover 族 tracked 子树用）。
    pub idmap_replay: Vec<(Vec<usize>, AuraNodeId)>,
}

/// 每 VmBridge 一张 memo 表 + 计数器（AC-03 断言/T-07 度量走 counts）。
#[derive(Debug, Default)]
pub struct MemoCache {
    entries: HashMap<MemoKey, MemoEntry>,
    order: VecDeque<MemoKey>,
    pub hits: u64,
    pub misses: u64,
    pub degraded: u64,
    pub evictions: u64,
}

impl MemoCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// 查表（LRU touch；hit 计数由调用方在判实命中——产物确被复用——时
    /// 经 [`Self::note_hit`] 记，慢路径 miss 不虚计）。
    pub fn lookup(&mut self, key: &MemoKey) -> Option<&MemoEntry> {
        if self.entries.contains_key(key) {
            if let Some(pos) = self.order.iter().position(|k| k == key) {
                if let Some(k) = self.order.remove(pos) {
                    self.order.push_back(k);
                }
            }
            self.entries.get(key)
        } else {
            None
        }
    }

    pub fn note_hit(&mut self) {
        self.hits += 1;
    }

    pub fn note_miss(&mut self) {
        self.misses += 1;
    }

    pub fn note_degraded(&mut self) {
        self.degraded += 1;
    }

    pub fn insert(&mut self, key: MemoKey, entry: MemoEntry) {
        if !self.order.contains(&key) {
            self.order.push_back(key.clone());
        }
        self.entries.insert(key, entry);
        while self.order.len() > MEMO_CACHE_CAP {
            if let Some(oldest) = self.order.pop_front() {
                if self.entries.remove(&oldest).is_some() {
                    self.evictions += 1;
                }
            }
        }
    }

    /// 刷新既有条目的 seq 基准（慢路径命中后调用——刚证明全部动态输入在
    /// 当前 seq 下未变，后续帧可走快速路径）。
    pub fn refresh_seq(&mut self, key: &MemoKey, seq: u64) {
        if let Some(e) = self.entries.get_mut(key) {
            e.seq_at_fill = seq;
        }
    }

    /// 全量失效（THEME_EPOCH 等全局面翻转时的兜底口；当前全局态走指纹
    /// 比对无需清表，此口留档与测试用）。
    pub fn clear(&mut self) {
        self.entries.clear();
        self.order.clear();
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// ─────────────────────────────────────────────────────────────────────
// 值指纹
// ─────────────────────────────────────────────────────────────────────

/// 对解析值做确定性指纹。堆引用（`VmRef`/`Int(id≥4M)`）经 `expand` 展开
/// 后再哈希——堆 id 不稳定（跨帧重建变 id）、id 同内容可被原地突变
/// （LIST_SET/PUSH），只有展开内容才是真值。`expand` 无法识别的堆引用
/// （未覆盖的类型化 ListData 等）→ `None`：调用方必须整条目降级（宁多
/// 失效，绝不按含 id 的指纹兜底——原地突变会漏检成陈旧命中）。超预算同
/// 样 → `None`。
pub fn fingerprint_value(v: &Value, expand: &dyn Fn(&Value) -> Value) -> Option<u64> {
    let mut budget = FINGERPRINT_BUDGET;
    fingerprint_inner(v, expand, &mut budget)
}

fn is_heap_ref(v: &Value) -> bool {
    match v {
        Value::VmRef(_) => true,
        Value::Int(id) => *id >= 4_000_000,
        _ => false,
    }
}

fn fingerprint_inner(
    v: &Value,
    expand: &dyn Fn(&Value) -> Value,
    budget: &mut usize,
) -> Option<u64> {
    if *budget == 0 {
        return None;
    }
    *budget -= 1;
    if is_heap_ref(v) {
        let m = expand(v);
        if is_heap_ref(&m) {
            // 展开器不识别（原样返回）→ 无法内容指纹 → 降级（None）。
            return None;
        }
        return fingerprint_inner(&m, expand, budget);
    }
    match v {
        Value::Array(arr) => {
            let mut h = std::collections::hash_map::DefaultHasher::new();
            "arr".hash(&mut h);
            arr.values.len().hash(&mut h);
            for el in &arr.values {
                fingerprint_inner(el, expand, budget)?.hash(&mut h);
            }
            Some(h.finish())
        }
        Value::Block(arr) => {
            let mut h = std::collections::hash_map::DefaultHasher::new();
            "blk".hash(&mut h);
            arr.values.len().hash(&mut h);
            for el in &arr.values {
                fingerprint_inner(el, expand, budget)?.hash(&mut h);
            }
            Some(h.finish())
        }
        Value::Obj(obj) => {
            let mut h = std::collections::hash_map::DefaultHasher::new();
            "obj".hash(&mut h);
            let pairs: Vec<(&auto_val::ValueKey, &Value)> = obj.iter().collect();
            pairs.len().hash(&mut h);
            for (k, val) in pairs {
                format!("{:?}", k).hash(&mut h);
                fingerprint_inner(val, expand, budget)?.hash(&mut h);
            }
            Some(h.finish())
        }
        _ => fingerprint_debug(v, budget),
    }
}

fn fingerprint_debug(v: &Value, budget: &mut usize) -> Option<u64> {
    if *budget == 0 {
        return None;
    }
    *budget -= 1;
    let mut h = std::collections::hash_map::DefaultHasher::new();
    format!("{:?}", v).hash(&mut h);
    Some(h.finish())
}

// ─────────────────────────────────────────────────────────────────────
// 静态读槽扫描与降级判定
// ─────────────────────────────────────────────────────────────────────

pub enum ScanVerdict {
    /// 子树静态可证：携带需 check 时重解析的顶层表达式（prop 表达式）。
    Slots(Vec<Expr>),
    /// 静态不可证 → 不建条目，走原始路径（正确性下限）。
    Degrade(&'static str),
}

/// 对 memo 候选子树做静态扫描。规则（宁缺勿错）：
/// - Element：prop `Expr` 按静态可证性分级（字面量不入槽；Ident/Dot/二元/
///   一元/Index/Object/Array/Pair 入槽——check 时经同一通道重解析）；
///   `StyleBinding` 降级；事件（handler 字符串）静态；children 递归。
/// - Text：字面量静态；插值（`${}`）降级（首批不涉，保守面）。
/// - ForLoop/Conditional/Component/Outlet：降级（首批 nav 块不涉）。
/// - Link：to/text 静态 + children 递归。
/// - 调用形态（`Expr::Call`/`Block`/`FStr`/Lambda/Closure 等）一律降级：
///   求值走 VM 代码，静态不可证其读面与纯度。
pub fn scan_subtree_static(node: &AuraNode) -> ScanVerdict {
    let mut slots = Vec::new();
    match scan_node(node, &mut slots) {
        Ok(()) => ScanVerdict::Slots(slots),
        Err(reason) => ScanVerdict::Degrade(reason),
    }
}

/// memo 门的实用形态：直接扫 (props, children)——转换器解构后拿到的就是
/// 这两样，无需合成节点。语义 = `scan_subtree_static` 的 Element 面。
pub fn scan_static(
    props: &HashMap<String, crate::aura::AuraPropValue>,
    children: &[AuraNode],
) -> ScanVerdict {
    let mut slots = Vec::new();
    for (_k, prop) in props.iter() {
        match prop {
            crate::aura::AuraPropValue::Expr(e) => {
                if let Err(reason) = scan_expr(e, &mut slots) {
                    return ScanVerdict::Degrade(reason);
                }
            }
            crate::aura::AuraPropValue::StyleBinding(_) => {
                return ScanVerdict::Degrade("style_binding");
            }
        }
    }
    for c in children {
        if let Err(reason) = scan_node(c, &mut slots) {
            return ScanVerdict::Degrade(reason);
        }
    }
    ScanVerdict::Slots(slots)
}

fn scan_node(node: &AuraNode, slots: &mut Vec<Expr>) -> Result<(), &'static str> {
    match node {
        AuraNode::Element { props, children, .. } => {
            for (_k, prop) in props.iter() {
                match prop {
                    crate::aura::AuraPropValue::Expr(e) => scan_expr(e, slots)?,
                    crate::aura::AuraPropValue::StyleBinding(_) => {
                        return Err("style_binding");
                    }
                }
            }
            for c in children {
                scan_node(c, slots)?;
            }
            Ok(())
        }
        AuraNode::Text(crate::aura::AuraTextContent::Literal(_)) => Ok(()),
        AuraNode::Text(crate::aura::AuraTextContent::Interpolated { .. }) => {
            Err("interpolated_text")
        }
        AuraNode::ForLoop { .. } => Err("for_loop"),
        AuraNode::Conditional { .. } => Err("conditional"),
        AuraNode::Component { .. } => Err("component"),
        AuraNode::Outlet => Err("outlet"),
        AuraNode::Link { children, .. } => {
            for c in children {
                scan_node(c, slots)?;
            }
            Ok(())
        }
    }
}

/// 单个 prop 表达式的静态可证性：字面量 → 静态（不入槽）；确定式解析形态
/// → 入槽（check 时重解析）；VM 代码/闭包/插值 → 降级。
fn scan_expr(e: &Expr, slots: &mut Vec<Expr>) -> Result<(), &'static str> {
    match e {
        // 字面量：值恒定，不依赖状态。
        Expr::Int(_)
        | Expr::Uint(_)
        | Expr::I8(_)
        | Expr::U8(_)
        | Expr::I64(_)
        | Expr::U64(_)
        | Expr::Byte(_)
        | Expr::Float(_, _)
        | Expr::Double(_, _)
        | Expr::Bool(_)
        | Expr::Char(_)
        | Expr::Str(_)
        | Expr::CStr(_) => Ok(()),
        // 确定式解析形态（bindings 空 + computed 空的门下，求值 = 纯 state
        // 读/物化/组合）——整表达式入槽，check 时经同一通道重解析。
        Expr::Ident(_) | Expr::Dot(_, _) | Expr::Unary(_, _) | Expr::Bina(_, _, _) => {
            slots.push(e.clone());
            Ok(())
        }
        Expr::Index(_, _) | Expr::Object(_) | Expr::Array(_) | Expr::Pair(_) => {
            slots.push(e.clone());
            Ok(())
        }
        // VM 代码/闭包/插值/借用系——静态不可证。
        Expr::Block(_)
        | Expr::Call(_)
        | Expr::Lambda(_)
        | Expr::Closure(_)
        | Expr::FStr(_)
        | Expr::Node(_)
        | Expr::Grid(_)
        | Expr::Ref(_)
        | Expr::View(_)
        | Expr::Mut(_)
        | Expr::Move(_)
        | Expr::Take(_)
        | Expr::Hold(_)
        | Expr::GenName(_) => Err("dynamic_expr"),
        // 兜底：未知/未列入形态一律降级（宁缺勿错）。
        _ => Err("unclassified_expr"),
    }
}

// ─────────────────────────────────────────────────────────────────────
// 静态骨架指纹（缓存键成分）
// ─────────────────────────────────────────────────────────────────────

/// 子树静态骨架指纹：tag/prop 键/字面量 prop 值/文本字面量/结构（children
/// 数与嵌套）——不含表达式求值值（那由慢路径 dyn_fp 覆盖）。纯 AST 漂步，
/// 零 VM 零分配（除哈希器）。调用前 scan 已通过（降级形态不会到这里）。
pub fn skeleton_fingerprint(
    props: &HashMap<String, crate::aura::AuraPropValue>,
    children: &[AuraNode],
) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    skeleton_props(props, &mut h);
    skeleton_children(children, &mut h);
    h.finish()
}

fn skeleton_props(props: &HashMap<String, crate::aura::AuraPropValue>, h: &mut std::collections::hash_map::DefaultHasher) {
    // 键序稳定化：HashMap 序不定，按键排序后哈希。
    let mut keys: Vec<&String> = props.keys().collect();
    keys.sort();
    keys.len().hash(h);
    for k in keys {
        k.hash(h);
        match &props[k] {
            crate::aura::AuraPropValue::Expr(e) => {
                "expr".hash(h);
                // 表达式只入"字面量值/形态判别"——值语义由 dyn_fp 承担；这里
                // 用 Debug 串（确定性）把字面量值与形态差异分开：字面量值变
                // → 骨架变（正确：字面量不重解析）；表达式形态变（如 Dot→
                // Index）→ 骨架变（保守，多失效不陈旧）。
                format!("{:?}", expr_shape(e)).hash(h);
            }
            crate::aura::AuraPropValue::StyleBinding(_) => {
                // 降级面不会到达；兜底计入区分度。
                "style_binding".hash(h);
            }
        }
    }
}

/// 表达式形态压缩：字面量保留值，其余按变体名（不计内部结构——内部读面
/// 由 dyn_fp 重解析覆盖）。
fn expr_shape(e: &Expr) -> String {
    match e {
        Expr::Int(v) => format!("Int({v})"),
        Expr::Uint(v) => format!("Uint({v})"),
        Expr::I8(v) => format!("I8({v})"),
        Expr::U8(v) => format!("U8({v})"),
        Expr::I64(v) => format!("I64({v})"),
        Expr::U64(v) => format!("U64({v})"),
        Expr::Byte(v) => format!("Byte({v})"),
        Expr::Float(v, _) => format!("Float({v})"),
        Expr::Double(v, _) => format!("Double({v})"),
        Expr::Bool(v) => format!("Bool({v})"),
        Expr::Char(v) => format!("Char({v})"),
        Expr::Str(v) => format!("Str({v})"),
        Expr::CStr(v) => format!("CStr({v})"),
        other => format!("{:?}", std::mem::discriminant(other)),
    }
}

fn skeleton_children(children: &[AuraNode], h: &mut std::collections::hash_map::DefaultHasher) {
    children.len().hash(h);
    for c in children {
        match c {
            AuraNode::Element { tag, props, children, .. } => {
                "el".hash(h);
                tag.hash(h);
                skeleton_props(props, h);
                skeleton_children(children, h);
            }
            AuraNode::Text(AuraTextContent::Literal(s)) => {
                "txt".hash(h);
                s.hash(h);
            }
            _ => {
                // 降级面不会到达（ForLoop/Conditional/Component/Outlet/插值
                // 已在 scan 拒绝）；兜底计入变体判别。
                "other".hash(h);
                std::mem::discriminant(c).hash(h);
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// 全局 episode 指纹
// ─────────────────────────────────────────────────────────────────────

/// 全局态指纹分量：主题代数 + 菜单/弹层开态 + action_config 身份。
/// 任一翻转 → 全部 memo 条目慢路径 miss（语义正确：产物依赖这些全局量）。
#[derive(Debug, Clone)]
pub struct GlobalEpisode {
    pub theme_epoch: u32,
    pub menubar_open: Option<String>,
    pub popover_open: Option<String>,
    /// `Arc::as_ptr` 身份——配置热重载换 Arc 即翻面。
    pub action_config_ptr: usize,
}

pub fn globals_fingerprint(ep: &GlobalEpisode) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    ep.theme_epoch.hash(&mut h);
    ep.menubar_open.hash(&mut h);
    ep.popover_open.hash(&mut h);
    ep.action_config_ptr.hash(&mut h);
    h.finish()
}

// ─────────────────────────────────────────────────────────────────────
// 测试
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aura::{AuraPropValue, AuraTextContent};
    use auto_val::{Array, AutoStr};

    fn ident_dot(field: &str) -> Expr {
        // `.field` 的解析形态：Dot(Ident("."), field)
        Expr::Dot(
            Box::new(Expr::Ident(AutoStr::from("."))),
            AutoStr::from(field),
        )
    }

    fn elem(tag: &str, props: Vec<(&str, Expr)>, children: Vec<AuraNode>) -> AuraNode {
        AuraNode::Element {
            tag: tag.to_string(),
            props: props
                .into_iter()
                .map(|(k, v)| (k.to_string(), AuraPropValue::Expr(v)))
                .collect(),
            events: HashMap::new(),
            children,
            span: None,
            debug_id: None,
        }
    }

    fn text_lit(s: &str) -> AuraNode {
        AuraNode::Text(AuraTextContent::Literal(s.to_string()))
    }

    fn passthrough(v: &Value) -> Value {
        v.clone()
    }

    #[test]
    fn fingerprint_value_deterministic_and_expanded() {
        let a = Value::Int(42);
        let b = Value::Int(42);
        assert_eq!(
            fingerprint_value(&a, &passthrough),
            fingerprint_value(&b, &passthrough)
        );
        // VmRef 经 expand 展开后按内容指纹（同内容同指纹，id 无关）。
        let r1 = Value::VmRef(auto_val::VmRef { id: 7 });
        let r2 = Value::VmRef(auto_val::VmRef { id: 9 });
        let expand = |v: &Value| match v {
            Value::VmRef(r) if r.id == 7 || r.id == 9 => Value::Str(AutoStr::from("same")),
            other => other.clone(),
        };
        assert_eq!(fingerprint_value(&r1, &expand), fingerprint_value(&r2, &expand));
        // 不可展开的堆引用：指纹 None（调用方据此整条目降级——原地突变
        // 检不出的形态绝不入缓存）。
        assert_eq!(
            fingerprint_value(&Value::VmRef(auto_val::VmRef { id: 7 }), &passthrough),
            None
        );
    }

    #[test]
    fn fingerprint_value_detects_array_content_change() {
        let mk = |n: i32| Value::Array(Array {
            values: (0..n).map(Value::Int).collect(),
        });
        assert_ne!(
            fingerprint_value(&mk(3), &passthrough),
            fingerprint_value(&mk(4), &passthrough)
        );
    }

    #[test]
    fn scan_literal_only_subtree_no_slots() {
        let node = elem(
            "menubar",
            vec![("class", Expr::Str(AutoStr::from("flex")))],
            vec![
                elem("menubar-trigger", vec![("text", Expr::Str(AutoStr::from("File")))], vec![]),
                text_lit("body"),
            ],
        );
        match scan_subtree_static(&node) {
            ScanVerdict::Slots(slots) => assert!(slots.is_empty(), "字面量子树应零读槽"),
            ScanVerdict::Degrade(r) => panic!("字面量子树不应降级: {}", r),
        }
    }

    #[test]
    fn scan_state_binding_prop_yields_slot() {
        let node = elem(
            "menubar-item",
            vec![
                ("text", Expr::Str(AutoStr::from("Show URLs"))),
                ("checked", ident_dot("show_urls")),
            ],
            vec![],
        );
        match scan_subtree_static(&node) {
            ScanVerdict::Slots(slots) => assert_eq!(slots.len(), 1, "checked: .show_urls 入一槽"),
            ScanVerdict::Degrade(r) => panic!("状态绑定 prop 不应降级: {}", r),
        }
    }

    #[test]
    fn scan_dynamic_forms_degrade() {
        // 插值文本
        let interp = AuraNode::Text(AuraTextContent::Interpolated {
            template: "x ${.count}".to_string(),
            bindings: vec!["count".to_string()],
        });
        assert!(matches!(
            scan_subtree_static(&interp),
            ScanVerdict::Degrade("interpolated_text")
        ));
        // 调用形态 prop（GenName 归入动态兜底族）
        let dyn_node = elem(
            "label",
            vec![("text", Expr::GenName(AutoStr::from("x")))],
            vec![],
        );
        assert!(matches!(
            scan_subtree_static(&dyn_node),
            ScanVerdict::Degrade("dynamic_expr")
        ));
        // ForLoop 子节点
        let for_node = AuraNode::ForLoop {
            var: "i".to_string(),
            index: None,
            iterable: ".items".to_string(),
            body: vec![],
            span: None,
            debug_id: None,
        };
        assert!(matches!(
            scan_subtree_static(&for_node),
            ScanVerdict::Degrade("for_loop")
        ));
    }

    #[test]
    fn scan_nested_children_merged() {
        let node = elem(
            "menubar",
            vec![],
            vec![elem(
                "menubar-menu",
                vec![],
                vec![elem(
                    "menubar-item",
                    vec![("checked", ident_dot("show_urls"))],
                    vec![],
                )],
            )],
        );
        match scan_subtree_static(&node) {
            ScanVerdict::Slots(slots) => assert_eq!(slots.len(), 1, "嵌套 children 读槽合并"),
            ScanVerdict::Degrade(r) => panic!("嵌套绑定不应降级: {}", r),
        }
    }

    fn mk_entry(seq: u64) -> MemoEntry {
        MemoEntry {
            seq_at_fill: seq,
            globals_fp: 0,
            read_exprs: vec![],
            dyn_fp: None,
            product: View::Empty,
            probe_replay: vec![],
            idmap_replay: vec![],
        }
    }

    #[test]
    fn memo_cache_lru_eviction_and_counters() {
        let mut c = MemoCache::new();
        for i in 0..(MEMO_CACHE_CAP + 8) {
            let key = MemoKey {
                ctx_state_obj: 1,
                site: 0,
                skeleton_fp: i as u64,
                probe_on: false,
            };
            c.insert(key, mk_entry(0));
        }
        assert_eq!(c.len(), MEMO_CACHE_CAP, "LRU 上限逐出");
        assert_eq!(c.evictions, 8);
        let hit_key = MemoKey { ctx_state_obj: 1, site: 0, skeleton_fp: (MEMO_CACHE_CAP + 7) as u64, probe_on: false };
        assert!(c.lookup(&hit_key).is_some());
        c.note_hit();
        assert_eq!(c.hits, 1);
        let miss_key = MemoKey { ctx_state_obj: 1, site: 0, skeleton_fp: 999_999, probe_on: false };
        assert!(c.lookup(&miss_key).is_none());
        c.note_miss();
        assert_eq!(c.misses, 1);
        c.refresh_seq(&hit_key, 5);
        let e = c.lookup(&hit_key).expect("still present");
        assert_eq!(e.seq_at_fill, 5);
    }

    #[test]
    fn skeleton_fingerprint_structural_identity() {
        // 同骨架（tag/字面量/结构同，表达式值不同）→ 同指纹。
        let a = elem("menubar-item", vec![("checked", ident_dot("show_urls"))], vec![]);
        let b = elem("menubar-item", vec![("checked", ident_dot("other_field"))], vec![]);
        let (fa, fb) = match (scan_subtree_static(&a), scan_subtree_static(&b)) {
            (ScanVerdict::Slots(_), ScanVerdict::Slots(_)) => {
                use crate::aura::AuraPropValue;
                let pa = std::collections::HashMap::from([("checked".to_string(), AuraPropValue::Expr(ident_dot("show_urls")))]);
                let pb = std::collections::HashMap::from([("checked".to_string(), AuraPropValue::Expr(ident_dot("other_field")))]);
                (skeleton_fingerprint(&pa, &[]), skeleton_fingerprint(&pb, &[]))
            }
            _ => panic!("两个绑定形态都应可证"),
        };
        assert_eq!(fa, fb, "表达式值差异不进骨架（由 dyn_fp 承担）");
        // 字面量值/结构差异 → 指纹翻面。
        let c = elem("menubar-item", vec![("text", Expr::Str(AutoStr::from("File")))], vec![]);
        let d = elem("menubar-item", vec![("text", Expr::Str(AutoStr::from("Edit")))], vec![]);
        use crate::aura::AuraPropValue;
        let pc = std::collections::HashMap::from([("text".to_string(), AuraPropValue::Expr(Expr::Str(AutoStr::from("File"))))]);
        let pd = std::collections::HashMap::from([("text".to_string(), AuraPropValue::Expr(Expr::Str(AutoStr::from("Edit"))))]);
        assert_ne!(
            skeleton_fingerprint(&pc, &[]),
            skeleton_fingerprint(&pd, &[]),
            "字面量值进骨架"
        );
        let _ = (c, d);
        // children 数差异 → 翻面。
        let e1 = elem("row", vec![], vec![text_lit("a")]);
        let e2 = elem("row", vec![], vec![text_lit("a"), text_lit("b")]);
        let (s1, s2) = (skeleton_fingerprint(&HashMap::new(), &[e1]), skeleton_fingerprint(&HashMap::new(), &[e2]));
        assert_ne!(s1, s2);
    }

    #[test]
    fn globals_fingerprint_flips_on_any_component_change() {
        let base = GlobalEpisode {
            theme_epoch: 1,
            menubar_open: None,
            popover_open: None,
            action_config_ptr: 0x1000,
        };
        let f0 = globals_fingerprint(&base);
        let mut menubar_open_some = base.clone();
        menubar_open_some.menubar_open = Some("menu-file".to_string());
        assert_ne!(f0, globals_fingerprint(&menubar_open_some));
        let mut theme2 = base.clone();
        theme2.theme_epoch = 2;
        assert_ne!(f0, globals_fingerprint(&theme2));
        let mut cfg2 = base.clone();
        cfg2.action_config_ptr = 0x2000;
        assert_ne!(f0, globals_fingerprint(&cfg2));
        let mut pop = base.clone();
        pop.popover_open = Some("dlg-1".to_string());
        assert_ne!(f0, globals_fingerprint(&pop));
    }
}
