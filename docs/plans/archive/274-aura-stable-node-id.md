# Plan 273: AURA 稳定节点 ID 体系重构

> **Status: ✅ 已完成** — `VNodeId` 稳定 ID 体系已实现（`crates/auto-lang/src/ui/vnode.rs`），解析阶段分配 ID 并传播到渲染层。

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 重新设计 AURA 节点 ID 体系，在解析阶段分配稳定 ID，一路传播到渲染层，实现精确的双向源码↔组件映射。

**Architecture:** 在 AuraNode 提取阶段分配 `AuraNodeId`（u32 自增），通过 sideband `DebugIdMap`（View 路径 → AuraNodeId）在 View 构建时建立映射，渲染层直接用 AuraNodeId 做 O(1) 查找。不修改 `View<M>` 泛型结构。

**Tech Stack:** Rust, iced 0.14, AutoLang compiler pipeline

---

## Context

当前 DevTools Inspector 的源码↔组件映射有根本性缺陷：
1. ID 在渲染层才分配（`wrap_debug` 中 `next_id()`），此时和源码 AST 的对应关系已经丢失
2. AuraNode 树和 View 树结构不同（`center` → `Container`，ForLoop → `Column` 等），导致 counter 不匹配
3. 没有反向映射（点击源码无法选中组件）
4. `tag_aliases()` 等补丁方案脆弱不可扩展

用户建议的核心思路：**在解析 AURA 源码时就分配 ID，后续转换全程保留映射关系**。

## 实施步骤

### Step 1: 定义 `AuraNodeId` 并添加到 AuraNode

**文件:** `crates/auto-lang/src/aura/types.rs`

添加新类型：
```rust
/// 稳定唯一 ID，在 AuraNode 提取时分配，用于 DevTools 源码↔组件双向映射
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AuraNodeId(pub u32);
```

为每个有 `span` 的 AuraNode variant 添加 `debug_id: Option<AuraNodeId>` 字段：
- `Element { ..., debug_id: Option<AuraNodeId> }`
- `ForLoop { ..., debug_id: Option<AuraNodeId> }`
- `Conditional { ..., debug_id: Option<AuraNodeId> }`
- `Component { ..., debug_id: Option<AuraNodeId> }`
- `Link { ..., debug_id: Option<AuraNodeId> }`

同时定义 `SpanInfo`：
```rust
#[derive(Debug, Clone)]
pub struct SpanInfo {
    pub span: Option<(usize, usize)>,  // 源码字节偏移和长度
    pub aura_tag: String,              // 原始 AuraNode tag（如 "center"）
    pub user_id: Option<String>,       // 用户指定的 id 属性（如 id: "my-btn"）
}
```

**同步更新** `crates/auto-lang/src/aura/extract.rs` 中所有构造 AuraNode 的地方，初始化 `debug_id: None`。

**同步更新**所有 match/构造 AuraNode 的文件（见下方文件清单）。使用 `..` 忽略 `debug_id` 字段即可，因为大多数 match 已用 `..` 忽略 `span`。

### Step 2: 提取阶段分配 ID

**文件:** `crates/auto-lang/src/aura/extract.rs`

新增函数 `assign_node_ids(root: &mut AuraNode) -> HashMap<AuraNodeId, SpanInfo>`：
- DFS 遍历 AuraNode 树，从 0 开始分配递增的 `AuraNodeId`
- 对于 `Element` 节点，检查 `props` 中是否有 `id` 属性，提取为 `user_id`
- 构建 `HashMap<AuraNodeId, SpanInfo>` 映射表
- 返回映射表

在 `extract_widget_from_decl()` 末尾调用 `assign_node_ids()`，将返回的 `SpanMap` 存入 `AuraWidget`。

**文件:** `crates/auto-lang/src/aura/types.rs`

为 `AuraWidget` 添加字段：
```rust
pub span_map: HashMap<AuraNodeId, SpanInfo>,
```

### Step 3: 创建 `DebugIdMap` sideband 映射

**新文件:** `crates/auto-lang/src/ui/debug_id_map.rs`

```rust
/// View 树路径 → AuraNodeId 的 sideband 映射
/// 路径用 Vec<usize> 表示，例如 [0, 2, 1] 表示 "根节点的第 0 个子节点的第 2 个子节点的第 1 个子节点"
#[derive(Debug, Clone, Default)]
pub struct DebugIdMap {
    entries: HashMap<Vec<usize>, AuraNodeId>,
}

impl DebugIdMap {
    pub fn record(&mut self, path: &[usize], id: AuraNodeId) { ... }
    pub fn get(&self, path: &[usize]) -> Option<AuraNodeId> { ... }
}
```

**文件:** `crates/auto-lang/src/ui/mod.rs` — 添加 `pub mod debug_id_map;`

### Step 4: AuraViewBuilder 跟踪路径并构建 DebugIdMap

**文件:** `crates/auto-lang/src/ui/aura_view_builder.rs`

当前 `build()` 签名：`pub fn build(&self, node: &AuraNode) -> View<DynamicMessage>`

修改为返回 `(View<DynamicMessage>, DebugIdMap)`：
```rust
pub fn build(&self, node: &AuraNode) -> (View<DynamicMessage>, DebugIdMap) {
    let mut id_map = DebugIdMap::default();
    let view = self.convert_node_tracked(node, &mut Vec::new(), &mut id_map);
    (view, id_map)
}
```

新增 `convert_node_tracked()` 方法：
- 接收 `path: &mut Vec<usize>` 和 `id_map: &mut DebugIdMap`
- 进入时检查 AuraNode 的 `debug_id`，如果有则 `id_map.record(path, aura_id)`
- 处理子节点时 `path.push(i)` → 递归 → `path.pop()`
- 结构变换时（如 `center` → Container），ID 跟随产出节点
- `ForLoop` 展开为 `Column` 时，body 子节点的路径继续递增

`convert_element()` 等已有方法签名不变，内部对子节点的 `convert_node()` 调用改为 `convert_node_tracked()`。

**注意:** `build()` 返回类型变更会影响所有调用方：
- `DynamicComponent::view()` — 改为只取 `.0`
- `DynamicComponent::view_with_debug()` — 新方法，取完整 tuple

### Step 5: DynamicComponent 存储 SpanMap + 提供 view_with_debug()

**文件:** `crates/auto-lang/src/ui/dynamic.rs`

当前 `DynamicComponent` 字段（行86-112）：`bridge`, `view_template`, `widget_name`, `dirty`, `source_path`, `last_modified`, `input_state_map`, `tick_interval`

添加字段：
```rust
span_map: HashMap<AuraNodeId, SpanInfo>,
```

在 `new()` 和 `reload()` 中从 `AuraWidget.span_map` 获取并存储。

当前 `view()` 方法（行479-487）调用 `AuraViewBuilder::build()`，修改为只取 `.0`：
```rust
fn view(&self) -> View<Self::Msg> {
    let builder = AuraViewBuilder::new(&self.bridge, &self.widget_name);
    builder.build(&self.view_template).0  // 只取 View，丢弃 DebugIdMap
}
```

新增方法：
```rust
pub fn view_with_debug(&self) -> (View<DynamicMessage>, DebugIdMap) {
    let builder = AuraViewBuilder::new(&self.bridge, &self.widget_name);
    builder.build(&self.view_template)
}
pub fn span_map(&self) -> &HashMap<AuraNodeId, SpanInfo> { &self.span_map }
```

移除旧的 `build_span_lookup()` (行269-274)、`collect_all_spans_global_dfs()` (行650-695)、`tag_aliases()` (行697-705) — 被 SpanMap 取代。

### Step 6: 重构 DebugRenderCtx 使用 AuraNodeId

**文件:** `crates/auto-lang/src/ui/iced/renderer.rs`

当前 `DebugRenderCtx`（行2794-2809）字段：`hovered_id`, `selected_id`, `counter`, `kind_counters`, `element_styles`, `tree_stack`, `component_tree`, `span_lookup`

重构为：

```rust
struct DebugRenderCtx {
    hovered_id: Option<String>,
    selected_id: Option<String>,
    wrapper_counter: RefCell<usize>,                    // 仅用于无 AuraNodeId 的合成节点
    span_map: HashMap<AuraNodeId, SpanInfo>,            // 替代 span_lookup
    id_to_aura: RefCell<HashMap<String, AuraNodeId>>,   // debug element id → aura id
    aura_to_id: RefCell<HashMap<AuraNodeId, String>>,   // aura id → debug element id
    debug_id_map: DebugIdMap,                           // view path → aura id
    element_styles: RefCell<HashMap<String, DebugElementInfo>>,
    tree_stack: RefCell<Vec<DebugTreeNode>>,
    component_tree: RefCell<Option<DebugTreeNode>>,
}
```

构建 `DebugRenderCtx` 的地方（行2094-2099 附近）改为：
```rust
// 旧: let span_lookup = state.component.build_span_lookup();
// 新: 从 view_with_debug() 获取 DebugIdMap + span_map
let (view, debug_id_map) = state.component.view_with_debug();
let span_map = state.component.span_map().clone();
// ... 构建 DebugRenderCtx { debug_id_map, span_map, ... }
```

**`wrap_debug()` 签名变更**：
```rust
fn wrap_debug(&self, view_path: &[usize], kind: &str, el: Element, props: Vec<(String, String)>) -> Element
```

- 从 `self.debug_id_map.get(view_path)` 获取 `AuraNodeId`
- 如果找到：用 `"aura_{id}"` 或 `user_id` 作为元素 ID，从 `span_map` 直接获取 span（O(1)，精确）
- 如果没找到（合成包装节点）：用 `wrapper_counter` 生成回退 ID，span 设为 None
- 不再需要 `kind_counters` 和 `span_lookup`

**`render_dynamic_view()` 变更**：

当前 `render_dynamic_view()` 递归渲染 `View<M>` 树，在 `wrap_debug()` 处添加 span 和样式元数据。

改为 `render_dynamic_view_at()`，增加路径跟踪：
```rust
fn render_dynamic_view_at(
    view: AbstractView<IcedMessage>,
    debug_ctx: Option<&DebugRenderCtx>,
    path: &mut Vec<usize>,
) -> Element { ... }
```

处理 children（Column/Row/Container 的 children）时 `path.push(i)` / `path.pop()`。所有 `wrap_debug` 调用增加 `path` 参数。

### Step 7: Inspector 源码点击 → 反向选中组件

**文件:** `crates/auto-lang/src/ui/iced/renderer.rs`

在 `DynamicState` 中添加反向映射：
```rust
/// 行号 → AuraNodeId 列表（源码点击 → 组件选中）
line_to_aura_ids: RefCell<HashMap<usize, Vec<AuraNodeId>>>,
```

构建时机：源码加载时，从 `span_map` 遍历所有条目，将 byte offset 转换为行号范围，建立索引。

在 `render_inspector_tab()` 的源码渲染中，为每一行包裹 `mouse_area`，点击时：
1. 查找 `line_to_aura_ids[line]`
2. 选择最内层（最长匹配）的 AuraNodeId
3. 通过 `aura_to_id` 获取 debug element ID
4. 设置为 `selected_widget` → 触发 UI 更新，左侧对应组件高亮

### Step 8: 清理旧代码

- 移除 `build_span_lookup()` 和 `collect_all_spans_global_dfs()`（`dynamic.rs`）
- 移除 `tag_aliases()`（`dynamic.rs`）
- 移除 `span_lookup_cache` 字段（`DynamicState`，行1537）
- 移除 `kind_counters` 字段（`DebugRenderCtx`）
- 移除 `span_lookup` 字段（`DebugRenderCtx`）
- 移除 `counter` 字段（`DebugRenderCtx`，由 `wrapper_counter` 替代）

---

## 关键文件清单

### 核心 AURA 层（Step 1-2）

| 文件 | 改动 |
|------|------|
| `crates/auto-lang/src/aura/types.rs` | 添加 `AuraNodeId`, `SpanInfo`; `AuraNode` 5个有span的variant加 `debug_id: Option<AuraNodeId>`; `AuraWidget` 加 `span_map` 字段 |
| `crates/auto-lang/src/aura/extract.rs` | 添加 `assign_node_ids()`; ~12处 AuraNode 构造加 `debug_id: None`; 提取 `user_id` |

### AURA 引用方 — match/构造处加 `debug_id`（编译修复，机械性改动）

| 文件 | 说明 |
|------|------|
| `crates/auto-lang/src/aura/atom.rs` | 7个match arm，已有 `..` 忽略span，同理忽略debug_id |
| `crates/auto-lang/src/a2ui/import.rs` | ~20处 `AuraNode::Element` 构造，加 `debug_id: None` |
| `crates/auto-lang/src/a2ui/export.rs` | match arm 加 `..` |
| `crates/auto-lang/src/ui_gen/ark/generator.rs` | match arm 加 `..` |
| `crates/auto-lang/src/ui_gen/jet/generator.rs` | match arm 加 `..` |
| `crates/auto-lang/src/ui_gen/vue.rs` | match arm 加 `..` |
| `crates/auto-lang/src/ui_gen/rust.rs` | match arm 加 `..` |

### UI 运行时层（Step 3-7）

| 文件 | 改动 |
|------|------|
| `crates/auto-lang/src/ui/debug_id_map.rs` | **新文件**: `DebugIdMap` sideband 映射 |
| `crates/auto-lang/src/ui/mod.rs` | 添加 `pub mod debug_id_map`（放在 `aura_view_builder` 旁边） |
| `crates/auto-lang/src/ui/aura_view_builder.rs` | `build()` 返回 `(View, DebugIdMap)`; 新增路径跟踪 `convert_node_tracked()` |
| `crates/auto-lang/src/ui/dynamic.rs` | 添加 `span_map` 字段, `view_with_debug()`, `view()` 改为 `.0`; 移除旧 span lookup |
| `crates/auto-lang/src/ui/iced/renderer.rs` | 重构 `DebugRenderCtx`, 路径跟踪渲染, 反向映射, 清理 `span_lookup_cache`/`kind_counters` |

## 不需要修改的文件

- `ui/view.rs` — `View<M>` 泛型结构不变
- `ast/ui.rs` — `ViewNode` 不变（ID 在 AuraNode 层分配）
- `aura/validate.rs` — 匹配的是 `ViewNode` 不是 `AuraNode`，无需改动
- 其他后端渲染器（gpui, headless）— 不使用 `debug_id`

## 验证

```bash
# 1. 编译
rtk cargo build -p auto --features ui-iced

# 2. 运行示例
auto examples/ui/002-counter/src/front/app.at

# 3. 验证正向映射
# F12 → 点击左侧 button/row/center → Inspector 源码高亮精确对应

# 4. 验证反向映射（Step 7 实现后）
# F12 → 点击 Inspector 源码中的某行 → 左侧对应组件高亮选中

# 5. 验证 hot-reload
# 编辑 .at 文件 → 保存 → 映射关系自动重建
```
