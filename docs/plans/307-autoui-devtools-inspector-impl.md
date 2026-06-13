# AutoUI DevTools 风格实时检视器 — 实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 把 VM 版 AutoUI 的 F12 检视从"源码视图"重构为 DevTools 风格的"computed DOM 视图"——VTree 成为 iced 路径的结构真相源，F12 门控的 `InspectorCache` 承载实时布局、解析样式、状态绑定、for 溯源、事件、源码定位。

**Architecture:** 方案 B（见设计 `307-autoui-devtools-inspector-design.md`）。VTree 每帧构建（精简），贵数据进 `InspectorCache`（仅 F12 on 时存在）。bounds 由现有 `LayoutCollector` 收集后经 `VNodeId ↔ aura_N` 映射回填。VNodeId 由 `path` 派生以让选中态跨帧存活。最后清理 `DebugLayer` / `DebugTreeNode` 两套并行实现。

**Tech Stack:** Rust；iced（UI 后端）；现有 `vnode.rs` / `view.rs` / `aura_view_builder.rs` / `iced/renderer.rs` / `iced/layout_collector.rs` / `style/`。

**测试约定：** 纯逻辑任务（VNode 字段、converter、cache、probe、style 解析）走 TDD 单元测试。iced 渲染器/UI 面板集成任务用"编译通过 + headless 路径集成测试 + 手动验收清单"，因为 iced widget 难以纯单元测试。每个任务结束 `cargo build -p auto`（CLAUDE.md 要求）后提交。

**关键参考文件：**
- 设计：`docs/plans/307-autoui-devtools-inspector-design.md`
- 现有结构：`crates/auto-lang/src/ui/vnode.rs`、`vnode_converter.rs`、`view.rs`、`debug/mod.rs`、`iced/renderer.rs`、`iced/layout_collector.rs`、`style/{mod,class,layout_extract}.rs`、`aura_view_builder.rs`、`debug_id_map.rs`

**清理顺序铁律：** 先让新通路（VTree 树 + InspectorCache 面板）跑通，再删旧通路（DebugLayer / DebugTreeNode），避免中间态断裂。

---

## Phase 0 — 基础类型（纯逻辑，无集成）

### Task 1: 给 VNode 增加 `path` 与 `source_span` 字段

**Files:**
- Modify: `crates/auto-lang/src/ui/vnode.rs`
- Create: `crates/auto-lang/src/ui/debug/source_span.rs`（或在 `debug/mod.rs` 内定义）

**Step 1: 定义 `SourceSpan`**

在 `debug/mod.rs`（或新 `source_span.rs` 并在 `debug/mod.rs` re-export）：

```rust
/// 源码 span（字节偏移区间），用于检视器定位。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SourceSpan {
    pub offset: usize,
    pub len: usize,
}
```

**Step 2: 写失败测试**

在 `vnode.rs` 的 `#[cfg(test)] mod tests` 末尾加：

```rust
#[test]
fn vnode_has_path_and_source_span() {
    let id = VNodeId::new(1);
    let mut node = VNode::new(id, VNodeKind::Text, VNodeProps::Text { content: "x".into() });
    assert!(node.path.is_empty());
    assert!(node.source_span.is_none());

    node.path = vec![0, 1, 2];
    node.source_span = Some(crate::ui::debug::SourceSpan { offset: 10, len: 3 });
    assert_eq!(node.path, vec![0, 1, 2]);
}
```

**Step 3: 加字段，跑测试**

在 `VNode` 结构体（`vnode.rs:243`）加两个字段，并在 `VNode::new`（`vnode.rs:265`）初始化为默认：

```rust
pub struct VNode {
    pub id: VNodeId,
    pub kind: VNodeKind,
    pub parent: Option<VNodeId>,
    pub children: Vec<VNodeId>,
    pub props: VNodeProps,
    pub label: String,
    /// 稳定的逻辑路径（从根到本节点的子索引序列）
    pub path: Vec<u16>,
    /// 源码 span（来自 DebugIdMap → AuraNodeId → span_map）
    pub source_span: Option<crate::ui::debug::SourceSpan>,
}
```

`VNode::new` 内 `path: Vec::new(), source_span: None`。可选加 builder：`.with_path(p)` / `.with_source_span(s)`。

Run: `cargo test -p auto-lang -- vnode_has_path_and_source_span`
Expected: PASS。然后 `cargo build -p auto`（新字段可能让其它构造点编译失败，逐一补默认值）。

**Step 4: Commit**

```bash
git add crates/auto-lang/src/ui/vnode.rs crates/auto-lang/src/ui/debug/mod.rs
git commit -m "feat(ui): add path and source_span fields to VNode"
```

---

### Task 2: `path` → 稳定 `VNodeId` 派生函数

**Files:**
- Modify: `crates/auto-lang/src/ui/vnode.rs`

**Step 1: 写失败测试**

```rust
#[test]
fn vnode_id_from_path_is_stable() {
    use crate::ui::vnode::id_from_path;
    // 同一 path → 同一 id
    assert_eq!(id_from_path(&[0, 1, 2]), id_from_path(&[0, 1, 2]));
    // 不同 path → 不同 id
    assert_ne!(id_from_path(&[0, 1, 2]), id_from_path(&[0, 1, 3]));
    // 空 path（根）→ 固定 id
    assert_eq!(id_from_path(&[]), id_from_path(&[]));
    // id 不为 0（避免与默认冲突）
    assert_ne!(id_from_path(&[]), 0);
}

#[test]
fn vnode_id_from_path_distinguishes_for_iterations() {
    use crate::ui::vnode::id_from_path;
    // for 展开第 0、1、2 项应有不同 id
    let a = id_from_path(&[0, 0]); let b = id_from_path(&[0, 1]); let c = id_from_path(&[0, 2]);
    assert_ne!(a, b); assert_ne!(b, c); assert_ne!(a, c);
}
```

**Step 2: 实现 `id_from_path`**

在 `vnode.rs` 加（FNV-1a 风格，确定性、无 `Math::random`）：

```rust
/// 由逻辑路径派生稳定的 VNodeId 数值。
/// 结构不变 → id 不变；for 不同迭代 path 不同 → id 不同。
pub fn id_from_path(path: &[u16]) -> u64 {
    // 非零种子，避免与默认/0 冲突
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &seg in path {
        h ^= seg as u64;
        h = h.wrapping_mul(0x1000_0000_01b3);
    }
    if h == 0 { h = 1; } // 保证非 0
    h
}
```

加便捷构造：`impl VTree { pub fn id_for_path(&self, path: &[u16]) -> VNodeId { VNodeId(id_from_path(path)) } }`。

Run: `cargo test -p auto-lang -- vnode_id_from_path`
Expected: PASS。

**Step 3: Commit**

```bash
git add crates/auto-lang/src/ui/vnode.rs
git commit -m "feat(ui): derive stable VNodeId from node path"
```

---

### Task 3: 迁移 `Rect` / `BoxModel` / `EdgeInsets` 到可复用模块

**Files:**
- Create: `crates/auto-lang/src/ui/debug/primitives.rs`
- Modify: `crates/auto-lang/src/ui/debug/mod.rs`（re-export，移除旧定义）

**Step 1:** 把 `debug/mod.rs` 中 `Rect`、`EdgeInsets`、`BoxModel` 及其 impl/测试 整体迁到新 `primitives.rs`。`debug/mod.rs` 改为 `mod primitives; pub use primitives::{Rect, EdgeInsets, BoxModel};`。

**Step 2:** `cargo test -p auto-lang -- debug::` 确保迁移后既有 box model 测试仍通过。

**Step 3: Commit**

```bash
git add crates/auto-lang/src/ui/debug/
git commit -m "refactor(ui): move Rect/BoxModel/EdgeInsets to debug/primitives"
```

---

## Phase 1 — VTree 在 iced 路径每帧构建（结构层）

### Task 4: 带路径与 span 的新 converter `view_to_vtree_with_paths`

**Files:**
- Modify: `crates/auto-lang/src/ui/vnode_converter.rs`

**Step 1: 写失败测试**

```rust
#[test]
fn vtree_assigns_path_derived_ids_and_spans() {
    use super::view_to_vtree_with_paths;
    // 构造 root col -> [text, row -> [button]]
    let view = View::<u32>::Column {
        children: vec![
            View::Text { content: "a".into(), style: None },
            View::Row { children: vec![
                View::Button { label: "b".into(), onclick: 0, style: None },
            ], spacing: 0, padding: 0, style: None },
        ],
        spacing: 0, padding: 0, style: None,
    };
    let tree = view_to_vtree_with_paths(view, |path: &[u16]| {
        // span 来源回调：root、[0]、[1]、[1,0]
        Some(crate::ui::debug::SourceSpan { offset: path.iter().map(|&x| x as usize).sum::<usize>(), len: 1 })
    });
    // 结构不变 → id 由 path 派生
    let root = tree.root().unwrap();
    assert_eq!(root.id.as_u64(), crate::ui::vnode::id_from_path(&[]));
    assert_eq!(root.path, vec![]);
    // 两次构建同结构 → id 一致（跨帧稳定）
    let again = view_to_vtree_with_paths(clone_view(), |_| None);
    assert_eq!(again.root().unwrap().id, root.id);
}
```
（`clone_view`/构造细节按实际 `View` 字段调整；关键断言是 id 由 path 派生且可重现。）

**Step 2: 实现 `view_to_vtree_with_paths`**

签名：

```rust
pub fn view_to_vtree_with_paths<M, F>(view: View<M>, span_for: F) -> VTree
where M: Clone + std::fmt::Debug, F: Fn(&[u16]) -> Option<SourceSpan>,
```

实现要点（对照现有 `convert_view_to_vnode`，`vnode_converter.rs:108`）：
- 递归时维护当前 `path: Vec<u16>`。
- 每个节点 `id = VTree::id_for_path(&path)`（用 Task 2）。
- `vnode.path = path.clone()`；`vnode.source_span = span_for(&path)`。
- 子节点 `path.push(child_index as u16)`，递归后 `pop`。
- 注意：VTree 内部 `next_id`/`set_root`/`add_node` 需容忍 id 由外部给定（当前实现靠 push，无 id 冲突检查，可继续用 push；但 `id_for_path` 已确定 id 值，构造 VNode 时直接用）。

Run: `cargo test -p auto-lang -- vtree_assigns_path_derived`
Expected: PASS。

**Step 3: Commit**

```bash
git add crates/auto-lang/src/ui/vnode_converter.rs
git commit -m "feat(ui): view_to_vtree_with_paths assigns path-derived ids + spans"
```

---

### Task 5: iced 渲染器每帧构建 VTree 并挂到 DynamicState

**Files:**
- Modify: `crates/auto-lang/src/ui/iced/renderer.rs`（DynamicState 定义区，~`renderer.rs:1900-2100`；每帧 view 构建处）

**Step 1: 定位点**

- `DynamicState` 结构体（grep `struct DynamicState`）加字段：
  `live_vtree: std::cell::RefCell<Option<crate::ui::vnode::VTree>>`
- 找到每帧构建 `View` 并调用渲染的位置（grep `render_dynamic_view` / `into_iced` / `wrap_debug` 调用链起点）。在该处，渲染前调用 `view_to_vtree_with_paths(&view, span_resolver)`，把结果写入 `state.live_vtree`。
- `span_resolver` 闭包：由 `DebugIdMap` + `span_map` 提供。`view_to_vtree_with_paths` 的 `span_for(path)` 内部：`debug_id_map.get(path)` → `AuraNodeId` → `span_map.get(&aura_id).span` → `SourceSpan`。（对照 `wrap_debug` 现有 `aura_id = self.debug_id_map.get(view_path)` 逻辑，`renderer.rs:3731`。）

**Step 2: 编译 + 集成测试（headless 路径不走这里，仅保证编译）**

Run: `cargo build -p auto`
Expected: 编译通过。`live_vtree` 此时只写入、未被读取（下个 Task 才用）。

**Step 3: Commit**

```bash
git add crates/auto-lang/src/ui/iced/renderer.rs
git commit -m "feat(ui): build live VTree per frame in iced renderer"
```

---

## Phase 2 — InspectorCache 与 ComputedNode

### Task 6: 定义 InspectorCache / ComputedNode 及子结构

**Files:**
- Create: `crates/auto-lang/src/ui/debug/inspector_cache.rs`
- Modify: `crates/auto-lang/src/ui/debug/mod.rs`（`mod inspector_cache; pub use inspector_cache::*;`）

**Step 1: 写失败测试**

```rust
#[test]
fn computed_node_default_renders_without_panic() {
    let cn = ComputedNode::default();
    assert!(cn.bounds.is_none());
    assert!(cn.computed_style.is_empty());
    assert!(cn.state_bindings.is_empty());
    assert!(cn.for_context.is_none());
    // 右栏不变量：即便全空也能产出渲染数据
    let summary = cn.summary("Button", &[0,1,2]);
    assert!(summary.contains("Button"));
}

#[test]
fn inspector_cache_round_trip_id_map() {
    let mut cache = InspectorCache::new();
    cache.set_iced_map(VNodeId::new(7), "aura_3_42".into());
    assert_eq!(cache.iced_to_vnode("aura_3_42"), Some(VNodeId::new(7)));
    assert_eq!(cache.vnode_to_iced(VNodeId::new(7)), Some("aura_3_42"));
}
```

**Step 2: 实现结构体**

```rust
use std::collections::HashMap;
use crate::ui::vnode::VNodeId;
use super::primitives::{Rect, BoxModel};
use super::SourceSpan;

#[derive(Debug, Clone, Default)]
pub struct StateBinding { pub expr: String, pub current_value: String }

#[derive(Debug, Clone, Default)]
pub struct ForIter { pub var: String, pub index: Option<usize>, pub value_repr: String, pub iterable_repr: String }

#[derive(Debug, Clone, Default)]
pub struct EventHandlerInfo { pub event: String, pub handler: String }

#[derive(Debug, Clone, Default)]
pub struct ComputedNode {
    pub bounds: Option<Rect>,
    pub box_model: Option<BoxModel>,
    pub computed_style: Vec<(String, String)>,
    pub raw_class: Option<String>,
    pub state_bindings: Vec<StateBinding>,
    pub for_context: Option<ForIter>,
    pub events: Vec<EventHandlerInfo>,
    pub source: Option<String>, // "app.at:42"
}

impl ComputedNode {
    /// 右栏渲染摘要（不变量：永不 panic）。
    pub fn summary(&self, kind: &str, path: &[u16]) -> String {
        let mut s = format!("{} {:?}", kind, path);
        if let Some(b) = &self.bounds {
            s.push_str(&format!(" @ {:.0},{:.0} {:.0}×{:.0}", b.x, b.y, b.width, b.height));
        }
        s
    }
}

#[derive(Debug, Clone, Default)]
pub struct InspectorCache {
    by_id: HashMap<VNodeId, ComputedNode>,
    id_to_iced: HashMap<VNodeId, String>,
    iced_to_id: HashMap<String, VNodeId>,
}

impl InspectorCache {
    pub fn new() -> Self { Self::default() }
    pub fn get(&self, id: VNodeId) -> Option<&ComputedNode> { self.by_id.get(&id) }
    pub fn get_mut_or_default(&mut self, id: VNodeId) -> &mut ComputedNode {
        self.by_id.entry(id).or_default()
    }
    pub fn set_iced_map(&mut self, id: VNodeId, iced_id: String) {
        if let Some(old) = self.id_to_iced.insert(id, iced_id.clone()) { self.iced_to_id.remove(&old); }
        self.iced_to_id.insert(iced_id, id);
    }
    pub fn vnode_to_iced(&self, id: VNodeId) -> Option<&String> { self.id_to_iced.get(&id) }
    pub fn iced_to_vnode(&self, iced_id: &str) -> Option<VNodeId> { self.iced_to_id.get(iced_id).copied() }
    pub fn clear(&mut self) { self.by_id.clear(); self.id_to_iced.clear(); self.iced_to_id.clear(); }
    pub fn ids(&self) -> impl Iterator<Item = VNodeId> + '_ { self.by_id.keys().copied() }
}
```

Run: `cargo test -p auto-lang -- computed_node_default_renders inspector_cache_round_trip`
Expected: PASS。

**Step 3: Commit**

```bash
git add crates/auto-lang/src/ui/debug/inspector_cache.rs crates/auto-lang/src/ui/debug/mod.rs
git commit -m "feat(ui): add InspectorCache + ComputedNode data model"
```

---

### Task 7: `computed_style` 解析（StyleClass → (prop,val)）

**Files:**
- Create: `crates/auto-lang/src/ui/debug/style_probe.rs`
- Modify: `crates/auto-lang/src/ui/debug/mod.rs`（re-export）

**Step 1: 写失败测试**

```rust
#[test]
fn style_probe_parses_class_string() {
    let pairs = crate::ui::debug::compute_style("w-full p-4 bg-blue-500");
    let m: std::collections::HashMap<&str, &str> = pairs.iter().map(|(k,v)| (k.as_str(), v.as_str())).collect();
    assert_eq!(m.get("width"), Some(&"100%"));
    assert!(m.contains_key("padding"));      // p-4 → padding
    assert!(m.contains_key("background-color")); // bg-blue-500
}

#[test]
fn style_probe_invalid_class_returns_empty() {
    assert!(crate::ui::debug::compute_style("__bogus__zzz").is_empty());
}
```

**Step 2: 实现 `compute_style`**

用现有 `Style::parse`（`style/mod.rs`）+ `StyleClass` 映射：

```rust
use crate::ui::style::{Style, StyleClass, SizeValue, Color};

pub fn compute_style(class: &str) -> Vec<(String, String)> {
    let Ok(style) = Style::parse(class) else { return Vec::new(); };
    style.classes.iter().filter_map(class_to_kv).collect()
}

fn class_to_kv(c: &StyleClass) -> Option<(String, String)> {
    use StyleClass::*;
    Some(match c {
        Padding(v) => ("padding", size(v)),
        Width(v) => ("width", size(v)),
        Height(v) => ("height", size(v)),
        Gap(v) => ("gap", size(v)),
        BackgroundColor(c) => ("background-color", color(c)),
        TextColor(c) => ("color", color(c)),
        BorderWidth(w) => ("border-width", format!("{w}")),
        BorderRadius(r) => ("border-radius", format!("{r}")),
        Opacity(o) => ("opacity", format!("{o}")),
        // 其余 variant 按需补充；未识别返回 None（不影响其它）
        _ => return None,
    }.map(|(k,v)| (k.to_string(), v)))
}

fn size(v: &SizeValue) -> String { /* Fixed(n)=>n*4px, Pixels(p)=>p px, Fill=>"100%" 等，按 SizeValue variant */ todo!() }
fn color(c: &Color) -> String { /* 转 #rrggbb */ todo!() }
```
（`size`/`color` 按 `style/class.rs` 的 `SizeValue` 与 `style/color.rs` 的 `Color` 真实 variant 补全；先覆盖测试用到的 w-full/p-4/bg-blue-500。）

Run: `cargo test -p auto-lang -- style_probe_`
Expected: PASS。

**Step 3: Commit**

```bash
git add crates/auto-lang/src/ui/debug/style_probe.rs crates/auto-lang/src/ui/debug/mod.rs
git commit -m "feat(ui): compute_style parses class string into property pairs"
```

---

### Task 8: `BuildProbe` 采集容器（按 path 索引）

**Files:**
- Create: `crates/auto-lang/src/ui/debug/build_probe.rs`
- Modify: `crates/auto-lang/src/ui/debug/mod.rs`

**Step 1: 写失败测试**

```rust
#[test]
fn build_probe_collects_by_path() {
    let mut probe = BuildProbe::new();
    probe.record_state(&[0,0], "${.count}", "3");
    probe.record_for(&[1,2], ForIter { var: "item".into(), index: Some(2), value_repr: "apple".into(), iterable_repr: "items".into() });
    probe.record_event(&[0,0], "onclick", "handle_click");
    let snap = probe.snapshot();
    let n = snap.get(&[0,0]).unwrap();
    assert_eq!(n.state_bindings[0].expr, "${.count}");
    assert_eq!(n.state_bindings[0].current_value, "3");
    assert_eq!(n.events[0].handler, "handle_click");
    let m = snap.get(&[1,2]).unwrap();
    assert_eq!(m.for_context.as_ref().unwrap().value_repr, "apple");
}
```

**Step 2: 实现 `BuildProbe`**

```rust
use std::collections::HashMap;
use super::inspector_cache::{StateBinding, ForIter, EventHandlerInfo};

#[derive(Debug, Default, Clone)]
pub struct ProbeEntry {
    pub state_bindings: Vec<StateBinding>,
    pub for_context: Option<ForIter>,
    pub events: Vec<EventHandlerInfo>,
}

#[derive(Debug, Default, Clone)]
pub struct BuildProbe {
    by_path: HashMap<Vec<u16>, ProbeEntry>,
}
impl BuildProbe {
    pub fn new() -> Self { Self::default() }
    fn entry(&mut self, path: &[u16]) -> &mut ProbeEntry { self.by_path.entry(path.to_vec()).or_default() }
    pub fn record_state(&mut self, p: &[u16], expr: impl Into<String>, val: impl Into<String>) {
        self.entry(p).state_bindings.push(StateBinding { expr: expr.into(), current_value: val.into() });
    }
    pub fn record_for(&mut self, p: &[u16], ctx: ForIter) { self.entry(p).for_context = Some(ctx); }
    pub fn record_event(&mut self, p: &[u16], event: impl Into<String>, handler: impl Into<String>) {
        self.entry(p).events.push(EventHandlerInfo { event: event.into(), handler: handler.into() });
    }
    pub fn snapshot(&self) -> &HashMap<Vec<u16>, ProbeEntry> { &self.by_path }
    pub fn clear(&mut self) { self.by_path.clear(); }
}
```

Run: `cargo test -p auto-lang -- build_probe_collects`
Expected: PASS。

**Step 3: Commit**

```bash
git add crates/auto-lang/src/ui/debug/build_probe.rs crates/auto-lang/src/ui/debug/mod.rs
git commit -m "feat(ui): add BuildProbe to collect debug data by path"
```

---

## Phase 3 — BuildProbe 接入 AuraViewBuilder

> 本阶段把"构建时即可知"的 AutoUI 特有数据（状态绑定、for 上下文、事件）采集进 `BuildProbe`。集成点在 `aura_view_builder.rs`。由于该文件尚未通读，每任务以"定位 grep + 接入点 + 验证"形式给出；执行者需读对应函数后插入采集调用。

### Task 9: 采集状态绑定 `${.x}`

**Files:**
- Modify: `crates/auto-lang/src/ui/aura_view_builder.rs`

**Step 1: 定位** —— grep `read_state` / `\$\{` / `interpolat` 找到解析 `${.field}` 并读取状态值的位置。

**Step 2: 接入** —— 在解析到 `${.field}` 绑定并取到当前值处，调用（仅 `debug_mode`/probe 存在时）：
```rust
if let Some(probe) = probe_opt {
    probe.record_state(current_path, format!("${{{}}}", expr), resolved_value.to_string());
}
```
需把 `current_path: &[u16]` 与 `probe: Option<&BuildProbe>` 作为参数贯穿 `build` 递归。

**Step 3: 验证** —— 新增 headless 集成测试：构造含 `${.count}` 的 widget，构建后断言 `probe.snapshot()` 中对应 path 有 `state_bindings`。

Run: `cargo test -p auto-lang -- probe_state`（新测试）+ `cargo build -p auto`
Expected: PASS / 编译通过。

**Step 4: Commit**

```bash
git add crates/auto-lang/src/ui/aura_view_builder.rs
git commit -m "feat(ui): BuildProbe captures state bindings in AuraViewBuilder"
```

---

### Task 10: 采集 for 循环迭代上下文

**Files:**
- Modify: `crates/auto-lang/src/ui/aura_view_builder.rs`（for 展开处理处，对应 `AuraNode::ForLoop`，见 `aura/types.rs`）

**Step 1: 定位** —— grep `ForLoop` / `iterable` 找到 for 展开、为每次迭代生成子节点的循环。

**Step 2: 接入** —— 在循环体每次迭代内，记录该迭代根节点的 `for_context`：
```rust
probe.record_for(iter_path, ForIter {
    var: var_name.into(),
    index: Some(i),
    value_repr: format!("{:?}", item_value),
    iterable_repr: iterable_expr.into(),
});
```

**Step 3: 验证** —— headless 测试：`for item in [a,b,c]` 展开后，probe 中三个迭代 path 各有 `for_context`，index=0/1/2。

Run: `cargo test -p auto-lang -- probe_for` + `cargo build -p auto`

**Step 4: Commit**

```bash
git add crates/auto-lang/src/ui/aura_view_builder.rs
git commit -m "feat(ui): BuildProbe captures for-loop iteration context"
```

---

### Task 11: 采集事件处理器绑定

**Files:**
- Modify: `crates/auto-lang/src/ui/aura_view_builder.rs`（事件绑定处，`AuraNode::Element.events` / `AuraEvent`）

**Step 1: 定位** —— grep `events` / `AuraEvent` / `DynamicMessage` 找到把节点事件绑定到 handler 的位置。

**Step 2: 接入** —— 对每个绑定的 event 记录：`probe.record_event(path, event_name, handler_fn_name)`。

**Step 3: 验证** —— headless 测试：带 `onclick="handle_click"` 的按钮构建后，probe 对应 path 有 `events=[{onclick, handle_click}]`。

Run: `cargo test -p auto-lang -- probe_event` + `cargo build -p auto`

**Step 4: Commit**

```bash
git add crates/auto-lang/src/ui/aura_view_builder.rs
git commit -m "feat(ui): BuildProbe captures event handler bindings"
```

---

## Phase 4 — Bounds 回填

### Task 12: iced 渲染器维护 `VNodeId ↔ aura_N` 映射

**Files:**
- Modify: `crates/auto-lang/src/ui/iced/renderer.rs`

**Step 1: 背景** —— `wrap_debug`（`renderer.rs:3726`）已用 `debug_id_map.get(view_path)` 得 `AuraNodeId`，并生成 `id_str = format!("aura_{}_{}", base_id, counter_val)`。VTree 节点同样按 path 派生 VNodeId。

**Step 2: 接入** —— 渲染每个被 wrap 的节点时，把 `VNodeId(id_from_path(view_path)) ↔ id_str` 写入 `InspectorCache::set_iced_map`（仅 `debug_mode`）。需要让渲染循环能拿到当前 `view_path`（`wrap_debug` 已有 `view_path: &[usize]`，转 `Vec<u16>` 即可）。

**Step 3: 验证** —— 单元测试：模拟 set_iced_map 后，`iced_to_vnode`/`vnode_to_iced` 双向正确（已在 Task 6 覆盖）；此处保证渲染器调用点编译通过。

Run: `cargo build -p auto`

**Step 4: Commit**

```bash
git add crates/auto-lang/src/ui/iced/renderer.rs
git commit -m "feat(ui): maintain VNodeId<->aura_N id map in renderer"
```

---

### Task 13: 布局后 bounds 回填 InspectorCache + box_model

**Files:**
- Modify: `crates/auto-lang/src/ui/iced/renderer.rs`（LayoutCollector 结果消费处，grep `LayoutCollector` / `BoundsMap`）

**Step 1: 定位** —— `LayoutCollector` 产出 `BoundsMap = HashMap<String, (f32,f32,f32,f32)>`（`layout_collector.rs:16`）。找到该结果被消费的位置（当前用于 MCP snapshot）。

**Step 2: 回填** —— 在 `debug_mode` 下，对每条 `(iced_id_str, (x,y,w,h))`：
```rust
if let Some(cache) = inspector_cache_opt.as_mut() {
    if let Some(vnid) = cache.iced_to_vnode(&iced_id_str) {
        let node = cache.get_mut_or_default(vnid);
        node.bounds = Some(Rect::new(x,y,w,h));
        // box_model: padding 取声明值（从该节点 raw_class 解析），content = bounds - padding
        let pad = parse_padding(node.raw_class.as_deref()); // 复用 compute_style/compute_box_layout
        node.box_model = Some(BoxModel::new(
            Rect::new(x + pad.left, y + pad.top, (w - pad.left - pad.right).max(0.0), (h - pad.top - pad.bottom).max(0.0)),
            pad, EdgeInsets::default(),
        ));
    }
}
```

**Step 3: 验证** —— 单元测试 `inspector_cache_bounds_backfill`：构造 cache + id_map + bounds map → 回填后 `get(id).bounds` 与 box_model.content 正确（content = bounds − padding）。

Run: `cargo test -p auto-lang -- inspector_cache_bounds_backfill` + `cargo build -p auto`

**Step 4: Commit**

```bash
git add crates/auto-lang/src/ui/iced/renderer.rs
git commit -m "feat(ui): backfill bounds + box_model into InspectorCache post-layout"
```

---

## Phase 5 — 面板（左树 + 右栏）

> 面板代码集中在 `iced/renderer.rs` 的 debug UI 渲染区（grep `render_inspector_tab`、`render_tree_into`、`component_tree`）。本阶段把数据源从旧的 `DebugTreeNode`/`element_styles` 切到 VTree + InspectorCache。

### Task 14: 左树改读 VTree

**Files:**
- Modify: `crates/auto-lang/src/ui/iced/renderer.rs`（左树渲染函数，~`renderer.rs:3114` `render_tree_into`）

**Step 1:** 新写/改写左树渲染：从 `state.live_vtree` 递归渲染。每行 `kind` + 摘要（text 取 props.content 前 N 字符；list/col 显示子节点数）。展开/折叠状态可先用"全展开"。

**Step 2:** 点击行 → 选中 `VNodeId` → 写入 `state.selected_vnode`（新字段，替代旧 `selected_widget: String`）。hover 行 → 写入 `state.hovered_vnode`。

**Step 3:** 编译 + 手动验收（左树出现、结构匹配可见 UI）。

Run: `cargo build -p auto`，手动运行 F12。

**Step 4: Commit**

```bash
git add crates/auto-lang/src/ui/iced/renderer.rs
git commit -m "feat(ui): left tree reads from live VTree"
```

---

### Task 15: 右栏面包屑 + Layout 标签

**Files:**
- Modify: `crates/auto-lang/src/ui/iced/renderer.rs`（`render_inspector_tab`，~`renderer.rs:3284`）

**Step 1:** 顶部面包屑：从选中 VNodeId 沿 `vnode.parent` 上溯到根，渲染可点击 chip，点祖先 → 设 `selected_vnode`。

**Step 2:** Layout 标签：读 `cache.get(selected).box_model`，画 Box Model 嵌套矩形 + 数值；padding 行标注"(声明值)"。bounds 缺失时显示"(布局中…)"。

**Step 3:** 编译 + 手动验收。

Run: `cargo build -p auto`

**Step 4: Commit**

```bash
git add crates/auto-lang/src/ui/iced/renderer.rs
git commit -m "feat(ui): breadcrumb + Layout (box model) tab"
```

---

### Task 16: 右栏 Computed / Props / AutoUI / Source 标签

**Files:**
- Modify: `crates/auto-lang/src/ui/iced/renderer.rs`

**Step 1:**
- **Computed**：`cache.get(id).computed_style` 键值对列表。
- **Props**：`raw_class` + VNode 原始 props（content/value/checked…）。
- **AutoUI**：`state_bindings`（expr = current_value，`<unresolved>` 降级）、`for_context`（`var=value, i=N`）、`events`（`event → handler`）。
- **Source**：单行可点击 `app.at:42`（来自 `cache.get(id).source` 或 VNode `source_span` → span_map 反查文件/行）。缺失显示"(无源码定位)"。

**Step 2:** 标签切换状态（`state.inspector_tab`）。

**Step 3:** 降级不变量测试：构造全空 `ComputedNode` → 渲染函数返回有效 Element、不 panic。

Run: `cargo test -p auto-lang -- computed_node_default_renders`（复用）+ `cargo build -p auto`

**Step 4: Commit**

```bash
git add crates/auto-lang/src/ui/iced/renderer.rs
git commit -m "feat(ui): Computed/Props/AutoUI/Source inspector tabs"
```

---

### Task 17: Hover/Select 经 VNodeId 打通 + overlay

**Files:**
- Modify: `crates/auto-lang/src/ui/iced/renderer.rs`（hit-test、overlay、`update_hover`/`select_hovered` 调用处）

**Step 1:** 应用区鼠标移动 → 用 `cache` 中各节点 `bounds` 做 hit-test（复用 `debug::hit_test`），命中 → `hovered_vnode`，左树同步高亮、overlay 蓝框。点击 → `selected_vnode`、overlay 橙框。

**Step 2:** 现有"源码点击 → 高亮"经 `iced_to_vnode` 映射到 VNodeId 后走同一选中通路。

**Step 3:** 手动验收 hover/选中联动。

Run: `cargo build -p auto`

**Step 4: Commit**

```bash
git add crates/auto-lang/src/ui/iced/renderer.rs
git commit -m "feat(ui): hover/select via VNodeId hit-test + overlay"
```

---

## Phase 6 — 性能门控 + 清理

### Task 18: 性能门控（F12 off 零开销）

**Files:**
- Modify: `crates/auto-lang/src/ui/iced/renderer.rs`、`aura_view_builder.rs`

**Step 1:** `InspectorCache` 改为 `Option<InspectorCache>`（DynamicState 字段），仅 F12 on 时 `Some`。所有填充/映射调用包在 `if let Some(cache) = …` 内。

**Step 2:** `BuildProbe` 同理：仅 `debug_mode` 时构造并贯穿 `build`；`aura_view_builder` 的 record_* 调用前判 `probe.is_some()`。

**Step 3:** VTree 仍每帧构建（结构需要），但 `span_for` 闭包在 `debug_mode` off 时直接返回 `None`（跳过 DebugIdMap 查询）。

**Step 4:** 防回归测试：用计数器探针断言 F12 off 时 `compute_style` / probe record 未被调用。

Run: `cargo test -p auto-lang -- f12_off_zero_overhead` + `cargo build -p auto`

**Step 5: Commit**

```bash
git add crates/auto-lang/src/ui/iced/renderer.rs crates/auto-lang/src/ui/aura_view_builder.rs
git commit -m "perf(ui): gate InspectorCache/BuildProbe behind debug_mode"
```

---

### Task 19: 删除死代码 `DebugLayer`

**Files:**
- Modify: `crates/auto-lang/src/ui/debug/mod.rs`

**Step 1:** 确认 `DebugLayer` / `DebugPanel` / `DebugState` / `LayoutReporter` 已无任何调用（grep 全仓）。

**Step 2:** 删除这些类型及其测试。保留 `primitives.rs`（Rect/BoxModel/EdgeInsets，已迁）、`hit_test`、`overlay`、`source_map`。

**Step 3:** `cargo build -p auto` + `cargo test -p auto-lang -- debug`。

**Step 4: Commit**

```bash
git add crates/auto-lang/src/ui/debug/mod.rs
git commit -m "refactor(ui): remove dead DebugLayer (replaced by InspectorCache)"
```

---

### Task 20: 删除 iced 渲染器旧 debug 通路

**Files:**
- Modify: `crates/auto-lang/src/ui/iced/renderer.rs`

**Step 1:** 确认新通路（VTree 树 + InspectorCache 面板）已完全接管后，删除：
- `DebugTreeNode` 及 `tree_enter/tree_exit`、`component_tree`、`tree_stack`（~`renderer.rs:3655-3723`）。
- `DebugElementInfo` / `element_styles` / `debug_element_styles`（~`renderer.rs:1906,2199,2900,3286`）。
- `wrap_debug` 内构建 component_tree / element_styles 的副作用（保留 bounds-probe container、mouse_area、hover/select）。

**Step 2:** `cargo build -p auto` + 全量 `cargo test -p auto-lang`。

**Step 3: Commit**

```bash
git add crates/auto-lang/src/ui/iced/renderer.rs
git commit -m "refactor(ui): remove DebugTreeNode/element_styles superseded by VTree+InspectorCache"
```

---

### Task 21: 手动验收 + 文档

**Files:**
- Modify: `docs/design/08-ui-systems.md`（若有 debug 章节）或新增 `docs/design/` 检视器说明

**Step 1:** 按 design §7.4 执行手动验收清单：
- F12 开 → 三栏出现，左树匹配可见 UI。
- hover 应用元素 → 左树 + overlay 同步。
- 点 for 列表第 3 项 → 右栏 AutoUI 显示 `for: item=…, i=2`，且重渲染后选中仍停留（跨帧存活）。
- 改状态重渲染 → 右栏 state 当前值更新。
- F12 关 → 无 overlay、帧率无明显下降。

**Step 2:** 更新文档描述新检视器（VTree = runtime DOM、InspectorCache、各标签含义）。

**Step 3: Commit**

```bash
git add docs/
git commit -m "docs(ui): document DevTools-style live inspector"
```

---

## 验收标准（Definition of Done）

- [ ] VTree 在 iced 路径每帧构建，VNodeId 由 path 派生、结构不变即稳定。
- [ ] `InspectorCache`（F12 门控）含 bounds/box_model/computed_style/state_bindings/for_context/events/source。
- [ ] 左树读 VTree，右栏含面包屑 + Layout/Computed/Props/AutoUI/Source 标签。
- [ ] hover/选中经 VNodeId 联动 + overlay；选中态跨帧存活。
- [ ] F12 off 时 computed 侧路完全旁路（零额外开销）。
- [ ] `DebugLayer` 与 `DebugTreeNode`/`element_styles` 已删除；`LayoutCollector`/`hit_test`/`overlay`/`source_map`/`DebugIdMap` 保留。
- [ ] `cargo test -p auto-lang` 全绿；`cargo build -p auto` 通过。
- [ ] 手动验收清单全过。

## 风险与回退

- **风险**：`aura_view_builder.rs` / `renderer.rs` 体量大、改动深，可能引入回归。
  **缓解**：每任务独立提交；Phase 3/5 集成任务后跑 `cargo test -p auto-lang`；新通路跑通（Phase 5 末）前不删旧通路。
- **风险**：path 派生 id 在条件渲染（`if` 分支）下，同 path 可能映射到不同节点 → 选中错位。
  **缓解**：验收 Task 21 专项检查条件渲染场景；必要时 path 编码加入分支标记。
- **回退**：任一阶段失败可 `git revert` 该阶段提交，旧通路（删之前）仍可用。
