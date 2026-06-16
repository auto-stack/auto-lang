# AutoUI MCP "实时样式 VTree" (real-time styled VTree) — Plan 314

> **For Claude:** 主战场是 `crates/auto-lang/src/ui/mcp_server.rs`（新工具 + 数据通道）+ 一个新的 `vtree_atom.rs`（VTree→Atom 序列化器）+ `crates/auto-lang/src/ui/iced/renderer.rs`（把 live VTree+cache 拷进 MCP SharedState，解耦 F12 门控）。构建 `cargo build -p auto`；回归 `cargo test -p auto-lang --lib 'ui::'`。手动验证用 `examples/ui/012-stopwatch`（VM 模式，bounds 已可用）。

## 目标（一句话）

新增 MCP 工具 `autoui_vtree`，把当前界面的**实时 VTree**（每个 node 与 VTree 中的 VNode 一一对应）序列化成 **Atom 格式**返回：node 名字对上原始 AutoUI widget（`col`/`button`/`center`…），用完整**盒模型**取代旧的简单 `rect`（盒模型内含 bounding box），其余 layout/style 属性作为 node 内部属性。让其它 AI Agent 拿到这棵 Atom 树就能精确感知"渲染后"的界面，无需截图。

## 架构（2-3 句）

复用 F12 DevTools 已 gather 的 `live_vtree` + `live_cache`（按 `VNodeId` 索引的 `ComputedNode`，含 `BoxModel`/`computed_style`/events/source）。把这条数据路径从"F12 门控"解耦成"F12 或 MCP 连接即激活"，并像现有 `set_layout_bounds` 那样每帧把一个可序列化快照拷进 MCP `SharedState`。新增 `VTreeAtomBuilder` 把 `VTree + InspectorCache` 转成 `auto_val::Node` 树，再经 `Display` 序列化为 Atom 文本。

## Tech Stack

- Rust；`auto_val::Node`（Atom 节点，builder + `Display` 序列化）。
- 复用：`crate::ui::vnode::{VTree, VNode, VNodeId, VNodeKind, VNodeProps}`、`crate::ui::debug::{InspectorCache, ComputedNode, BoxModel, Rect, EdgeInsets}`、`crate::ui::mcp_server::SharedState`。
- iced 渲染器侧：`DynamicState`（VM，renderer.rs:2059/2068）、`DevToolsWrapper`（rust，renderer.rs:6131/6132）。

---

## 现状分析（为什么这是"升级"而非新建）

### 现有 MCP（`crates/auto-lang/src/ui/mcp_server.rs`）

- 工具集：`autoui_snapshot` / `autoui_inspect` / `autoui_action` / `autoui_check` / `autoui_screenshot` / `autoui_state` / `autoui_wait` / `autoui_type` / `autoui_keyboard`。
- `autoui_snapshot` 走的是 **`view_template`（AURA 模板，build-time）+ `AuraSnapshotBuilder`**：
  - 节点 id 是 **`AuraNodeId`（`aura_N`，组件级、build-time）**，不是实例级。
  - 属性是模板里的**未求值 prop 表达式**，由 `eval_prop_value` 现场求值。
  - 几何只有 `include_bounds=true` 时附加的 `@rect x,y,w,h`（来自 `SharedState.layout_bounds: HashMap<String,(f32,f32,f32,f32)>`，按 `aura_N` 索引）——**只是 4 元组 rect，没有完整盒模型，没有 computed style**。
  - 输出是自定义 "AuraUI Snapshot v1" 文本，**不是 Atom**。
- 数据通道：iced 渲染器每帧 `mcp.lock().unwrap().set_layout_bounds(bounds_map)`（renderer.rs:2283）把测量 rect 写进 SharedState；`view_template` 来自 `state.component.view_template()`（renderer.rs:3144）。**先例明确**：渲染器→SharedState 的每帧拷贝是既定模式。

### F12 Debug 模式已可得的信息（数据富矿）

- `live_vtree: RefCell<Option<VTree>>`（renderer.rs:2059 / 6131）——实例级、path-based 的 `VNodeId`，for 循环每次展开唯一。
- `live_cache: RefCell<Option<InspectorCache>>`（renderer.rs:2068 / 6132）——按 `VNodeId` 索引，每个 `ComputedNode`（inspector_cache.rs:54）含：
  - `bounds: Option<Rect>` —— iced 测量的 border-box。
  - `box_model: Option<BoxModel>` —— content + padding + border + margin（primitives.rs:89），带 `border_box()`/`padding_box()`/`margin_box()`。
  - `computed_style: Vec<(String,String)>` —— class 解析后的 computed k/v（颜色/字号/圆角…）。
  - `raw_class`、`state_bindings`、`for_context`、`events`、`source`。
- `VNode`（vnode.rs:243）自带 `kind: VNodeKind`、`props: VNodeProps`、`path: Vec<u16>`、`source_span`。
- **关键约束**：`live_cache` 当前**只在 debug 模式（F12 开）下填充**（renderer.rs:2275 注释）。本计划要解耦这条门控。

### Atom 序列化底座（已就绪）

- `auto_val::Node`（node.rs:100）：`Node::new(name).with_arg(v).with_prop(k,v).with_child(node)`。
- `impl Display for Node`（node.rs:829）→ 输出 `name id? (args) { props; children }`。
- 源 widget 关键字（node_converter.rs 测试确认）：`col` / `row` / `text` / `button` …。

### 结论

新工具 = **把 MCP 从"build-time 模板 + rect"切换到"runtime VTree + 完整盒模型 + computed style + Atom"**，数据源就是 F12 的 `live_vtree`/`live_cache`。二者共享同一数据层，是自然收敛。

---

## 目标设计

### 1. Atom 节点 schema

每个 VNode → 一个 Atom `Node`：

| Node 字段 | 来源 | 说明 |
|---|---|---|
| `name` | `VNodeKind` 经映射表 → 源关键字 | `col`/`row`/`button`/`center`…（见下表） |
| `id`（main arg） | `VNodeId` → `"vnode_<n>"` | 实例级、稳定；取代旧的 `aura_N` |
| props: widget 属性 | `VNodeProps` 展开 | `Text→content`、`Button→label`、`Input→{placeholder,value,password}`、`Checkbox→{label,checked}`、`Select→{options,selected}`、`Slider→{min,max,value,step}`、`ProgressBar→progress`、`Layout→{spacing,padding}`、`Container→{padding,center_x,center_y}` 等 |
| prop: `bbox` | `ComputedNode.bounds`（border-box） | `Obj{x,y,w,h}` —— **= 旧 rect 的超集/等价**，bounding box |
| prop: `box` | `ComputedNode.box_model` | `Obj{ content:{x,y,w,h}, padding:{t,r,b,l}, border:{t,r,b,l}, margin:{t,r,b,l} }`；缺失则省略 |
| prop: `style` | `ComputedNode.computed_style` | `Obj{ color:"#ff0000", font_size:14, radius:8, … }`；空则省略 |
| prop: `events` | `ComputedNode.events` | `Array<Node>` 或字符串数组：`[press -> .AddTodo, change -> .Search]` |
| prop: `source` | `ComputedNode.source` | `"app.at:42"` |
| prop: `for_iter` | `ComputedNode.for_context` | `{index, item}`；非 for 子节点省略 |
| prop: `class` | `ComputedNode.raw_class` | 原始 class 字符串（便于 AI 对照源码） |
| **kids（子节点）** | `VNode.children` | **严格 1:1** —— 只有真正的 widget 子节点进 kids |

**关键约束（写进实现注记）**：`bbox`/`box`/`style`/`events`/`source`/`for_iter`/`class` 全部是 **props（Value::Obj/Array/Str）**，**不是 children**。这样 node 的 children 拓扑严格 = VTree 的 children 拓扑，满足"每一层 node 与 VTree 中的 Node 一一对应"。盒模型等元数据作为属性挂在 node 内部，不破坏层级。

示例输出：

```
col vnode_0 {
  widget: NotesApp
  bbox: { x: 0; y: 0; w: 1600; h: 900 }
  box: { content: {x:0 y:0 w:1600 h:900}; padding: {t:0 r:0 b:0 l:0}; border: {t:0 r:0 b:0 l:0}; margin: {t:0 r:0 b:0 l:0} }
  style: { direction: "column"; full: true }
  class: "w-full h-screen bg-white flex-col"
  source: "app.at:14"
  row vnode_1 {
    bbox: { x: 0; y: 0; w: 1600; h: 64 }
    style: { direction: "row"; align: "center" }
    class: "w-full items-center p-4 border-b border-gray-200"
    text vnode_2 {
      content: "Notes"
      bbox: { x: 16; y: 22; w: 96; h: 32 }
      style: { font_size: 30; weight: "bold"; color: "#1f2937" }
      class: "text-3xl font-bold text-gray-800"
    }
    button vnode_3 {
      label: "+ New"
      bbox: { x: 1480; y: 16; w: 104; h: 36 }
      style: { bg: "#3b82f6"; color: "#ffffff"; radius: 8 }
      events: [press -> .NewNote]
      class: "ml-auto px-4 py-2 bg-blue-500 ..."
      source: "app.at:18"
    }
  }
}
```

### 2. Widget 名字映射表（VNodeKind → 源关键字）

node 名字必须对上**用户在 .at 源码里写的 widget 关键字**。下表为**初值**，Task 1 须对照 `node_converter.rs` / `aura_view_builder.rs` 核对每个关键字（`col` vs `column` 等）后定稿：

| VNodeKind | 源关键字（待核） | VNodeKind | 源关键字（待核） |
|---|---|---|---|
| Column | `col` | List | `list` |
| Row | `row` | Table | `table` |
| Container | `container` | Slider | `slider` |
| Scrollable | `scrollable` | ProgressBar | `progress` |
| Center | `center` | Accordion | `accordion` |
| Text | `text` | Sidebar | `sidebar` |
| Button | `button` | Tabs | `tabs` |
| Input | `input` | NavigationRail | `nav_rail` |
| Textarea | `textarea` | Checkbox | `checkbox` |
| Radio | `radio` | Select | `select` |

### 3. 工具契约：`autoui_vtree`

- **返回**：Atom 文本（一棵 Node 树）。
- **参数（token 控制）**：
  - `scope`（可选，默认根）：`vnode_<n>` 或 path 数组 `[0,1,2]`，只返回该子树。
  - `depth`（可选，默认全量）：最大渲染深度，超出用 `... (N children)` 折叠。
  - `include_box`（默认 true）：盒模型 + bbox。
  - `include_style`（默认 true）：computed style + class。
  - `include_events`（默认 true）：events。
  - `include_source`（默认 true）：source/for_iter。
  - `include_props`（默认 true）：widget 属性（content/label/value…）。
- **降级**：任一 `ComputedNode` 字段缺失（bounds=None、rust 模式未测几何等）→ 该 prop 省略，node 仍输出，**永不报错**（不变量：空字段即省略）。

### 4. 数据通道（核心集成）

引入可序列化快照类型 + 每帧拷贝，镜像 `set_layout_bounds`：

```rust
// mcp_server.rs
pub struct StyledNodeSnapshot {
    pub vtree: crate::ui::vnode::VTree,          // Clone
    pub computed: std::collections::HashMap<VNodeId, ComputedNodeLite>, // 序列化友好的子集
}
pub struct ComputedNodeLite {
    pub bounds: Option<(f32,f32,f32,f32)>,
    pub box_model: Option<BoxModel>,
    pub computed_style: Vec<(String,String)>,
    pub raw_class: Option<String>,
    pub events: Vec<(String,String)>,
    pub source: Option<String>,
    pub for_context: Option<(usize, String)>,
}
impl SharedState {
    pub fn set_styled_vtree(&mut self, snap: StyledNodeSnapshot); // 每帧调用
}
```

渲染器侧（renderer.rs，紧挨 `set_layout_bounds` 调用处）：
```rust
if state.live_vtree.borrow().is_some() {
    let snap = StyledNodeSnapshot::from_live(&state); // 读 live_vtree + live_cache
    mcp.lock().unwrap().set_styled_vtree(snap);
}
```

### 5. 解耦 F12 门控

当前 `live_cache` 仅 debug 下填充。改为：`DynamicState`（VM）与 `DevToolsWrapper`（rust）新增 `mcp_active: Cell<bool>`；VTree + cache 的捕获/填充条件由 `devtools_open` 改为 **`devtools_open || mcp_active`**。MCP 连接时置 `mcp_active=true`，断开置 false。这样 **AI Agent 无需打开 F12** 即可拿到完整 VTree。

> VM 模式 bounds 已可用（layout_bounds 始终流）；rust 模式 bounds 属 Plan 311 P2-B-3（deferred），故 rust 模式 `bbox`/`box` 暂省略，schema 自动降级。

---

## 任务（TDD，bite-sized）

### Task 1 — Widget 名字映射表 + 单测

**Files:** Modify `crates/auto-lang/src/ui/vnode.rs`（或新模块 `vtree_atom.rs`）；Test 同文件 `#[cfg(test)]`。

1. 写失败测试：`fn widget_keyword()` 断言 `kind_keyword(VNodeKind::Column)=="col"`、`Button=="button"`、`Center=="center"` 等全表。
2. 实现 `pub fn kind_keyword(k: VNodeKind) -> &'static str`（match 全枚举）。
3. **核对源关键字**：grep `node_converter.rs` / `aura_view_builder.rs`，确认每个关键字与源码一致（尤其 col/row、progress vs progress_bar、nav_rail）；不一致以源码为准并更新测试。
4. `cargo test -p auto-lang --lib vnode::tests::widget_keyword`。

### Task 2 — `ComputedNodeLite` + `StyledNodeSnapshot`（序列化友好子集）

**Files:** Modify `crates/auto-lang/src/ui/mcp_server.rs`。

1. 定义 `ComputedNodeLite` / `StyledNodeSnapshot`（见设计 §4）。
2. 写 `StyledNodeSnapshot::from_live(state)`：从 `DynamicState`（VM）读 `live_vtree` + `live_cache`，组装。**抽 trait** `LiveVTreeSource`（`fn live_vtree(&self) -> Ref<Option<VTree>>; fn live_cache(&self) -> Ref<Option<InspectorCache>>`），让 VM 的 `DynamicState` 与 rust 的 `DevToolsWrapper` 都 impl，复用同一组装逻辑。
3. 单测：构造一个 2 节点 VTree + 手填 InspectorCache，断言 `from_live` 产出正确 `StyledNodeSnapshot`。
4. `cargo test -p auto-lang --lib mcp_server`。

### Task 3 — `VTreeAtomBuilder`（VTree+cache → `auto_val::Node`）

**Files:** Create `crates/auto-lang/src/ui/vtree_atom.rs`。

1. 写失败测试：构造 VTree（`col` 含 `text("a")` + `button("b")`）+ cache（给每个 VNodeId 填 bounds + 一条 computed_style + 一个 event），调 `VTreeAtomBuilder::build(&snap, &Options::default()).to_string()`，断言输出含：
   - `col vnode_0 {`、`text vnode_1 { content: "a"`、`button vnode_2 { label: "b"`；
   - `bbox: { x:`、`style:`、`events: [press -> .X]`；
   - 子节点拓扑 1:1（col 的 children 只有 text/button，box/style 是 prop 不是 child）。
2. 实现 builder：
   - DFS 遍历 VTree（按 `VNode.children`），递归建 `Node`。
   - node.name = `kind_keyword(vnode.kind)`；main arg = `format!("vnode_{}", vnode.id.as_u64())`。
   - 按 `VNodeProps` 展开 widget 属性 prop。
   - 从 `computed` map 取 `ComputedNodeLite`，按 Options 决定是否附 `bbox`/`box`/`style`/`events`/`source`/`for_iter`/`class`（**全部用 `with_prop(k, Value::Obj/Array/Str)`，绝不 with_child**）。
   - children 用 `with_child` 递归挂上。
   - `Options { scope, depth, include_* }`：scope 过滤根、depth 折叠、include_* 开关。
3. `cargo test -p auto-lang --lib vtree_atom`。

### Task 4 — 解耦捕获门控（`devtools_open || mcp_active`）

**Files:** Modify `renderer.rs`（VM `DynamicState` + rust `DevToolsWrapper`）。

1. 给 `DynamicState` 与 `DevToolsState`（DevToolsWrapper 内）加 `mcp_active: Cell<bool>`。
2. 把"`live_cache` 仅 debug 填充"的门控条件（renderer.rs:2275 一带）改为 `*devtools_open.borrow() || mcp_active.get()`；`live_vtree` 构建同理。
3. 单测：`mcp_active=true && devtools_open=false` 时 `live_vtree`/`live_cache` 仍被填充。
4. `cargo test -p auto-lang --lib 'ui::iced'` + `cargo test -p auto-lang --lib 'ui::debug'`（回归 80+12）。

### Task 5 — 渲染器每帧拷贝快照进 SharedState

**Files:** Modify `renderer.rs`（紧邻 `set_layout_bounds` 调用处，~2283）。

1. 在已有的 per-frame `set_layout_bounds` 之后，若 `live_vtree.is_some()`：`let snap = StyledNodeSnapshot::from_live(&*state); mcp.lock().unwrap().set_styled_vtree(snap);`。
2. rust 模式（DevToolsWrapper）镜像同样调用（bounds 暂缺失，自动降级）。
3. 手动验证：`auto r`（VM）跑 `012-stopwatch`，另起进程连 MCP 调 `autoui_vtree`，确认 SharedState 有快照。

### Task 6 — 注册 `autoui_vtree` 工具

**Files:** Modify `mcp_server.rs`。

1. 在 `dispatch_tool_static`（~540）加 `"autoui_vtree" => tool_vtree(shared, args)`。
2. 在工具 schema 列表（~306 一带）加 `autoui_vtree` 的 JSON schema（参数见设计 §3）。
3. `tool_vtree`：读 Options → 取 `shared.styled_vtree`（`Option<StyledNodeSnapshot>`）→ `VTreeAtomBuilder::build` → `text_result(atom_string)`；快照为 None 时返回友好错误（"UI 尚未渲染"）。
4. 单测：mock SharedState 带 snap，调 `tool_vtree`，断言返回 Atom 文本含预期 node。
5. `cargo test -p auto-lang --lib mcp_server`。

### Task 7 — 文档 + examples 更新

**Files:** Modify `docs/design/14-developer-tools.md`（MCP Server 节）；可选 `docs/design/07-data-structures.md`。

1. 记录 `autoui_vtree` 契约、schema、与 `autoui_snapshot`（旧，build-time）的关系：**新工具是 runtime/computed 主信道，旧工具保留向后兼容**。
2. 明确"截图仍是像素级验证的次信道"，VTree 是结构/布局/样式的主信道。
3. 给一个 example Atom 输出片段。

---

## 验证

1. `cargo build -p auto` —— 全绿，VM/rust 双模式都能编译。
2. `cargo test -p auto-lang --lib 'ui::'` —— 全绿（含 Task 1/2/3/6 新测试 + Task 4 回归）。
3. 手动（VM，`examples/ui/012-stopwatch`）：
   - `auto r` 启动；用 MCP 客户端调 `autoui_vtree`（不开 F12）。
   - 返回的 Atom 树：每个 node 名字 = 源 widget（col/button/text…）；leaf 有 `bbox` + `style`；button 有 `events`；for 循环展开成多个实例 node，每个 `vnode_<n>` 唯一。
   - `scope=vnode_3` 只返回子树；`depth=1` 折叠深层；`include_style=false` 时无 `style` prop。
   - 与 `autoui_screenshot` 对比：bbox 数值与屏幕像素位置吻合（主信道正确性）。
4. 手动（rust，`examples/ui/012-stopwatch` `auto r -r rust`）：
   - `autoui_vtree` 返回树 + widget 属性 + style；`bbox`/`box` 因 P2-B-3 缺省省略（降级正常，不报错）。

---

## Follow-ups（不在本计划）

- **rust 模式 measured bounds**（Plan 311 P2-B-3）：完成后 rust 模式 `autoui_vtree` 即有 `bbox`/`box`。
- **diff 模式** `diff_since=last`：只返回变更节点，省 token。
- **`autoui_snapshot` 旧工具的废弃/重定向**：稳定后再决定是否把 `autoui_snapshot` 重定向到 `autoui_vtree`（破坏性变更，需评估下游）。
- **filter 谓词**：按 kind/prop/class 过滤子集（CSS-selector 式）。
- **state bindings**：把 `${.field}` 运行时绑定值纳入 node（VM 模式可做；rust 模式无运行时 VM，N/A）。

## 相关

- 数据路径分歧：[[307-devtools-path-schemes]] —— 本计划统一到 VNodeId（View-structural）方案。
- rust 模式 DevTools：[[rust-mode-devtools-parity]]（Plan 311）。
- 旧 MCP 协议：`docs/plans/299-autoui-mcp-v2.md`。
