# AutoUI DevTools 风格实时检视器（重构 VM 版 F12 Debug）

- 状态：设计已确认，待转实现计划
- 日期：2026-06-13
- 范围：VM 版 AutoUI 的 iced 渲染路径，F12 Debug 检视功能重构

## 1. 背景与动机

当前 F12 检视功能的右侧面板，试图把左侧选中的 widget 映射回 Auto 源码里 `view`
部分并高亮对应 span。由于 span 映射脆弱，"很难对准，效果较差"。

Chrome DevTools 的检视之所以准，是因为它展示的不是 HTML 源码，而是**实时计算
出来的 DOM**：左侧选中的节点和渲染结果一一对应，每个节点的 layout / 样式都是每帧
实时计算结果，可直接查看 computed 属性。

AutoUI 没有 DOM，但存在一个等价的"实时计算结果树"。本设计的目标是：把这棵树
正式确立为运行时 DOM，让右侧面板从"源码视图"改为"computed 视图"，实现 DevTools
风格的实时检视。

### 现状（重构前）

经代码确认的关键事实：

1. **VTree/VNode 在 iced 路径中并未使用。** `vnode.rs` 定义的 VTree 只服务于 GPUI
   后端（`gpui/vnode_entity.rs`）和 headless 渲染器（`headless/renderer.rs`）。iced
   后端是 `View<M>` → `into_iced()` 直接渲染，**没有 VTree 中间层**。
2. **iced 路径已有一棵实时计算树**，只是没被当 DOM 用：`iced/renderer.rs` 的
   `wrap_debug` 通过 `tree_enter/tree_exit` 边渲染边构建 `DebugTreeNode`
   （`id`/`kind`/`children`），存入 `component_tree`。**这就是等价的 computed DOM**
   （含 for 展开后的节点）。
3. **实时 layout 已可获取**：`iced/layout_collector.rs` 的 iced `Operation` 已收集每
   个节点经真实布局后的 bounds（x/y/w/h），key 为 `aura_N` 字符串。但这些 bounds
   目前只喂给 MCP snapshot 的 `@rect` 标注，**未显示在检视面板**。
4. **右侧面板当前显示源码**：`render_inspector_tab` 顶部显示 `kind + 原始 props`，
   下半部分通过 span 反查显示并高亮源码——即"难对准"的部分。
5. **存在一套未接通的死代码**：`ui/debug/DebugLayer` 设计干净（bounds/hover/select
   /panel/box_model/source_map 齐全），但 `populate_panel` 把 widget 类型写死成
   `Container`、styles 写死成空 HashMap，未与真实树接通。

**结论：差距不是"没有 DOM"，而是已有实时计算树与实时布局未被当作 computed 属性
展示，反而去展示源码；且存在两套并行 debug 实现。**

## 2. 方案选择

| 方案 | 描述 | 取舍 |
|---|---|---|
| A. 全部数据塞进 VNode | 给 VNode 加 bounds/resolved_style/state_bindings 等字段 | 最直观，但 VTree 每帧都胖，F12 off 仍付成本 |
| **B. VTree 作结构骨架 + InspectorCache 侧表（选定）** | VTree 精简、每帧构建；贵数据进 F12 门控的侧表 | 性能门控干净，VNode 精简，GPUI/headless 不受影响 |
| C. 不重构 iced，只换面板 | 保留 DebugTreeNode 当 DOM | 与"统一用 VTree"目标冲突，否决 |

**选定方案 B。** VTree 是结构真相源（左树、面包屑都从它读），DevTools 那些"贵且
可选"的 computed（bounds 须布局后回填、状态值须查 VmBridge）隔离到 F12 门控的
`InspectorCache`，复用现有 `LayoutCollector` 产出，使 F12 关闭时近乎零开销。

## 3. 整体架构与每帧数据流

VTree 成为 iced 路径结构的唯一真相源，每帧构建（廉价）。所有贵且可选的 computed
数据进 `InspectorCache`，仅 F12 开启时填充，按 `VNodeId` 索引；bounds 等布局后才
知道的数据在布局完成后回填侧表。

```
AuraNode 模板
   │  AuraViewBuilder（解析状态绑定 ${.x}、展开 for、绑定事件）
   ▼
View<M>  ──① view_to_vtree（iced 变体）──►  VTree  ★结构真相源★
   │                                          每节点带: VNodeId, kind, props,
   │                                          parent/children, source_span, path
   │                                          构建时顺带记录 for上下文/状态绑定/事件
   ▼
iced::Element（渲染）
   │  ② 渲染时维护 id_map: VNodeId ↔ iced.Id("aura_N")
   ▼
iced 布局 ──③ LayoutCollector──► HashMap<iced_id, Rect>
   │
   ▼  ④ 布局后回填：经 id_map 把 bounds 并入 InspectorCache
InspectorCache[VNodeId] = ComputedNode {
     bounds, box_model,               ← 来自 LayoutCollector
     computed_style,                  ← 解析 class 字符串
     state_bindings + 当前值,          ← AuraViewBuilder 捕获 + VmBridge 快照
     for_context,                     ← for 展开时记录 (var, i, iterable)
     events,                          ← AuraNode 事件 → 处理函数名
     source,                          ← span_map
}
   │
   ▼  ⑤ 左侧树 + 右侧面板 都从这里读
```

**F12 关闭时**：VTree 仍构建（结构需要），但 computed 填充、id_map 维护、
InspectorCache 构建全部跳过 → 近乎零额外开销。把现有 `wrap_debug` 里零散的
`if !debug_mode` 门控升级为"整条 computed 侧路旁路"。

### 三个硬关联

1. **`VNodeId ↔ aura_N ↔ iced.Id` 三方映射** —— bounds 才能回填。
2. **VNodeId 稳定性** —— 当前 iced 用每帧重置的 `counter_val` 给 for 展开节点去重，
   选中态跨帧会丢。改用**位置/路径稳定**的 VNodeId 分配（结构不变则 id 不变）。
3. **box model 的 padding 来源** —— iced `Operation` 只回 bounds，拿不到 padding
   insets。padding 取自**声明的 class 解析值**，content rect = bounds − declared
   padding。面板中标注"padding 为声明值"。

## 4. 数据结构

原则：VNode 保持精简（GPUI/headless 不受影响）；重型数据全进 `InspectorCache`。

### 4.1 VNode 增量（`vnode.rs`）

只加两个字段，承载"构建时即可知且便宜"的信息：

```rust
pub struct VNode {
    // ...现有 id/kind/parent/children/props/label 不变...
    /// 构建时的源码 span（来自 DebugIdMap → AuraNodeId → span_map）
    pub source_span: Option<SourceSpan>,
    /// 稳定的逻辑路径（如 [0, 1, 2]），用于位置稳定的 id 分配与跨帧选中
    pub path: Vec<u16>,
}
```

`path` 是从根到该节点的子索引序列。**VNodeId 改为由 path 派生**（哈希或递推编码），
保证"结构不变 → id 不变"，解决跨帧选中丢失，也天然处理 for 展开节点（每次迭代产生
不同 path）。

### 4.2 InspectorCache + ComputedNode（新文件 `ui/debug/inspector_cache.rs`）

```rust
/// 仅 F12 开启时填充。按 VNodeId 索引。
pub struct InspectorCache {
    by_id: HashMap<VNodeId, ComputedNode>,
    /// VNodeId ↔ iced widget id("aura_N") 双向映射
    id_to_iced: HashMap<VNodeId, String>,
    iced_to_id: HashMap<String, VNodeId>,
}

pub struct ComputedNode {
    // —— 实时布局（布局后回填）——
    pub bounds: Option<Rect>,              // 来自 LayoutCollector
    pub box_model: Option<BoxModel>,       // content = bounds − declared_padding
    // —— 解析后的 computed 样式 ——
    pub computed_style: Vec<(String, String)>,  // [("width","100%"),("background-color","#3b82f6")]
    pub raw_class: Option<String>,         // 原始 class 字符串
    // —— AutoUI 特有 ——
    pub state_bindings: Vec<StateBinding>, // 状态绑定 + 当前值
    pub for_context: Option<ForIter>,      // for 迭代溯源
    pub events: Vec<EventHandlerInfo>,     // 事件绑定
    // —— 源码 ——
    pub source: Option<SourceLocation>,    // 可点击链接
}

pub struct StateBinding { pub expr: String, pub current_value: String }  // "${.count}", "3"
pub struct ForIter { pub var: String, pub index: Option<usize>, pub value_repr: String, pub iterable_repr: String }
pub struct EventHandlerInfo { pub event: String, pub handler: String }  // "onclick", "handle_click"
```

### 4.3 数据来源映射

| 字段 | 来源 | 时机 |
|---|---|---|
| `bounds` | `LayoutCollector`（现有） | 布局后，经 `id_to_iced` 回填 |
| `box_model` | bounds − 解析出的 Padding class | 回填时计算 |
| `computed_style` | `Style::parse(raw_class)` → StyleClass 映射成 (prop,val) | 构建时 |
| `state_bindings` | AuraViewBuilder 解析 `${.x}` 时捕获 + VmBridge 快照取值 | 构建时捕获，回填时取值 |
| `for_context` | AuraViewBuilder for 展开时记录 | 构建时 |
| `events` | AuraNode `events` map → handler 名 | 构建时 |
| `source` | `span_map` via AuraNodeId（现有） | 构建时 |

AuraViewBuilder 新增**采集通道 `BuildProbe`**：构建 View 时顺带把 state_bindings /
for_context / events 收集，按 path 对齐到 VNode。这是除"换面板"外的主要代码落点。

## 5. 左右面板布局与交互

DevTools 风格三栏（F12 开启时叠加在应用右侧/抽屉）：

```
┌──────────────┬─────────────────────────────────┐
│ 左：VTree 树  │ 右：检视面板（选中节点）          │
│              │ root › col › row ▸ [button]   ← 面包屑（可点祖先）
│ ▼ root:col   │ ─────────────────────────────    │
│   ▼ row      │ [Layout] [Computed] [Props]      │ ← 标签页
│   ▼ col      │ ─────────────────────────────    │
│     • text   │  Box Model 可视化                 │
│     ▸ button │  content x10 y20 120×36          │
│   ▸ for(...) │  padding: 4  margin: 0           │
│              │  ── Computed ──                   │
│ (hover→高亮  │  width 100%  background #3b82f6   │
│  select→选中)│  ── AutoUI ──                     │
│              │  state: ${.count} = 3             │
│              │  for: item=apple, i=2             │
│              │  events: onclick → handle_click   │
│              │  ── Source ──  app.at:42  (链接)   │
└──────────────┴─────────────────────────────────┘
```

**左栏（VTree 树）**：从 VTree 根递归渲染，替代现有 `DebugTreeNode` 树。每行：`kind`
+ 简短摘要（text 显示前若干字符、for 显示迭代数）。展开/折叠。hover→蓝色 overlay
高亮该节点 bounds，click→橙色选中 + 联动右栏。**for 展开节点直接平铺为子节点**
（VTree 相对源码的优势——已展开）。

**右栏面包屑**：从选中节点沿 `parent` 上溯到根，可点击任意祖先跳转选中。

**右栏标签页**（复用现有标签机制）：
- **Layout**：Box Model 可视化 + 数值。padding 标注"声明值"。
- **Computed**：`computed_style` 键值对。
- **Props**：`raw_class` + 原始 props（text content / value / checked 等）。
- **AutoUI**：state_bindings（含当前值）、for_context、events。相对 DevTools 的增量。
- **Source**：单行可点击 `app.at:42`，跳源码定位（不再整块展示源码）。

**交互闭环**：
- 应用区 hover → 命中 VNodeId（bounds hit-test）→ 左树同步高亮 + overlay。
- 左树点击 → 选中 VNodeId → 右栏填充。
- 面包屑点祖先 → 切换选中。
- 现有"源码点击 → 高亮组件"保留，经 `iced_to_id` 映射到 VNodeId 后走同一选中通路。

**选中态跨帧存活**：VNodeId 由 path 派生，结构不变时同一节点 id 不变，选中态自然延续。

## 6. 错误处理 / 降级 + 死代码清理

### 6.1 降级策略（每个字段独立失败，不连累整体）

| 失败情形 | 降级行为 |
|---|---|
| 节点未布局 / `id_to_iced` 缺映射 | `bounds=None`，Layout 标签显示"(布局中…)"，其他标签正常 |
| `class` 解析失败 / 空 | `computed_style=[]`，Props 标签回退显示 `raw_class` 原文 |
| 状态绑定取值失败 | `current_value="<unresolved>"`，仍显示绑定表达式 |
| for 上下文缺失 | AutoUI 标签不显示 for 行，无报错 |
| 事件无绑定 | events 行省略 |
| 源码 span 缺失 | Source 显示"(无源码定位)" |

**不变量**：选中任何 VNodeId，右栏**永远能渲染**（至少 kind + path），绝不因某项
computed 取不到而白屏或 panic。靠 `ComputedNode` 全字段 `Option`/`Vec`（空即可）保证。

### 6.2 性能门控（F12 off 的零开销）

- `InspectorCache` **整对象**只在 `debug_mode` 时存在（`Option<InspectorCache>`）。
- `BuildProbe`、`id_to_iced` 维护、`Style::parse`、状态取值——全部在 `if debug_mode` 守卫内。
- VTree 仍构建（结构需要），构建时不触发 probe。
- F12 off 时相对无 debug 路径**仅多一棵 VTree 构建开销**（O(节点数)，纯内存），computed 侧路完全旁路。

### 6.3 死代码清理

方案 B 落地后被 VTree + InspectorCache 取代：

1. **`ui/debug/DebugLayer`**（`populate_panel` 写死 Container 那套）→ **删除**。可复用纯类型（`Rect`/`BoxModel`/`EdgeInsets`）迁到 `inspector_cache.rs` 或公共模块。
2. **iced `DebugTreeNode` + `tree_enter/tree_exit` + `wrap_debug` 内 component_tree 构建**（`renderer.rs:3655+`）→ **移除**，左树改读 VTree。
3. **`DebugElementInfo` / `element_styles`** → kind/props/span 并入 ComputedNode，**移除**并行表。
4. **保留**：`LayoutCollector`（核心资产）、`hit_test`、`overlay`、`source_map`、`DebugIdMap`（VNode 的 `source_span` 仍靠它）。

清理顺序：先让新通路跑通（VTree 树 + InspectorCache 面板），再删旧通路，避免中间态断裂。

## 7. 测试策略

按"数据正确性 → 映射稳定性 → 端到端"三层，从底往上可独立验证，不依赖 iced 实际窗口。

### 7.1 单元测试（纯逻辑）

- **VTree 构建 + path 稳定 id**：同一 View 树构建两次 → VNodeId 完全一致；for 展开 N
  项 → N 个子节点 path 各异；增删节点只影响该子树 id。
- **computed_style 解析**：给定 class → `Style::parse` → 预期 (prop,val) 对（复用现有 style 测试）。
- **BuildProbe 采集**：带 `${.count}`/for/onclick 的 View → probe 按 path 对齐输出正确。
- **InspectorCache 回填**：喂入 `id_to_iced` + LayoutCollector bounds → 对应 VNodeId 的
  `bounds`/`box_model` 正确（content = bounds − declared padding）。

### 7.2 集成测试（headless 渲染器，已存在）

复用 `headless/renderer.rs` 的 `view_to_vtree` 路径：Auto 源码片段（含 for + 状态 +
样式）→ AuraViewBuilder → VTree → 断言树结构、节点数、for 展开数、每节点 source_span
非空。不依赖 iced，可在 CI 跑。

### 7.3 降级与不变量测试（防回归）

- "bounds 缺失 + class 解析失败 + 状态取不到"的 ComputedNode → 右栏渲染函数不 panic、返回有效结构。
- F12 off → `InspectorCache` 为 None，VTree 仍构建，断言 computed 侧路未被触发（计数器探针验证 probe 未调用）。

### 7.4 手动验收清单（需真实 iced 窗口）

- F12 开 → 三栏出现，左树结构匹配可见 UI。
- hover 应用元素 → 左树 + overlay 同步。
- 点击 for 列表第 3 项 → 右栏 AutoUI 标签显示 `for: item=..., i=2`，且再次渲染后选中仍停留该项（跨帧存活）。
- 改状态触发重渲染 → 右栏 state 当前值更新。
- F12 关 → UI 无 overlay、无明显帧率下降。

### 7.5 不写测试的部分

iced widget id 的 `aura_N` 字符串解析、overlay/hit_test 已有测试，保留。

## 8. 已确认的关键决策

- 方案 B：VTree 作结构真相源 + InspectorCache 侧表。
- 右侧面板展示：实时 Box Model、解析后 computed 样式、原始 class + props、源码定位链接（次要）、状态绑定+当前值、for 迭代溯源、事件处理器绑定、父节点链面包屑。
- VNodeId 改为由 path 派生（位置稳定）。
- AuraViewBuilder 加 `BuildProbe` 采集通道。
- 删除 `DebugLayer` 与 `DebugTreeNode` 两套并行实现，保留 `LayoutCollector`/`hit_test`/`overlay`/`source_map`/`DebugIdMap`。
