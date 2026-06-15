# AutoUI DevTools 风格实时检视器（重构 VM 版 F12 Debug）

- 状态：**主体已实现并合并至 master**（merge commit `930a6cd2`）；清理任务 19/20 与手动验收 21 待完成
- 日期：2026-06-13（设计 + 实现计划）；2026-06-15（合并、合并本文档）
- 范围：VM 版 AutoUI 的 iced 渲染路径，F12 Debug 检视功能重构

> 本文档由原 `307-autoui-devtools-inspector-design.md`（设计）与
> `307-autoui-devtools-inspector-impl.md`（实现计划）合并而成。

---

## 实现进度（2026-06-15 快照）

| 任务 | 标题 | 状态 | Commit |
|---|---|---|---|
| Task 1 | 给 VNode 增加 `path` 与 `source_span` 字段 | ✅ 已实现 | `638e7a4d` |
| Task 2 | `path` → 稳定 `VNodeId` 派生函数 | ✅ 已实现 | `6195f047` |
| Task 3 | 迁移 `Rect`/`BoxModel`/`EdgeInsets` 到可复用模块 | ✅ 已实现 | `042d4d13` |
| Task 4 | 带路径与 span 的 `view_to_vtree_with_paths` | ✅ 已实现 | `c7cf1b83` |
| Task 5 | iced 渲染器每帧构建 VTree | ✅ 已实现 | `0d83a882` |
| Task 6 | 定义 InspectorCache / ComputedNode | ✅ 已实现 | `bc08ff7a` |
| Task 7 | `computed_style` 解析 | ✅ 已实现 | `53b5a438` |
| Task 8 | `BuildProbe` 采集容器 | ✅ 已实现 | `0c4818db` |
| Task 9 | 采集状态绑定（深度贯穿真实 converter） | ✅ 已实现 | `f330fe19` |
| Task 10 | 采集 for 循环迭代上下文 | ✅ 已实现 | `58c2ef13` |
| Task 11 | 采集事件处理器绑定 | ✅ 已实现 | `f1ad493e` |
| Task 12 | iced 渲染器维护 `VNodeId ↔ aura_N` 映射 | ✅ 已实现 | `91d5a284` |
| Task 13 | 布局后 bounds 回填 + box_model | ✅ 已实现 | `2c2b4d29` |
| Task 14 | 左树改读 VTree | ✅ 已实现 | `994f0ccd` |
| Task 15 | 右栏面包屑 + Layout 标签 | ✅ 已实现 | `380de961` |
| Task 16 | 右栏 Computed/Props/AutoUI/Source 标签 | ✅ 已实现 | `15b59430` |
| Task 17 | Hover/Select 经 VNodeId 打通 + overlay | ✅ 已实现 | `34efe86d` |
| Task 18 | 性能门控（F12 off 零开销） | ✅ 已实现 | `4683acaf` |
| Task 19 | 删除死代码 `DebugLayer` | ⏳ 待清理 | — |
| Task 20 | 删除 iced 渲染器旧 debug 通路 | ⏳ 待清理 | — |
| Task 21 | 手动验收 + 文档 | ⏳ 待验收 | — |

> 相关：路径方案分歧（View-structural vs build-time）对 for 循环体节点造成
> AutoUI 标签降级，见 §6.1 与项目记忆 `307-devtools-path-schemes`。
> 后续 `render=rust` 模式的同类检视器见项目记忆 `rust-mode-devtools-parity`。

---

# 一、设计

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

> **实现注记（for 循环体节点）**：存在两套路径方案——View-结构路径（VTree VNodeId、
> bounds 回填）与构建时路径（`BuildProbe`，for 用 `[iter,body]` 两段编码）。两者在
> for 循环体节点处分歧，导致 AutoUI 标签的 probe 数据在该处按 VNodeId 查不到 → 显示
> "(本节点无 AutoUI 元数据)"。非循环节点两方案一致，查得到。属设计内可接受的降级，
> 不改动任一全局方案。

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

---

# 二、实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 把 VM 版 AutoUI 的 F12 检视从"源码视图"重构为 DevTools 风格的"computed DOM 视图"——VTree 成为 iced 路径的结构真相源，F12 门控的 `InspectorCache` 承载实时布局、解析样式、状态绑定、for 溯源、事件、源码定位。

**Architecture:** 方案 B（见本文「一、设计」）。VTree 每帧构建（精简），贵数据进 `InspectorCache`（仅 F12 on 时存在）。bounds 由现有 `LayoutCollector` 收集后经 `VNodeId ↔ aura_N` 映射回填。VNodeId 由 `path` 派生以让选中态跨帧存活。最后清理 `DebugLayer` / `DebugTreeNode` 两套并行实现。

**Tech Stack:** Rust；iced（UI 后端）；现有 `vnode.rs` / `view.rs` / `aura_view_builder.rs` / `iced/renderer.rs` / `iced/layout_collector.rs` / `style/`。

**测试约定：** 纯逻辑任务（VNode 字段、converter、cache、probe、style 解析）走 TDD 单元测试。iced 渲染器/UI 面板集成任务用"编译通过 + headless 路径集成测试 + 手动验收清单"，因为 iced widget 难以纯单元测试。每个任务结束 `cargo build -p auto`（CLAUDE.md 要求）后提交。

**关键参考文件：**
- 设计：见本文档「一、设计」
- 现有结构：`crates/auto-lang/src/ui/vnode.rs`、`vnode_converter.rs`、`view.rs`、`debug/mod.rs`、`iced/renderer.rs`、`iced/layout_collector.rs`、`style/{mod,class,layout_extract}.rs`、`aura_view_builder.rs`、`debug_id_map.rs`

**清理顺序铁律：** 先让新通路（VTree 树 + InspectorCache 面板）跑通，再删旧通路（DebugLayer / DebugTreeNode），避免中间态断裂。

---

## Phase 0 — 基础类型（纯逻辑，无集成）

### Task 1: 给 VNode 增加 `path` 与 `source_span` 字段 ✅

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
git commit -m "feat(ui): add path and source_span fields to VNode"
```

---

### Task 2: `path` → 稳定 `VNodeId` 派生函数 ✅

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

**Step 3: Commit** — `feat(ui): derive stable VNodeId from node path`

---

### Task 3: 迁移 `Rect` / `BoxModel` / `EdgeInsets` 到可复用模块 ✅

**Files:**
- Create: `crates/auto-lang/src/ui/debug/primitives.rs`
- Modify: `crates/auto-lang/src/ui/debug/mod.rs`（re-export，移除旧定义）

**Step 1:** 把 `debug/mod.rs` 中 `Rect`、`EdgeInsets`、`BoxModel` 及其 impl/测试 整体迁到新 `primitives.rs`。`debug/mod.rs` 改为 `mod primitives; pub use primitives::{Rect, EdgeInsets, BoxModel};`。

**Step 2:** `cargo test -p auto-lang -- debug::` 确保迁移后既有 box model 测试仍通过。

**Step 3: Commit** — `refactor(ui): move Rect/BoxModel/EdgeInsets to debug/primitives`

---

## Phase 1 — VTree 在 iced 路径每帧构建（结构层）

### Task 4: 带路径与 span 的新 converter `view_to_vtree_with_paths` ✅

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
        Some(crate::ui::debug::SourceSpan { offset: path.iter().map(|&x| x as usize).sum::<usize>(), len: 1 })
    });
    let root = tree.root().unwrap();
    assert_eq!(root.id.as_u64(), crate::ui::vnode::id_from_path(&[]));
    assert_eq!(root.path, vec![]);
    let again = view_to_vtree_with_paths(clone_view(), |_| None);
    assert_eq!(again.root().unwrap().id, root.id);
}
```

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

**Step 3: Commit** — `feat(ui): view_to_vtree_with_paths assigns path-derived ids + spans`

---

### Task 5: iced 渲染器每帧构建 VTree 并挂到 DynamicState ✅

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

**Step 3: Commit** — `feat(ui): build live VTree per frame in iced renderer`

---

## Phase 2 — InspectorCache 与 ComputedNode

### Task 6: 定义 InspectorCache / ComputedNode 及子结构 ✅

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

**Step 3: Commit** — `feat(ui): add InspectorCache + ComputedNode data model`

---

### Task 7: `computed_style` 解析（StyleClass → (prop,val)）✅

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
        _ => return None,
    }.map(|(k,v)| (k.to_string(), v)))
}
```

（`size`/`color` 按 `style/class.rs` 的 `SizeValue` 与 `style/color.rs` 的 `Color` 真实 variant 补全；先覆盖测试用到的 w-full/p-4/bg-blue-500。）

Run: `cargo test -p auto-lang -- style_probe_`
Expected: PASS。

**Step 3: Commit** — `feat(ui): compute_style parses class string into property pairs`

---

### Task 8: `BuildProbe` 采集容器（按 path 索引）✅

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

**Step 3: Commit** — `feat(ui): add BuildProbe to collect debug data by path`

---

## Phase 3 — BuildProbe 接入 AuraViewBuilder

> 本阶段把"构建时即可知"的 AutoUI 特有数据（状态绑定、for 上下文、事件）采集进 `BuildProbe`。集成点在 `aura_view_builder.rs`。
>
> **实现注记：** 实际落地采用**深度重构**——新增 `convert_node_tracked_ctx` /
> `convert_element_tracked_ctx` / `convert_column_tracked_ctx` 等"追踪双生子"，把
> `path` + `DebugIdMap` + `BuildProbe` 贯穿真实 element converter，使**所有嵌套节点**
> 都被 path 追踪，状态绑定在 VTree 显示处全部捕获（而非浅层委托）。

### Task 9: 采集状态绑定 `${.x}` ✅

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

**Step 4: Commit** — `feat(ui): BuildProbe captures state bindings in AuraViewBuilder`

---

### Task 10: 采集 for 循环迭代上下文 ✅

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

**Step 4: Commit** — `feat(ui): BuildProbe captures for-loop iteration context`

---

### Task 11: 采集事件处理器绑定 ✅

**Files:**
- Modify: `crates/auto-lang/src/ui/aura_view_builder.rs`（事件绑定处，`AuraNode::Element.events` / `AuraEvent`）

**Step 1: 定位** —— grep `events` / `AuraEvent` / `DynamicMessage` 找到把节点事件绑定到 handler 的位置。

**Step 2: 接入** —— 对每个绑定的 event 记录：`probe.record_event(path, event_name, handler_fn_name)`。

**Step 3: 验证** —— headless 测试：带 `onclick="handle_click"` 的按钮构建后，probe 对应 path 有 `events=[{onclick, handle_click}]`。

Run: `cargo test -p auto-lang -- probe_event` + `cargo build -p auto`

**Step 4: Commit** — `feat(ui): BuildProbe captures event handler bindings`

---

## Phase 4 — Bounds 回填

### Task 12: iced 渲染器维护 `VNodeId ↔ aura_N` 映射 ✅

**Files:**
- Modify: `crates/auto-lang/src/ui/iced/renderer.rs`

**Step 1: 背景** —— `wrap_debug`（`renderer.rs:3726`）已用 `debug_id_map.get(view_path)` 得 `AuraNodeId`，并生成 `id_str = format!("aura_{}_{}", base_id, counter_val)`。VTree 节点同样按 path 派生 VNodeId。

**Step 2: 接入** —— 渲染每个被 wrap 的节点时，把 `VNodeId(id_from_path(view_path)) ↔ id_str` 写入 `InspectorCache::set_iced_map`（仅 `debug_mode`）。需要让渲染循环能拿到当前 `view_path`（`wrap_debug` 已有 `view_path: &[usize]`，转 `Vec<u16>` 即可）。

**Step 3: 验证** —— 单元测试：模拟 set_iced_map 后，`iced_to_vnode`/`vnode_to_iced` 双向正确（已在 Task 6 覆盖）；此处保证渲染器调用点编译通过。

Run: `cargo build -p auto`

**Step 4: Commit** — `feat(ui): maintain VNodeId<->aura_N id map in renderer`

---

### Task 13: 布局后 bounds 回填 InspectorCache + box_model ✅

**Files:**
- Modify: `crates/auto-lang/src/ui/iced/renderer.rs`（LayoutCollector 结果消费处，grep `LayoutCollector` / `BoundsMap`）

**Step 1: 定位** —— `LayoutCollector` 产出 `BoundsMap = HashMap<String, (f32,f32,f32,f32)>`（`layout_collector.rs:16`）。找到该结果被消费的位置（当前用于 MCP snapshot）。

**Step 2: 回填** —— 在 `debug_mode` 下，对每条 `(iced_id_str, (x,y,w,h))`：
```rust
if let Some(cache) = inspector_cache_opt.as_mut() {
    if let Some(vnid) = cache.iced_to_vnode(&iced_id_str) {
        let node = cache.get_mut_or_default(vnid);
        node.bounds = Some(Rect::new(x,y,w,h));
        let pad = parse_padding(node.raw_class.as_deref());
        node.box_model = Some(BoxModel::new(
            Rect::new(x + pad.left, y + pad.top, (w - pad.left - pad.right).max(0.0), (h - pad.top - pad.bottom).max(0.0)),
            pad, EdgeInsets::default(),
        ));
    }
}
```

**Step 3: 验证** —— 单元测试 `inspector_cache_bounds_backfill`：构造 cache + id_map + bounds map → 回填后 `get(id).bounds` 与 box_model.content 正确（content = bounds − padding）。

Run: `cargo test -p auto-lang -- inspector_cache_bounds_backfill` + `cargo build -p auto`

**Step 4: Commit** — `feat(ui): backfill bounds + box_model into InspectorCache post-layout`

---

## Phase 5 — 面板（左树 + 右栏）

> 面板代码集中在 `iced/renderer.rs` 的 debug UI 渲染区（grep `render_inspector_tab`、`render_tree_into`、`component_tree`）。本阶段把数据源从旧的 `DebugTreeNode`/`element_styles` 切到 VTree + InspectorCache。

### Task 14: 左树改读 VTree ✅

**Files:**
- Modify: `crates/auto-lang/src/ui/iced/renderer.rs`（左树渲染函数，~`renderer.rs:3114` `render_tree_into`）

**Step 1:** 新写/改写左树渲染：从 `state.live_vtree` 递归渲染。每行 `kind` + 摘要（text 取 props.content 前 N 字符；list/col 显示子节点数）。展开/折叠状态可先用"全展开"。

**Step 2:** 点击行 → 选中 `VNodeId` → 写入 `state.selected_vnode`（新字段，替代旧 `selected_widget: String`）。hover 行 → 写入 `state.hovered_vnode`。

**Step 3:** 编译 + 手动验收（左树出现、结构匹配可见 UI）。

Run: `cargo build -p auto`，手动运行 F12。

**Step 4: Commit** — `feat(ui): left tree reads from live VTree`

---

### Task 15: 右栏面包屑 + Layout 标签 ✅

**Files:**
- Modify: `crates/auto-lang/src/ui/iced/renderer.rs`（`render_inspector_tab`，~`renderer.rs:3284`）

**Step 1:** 顶部面包屑：从选中 VNodeId 沿 `vnode.parent` 上溯到根，渲染可点击 chip，点祖先 → 设 `selected_vnode`。

**Step 2:** Layout 标签：读 `cache.get(selected).box_model`，画 Box Model 嵌套矩形 + 数值；padding 行标注"(声明值)"。bounds 缺失时显示"(布局中…)"。

**Step 3:** 编译 + 手动验收。

Run: `cargo build -p auto`

**Step 4: Commit** — `feat(ui): breadcrumb + Layout (box model) tab`

---

### Task 16: 右栏 Computed / Props / AutoUI / Source 标签 ✅

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

**Step 4: Commit** — `feat(ui): Computed/Props/AutoUI/Source inspector tabs`

---

### Task 17: Hover/Select 经 VNodeId 打通 + overlay ✅

**Files:**
- Modify: `crates/auto-lang/src/ui/iced/renderer.rs`（hit-test、overlay、`update_hover`/`select_hovered` 调用处）

**Step 1:** 应用区鼠标移动 → 用 `cache` 中各节点 `bounds` 做 hit-test（复用 `debug::hit_test`），命中 → `hovered_vnode`，左树同步高亮、overlay 蓝框。点击 → `selected_vnode`、overlay 橙框。

**Step 2:** 现有"源码点击 → 高亮"经 `iced_to_vnode` 映射到 VNodeId 后走同一选中通路。

**Step 3:** 手动验收 hover/选中联动。

Run: `cargo build -p auto`

**Step 4: Commit** — `feat(ui): hover/select via VNodeId + overlay`

---

## Phase 6 — 性能门控 + 清理

### Task 18: 性能门控（F12 off 零开销）✅

**Files:**
- Modify: `crates/auto-lang/src/ui/iced/renderer.rs`、`aura_view_builder.rs`

**Step 1:** `InspectorCache` 改为 `Option<InspectorCache>`（DynamicState 字段），仅 F12 on 时 `Some`。所有填充/映射调用包在 `if let Some(cache) = …` 内。

**Step 2:** `BuildProbe` 同理：仅 `debug_mode` 时构造并贯穿 `build`；`aura_view_builder` 的 record_* 调用前判 `probe.is_some()`。

**Step 3:** VTree 仍每帧构建（结构需要），但 `span_for` 闭包在 `debug_mode` off 时直接返回 `None`（跳过 DebugIdMap 查询）。

**Step 4:** 防回归测试：用计数器探针断言 F12 off 时 `compute_style` / probe record 未被调用。

Run: `cargo test -p auto-lang -- f12_off_zero_overhead` + `cargo build -p auto`

**Step 5: Commit** — `perf(ui): gate BuildProbe/InspectorCache behind debug_mode`

> **实现注记：** 落地为 `BuildProbe::new_disabled()` + `view_with_debug_gated(capture)`：
> MCP 同步路径传 `false`（不需要 probe），渲染路径传 `state.debug_mode`。`record_*`
> 在 `!enabled` 时早退。`live_cache`/`live_probe` 仅在 `debug_mode` 时保留，否则 `None`。

---

### Task 19: 删除死代码 `DebugLayer` ⏳

**Files:**
- Modify: `crates/auto-lang/src/ui/debug/mod.rs`

**Step 1:** 确认 `DebugLayer` / `DebugPanel` / `DebugState` / `LayoutReporter` 已无任何调用（grep 全仓）。

**Step 2:** 删除这些类型及其测试。保留 `primitives.rs`（Rect/BoxModel/EdgeInsets，已迁）、`hit_test`、`overlay`、`source_map`。

**Step 3:** `cargo build -p auto` + `cargo test -p auto-lang -- debug`。

**Step 4: Commit** — `refactor(ui): remove dead DebugLayer (replaced by InspectorCache)`

---

### Task 20: 删除 iced 渲染器旧 debug 通路 ⏳

**Files:**
- Modify: `crates/auto-lang/src/ui/iced/renderer.rs`

**Step 1:** 确认新通路（VTree 树 + InspectorCache 面板）已完全接管后，删除：
- `DebugTreeNode` 及 `tree_enter/tree_exit`、`component_tree`、`tree_stack`（~`renderer.rs:3655-3723`）。
- `DebugElementInfo` / `element_styles` / `debug_element_styles`（~`renderer.rs:1906,2199,2900,3286`）。
- `wrap_debug` 内构建 component_tree / element_styles 的副作用（保留 bounds-probe container、mouse_area、hover/select）。

**Step 2:** `cargo build -p auto` + 全量 `cargo test -p auto-lang`。

**Step 3: Commit** — `refactor(ui): remove DebugTreeNode/element_styles superseded by VTree+InspectorCache`

---

### Task 21: 手动验收 + 文档 ⏳

**Files:**
- Modify: `docs/design/08-ui-systems.md`（若有 debug 章节）或新增 `docs/design/` 检视器说明

**Step 1:** 按 design §7.4 执行手动验收清单：
- F12 开 → 三栏出现，左树匹配可见 UI。
- hover 应用元素 → 左树 + overlay 同步。
- 点 for 列表第 3 项 → 右栏 AutoUI 显示 `for: item=…, i=2`，且重渲染后选中仍停留（跨帧存活）。
- 改状态重渲染 → 右栏 state 当前值更新。
- F12 关 → 无 overlay、帧率无明显下降。

**Step 2:** 更新文档描述新检视器（VTree = runtime DOM、InspectorCache、各标签含义）。

**Step 3: Commit** — `docs(ui): document DevTools-style live inspector`

---

## 验收标准（Definition of Done）

- [x] VTree 在 iced 路径每帧构建，VNodeId 由 path 派生、结构不变即稳定。
- [x] `InspectorCache`（F12 门控）含 bounds/box_model/computed_style/state_bindings/for_context/events/source。
- [x] 左树读 VTree，右栏含面包屑 + Layout/Computed/Props/AutoUI/Source 标签。
- [x] hover/选中经 VNodeId 联动 + overlay；选中态跨帧存活。
- [x] F12 off 时 computed 侧路完全旁路（零额外开销）。
- [ ] `DebugLayer` 与 `DebugTreeNode`/`element_styles` 已删除；`LayoutCollector`/`hit_test`/`overlay`/`source_map`/`DebugIdMap` 保留。
- [x] `cargo test -p auto-lang`（307 相关：build_probe 6 / aura_view_builder 18 / inspector_cache 10）全绿；`cargo build -p auto` 通过。
- [ ] 手动验收清单全过。

## 风险与回退

- **风险**：`aura_view_builder.rs` / `renderer.rs` 体量大、改动深，可能引入回归。
  **缓解**：每任务独立提交；Phase 3/5 集成任务后跑 `cargo test -p auto-lang`；新通路跑通（Phase 5 末）前不删旧通路。
- **风险**：path 派生 id 在条件渲染（`if` 分支）下，同 path 可能映射到不同节点 → 选中错位。
  **缓解**：验收 Task 21 专项检查条件渲染场景；必要时 path 编码加入分支标记。
- **风险**（已发现）：两套路径方案在 for 循环体节点分歧 → AutoUI 标签查不到 probe 数据。
  **缓解**：按 §6.1 降级显示，不改动全局方案。
- **回退**：任一阶段失败可 `git revert` 该阶段提交，旧通路（删之前）仍可用。

---

# 续篇：Chrome-DevTools-Style Refinements（原暂存计划「Plan 309」）

> 本节为 307 检视器落地后的细化工作。用户原始诉求：「请参考 DevTool（Chrome），继续细化 F12 Debug 的更多内容。」经 AskUserQuestion 确认实施全部四个方向。因 `docs/plans/309` 已被 ash roadmap 占用，本续篇按用户决定合并写入 307（下文沿用「细化阶段」表述）。

## 背景：四个差距

1. **AutoUI 数据标签对 `for` 循环体节点为空**（最常见场景：列表每项一个 widget）。根因：tracked 构建路径与扁平化 VTree 路径方案分歧。
2. **Box Model + Computed Style 显示 0 / 「(待入)」** —— `ComputedNode.computed_style`/`raw_class`/box-model insets 从未写入。
3. **事件监听器 + 源码导航** —— Source 子标签是占位符；行点击设置了错误的状态字段。
4. **元素检视光标模式** —— hover overlay 常开（噪声大）；无 Chrome 式取色器开关。

## 细化阶段设计（5 个独立、可回滚阶段，按 1→5 实施）

### 阶段 1 — 路径调和（Fix A）

**问题：** tracked 构建对 ForLoop 体节点记录 `[p, i, 0]`（push `[iter, body_index]`），但 `body.len()==1` 时迭代直接产出裸 body view（无包裹 Column），VTree 实际路径是 `[p, i]`。路径不匹配 → `probe.snapshot().get(&node.path)` 落空 → AutoUI 标签为空；`debug_id_map.get(view_path)` 落空 → 走合成回退分支。

**修复：** 仅当 body 节点数 >1 时才 push body-index 段（ForLoop 与 Conditional 两处同形 bug）。

- `aura_view_builder.rs` ForLoop 臂：`let body_len = body.len();` 后 `path.push(i); if body_len > 1 { path.push(bi); }`，pop 对称。
- Conditional 臂：同样门控。
- 新增测试：`build_with_debug_for_loop_single_body_path_matches_vtree`（断言键是 `[0],[1]` 而非 `[0,0]`）、`build_with_debug_for_loop_multi_body_path_keeps_body_index`。

**已知范围外限制（记录不修）：** `matches_search` 会过滤迭代、使后续 VTree 子索引偏移 —— 既有问题，见 307 Known Issues。

### 阶段 2 — Computed style + raw_class 填充

- **2a computed_style：** `wrap_debug` 把已解析 `props`（`debug_style_props` 输出）克隆写入 `ComputedNode.computed_style`（aura 与回退两分支）。
- **2b raw_class：** `ProbeEntry` 增 `raw_class: Option<String>` + `record_raw_class`（disabled 或 None 时 no-op，避免无 class 元素产生空条目）；`convert_element_tracked_ctx` 经 `extract_string(props,"class")`（回退 `"style"`）记录；renderer 脏构建分支把 probe snapshot 按 path→VNodeId 合并进 cache。
- **2c Computed 标签：** 占位符 `(CSS class 解析待入)` 替换为 `class` 行 + 遍历 `computed_style`；皆空时才回退 `(无 computed 样式)`。
- 测试：`record_raw_class` 往返 + disabled no-op。

### 阶段 3 — Box Model 图 + 各边 insets

- **3.1 `BoxModel.border`：** `primitives.rs` 增 `border: EdgeInsets` 字段、`border_box()`（padding_box+border）；`margin_box()` 改为包 border_box；`EdgeInsets` derive 加 `PartialEq`；`render()` 增 Border 行。
- **3.2 `backfill_bounds` 保留 insets：** 不再用零 insets 的 `from_bounds` 覆盖；保留 wrap_debug 写入的 padding/border/margin，从测量 rect（=border-box）减去 padding+border（`.max(0.0)` 钳制）派生 `content`。
- **3.3 `wrap_debug` 读 Style：** 增 `style: Option<&Style>` 参数；新增 `debug_style_insets(style)`（per-side > axis > uniform；border 取 `border_width` 统一值），把含 insets 的 `BoxModel` 写入 cache；7 处调用点更新（input/col/row/container/scroll 传 `style.as_ref()`；textarea 传 `None`；catch-all 先 `extract_view_style(&view).cloned()` 再 move）。
- **3.4 Layout 标签图：** `render_box_model_diagram(bm)` 嵌套容器 margin→border→padding→content，着色背景 + 图例条，insets 封顶 28px；插入「盒模型」标题下方；「Border box」行改用 `bm.border_box()`。
- 测试：`box_model_border_box`、`backfill_bounds_preserves_declared_insets_and_derives_content`。

### 阶段 4 — 源码导航：真实 viewer + 双向选中

- **4.1：** 抽出 `fn render_source_viewer(state, highlight_range)` + `fn ensure_source_loaded(state)`（懒加载源码/行偏移/高亮缓存/line→AuraNodeId 索引）；`render_inspector_source_tab` 由选中 VNode 的 `source_span` 经 partition_point 解析为 0 基半开 `(start,end)` 高亮区间，委托 viewer；删除 `(点击导航待接入)` 占位符。
- **4.2：** `SRC_CLICK_PREFIX` 处理器在设 `selected_widget` 后，经 `live_cache.iced_to_vnode(&debug_id)` 派生 `selected_vnode`（右栏以 VNodeId 为键，此前点击源码行后右栏为空）。
- **4.3：** `DEBUG_SELECT_VNODE_PREFIX` 处理器增 `ensure_source_loaded` + 按 VNode span 设 `pending_scroll_to_center`（**有意偏离计划**：计划建议在 render 内设，实际放进 update 处理器以保持 render 无副作用）。

### 阶段 5 — 元素检视光标模式

- **5.1：** `DynamicState` 增 `inspect_mode: RefCell<bool>`（默认 false）；F12 关闭 / `__close_devtools` / DEBUG_SELECT_PREFIX 选中后三处复位；`__toggle_inspect` 处理器（开启时强制 `debug_mode=true`+`devtools_open=true`）。
- **5.2：** `render_devtools_panel` 增「🔍 检视」开关按钮（`tab_style_fn(inspect_active)` 高亮），发 `__toggle_inspect`。
- **5.3：** `DebugRenderCtx` 增 `inspect_mode: bool`；`wrap_debug` 末尾门控由 `if !self.debug_mode` 改为 `if !(self.debug_mode && self.inspect_mode)`。bounds-probe 容器 + InspectorCache 写入 + `set_iced_map` 均在门控之前，MCP 快照与 cache 不受影响。

## 实施结果（截至本节写入）

| 阶段 | 状态 | 单测 | 构建 |
|---|---|---|---|
| 1 路径调和 | ✅ 完成（spec+质量两段评审 APPROVED） | `ui::debug` 80 / `aura_view_builder` 20 通过 | ✅ |
| 2 Computed style + raw_class | ✅ APPROVED | 通过 | ✅ |
| 3 Box Model 图 + insets | ✅ APPROVED | 通过 | ✅ |
| 4 源码导航 + 双向选中 | ✅ APPROVED（记录 1 处有益偏离） | 通过 | ✅ |
| 5 检视光标模式 | ✅ APPROVED（采用计划字面全门控，非拆分门控） | 通过 | ✅ |

**改动文件：** `crates/auto-lang/src/ui/iced/renderer.rs`、`crates/auto-lang/src/ui/aura_view_builder.rs`、`crates/auto-lang/src/ui/debug/primitives.rs`、`crates/auto-lang/src/ui/debug/inspector_cache.rs`、`crates/auto-lang/src/ui/debug/build_probe.rs`。

**关键设计判定（评审中确认）：**
- 阶段 3 测量 rect 模型 = border-box；`content = 测量 − padding − border`；margin 仅声明（iced 不测量）。
- 阶段 4 `SRC_CLICK` 的 `debug_id` 与 `live_cache.iced_to_id` 键同形（均为 `aura_N` / `aura_N_cnt`），查表不会静默失败。
- 阶段 5 门控要求 F12 **与** 检视模式同时开；普通 F12 模式下应用内点击不再直接选中（选中改经 Elements 树，发 `DEBUG_SELECT_VNODE_PREFIX`）。评审认定此为可接受的、噪声更低的模型；如需「普通 F12 也能点击选中」可局部拆分门控（提 mouse_area+selected 至门控前，仅 hover 分支保持门控）。

**验证：**
- `cargo build -p auto` 通过（仅既有 warning，无新增 error）。
- `ui::debug`（80）、`ui::aura_view_builder`（20）全绿。
- `view::tests`/`style::layout_extract`/`vm_bridge` 中 4 个失败为 **既有失败**（git stash 验证基线即失败，且不在本次触及文件中）。

**⚠️ 仍待人工验收（无法 headless 验证）：** VM 模式运行 `examples/ui/015-notes`，F12 后点一个 note 按钮（循环体），确认：
1. AutoUI 标签显示 `for_context` + 事件（阶段 1，此前为空）。
2. Computed 标签显示真实 `class` + 样式属性（阶段 2）。
3. Layout 标签显示彩色盒模型图 + 各边数值（阶段 3）。
4. Source 标签源码居中于该元素；行点击可回选（阶段 4）。
5. 🔍 检视开关门控 hover overlay；点击后自动退出（阶段 5）。

**未提交：** 遵循项目约定（除非用户要求否则不提交）。视觉验收通过后再统一提交。

## 与 307 既有「验收标准」的衔接

- 上文 DoD 中「手动验收清单全过」仍待本续篇人工验收完成后勾选。
- 「`DebugLayer` 与 `DebugTreeNode`/`element_styles` 已删除」仍为 ⏳：本续篇未删除（`render_inspector_source_section` 仍 `#[allow(dead_code)]` 保留，与 `render_source_viewer` 逻辑重复），归入 307 Task 19/20 清理批次统一退役，避免两份实现漂移。
- 路径方案分歧项：阶段 1 已对常见单 body 循环场景调和，DoD 中「按 §6.1 降级」风险项对应不再是常见失败路径。
