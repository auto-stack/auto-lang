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

/// 每 builder（=每 VmBridge）缓存条目上限默认值（LRU 逐出最旧）。
/// PLAN-045 Q-03：64/组件初值；此为每桥全局口径（首批量级 ≪64）。
const MEMO_CACHE_CAP: usize = 256;

/// PLAN-046 Q-03：keyed-for 大列表（syslog 千行流）单帧 fill 需 ≥列表长度
/// 的容量，否则 LRU 抖动使重求值计数无法归零（AC-07）。仍共池单 LRU（机制
/// 单源），键化门按需抬升、硬顶此值（内存实测在 T-06；超顶=部分条目走
/// LRU 淘汰语义，正确性不受影响——只允许变慢）。
const MEMO_CACHE_CAP_MAX: usize = 4096;

/// PLAN-046 站点常量（MemoKey.site）：keyed-for 项级条目。档 A 组件族
/// 常量（1..=5）与 outlet 页（6）在 aura_view_builder，此处 7 起新面。
pub const MEMO_SITE_FOR_ITEM: u8 = 7;

/// PLAN-046 站点常量：显式 memo 块条目（T-04）。
pub const MEMO_SITE_MEMO_BLOCK: u8 = 8;

/// PLAN-046 §4：per-site 分解计数（for_item/memo_block/outlet 门经
/// `note_*_site` 记账；档 A 组件门只走全局计数器）。PLAN-047 T-05：
/// check-kind 分解（seq_fast/version_fast/fp_slow——三级判定的观测面，
/// version_fast = 动态 dep 集版本全同的零重解析命中）。
#[derive(Debug, Default, Clone)]
pub struct SiteCounts {
    pub hits: u64,
    pub misses: u64,
    pub degraded: u64,
    pub seq_fast: u64,
    pub version_fast: u64,
    pub fp_slow: u64,
}

// ─────────────────────────────────────────────────────────────────────
// 键与条目
// ─────────────────────────────────────────────────────────────────────

/// 缓存键。`ctx_state_obj` 区分子 widget 状态作用域（child 渲染时
/// `override_state_obj_id` 生效）；`skeleton_fp` 是子树静态骨架指纹
/// （tag/字面量 prop/文本/结构——纯 AST 漫步，不含表达式值；模板在桥
/// 生命周期内不可变 → 骨架同 ⇒ 结构同，表达式值差异由 dyn_fp 兜住）；
/// `site` 防不同 convert 家族骨架撞键；probe_on 进键（PLAN-045 §4：
/// probe-off 先填充、probe-on 后命中会吞 acceptance 事件索引——两态各存
/// 各的条目）。PLAN-046 T-03：`item_key` = keyed-for 项级条目的 key 值
/// 指纹（`None` = 档 A 组件/outlet 条目——档 A 键面零变化）。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemoKey {
    pub ctx_state_obj: u64,
    pub site: u8,
    pub skeleton_fp: u64,
    pub probe_on: bool,
    pub item_key: Option<u64>,
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
    /// PLAN-046 T-03：keyed-for 项级条目 = combine3(读槽, 项值, index)。
    pub dyn_fp: Option<u64>,
    /// fill 产物（`View: Clone`，命中按值克隆返回）。
    pub product: View<DynamicMessage>,
    /// fill 期间 base path 前缀下新增的 probe 条目（命中重放，保 acceptance
    /// 事件索引——AC-06）。
    pub probe_replay: Vec<(Vec<u16>, ProbeEntry)>,
    /// fill 期间新增的 id_map 条目（dialog/popover 族 tracked 子树用）。
    pub idmap_replay: Vec<(Vec<usize>, AuraNodeId)>,
    /// PLAN-046 T-03：keyed-for 项级条目的重放面按**项内相对路径**存储
    /// （命中帧以当前基路径+[新 index] 前缀化重放、ForIter.index 补丁）；
    /// `false` = 档 A 绝对路径重放（组件/outlet 结构稳定面，行为不变）。
    pub replay_relative: bool,
    /// PLAN-047 T-05（档 C SD-11）: fill 期动态依赖集 + 基线版本对——
    /// check 的 `version_fast` 判定面（零重解析）。`None` = 录制空集/
    /// 超预算（落回既有静态扫描+指纹慢路径，行为与档 A/B 一致）。
    pub dyn_deps: Option<Vec<(DepKey, u64)>>,
}

/// 每 VmBridge 一张 memo 表 + 计数器（AC-03 断言/T-07 度量走 counts）。
#[derive(Debug)]
pub struct MemoCache {
    entries: HashMap<MemoKey, MemoEntry>,
    order: VecDeque<MemoKey>,
    cap: usize,
    pub hits: u64,
    pub misses: u64,
    pub degraded: u64,
    pub evictions: u64,
    pub site_counts: HashMap<u8, SiteCounts>,
    /// PLAN-047 T-05: check-kind 全局分解（观测面——AC-02 断言用）。
    pub seq_fast: u64,
    pub version_fast: u64,
    pub fp_slow: u64,
}

impl Default for MemoCache {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
            order: VecDeque::new(),
            cap: MEMO_CACHE_CAP,
            hits: 0,
            misses: 0,
            degraded: 0,
            evictions: 0,
            site_counts: HashMap::new(),
            seq_fast: 0,
            version_fast: 0,
            fp_slow: 0,
        }
    }
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

    /// PLAN-046 §4：带 site 分解的计数（全局计数同步递增）。
    pub fn note_hit_site(&mut self, site: u8) {
        self.hits += 1;
        self.site_counts.entry(site).or_default().hits += 1;
    }

    pub fn note_miss_site(&mut self, site: u8) {
        self.misses += 1;
        self.site_counts.entry(site).or_default().misses += 1;
    }

    pub fn note_degraded_site(&mut self, site: u8) {
        self.degraded += 1;
        self.site_counts.entry(site).or_default().degraded += 1;
    }

    /// PLAN-046 Q-03：按需抬升容量（共池 LRU，硬顶 [`MEMO_CACHE_CAP_MAX`]）。
    pub fn ensure_capacity(&mut self, min_cap: usize) {
        let want = min_cap.max(MEMO_CACHE_CAP).min(MEMO_CACHE_CAP_MAX);
        if want > self.cap {
            self.cap = want;
        }
    }

    pub fn insert(&mut self, key: MemoKey, entry: MemoEntry) {
        if !self.order.contains(&key) {
            self.order.push_back(key.clone());
        }
        self.entries.insert(key, entry);
        while self.order.len() > self.cap {
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

    /// PLAN-047 T-05: 刷新既有条目的动态 dep 基线版本（fp_slow 命中后调用
    /// ——值同证明版本前进无害，基线前移让后续帧回到 version_fast）。
    pub fn refresh_deps(&mut self, key: &MemoKey, pairs: Vec<(DepKey, u64)>) {
        if let Some(e) = self.entries.get_mut(key) {
            if e.dyn_deps.is_some() {
                e.dyn_deps = Some(pairs);
            }
        }
    }

    /// PLAN-047 T-05: check-kind 计数（全局 + 可选 per-site 分解）。
    pub fn note_seq_fast(&mut self, site: Option<u8>) {
        self.seq_fast += 1;
        if let Some(s) = site {
            self.site_counts.entry(s).or_default().seq_fast += 1;
        }
    }

    pub fn note_version_fast(&mut self, site: Option<u8>) {
        self.version_fast += 1;
        if let Some(s) = site {
            self.site_counts.entry(s).or_default().version_fast += 1;
        }
    }

    pub fn note_fp_slow(&mut self, site: Option<u8>) {
        self.fp_slow += 1;
        if let Some(s) = site {
            self.site_counts.entry(s).or_default().fp_slow += 1;
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
// PLAN-047 T-01: 依赖录制器（档 C SD-08）
//
// T-04 起核心类型单源下沉 `crate::vm::dep_track`（vm 模块不依赖 ui 门控
// 面；AutoVM 录制槽与读臂挂钩共用同组类型）——此处转发导出保持 T-01
// 引用面（`memo_deps::DepKey` 等）零变化。
// ─────────────────────────────────────────────────────────────────────

pub use crate::vm::dep_track::{DepKey, RecState, DEP_PATH_ANY, REC_DEP_BUDGET};

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
    scan_static_with_components(props, children, None)
}

/// T-05b 组件模板感知版：`Component` 节点的模板（registry 子 widget 的
/// view_tree）一并扫描——模板内的状态读同为读槽；visited 集合破自引用环。
/// registry 缺席（None）时 Component 退回降级（宁缺勿错）。
pub fn scan_static_with_components(
    props: &HashMap<String, crate::aura::AuraPropValue>,
    children: &[AuraNode],
    registry: Option<&crate::ui::widget_registry::WidgetRegistry>,
) -> ScanVerdict {
    let mut slots = Vec::new();
    let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();
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
        if let Err(reason) = scan_node_registry(c, &mut slots, registry, &mut visited) {
            return ScanVerdict::Degrade(reason);
        }
    }
    ScanVerdict::Slots(slots)
}

fn scan_node_registry(
    node: &AuraNode,
    slots: &mut Vec<Expr>,
    registry: Option<&crate::ui::widget_registry::WidgetRegistry>,
    visited: &mut std::collections::HashSet<String>,
) -> Result<(), &'static str> {
    if let AuraNode::Component { name, props, children, .. } = node {
        // Component props = Vec<(String, Expr)>（Element 的 HashMap 形态不同）。
        for (_k, e) in props.iter() {
            scan_expr(e, slots)?;
        }
        for c in children {
            scan_node_registry(c, slots, registry, visited)?;
        }
        if let Some(reg) = registry {
            if let Some(w) = reg.get(name) {
                if visited.insert(name.clone()) {
                    scan_node_registry(&w.view_tree, slots, Some(reg), visited)?;
                }
            }
        }
        return Ok(());
    }
    scan_node(node, slots)
}

/// PLAN-046 T-03：keyed-for **项级**扫描。正确性判据：项值指纹覆盖循环变量
/// 的一切展开（key/条件/插值引用循环变量 → 项变即失效），扫描只负责把
/// **外部状态读**找出来入槽（check 时经同一通道重解析）。规则（宁缺勿错）：
/// - Element：prop 表达式按 [`scan_expr`] 分级（槽/降级）；children 递归；
/// - Text：字面量静态；插值 bindings 全部 ⊆ 循环变量名 → 项值覆盖（安全），
///   含外部名 → 降级（v1 保守——外部读插值的槽化不在档 B 面）；
/// - Conditional：条件串经 `parse_expr_fragment` 解析入槽（解析失败降级），
///   双臂递归；
/// - 嵌套 ForLoop：iterable 根 ∈ 循环变量（`r.guides`）→ 项值覆盖，递归
///   体（内层变量并入循环变量集）；根为状态 → 降级。Outlet：降级；
/// - Component：prop 槽 + children + registry 模板递归（`scan_node_registry`
///   同型语义；registry 缺席 → 降级）；
/// - Link：children 递归。
pub fn scan_for_item_body(
    body: &[AuraNode],
    registry: Option<&crate::ui::widget_registry::WidgetRegistry>,
    loop_vars: &std::collections::HashSet<String>,
) -> ScanVerdict {
    let mut slots = Vec::new();
    let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();
    for c in body {
        if let Err(reason) = scan_item_node(c, &mut slots, registry, loop_vars, &mut visited) {
            return ScanVerdict::Degrade(reason);
        }
    }
    ScanVerdict::Slots(slots)
}

/// PLAN-046 T-06：项级 prop 表达式扫描——FStr（f-string prop，如
/// `class: f"... ${r.state}"`）的插值根段全 ∈ 循环变量 → 项值覆盖（安全，
/// 不入槽——值随项值指纹翻面）；其余形态按 [`scan_expr`] 分级。
fn scan_item_expr(e: &Expr, slots: &mut Vec<Expr>, loop_vars: &std::collections::HashSet<String>) -> Result<(), &'static str> {
    if let Expr::FStr(f) = e {
        for part in &f.parts {
            match part {
                Expr::Str(_) => {}
                Expr::Ident(name) => {
                    let root = name.as_str().trim_start_matches('.').split('.').next().unwrap_or("");
                    if !loop_vars.contains(root) {
                        return Err("fstr_external");
                    }
                }
                Expr::Dot(obj, _) => {
                    let root = match obj.as_ref() {
                        Expr::Ident(n) => n.as_str().trim_start_matches('.').split('.').next().unwrap_or(""),
                        _ => "",
                    };
                    if root.is_empty() || !loop_vars.contains(root) {
                        return Err("fstr_external");
                    }
                }
                _ => return Err("fstr_dynamic"),
            }
        }
        return Ok(());
    }
    scan_expr(e, slots)
}

fn scan_item_node(
    node: &AuraNode,
    slots: &mut Vec<Expr>,
    registry: Option<&crate::ui::widget_registry::WidgetRegistry>,
    loop_vars: &std::collections::HashSet<String>,
    visited: &mut std::collections::HashSet<String>,
) -> Result<(), &'static str> {
    match node {
        AuraNode::Element { props, children, .. } => {
            for (_k, prop) in props.iter() {
                match prop {
                    crate::aura::AuraPropValue::Expr(e) => scan_item_expr(e, slots, loop_vars)?,
                    crate::aura::AuraPropValue::StyleBinding(_) => {
                        return Err("style_binding");
                    }
                }
            }
            for c in children {
                scan_item_node(c, slots, registry, loop_vars, visited)?;
            }
            Ok(())
        }
        AuraNode::Text(crate::aura::AuraTextContent::Literal(_)) => Ok(()),
        AuraNode::Text(crate::aura::AuraTextContent::Interpolated { bindings, .. }) => {
            // 插值名根段（`r.rowcls` → `r`）∈ 循环变量 → 项值指纹已覆盖；
            // 外部名（`.count` → `count`）其读面不在项值内 → 降级（v1 保守）。
            for b in bindings {
                let root = b.split('.').next().unwrap_or(b);
                if !loop_vars.contains(root) {
                    return Err("interpolated_text_external");
                }
            }
            Ok(())
        }
        AuraNode::Conditional { condition, then_body, else_body, .. } => {
            match crate::parser::Parser::parse_expr_fragment(condition) {
                Some(e) => scan_expr(&e, slots)?,
                None => return Err("conditional_unprovable"),
            }
            for c in then_body {
                scan_item_node(c, slots, registry, loop_vars, visited)?;
            }
            if let Some(els) = else_body {
                for c in els {
                    scan_item_node(c, slots, registry, loop_vars, visited)?;
                }
            }
            Ok(())
        }
        AuraNode::ForLoop { iterable, var, index, body, .. } => {
            // PLAN-046 T-06 扩展：嵌套 for 的 iterable **根段 ∈ 循环变量**
            //（如 filetree 的 `for g in r.guides`）→ 其求值面是项值的纯
            // 函数（项变即失效）→ 递归体（内层 var/index 并入循环变量集）；
            // iterable 根是状态（`.` 前缀或外部名）→ 降级（保守不变）。
            let root = iterable.trim_start_matches('.').split('.').next().unwrap_or("");
            if iterable.starts_with('.') || !loop_vars.contains(root) {
                return Err("for_loop");
            }
            let mut inner = loop_vars.clone();
            inner.insert(var.clone());
            if let Some(iv) = index {
                inner.insert(iv.clone());
            }
            inner.insert(root.to_string());
            for c in body {
                scan_item_node(c, slots, registry, &inner, visited)?;
            }
            Ok(())
        }
        AuraNode::Outlet { .. } => Err("outlet"),
        AuraNode::MemoBlock { .. } => Err("memo_block"),
        AuraNode::Component { name, props, children, .. } => {
            let Some(reg) = registry else {
                return Err("component");
            };
            for (_k, e) in props.iter() {
                scan_expr(e, slots)?;
            }
            for c in children {
                scan_item_node(c, slots, registry, loop_vars, visited)?;
            }
            if let Some(w) = reg.get(name) {
                if visited.insert(name.clone()) {
                    scan_item_node(&w.view_tree, slots, Some(reg), loop_vars, visited)?;
                }
            }
            Ok(())
        }
        AuraNode::Link { children, .. } => {
            for c in children {
                scan_item_node(c, slots, registry, loop_vars, visited)?;
            }
            Ok(())
        }
    }
}

/// PLAN-046 T-03：keyed-for **站点**骨架指纹——循环头（var/index/iterable/
/// key 表达式全形态）+ 体读槽表达式全形态 + 体结构（Element/Text/
/// Component 模板递归，同 `skeleton_children_registry`）。同一桥生命周期内
/// ForLoop AST 不可变 → 同站恒同指纹；不同 for 站点由头差异、读槽差异或
/// 体结构差异拆键。
pub fn for_site_fingerprint(
    var: &str,
    index: Option<&str>,
    iterable: &str,
    key_expr: &crate::ast::Expr,
    body: &[AuraNode],
    registry: Option<&crate::ui::widget_registry::WidgetRegistry>,
    slots: &[Expr],
) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    var.hash(&mut h);
    index.hash(&mut h);
    iterable.hash(&mut h);
    // key 表达式全形态入指纹（有界解析器产物，Debug 串有界且确定）。
    format!("{:?}", key_expr).hash(&mut h);
    // 体读槽表达式全形态入指纹：骨架的 expr_shape 只到判别式（Dot vs Dot
    // 不分读目标），跨站点撞键时读目标差异必须进站点指纹——否则 A 站条目
    // 在 B 站慢路径比对的是 A 的槽，B 的真实读面（如 a+c vs a+b）永不复检
    // → 陈旧风险。
    slots.len().hash(&mut h);
    for s in slots {
        format!("{:?}", s).hash(&mut h);
    }
    let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();
    skeleton_children_registry(body, &mut h, registry, &mut visited);
    h.finish()
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
        AuraNode::Outlet { .. } => Err("outlet"),
        AuraNode::MemoBlock { .. } => Err("memo_block"),
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
    skeleton_fingerprint_with_components(props, children, None)
}

/// T-05b：Component 模板并入骨架（registry 模板结构进指纹；visited 破环）。
pub fn skeleton_fingerprint_with_components(
    props: &HashMap<String, crate::aura::AuraPropValue>,
    children: &[AuraNode],
    registry: Option<&crate::ui::widget_registry::WidgetRegistry>,
) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    skeleton_props(props, &mut h);
    let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();
    skeleton_children_registry(children, &mut h, registry, &mut visited);
    h.finish()
}

fn skeleton_children_registry(
    children: &[AuraNode],
    h: &mut std::collections::hash_map::DefaultHasher,
    registry: Option<&crate::ui::widget_registry::WidgetRegistry>,
    visited: &mut std::collections::HashSet<String>,
) {
    children.len().hash(h);
    for c in children {
        if let AuraNode::Component { name, props, children, .. } = c {
            "comp".hash(h);
            name.hash(h);
            props.len().hash(h);
            for (k, e) in props.iter() {
                k.hash(h);
                format!("{:?}", expr_shape(e)).hash(h);
            }
            skeleton_children_registry(children, h, registry, visited);
            if let Some(reg) = registry {
                if let Some(w) = reg.get(name) {
                    if visited.insert(name.clone()) {
                        skeleton_children_registry(
                            std::slice::from_ref(&w.view_tree),
                            h,
                            Some(reg),
                            visited,
                        );
                    }
                }
            }
            continue;
        }
        match c {
            AuraNode::Element { tag, props, children, .. } => {
                "el".hash(h);
                tag.hash(h);
                skeleton_props(props, h);
                skeleton_children_registry(children, h, registry, visited);
            }
            AuraNode::Text(AuraTextContent::Literal(s)) => {
                "txt".hash(h);
                s.hash(h);
            }
            _ => {
                "other".hash(h);
                std::mem::discriminant(c).hash(h);
            }
        }
    }
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

/// PLAN-046 T-04：显式 memo 块扫描（默认语义）。与 [`scan_for_item_body`]
/// 同构，差异：VM 代码形态（Call/FStr/Lambda/StyleBinding 等）**不入槽
/// 不降级**——块的 deps 声明即为其覆盖面（canonical 档：隐藏读面由 deps
/// 声明覆盖）；插值 bindings 逐名入槽（外部读的可靠上界）；条件串可解析
/// 入槽、不可解析容忍。降级仅限结构性不可证形态：嵌套 ForLoop / Outlet /
/// 嵌套 memo 块。
pub fn scan_memo_block_body(
    body: &[AuraNode],
    registry: Option<&crate::ui::widget_registry::WidgetRegistry>,
) -> ScanVerdict {
    let mut slots = Vec::new();
    let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();
    for c in body {
        if let Err(reason) = scan_block_node(c, &mut slots, registry, &mut visited) {
            return ScanVerdict::Degrade(reason);
        }
    }
    ScanVerdict::Slots(slots)
}

fn scan_block_node(
    node: &AuraNode,
    slots: &mut Vec<Expr>,
    registry: Option<&crate::ui::widget_registry::WidgetRegistry>,
    visited: &mut std::collections::HashSet<String>,
) -> Result<(), &'static str> {
    match node {
        AuraNode::Element { props, children, .. } => {
            for (_k, prop) in props.iter() {
                match prop {
                    // 容忍：可证形态入槽，VM 代码形态跳过（deps 覆盖）。
                    crate::aura::AuraPropValue::Expr(e) => {
                        let _ = scan_expr(e, slots);
                    }
                    crate::aura::AuraPropValue::StyleBinding(_) => {}
                }
            }
            for c in children {
                scan_block_node(c, slots, registry, visited)?;
            }
            Ok(())
        }
        AuraNode::Text(crate::aura::AuraTextContent::Literal(_)) => Ok(()),
        AuraNode::Text(crate::aura::AuraTextContent::Interpolated { bindings, .. }) => {
            // 块内插值名 = 状态/计算读——逐名入槽（resolve_expr_to_value 是
            // 实际读通道的可靠上界：多失效只变慢，绝不陈旧）。
            for b in bindings {
                slots.push(Expr::Ident(crate::ast::Name::from(
                    b.trim_start_matches('.'),
                )));
            }
            Ok(())
        }
        AuraNode::Conditional { condition, then_body, else_body, .. } => {
            if let Some(e) = crate::parser::Parser::parse_expr_fragment(condition) {
                let _ = scan_expr(&e, slots);
            }
            for c in then_body {
                scan_block_node(c, slots, registry, visited)?;
            }
            if let Some(els) = else_body {
                for c in els {
                    scan_block_node(c, slots, registry, visited)?;
                }
            }
            Ok(())
        }
        // 结构性不可证：嵌套循环/路由口/嵌套块 → 整块降级（Q-02 保守）。
        AuraNode::ForLoop { .. } => Err("for_loop"),
        AuraNode::Outlet { .. } => Err("outlet"),
        AuraNode::MemoBlock { .. } => Err("memo_block"),
        AuraNode::Component { name, props, children, .. } => {
            for (_k, e) in props.iter() {
                let _ = scan_expr(e, slots);
            }
            for c in children {
                scan_block_node(c, slots, registry, visited)?;
            }
            if let Some(reg) = registry {
                if let Some(w) = reg.get(name) {
                    if visited.insert(name.clone()) {
                        scan_block_node(&w.view_tree, slots, Some(reg), visited)?;
                    }
                }
            }
            Ok(())
        }
        AuraNode::Link { children, .. } => {
            for c in children {
                scan_block_node(c, slots, registry, visited)?;
            }
            Ok(())
        }
    }
}

/// PLAN-046 T-04：memo 块站点指纹——exact 旗 + deps 全形态 + 体结构。
/// 同一桥生命周期内块 AST 不可变 → 同站恒同；不同块由 deps/exact/体结构
/// 差异拆键。
pub fn memo_block_site_fingerprint(
    deps: &[Expr],
    exact: bool,
    body: &[AuraNode],
    registry: Option<&crate::ui::widget_registry::WidgetRegistry>,
) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    "memo_block_site".hash(&mut h);
    exact.hash(&mut h);
    deps.len().hash(&mut h);
    for d in deps {
        format!("{:?}", d).hash(&mut h);
    }
    let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();
    skeleton_children_registry(body, &mut h, registry, &mut visited);
    h.finish()
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

    // ─────────────────────────────────────────────────────────────────
    // PLAN-047 T-01: plan047_recorder_tests（Recorder 语义 4 条）
    // ─────────────────────────────────────────────────────────────────

    #[test]
    fn plan047_recorder_record_and_key_shapes() {
        let mut rec = RecState::default();
        rec.record(DepKey::field(7, "count"));
        rec.record(DepKey::any(9));
        assert_eq!(rec.deps.len(), 2);
        assert!(rec.deps.contains(&DepKey {
            heap_id: 7,
            path: "count".to_string()
        }));
        assert!(
            rec.deps.contains(&DepKey {
                heap_id: 9,
                path: DEP_PATH_ANY.to_string()
            }),
            "any() 记 DEP_PATH_ANY path"
        );
        assert!(!rec.overflow);
        // 重复录同键 = 幂等（BTreeSet 集语义）。
        rec.record(DepKey::field(7, "count"));
        assert_eq!(rec.deps.len(), 2);
    }

    #[test]
    fn plan047_recorder_budget_overflow_discards_whole_set() {
        let mut rec = RecState::default();
        for i in 0..REC_DEP_BUDGET {
            rec.record(DepKey::field(1, &format!("f{i}")));
        }
        assert!(!rec.overflow);
        assert_eq!(rec.deps.len(), REC_DEP_BUDGET);
        // 第 257 条 → overflow 置位且**整集弃置**（半录制集 = 盲区，绝不半信）。
        rec.record(DepKey::field(1, "one-too-many"));
        assert!(rec.overflow);
        assert!(rec.deps.is_empty());
        // overflow 后继续录不再恢复。
        rec.record(DepKey::field(1, "more"));
        assert!(rec.overflow && rec.deps.is_empty());
    }

    #[test]
    fn plan047_recorder_absorb_union_and_overflow_propagation() {
        let mut outer = RecState::default();
        outer.record(DepKey::field(1, "a"));
        let mut inner = RecState::default();
        inner.record(DepKey::field(2, "b"));
        inner.record(DepKey::field(1, "a")); // 与外层重叠 → 并集去重
        outer.absorb(&inner);
        assert_eq!(outer.deps.len(), 2);
        assert!(!outer.overflow);

        // overflow 传播：内层溢出 → 外层弃集置溢（外层集不完整 = 盲区）。
        let mut outer2 = RecState::default();
        outer2.record(DepKey::field(1, "x"));
        let mut inner2 = RecState::default();
        for i in 0..=REC_DEP_BUDGET {
            inner2.record(DepKey::field(3, &format!("g{i}")));
        }
        assert!(inner2.overflow);
        outer2.absorb(&inner2);
        assert!(outer2.overflow);
        assert!(outer2.deps.is_empty());
    }

    #[test]
    fn plan047_recorder_absorb_budget_boundary() {
        // 外层已有 256-1 条，吸收 2 条 → 越界弃集（并集也守预算）。
        let mut outer = RecState::default();
        for i in 0..REC_DEP_BUDGET - 1 {
            outer.record(DepKey::field(1, &format!("o{i}")));
        }
        let mut inner = RecState::default();
        inner.record(DepKey::field(2, "i1"));
        inner.record(DepKey::field(2, "i2"));
        outer.absorb(&inner);
        assert!(outer.overflow);
        assert!(outer.deps.is_empty());
    }

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
            key_expr: None,
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
            replay_relative: false,
            dyn_deps: None,
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
                item_key: None,
            };
            c.insert(key, mk_entry(0));
        }
        assert_eq!(c.len(), MEMO_CACHE_CAP, "LRU 上限逐出");
        assert_eq!(c.evictions, 8);
        let hit_key = MemoKey { ctx_state_obj: 1, site: 0, skeleton_fp: (MEMO_CACHE_CAP + 7) as u64, probe_on: false, item_key: None };
        assert!(c.lookup(&hit_key).is_some());
        c.note_hit();
        assert_eq!(c.hits, 1);
        let miss_key = MemoKey { ctx_state_obj: 1, site: 0, skeleton_fp: 999_999, probe_on: false, item_key: None };
        assert!(c.lookup(&miss_key).is_none());
        c.note_miss();
        assert_eq!(c.misses, 1);
        c.refresh_seq(&hit_key, 5);
        let e = c.lookup(&hit_key).expect("still present");
        assert_eq!(e.seq_at_fill, 5);
    }

    #[test]
    fn memo_cache_site_counts_and_capacity_raise() {
        // PLAN-046 §4/Q-03：per-site 分解计数 + 按需抬升容量（共池单 LRU）。
        let mut c = MemoCache::new();
        assert_eq!(c.len(), 0);
        c.ensure_capacity(1000);
        for i in 0..1000 {
            let key = MemoKey {
                ctx_state_obj: 1,
                site: MEMO_SITE_FOR_ITEM,
                skeleton_fp: 7,
                probe_on: false,
                item_key: Some(i),
            };
            c.insert(key, mk_entry(0));
        }
        assert_eq!(c.len(), 1000, "抬升后容量容纳千行列表（无逐出）");
        assert_eq!(c.evictions, 0);
        c.note_hit_site(MEMO_SITE_FOR_ITEM);
        c.note_miss_site(MEMO_SITE_FOR_ITEM);
        c.note_degraded_site(MEMO_SITE_FOR_ITEM);
        assert_eq!(c.hits, 1);
        assert_eq!(c.misses, 1);
        assert_eq!(c.degraded, 1);
        let sc = &c.site_counts[&MEMO_SITE_FOR_ITEM];
        assert_eq!((sc.hits, sc.misses, sc.degraded), (1, 1, 1));
        // 硬顶封口：超过 MAX 的抬升请求被截断（cap ≤ MAX，溢出走 LRU 逐出）。
        c.ensure_capacity(usize::MAX);
        for i in 0..(MEMO_CACHE_CAP_MAX + 8) {
            let key = MemoKey { ctx_state_obj: 2, site: 9, skeleton_fp: i as u64, probe_on: false, item_key: None };
            c.insert(key, mk_entry(0));
        }
        assert!(c.evictions > 0, "溢出硬顶走 LRU 逐出");
        assert!(c.len() <= MEMO_CACHE_CAP_MAX + 1000);
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

    // ── PLAN-046：keyed-for 项级扫描（scan_for_item_body）──

    fn item_elem(tag: &str, props: Vec<(&str, Expr)>) -> AuraNode {
        elem(tag, props, vec![])
    }

    fn loop_vars_of(names: &[&str]) -> std::collections::HashSet<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    /// 条件臂递归 + 条件串入槽：`if r.open { text (checked: .ftExpanded) }`
    /// → 条件解析槽 + prop 槽（filetree 形态的正确性前提）。
    #[test]
    fn scan_item_conditional_slots_and_recurses() {
        let body = vec![AuraNode::Conditional {
            condition: "r.open".to_string(),
            then_body: vec![item_elem(
                "text",
                vec![("checked", ident_dot("ftExpanded"))],
            )],
            else_body: None,
            span: None,
            debug_id: None,
        }];
        match scan_for_item_body(&body, None, &loop_vars_of(&["r"])) {
            ScanVerdict::Slots(slots) => assert_eq!(slots.len(), 2, "条件槽 + prop 槽"),
            ScanVerdict::Degrade(r) => panic!("条件臂应可证: {}", r),
        }
    }

    /// 条件串不可解析（VM 代码形态）→ 降级（宁缺勿错）。
    #[test]
    fn scan_item_unparsable_condition_degrades() {
        let body = vec![AuraNode::Conditional {
            condition: "heavy_call(.x)".to_string(),
            then_body: vec![],
            else_body: None,
            span: None,
            debug_id: None,
        }];
        // "heavy_call(.x)" 经 parse_expr_fragment 解析为 Call 形态 →
        // scan_expr 以 dynamic_expr 拒绝（解析通道接受全表达式语法，
        // 不可证形态统一由 scan_expr 分级拦截）。
        assert!(matches!(
            scan_for_item_body(&body, None, &loop_vars_of(&["r"])),
            ScanVerdict::Degrade("dynamic_expr")
        ));
    }

    /// 插值 bindings 全为循环变量 → 项值覆盖（安全）；含外部名 → 降级。
    #[test]
    fn scan_item_interpolation_external_name_degrades() {
        let loop_only = AuraNode::Text(AuraTextContent::Interpolated {
            template: "row ${r.id}".to_string(),
            bindings: vec!["r".to_string()],
        });
        assert!(matches!(
            scan_for_item_body(&[loop_only], None, &loop_vars_of(&["r"])),
            ScanVerdict::Slots(_)
        ));
        let external = AuraNode::Text(AuraTextContent::Interpolated {
            template: "n ${.count}".to_string(),
            bindings: vec!["count".to_string()],
        });
        assert!(matches!(
            scan_for_item_body(&[external], None, &loop_vars_of(&["r"])),
            ScanVerdict::Degrade("interpolated_text_external")
        ));
    }

    /// 嵌套 ForLoop（T-06 扩展）：iterable 根 ∈ 循环变量 → 可证（递归）；
    /// 根为外部状态 → 降级。Outlet 在项体内 → 降级。
    #[test]
    fn scan_item_nested_for_and_outlet_degrade() {
        // `for g in r.guides`：根 r ∈ 循环变量 → 项值覆盖，Slots。
        let nested = AuraNode::ForLoop {
            var: "g".to_string(),
            index: None,
            iterable: "r.guides".to_string(),
            key_expr: None,
            body: vec![],
            span: None,
            debug_id: None,
        };
        assert!(matches!(
            scan_for_item_body(&[nested], None, &loop_vars_of(&["r"])),
            ScanVerdict::Slots(_)
        ));
        // `for g in .external`：状态根 → 降级。
        let external = AuraNode::ForLoop {
            var: "g".to_string(),
            index: None,
            iterable: ".external".to_string(),
            key_expr: None,
            body: vec![],
            span: None,
            debug_id: None,
        };
        assert!(matches!(
            scan_for_item_body(&[external], None, &loop_vars_of(&["r"])),
            ScanVerdict::Degrade("for_loop")
        ));
        assert!(matches!(
            scan_for_item_body(&[AuraNode::Outlet { memo: false }], None, &loop_vars_of(&["r"])),
            ScanVerdict::Degrade("outlet")
        ));
    }

    /// 站点指纹：同站恒同；头差异（key 形态/iterable/var）或体结构差异拆键。
    #[test]
    fn for_site_fingerprint_distinguishes_sites() {
        let key_a = Expr::Dot(Box::new(Expr::Ident(AutoStr::from("r"))), AutoStr::from("id"));
        let key_b = Expr::Dot(Box::new(Expr::Ident(AutoStr::from("r"))), AutoStr::from("name"));
        let body = vec![item_elem("text", vec![("value", ident_dot("x"))])];
        let slots_a = match scan_for_item_body(&body, None, &loop_vars_of(&["r"])) {
            ScanVerdict::Slots(s) => s,
            ScanVerdict::Degrade(r) => panic!("扫描应通过: {}", r),
        };
        let f1 = for_site_fingerprint("r", None, ".rows", &key_a, &body, None, &slots_a);
        let f2 = for_site_fingerprint("r", None, ".rows", &key_a, &body, None, &slots_a);
        assert_eq!(f1, f2, "同站恒同");
        assert_ne!(
            f1,
            for_site_fingerprint("r", None, ".rows", &key_b, &body, None, &slots_a),
            "key 形态差异拆键"
        );
        assert_ne!(
            f1,
            for_site_fingerprint("r", None, ".other", &key_a, &body, None, &slots_a),
            "iterable 差异拆键"
        );
        // 体读目标差异（x vs y，同为 Dot 判别式）→ 槽形态差异拆键。
        let body2 = vec![item_elem("text", vec![("value", ident_dot("y"))])];
        let slots_b = match scan_for_item_body(&body2, None, &loop_vars_of(&["r"])) {
            ScanVerdict::Slots(s) => s,
            ScanVerdict::Degrade(r) => panic!("扫描应通过: {}", r),
        };
        assert_ne!(
            f1,
            for_site_fingerprint("r", None, ".rows", &key_a, &body2, None, &slots_b),
            "体读槽差异拆键"
        );
    }
}
