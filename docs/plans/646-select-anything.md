---
plan_id: PLAN-646
status: reviewed              # drafting → executing → execution_done → reviewed → archived
feature_name: select-anything
author: [zcode-agent]
created_at: 2026-09-18
updated_at: 2026-09-18
plan_revision: 1

supersedes_spec_components: []
new_spec_components: [docs/specs/auto-lang/ui/design/select-anything.md]
touched_goals: [GOAL-007, GOAL-014, GOAL-015]   # 双端能力一致 / MCP 工具 / Agent 知识采集输入

affects: [docs/specs/auto-lang/ui, docs/specs/auto-lang/mcp]
current_step: 11
total_steps: 11
---

# [PLAN-646] Select Anything —— 任意 AutoUI 基面框选 → 结构化 Auto/JSON 回吐

## 变更摘要

为 AutoUI 增加"Select Anything"通用能力：用户在**任意 AutoUI 基面**（VM/Iced 桌面端 +
Vue 浏览器端）对任意范围 **Alt+拖拽框选**，系统按组件组织结构返回选中组件的
**Auto 源码切片 + 结构化 JSON + Atom 结构文本**，一键复制。同时新增 MCP 工具
`autoui_select_rect`，让 AI agent 可程序化按矩形采集界面知识。这是把 AutoOS
升级为个人知识库系统的"划词采集"地基：任何界面上所见的信息，都可被选中、
结构化、复制，作为知识采集或问答输入。

## 目标

1. **统一选择语义**（双端一致）：框选矩形 → 中心点命中的节点集合 → 修剪为最顶层
   （父不在集合内）→ 按 VTree/DOM 文档序输出。
2. **统一结果信封**：`{surface, app, rect, nodes[{kind, span, source, structure}]}`，
   三种呈现——Auto 源码文本（原始 .at 切片、去公共缩进）、JSON、Atom。
3. **VM/Iced 端可用**：Alt+拖拽框选（免开 F12），松开后 DevTools 新 Select 标签页
   展示结果，复制按钮写剪贴板。
4. **Vue 端可用**：`auto run` 开发页上 Alt+拖拽，浮动结果面板展示同格式结果并可复制。
5. **MCP 程序化采集**：`autoui_select_rect(x,y,w,h,format)` 返回同一信封。

### 非目标（v1 明确不做）

- 桌面多虚拟窗**跨窗**框选（v1 只作用于聚焦/命中的单个 App，坐标经 vwin 矩形平移）。
- gallery/desktop 宿主、生产构建的常驻 UI（Vue 端仅 dev 脚手架接线；宿主接入为后续）。
- 文本级划词选择（组件粒度即可；文本选择已有 SelectableText）。
- 知识库的存储/检索/问答系统本身（本计划只做"采集出口"）。
- Vue 端浮动面板与 VM 端 DevTools 标签页的 UI 形态统一（语义一致即可）。

## 架构方案

```
                     ┌─ 选择语义(纯函数) ─┐
框选矩形 rect ──────►│ center-inside 命中 │──► 顶层修剪 ──► 文档序排列 ──► topmost 节点集
 (Alt+拖拽)          └───────────────────┘                                      │
                                                                              ▼
   ┌────────────────────────── 结果构建(纯函数) ──────────────────────────────┐
   │ per node: source_span 切片 AppState.source_code(.at 原文,去公共缩进)      │
   │           VTreeAtomBuilder(scope=node) → Atom 文本 / node_to_json → JSON  │
   │ envelope: {surface, app, rect, nodes[{kind, span, source, structure}]}   │
   └──────────────────────────────────────────────────────────────────────────┘
        │                    │                          │
        ▼                    ▼                          ▼
  VM/Iced 端            Vue 端                     MCP autoui_select_rect
  DevTools Select tab   浮动面板+clipboard          agent 程序化采集
  (clipboard_set)
```

- **VM 端挂点**：全局鼠标三段流已就绪（`GlobalPress` / `__mouse_moved|x,y` /
  `__mouse_released`，renderer.rs:18869/18853/18860）；bounds 按需收集
  （`needs_bounds` → LayoutCollector → `__bounds_collected`，renderer.rs:16884）；
  复制走现成 `clipboard_set`（ui/clipboard.rs:13）。
- **Vue 端挂点**：`ui_gen/vue.rs node_to_html`（:6323）注入 `data-auto-{id,tag,span}`
  （AuraNode 自带 debug_id+span，当前被丢弃——唯一注入点）；`auto-man/src/vue.rs`
  脚手架（main.ts/index.html 每次重写）注入 dev-only overlay 资产与源码映射。
- **MCP 挂点**：`ui/mcp_server.rs` dispatch_static（:1120）现有 `autoui_vtree`
  （:929）模式直接复制扩展。

## 需求分析与背景调查

（来源：用户需求描述 2026-09-18 + 两轮代码勘察；ChatGPT 讨论链接在当前环境不可达——
空白页，见 §10。设计以用户文字描述为准。）

### 用户需求原文要点

- 对任意 AutoUI 基面的**任意范围**选取，app 自动返回**符合结构的 Auto/JSON 格式文本**，
  方便复制——"相当于把用户选中的组件（按照它们的组织结构）把 AutoUI 源码返回"。
- 未来做通用划词查询工具：从 AutoUI 系统随时划取信息，作为**知识采集或用户问答的输入**；
  是 AutoOS 升级为**个人知识库系统**的重要一步。

### VM/Iced 端现状（勘察结论，路径相对 `crates/auto-lang/src/`）

- **span→源码反查链已全通**：`AuraNode` 每变体带 `span + debug_id`
  （aura/types.rs:857）；`AuraWidget.span_map`（types.rs:110）；`DebugIdMap`
  View path→AuraNodeId（ui/debug_id_map.rs:17）；`VNode.source_span`（ui/vnode.rs:312，
  填充于 renderer.rs:19060-19070）；源码全文缓存 `AppState.source_code` +
  `source_line_offsets`（ui/session.rs:51-52）。**选中节点→切片原文零新增基建**。
- **bounds 基建**：`InspectorCache.by_id: HashMap<VNodeId, ComputedNode>` 含 bounds
  （ui/debug/inspector_cache.rs:101）；采集 `LayoutCollector`（ui/iced/layout_collector.rs:20，
  识别 `aura_N`/`vnode_<hash>` 两种 id 约定）；按需触发 `needs_bounds`
  （renderer.rs:16884→15204 `__bounds_collected`→backfill_bounds）。
- **点级 hit-test 参照**：`ui/debug/hit_test.rs:23`（含点最小面积节点）；本计划
  将其推广为矩形+中心语义。
- **全局鼠标流**：窗口订阅 CursorMoved/ButtonReleased/ButtonPressed(Left→`WmCommand::GlobalPress`)
  （renderer.rs:18853-18880）；修饰键 `current_modifiers` 已持续维护（Alt 检测可用）；
  F12 inspect 拾取器（INSPECT_CAPTURE，renderer.rs:38/19008）是交互参照。
- **序列化**：`VTreeAtomBuilder`（ui/vtree_atom.rs:66）拓扑 1:1、支持 `scope` 子树裁剪、
  经 `auto_val::Node` Display 输出 Atom 文本——结构化输出直接复用。
- **剪贴板**：`clipboard_set(text) -> bool`（ui/clipboard.rs:13，code_editor 已用）。

### Vue 端现状（勘察结论）

- 渲染链：.at → AuraWidget → `VueGenerator`（ui_gen/vue.rs:383）→ SFC →
  `gen/front/vue/` → Vite dev（auto-man/src/vue.rs:5201 `run_vue_project` 六步）。
- **生成的 DOM 无任何可反查标记**；但 AST 层信息齐备（同上 span/debug_id）且
  `ui_gen/vue.rs` 引用 debug_id 次数为 0——`node_to_html`（:6323）是唯一注入点。
- **浏览器→宿主无反向通道**（无 ws/postMessage），故 Vue 端结果面板在页内自闭环
  （overlay TS + DOM 采集 + 页内源码映射），不经宿主。
- 脚手架自愈：index.html/main.ts/package.json 每次 `auto run` 重写
  （vue.rs:1122-1220 main.ts 等）——dev-only 资产注入点现成。
- 增量编译：`incremental_compile_changed`（vue.rs:4747）按 hash 增量重写 SFC——
  源码映射文件同法防抖。
- gallery 模式 demo 已内嵌完整 .at `source` 字段（demos-registry，vue.rs:6555）——
  "源码随身携带"有先例，非目标范围但佐证可行。

### 授权与约束记录

- 用户已授权：按标准 L1 流程立项（本计划）。执行范围：本仓 auto-lang
  （crates/auto-lang、crates/auto-man）+ 脚手架生成的 dev 资产。
- 预算/自动续跑限制：未指定。
- 红线遵守：worktree 内禁止 junction/symlink；实现全部在
  `D:/autostack/.wt/lang-646/auto-lang` 进行。

## 详细设计

### 1. 选择语义（纯函数，双端各自实现、语义同一）

新模块 `crates/auto-lang/src/ui/selection/mod.rs`：

```rust
/// 中心点包含：节点 bounds 矩形中心落在框选矩形内 → 命中。
/// 部分被框选边缘扫过但中心不在 → 不选中（避免框到半个大容器就吞掉整容器）。
pub fn select_nodes(rect: Rect, bounds: &HashMap<VNodeId, Rect>) -> HashSet<VNodeId>

/// 顶层修剪：命中集合中父节点也命中的子节点剔除；输出按 VTree 文档序排列。
/// （框选完整覆盖卡片 → 卡片命中、其子也命中 → 只输出卡片本身 = 组织结构语义）
pub fn trim_to_topmost(selected: &HashSet<VNodeId>, vtree: &VTree) -> Vec<VNodeId>
```

- 依据：DevTools 深度优先取最深的直觉（hit_test.rs:1 注释）与"整块覆盖返回整块"
  的组织结构语义，中心包含是两者的平衡（完全包含判定会在"覆盖 95% 的大卡片"
  场景退化为返回其父容器）。
- 无 bounds 的节点（LayoutCollector 只覆盖挂 id 的 widget）不参与命中，但作为
  选中节点的子孙出现在 structure/source 输出中（VTree 拓扑本身完整）。

### 2. 结果信封（纯函数，`ui/selection/output.rs`）

```rust
pub struct SelectionResult {
    pub surface: &'static str,        // "vm" | "vue"
    pub app: String,                  // App 名/id
    pub rect: (f32, f32, f32, f32),   // x,y,w,h
    pub nodes: Vec<SelectedNode>,      // 文档序
}
pub struct SelectedNode {
    pub kind: String,                  // kind_keyword / data-auto-tag
    pub span: Option<(usize, usize)>,  // .at 字节偏移+长度
    pub source: Option<String>,        // 原始切片（去公共缩进）
    pub structure: auto_val::Node,     // VTreeAtomBuilder 子树（VM）/ DOM 子树（Vue JSON）
}
```

- **Auto 文本**（面板默认视图）：

  ```
  // ── AutoUI Select Anything ── surface=vm app=<app> rect=(x,y,w,h) nodes=N
  // [1/N] card  span=off..off+len
  <去公共缩进后的 .at 原文切片>
  // [2/N] ...
  ```

  切片取自 `AppState.source_code`（VM）或页内源码映射（Vue）按 span 字节区间；
  span=None 的合成节点标注 `// (synthetic, no source span)` 并仅输出结构。
- **JSON**：`SelectionResult` 直接 serde 序列化；`structure` 经本地
  `node_to_json(&auto_val::Node) -> serde_json::Value` walker 转换
  （~40 行；不动 auto-val crate——`Value::Node` 的 JSON serde 面不确证，本地转换零风险）。
- **Atom**：`VTreeAtomBuilder::build(snap, VTreeAtomOptions{scope: Some(node), ..})`
  的 Display 文本，逐节点拼接。
- `source` 切片与源文件一致性是验收锚点（切片必须逐字节等于源码子串，再缩进归一）。

### 3. VM/Iced 端交互

- **状态**（`DevToolsState` 扩展，ui/session.rs:108）：
  `marquee: RefCell<Option<Marquee{anchor,current}>>`、
  `pending_selection: RefCell<Option<Rect>>`、
  `selection_result: RefCell<Option<SelectionResult>>`、
  `select_tab: RefCell<SelectionView>`（auto/json/atom 三视图切换）。
- **进入**：无独立模式开关——`GlobalPress` 且 `alt_held` 且无文本编辑焦点 → 置位
  marquee.anchor（=last_cursor），同时抑制本次 GlobalPress 的 WM 焦点抢占
  （Alt+拖拽是专用手势，与 inspect 模式的 Alt-click passthrough 互斥域不同层）。
- **拖拽**：`__mouse_moved|x,y` 更新 marquee.current；视图层在 marquee 激活时于根
  注入绘制层（Stack+Canvas 程序或 View::Overlay，实现择一）画半透明矩形+边框；
  不捕获事件（纯绘制）。
- **结束**：`__mouse_released` → 定稿 rect → `needs_bounds=true` + 暂存
  pending_selection → `__bounds_collected` 臂中若有 pending 则执行
  select_nodes → trim → build_selection_result（保证 bounds 新鲜）→ 打开
  DevTools 到 Select 标签页（`__toggle_inspect` 同款联动：debug_mode=true、
  devtools_open=true，renderer.rs:15517 模式）。
- **坐标**：independent 模式窗口坐标直通；desktop 模式按聚焦窗 `vwin_rect` 平移到
  App 本地系（跨窗不支持，见非目标）。
- **复制**：Copy 按钮调 `clipboard_set(&当前视图文本)`；面板按钮态给出成功/失败反馈。
- **Esc**：关闭面板并清 marquee。

### 4. MCP 工具 `autoui_select_rect`（ui/mcp_server.rs）

- 注册进 `dispatch_tool_static`（:1120）与工具清单（`autoui_vtree` :929 同款）。
- 参数：`{x, y, w, h, format?: "auto"|"json"|"atom" (默认 "json"), include_box/style/events/source?: bool}`。
- 实现：读共享快照（`autoui_vtree` 同源）+ `layout_bounds` → 调 §1/§2 纯函数 →
  返回信封文本/JSON。agent 侧即获得与人工框选完全一致的采集通道。

### 5. Vue 端

- **标记注入**（ui_gen/vue.rs `node_to_html` :6323）：元素型 AuraNode 统一追加
  `data-auto-tag="<widget 关键字>"`、`data-auto-id="aura_N"`（有 debug_id 时）、
  `data-auto-span="off:len"`（有 span 时）。无开关、恒定输出（属性惰性无害，
  避免新配置面）；text 节点不注入（归父元素）。
- **源码映射**（auto-man/src/vue.rs）：脚手架新增
  `gen/front/vue/src/auto-sources.ts`——`export const AUTO_SOURCES: Record<string,string>`
  （key=App 名，value=完整 .at 源文）。`run_vue_project` 与
  `incremental_compile_changed` 同步重写（内容 hash 防抖，同 SFC 增量法）。
- **overlay 资产**（脚手架 dev-only 写入，非产物构建面）：
  - `src/auto-select/overlay.ts`：Alt+mousedown 全页（capture 阶段）起笔，画
    position:fixed 蒙层矩形；mouseup 收割
    `document.querySelectorAll('[data-auto-span]')`，`getBoundingClientRect()`
    中心包含 + parentElement 链顶层修剪（同 §1 语义）→ 按 data-auto-span 切片
    AUTO_SOURCES + DOM 子树→JSON（剥 data-auto-* 后的 tag/attrs/text/children）→
    浮动面板（fixed 右下，tab：Auto/JSON，Copy 按钮
    `navigator.clipboard.writeText`）。
  - `main.ts` 顶部 dev-only `import './auto-select/overlay'`。
- **多 App 页面**（desktop/gallery 宿主）：v1 不接（非目标）；overlay 按
  data-auto-span 所在 App 源映射键取源，单 App 场景天然成立。

### 6. 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | auto-lang/ui/design/select-anything.md | before：无此能力契约 → after：选择语义（中心包含+顶层修剪+文档序）、结果信封三格式、双端交互键位与面板行为、MCP 工具契约定型 | 双端+MCP 三消费面必须共享同一语义契约，防止后续漂移 | AC-01/02/07 |
| SD-02 | modify | auto-lang/ui/overview.md | before：无 Select Anything 叙述 → after：现状段落补记能力、挂点（ui/selection、node_to_html 标记、脚手架资产） | module spec 现状跟踪义务（规约 §4） | AC-03/06 |
| SD-03 | modify | auto-lang/mcp/overview.md | before：工具清单无 select → after：登记 `autoui_select_rect` 参数与信封返回 | MCP 工具面登记 | AC-07 |

## 测试设计

- **纯函数单测**（`ui/selection/` 内联 #[cfg(test)]，进日常档）：
  - select_nodes：空 bounds、中心包含/边缘扫过不命中、嵌套全命中；
  - trim_to_topmost：全命中返回顶层单节点、分散命中返回多节点、文档序；
  - 信封构建：切片==源码子串（锚点断言）、去公共缩进、span=None 降级标注、
    node_to_json 结构往返、Atom Display 可输出。
- **vue 生成器测试**（ui_gen/vue.rs:18254 mod tests 内新增）：生成模板含
  `data-auto-tag`/`data-auto-span` 且 span 值与 AuraNode.span 一致。
- **VM 交互**：交互臂以实机走查为准（无头不可达），逻辑核心已抽纯函数；
  走查用 autoui-verifier 技能（`auto run -r vm` + MCP 脚本）。
- **端到端**：
  - `scripts`（autoui-verifier）：`test_vm_mcp.py` 扩展 `autoui_select_rect` 用例
    （rect→JSON 信封→节点 kind 集合断言）；
  - `test_vue_playwright.mjs` 扩展：断言示例页存在 data-auto-span 元素、
    Alt+拖拽后面板出现且切片文本出现在面板中。
- **门禁分级**（AGENTS.md Category B）：
  - 日常：`cargo check -p auto-lang`；局部 `cargo t selection`、`cargo t vue`；
  - UI 生成器/VM 改动收敛后：`cargo t iced` 局部 + review 前一次 `cargo tf`。
  - 不触碰 aavm/trans/book 面 → 不跑 taa/tt/tb。

## 验收标准

- **AC-01 选择语义**：`ui/selection` 纯函数单测全绿（中心包含、顶层修剪、文档序、
  空集安全）。验证：`cargo t selection`。
- **AC-02 结果信封**：单测断言 Auto 切片逐字节等于源文件子串（缩进归一前）、
  JSON 信封字段完整、Atom 可序列化。验证：`cargo t selection`。
- **AC-03 VM 端框选**：`auto run -r vm examples/ui/015-notes`（或等价示例）实机
  Alt+拖拽后，DevTools Select 标签页出现结果，Auto 视图内容可在对应 .at 源文件中
  找到（人工/走查记录截图）。
- **AC-04 VM 复制**：Copy 按钮后剪贴板内容==当前视图全文（走查断言
  clipboard_set 返回 true 并粘贴核对）。
- **AC-05 Vue 标记注入**：`cargo t vue` 快照断言生成模板含 data-auto-tag/span。
- **AC-06 Vue 端框选**：`auto run` Vue 模式实机/Playwright：Alt+拖拽 → 浮动面板
  出现，Auto 视图切片可在 .at 源文件找到，Copy 后剪贴板一致。
- **AC-07 MCP 工具**：`test_vm_mcp.py` 调 `autoui_select_rect` 返回 JSON 信封，
  命中节点 kind 集合与 AC-01 语义人工核对一致；错误参数返回结构化错误。
- **AC-08 无回归**：`cargo tf` 全绿（在案预存红除外）；走查双端原有交互无破坏。

## 执行步骤

（在 worktree `D:/autostack/.wt/lang-646/auto-lang`（分支 plan-646-dev）执行；
计划簿记留在主检出。）

### Phase A —— 选择语义与信封内核（纯函数）

- **[x] T-01** 新建 `crates/auto-lang/src/ui/selection/mod.rs`：`Rect` 复用
  `ui::debug::Rect`；`select_nodes` + `trim_to_topmost` + `Marquee` 几何；
  lib.rs/`ui/mod.rs` 挂模块。内联单测（AC-01）。
  验证：`cargo check -p auto-lang && cargo t selection`。
  [✅ 已完成] commit f47012fb2；selection 模块门控 ui-interpreter（依赖
  mcp_server/vtree_atom 同门控，default-features API 构建修复 79145ed32 后随
  c6579082e 线收编）。
- **[x] T-02** 新建 `crates/auto-lang/src/ui/selection/output.rs`：
  `SelectionResult`/`SelectedNode` + `build_selection_result(...)`（入参：
  surface/app/rect/topmost 节点、VTree、computed 映射、源码全文）+
  `node_to_json` walker + 三格式渲染（auto/json/atom）。内联单测（AC-02）。
  验证：`cargo t selection`。
  [✅ 已完成] 同上 commit；`select_envelope` 一步式包装（VM 臂/MCP 共用）。

### Phase B —— VM/Iced 端交互

- **[x] T-03** `ui/session.rs` DevToolsState 扩展四字段 + 初始化；
  `renderer.rs` `GlobalPress`（alt_held 门控、抑制 WM 焦点抢占）、`__mouse_moved`、
  `__mouse_released` 三臂接入 marquee 状态机。验证：`cargo check` +
  `auto run -r vm` 冒烟（拖拽出现矩形轨迹日志）。
  [✅ 已完成] commit 096e52825。DevToolsState 实扩 6 字段（+last_cursor/
  select_copy_feedback）；GlobalPress 臂经 `selection_press_anchor`（primary
  App split_mut）。人工拖拽冒烟属 AC-03 残项（见 §9 走查记录）。
- **[x] T-04** marquee 绘制层（Stack+Canvas 或 View::Overlay 择一）+
  release→needs_bounds→`__bounds_collected` 臂内延迟计算（pending_selection 消费）
  + desktop 模式 vwin 平移。验证：实机框选产出 selection_result 日志（AC-03 前置）。
  [✅ 已完成] Stack+canvas（MarqueePainter，非 opaque 层穿透）。**vwin 平移
  实证豁免**：LayoutCollector bounds 与订阅面 CursorMoved 同为窗口逻辑坐标
  （desktop 模式 vwin 平移渲染已含在 bounds 内），直接同空间比较，无需平移
  （语义等价，实现更简）。
- **[x] T-05** DevTools 面板新 `DevToolsTab::Select`：三视图切换 + Copy
  （clipboard_set）+ Esc 关闭。验证：实机走查（AC-03/AC-04），截图留证。
  [✅ 已完成] 「采集」chip + `__tab_select`/`__select_view_*`/`__select_copy`/
  `__select_esc` 四臂；Esc 走 keyboard_event_message 尾部（app 级 Escape
  binding 优先）；F12/✕ 关面板顺带清 marquee 在途态。

### Phase C —— MCP 工具

- **[x] T-06** `ui/mcp_server.rs`：`autoui_select_rect` 工具注册 + 实现（复用
  T-01/T-02 纯函数与 vtree 快照获取模式）；扩展
  `.agents/skills/autoui-verifier/scripts/test_vm_mcp.py` 用例。验证：
  `python .agents/skills/autoui-verifier/scripts/test_vm_mcp.py`（AC-07）。
  [✅ 已完成] commit b777994f7 + 9a5ccd6c6 线。**参数面收敛**：include_box/
  style/events/source 四 flag 未实现——信封 structure 由无 computed 快照构建
  （盒模型/样式本就缺席，flag 恒 no-op，如实去掉）；信封新增 `id` 字段。
  **走查修复**：SharedState 增 source_code 随帧发布（ensure_source_loaded）+
  `__hot_reload` 泵臂代消费 needs_bounds（静默会话 bounds 采集闭环，Plan 282
  补线）。e2e：013-todo 全窗 rect → nodes=1 kinds=['col']（顶层修剪语义）+
  结构化错误路径全绿。

### Phase D —— Vue 端

- **[x] T-07** `ui_gen/vue.rs node_to_html` 注入 data-auto-{id,tag,span}；mod tests
  新增快照断言。验证：`cargo t vue`（AC-05）。
  [✅ 已完成] commit c6579082e。**偏离计划：AUTOUI_SELECT_MARKERS dev 门控**
  （原定"无开关恒定输出"被实证证伪——304 项 vue 快照测试锚定精确 HTML，恒定
  注入大面积破坏；产物构建不设 env 保持逐字节不变，语义等价 dev-only）。
  走查增注 `data-auto-src`（源 .at stem——子件 span 归各自源文）。
- **[x] T-08** `crates/auto-man/src/vue.rs`：脚手架写 `src/auto-sources.ts`
  （run_vue_project + incremental_compile_changed 同步、hash 防抖）；
  dev-only overlay 资产（overlay.ts + 面板样式）+ main.ts 接线。
  验证：`auto run` 启动无错，页面含 overlay 注入（AC-06 前置）。
  [✅ 已完成] commit 444b87fec + 9a5ccd6c6。main.ts 以
  `import.meta.env.DEV` 动态引用（产物 tree-shake）；overlay 资产随
  run_vue_project 自愈刷新（旧工程 scaffold 停旧版 overlay 实证后补）。
- **[x] T-09** overlay.ts 交互逻辑：Alt+drag marquee、DOM 采集（中心包含+顶层修剪）、
  切片+JSON、复制。扩展 `test_vue_playwright.mjs` 用例。
  验证：`node .agents/skills/autoui-verifier/scripts/test_vue_playwright.mjs`（AC-06）。
  [✅ 已完成] playwright 增 altDrag/assertDataAuto/assertPanel 三动作；
  015-notes e2e 全绿（93 标记元素 → Alt 拖拽 → 面板切片=sidebar.at 字节
  区间原文）。走查修复：**UTF-8 字节切片**（.at span 字节口径 vs JS UTF-16
  码元，中文注释错位实证）。

### Phase E —— 收口

- **[x] T-10** 双端实机走查留证（截图/录屏入 plan）；`cargo tf` 全量门禁（AC-08）。
  [✅ 已完成] Vue 端 e2e 截图 `docs/plans/reports/p646/`（initial + panel：
  面板 span=4193..12730 切片=sidebar.at 原文）；VM 端 MCP e2e（013-todo）。
  tf 全绿（唯一红 plan367 real_sidebar_at_parses_with_navtree=master 预存，
  sidebar.at PLAN-637 内容漂移 vs plan367 测试，与本计划无关——两文件在
  master/分支逐字节一致实证）。
- **[x] T-11** spec 沉淀：SD-01 新建设计文档、SD-02/03 回写；`.autoos/specs.json`
  upsert + `python scripts/spec-index.py`。
  [✅ 已完成（worktree 侧）] SD-01 `docs/specs/auto-lang/ui/design/
  select-anything.md` + SD-02 ui/overview + SD-03 mcp/overview 已落 worktree
  随分支合并；specs.json upsert + spec-index.py 属主检出台账面，归
  /auto-plan:merge 执行。

## 复审记录

- 2026-09-18 /auto-plan:new 起草：基于双端代码勘察定稿 v1 契约，提交用户确认。
  stage: new，outcome: pass（待用户确认后转 executing），next: work（T-01 起）。

### 9.1 work 记录（2026-09-18，T-01..T-11）

`stage: work | plan_id: PLAN-646 | plan_revision: 1 | outcome: pass（附 3 项
人工走查残项，见 §10.6） | code_commit: 911c6a93d（worktree plan-646-dev，
base 1f4e3e32c）| task_ids: T-01..T-11 全闭环`

- **提交链**（worktree `D:/autostack/.wt/lang-646/auto-lang`，依赖兄弟
  auto-down detached @fae21d90 仅解 cargo path）：
  f47012fb2（T-01/02 selection 纯函数内核）→ 096e52825（T-03/04/05 VM 交互）
  → b777994f7（T-06 MCP 工具）→ c6579082e（T-07 vue 标记注入）→ 444b87fec
  （T-08 脚手架资产）→ bfb654a37（T-09 playwright 动作）→ c7e725324
  （selection 门控修复）→ 9a5ccd6c6（走查三修）→ eba552e50（needs_bounds
  泵臂代消费）→ 911c6a93d（T-11 spec）。
- **验证证据**：`cargo t selection` 24/24；vue 生成器 304/304（含
  plan646 标记测试）；mcp select_rect 5/5；tf 门禁见 §9.2；Vue e2e 截图
  `docs/plans/reports/p646/`（93 标记元素，面板 span=4193..12730 切片=
  sidebar.at 字节原文）；VM MCP e2e 013-todo（全窗 rect → nodes=1
  kinds=['col'] = 顶层修剪语义直证 + 结构化错误路径）。
- **AC 对账**：AC-01/02（selection 单测+字节锚点）✅；AC-05（生成器快照）✅；
  AC-06（Vue e2e 机器驱动，截图留证）✅；AC-07（test_vm_mcp.py 机器驱动）✅；
  AC-08（tf 预存红除外全绿，见 §9.2）✅。**AC-03/AC-04 半开放**：交互链路
  逻辑核（GlobalPress 起笔→marquee→release 定稿→bounds 臂信封→面板/Copy）
  与 MCP 路径共用同一信封代码且 MCP 侧机器实证；人工 Alt+拖拽与剪贴板
  粘贴核对属 30 秒目视残项（§10.6），复审时用户一拖即验。

### 9.2 tf 全量门禁（AC-08）

- `cargo tf` 全档（3616 测）：两预存红均与本计划无关——
  ①`plan367 real_sidebar_at_parses_with_navtree`：sidebar.at（PLAN-637 B1
  内容漂移，undefined `caption_text`）vs plan367 测试；**master 同红实证**
  （两文件 master/分支逐字节一致）。②`ffi_dual_019`（+并发批跑时
  dep_parity_018/ffi_dual_018 同 flake）：**master 隔离复跑同红**、本分支
  三测隔离全绿——在案预存并发 flaky（Plan 621/634 同族记录）。
  排除①后带 --retries 2 全量复跑（§9.1 证据行，3.7k+ 测）最终绿。
- 走查顺带修复两处非本计划破损面：selection 模块门控（default-features
  API 构建断裂）、`__hot_reload` 泵臂代消费 needs_bounds（静默会话 bounds
  采集闭环——Plan 282 既有链路缺口，非本计划引入，实机 trace armed 4/
  consumed 0 定罪后修复）。

### 9.3 review 记录（2026-09-18）

`stage: review | plan_id: PLAN-646 | plan_revision: 1 | outcome: pass |
reviewed_commit: 738ec4ecd（worktree plan-646-dev） | base_commit: 1f4e3e32c |
dependency_revisions: auto-down detached @fae21d90（仅 cargo path 解析） |
spec_inputs: docs/specs/auto-lang/ui/design/select-anything.md（新件，
SD-01/02/03 已落 worktree 提交 911c6a93d）`

**独立性声明**：复审在实现会话内进行（无独立会话可用），结论全部从工件
重建——复审期新增/复跑的命令与截图均在本节记录，不采信执行期摘要。

**复审期修复（回 work 两小项，均已验证）**：
- R646-F1（defect，已修 738ec4ecd）：Select 面板正文在深色主题下白字白底
  隐身（text 缺省色随主题，面板浅底硬编码）——实机合成输入驱动真窗口后
  截图定罪，正文加显式深灰。
- R646-F2（style，已修 65976a912）：selection 新件 rustfmt 归位。

**验收对账（复审复跑证据）**：
- AC-01 ✅ pass：`cargo t selection`（复审批 25/25，含中心包含/边缘扫过
  不命中/顶层修剪/文档序/死区/空集安全）。
- AC-02 ✅ pass：切片逐字节锚点（`slice_is_byte_exact_substring`）、JSON
  字段完整、Atom Display、node_to_json 往返。
- AC-03 ✅ pass（实机机器驱动）：PowerShell SendInput 合成 Alt+拖拽驱动
  真窗口（"待办清单"，013-todo），trace 全链 `GlobalPress alt=true →
  released drag=true → select bounds=12 nodes=1`；截图证明采集 tab 激活+
  信封头可见（`docs/plans/reports/p646/`）；R646-F1 修复后正文可视（修复
  后面板终态截图因桌面物理输入竞争未捕获，行为层由修复版 trace + 单行
  颜色 diff 佐证——证据层瑕疵已如实标注）。
- AC-04 ✅ pass（附人工残项）：clipboard 原语有 roundtrip 单测
  （clipboard.rs set_then_get），按钮→`__select_copy`→`clipboard_set`→
  `render(当前视图)` 接线经代码审查；物理粘贴核对留用户目视（§10.6）。
- AC-05 ✅ pass：`plan646_vue_emits_data_auto_markers`（tag/src/id/span+
  off:len 形态断言）；vue 生成器 304/304。
- AC-06 ✅ pass：playwright e2e 复审复跑全绿（assertDataAuto 30 元素 →
  altDrag → assertPanel surface=vue/span=），截图
  `docs/plans/reports/p646/p646-review-vue-*.png`。
- AC-07 ✅ pass：test_vm_mcp.py 复审复跑（envelope surface=vm nodes=1
  kinds=['col'] + 结构化错误路径）。
- AC-08 ✅ pass：**复审提交点 738ec4ecd 全量 tf 3615/3615 绿**（排除
  plan367 sidebar——master 同红实证；ffi_dual_019 flaky 一次重试过=在案
  预存并发 flake，Plan 621/634 同族）；scoped 25/25。

**知识增量复审**：SD-01/02/03 描述与实现一致（含走查修订：data-auto-src、
UTF-8 字节切片、AUTOUI_SELECT_MARKERS 门控、泵臂代消费）；`new_spec_
components`/`touched_goals`（GOAL-007/014/015，goals.md 注册在案）核对
无误；台账 upsert 留 merge。

**发现汇总**：R646-F1/F2 已修复；无未决 finding。**残项**（§10.6）：
VM 物理粘贴核对、Vue 面板 Copy（headless 剪贴板权限不可驱）——人工 30 秒
目视项，不阻塞。

`next: merge`

### 9.4 worktree 事件记录（外来 WIP 冲突）

执行中段发现**另一会话的 PLAN-022 terminal WIP 落入本 worktree 工作树**
（renderer.rs 片段曾误入一次本计划提交，已 reset --soft 撤销并恢复）。
外来改动已全额保全至 `D:/autostack/.wt/lang-646/foreign-wip-plan022-terminal.patch`
（terminal/{mod,iced/widget}.rs + renderer.rs 片段，608 行），本分支工作树
已还原为仅含本计划改动。**归属裁定待用户**：该 patch 归 PLAN-022 属主认领
（pop 恢复或弃置）；本计划未据其构建、未携带其任何行。

## 待澄清事项

1. **ChatGPT 讨论链接不可达**（chatgpt.com 分享页在本环境渲染为空白，多种方式
   尝试失败）。本设计以需求文字描述为准；若讨论中已有既定决策（如输出格式细节、
   交互方式），请指出差异，我将按讨论内容修订计划（plan_revision +1）。
2. **交互键位**：双端统一 **Alt+拖拽**（VM 端免开 F12；浏览器端 Alt 无冲突）。
   备选：VM 端 F11 独立模式 / Vue 端 Ctrl+Shift+S。若你偏好其他键位请指定。
3. **结果面板形态**：v1 VM 端复用 DevTools 标签页（基建最短）、Vue 端浮动面板；
   语义一致。统一为双端浮动面板可作后续迭代。
4. **桌面多虚拟窗**：v1 仅聚焦窗（vwin 平移），跨窗框选不支持——符合预期否？
5. **默认视图**：面板默认展示 Auto 源码（JSON/Atom 切换）。若知识采集场景期望
   默认 JSON，请指出。
6. **work 残项（复审时 30 秒目视确认，非阻塞）**：①VM 端人工 Alt+拖拽
   （交互臂逻辑与 MCP 共用信封代码且 MCP 侧机器实证，拖拽手势本身无头不可
   驱动）；②VM 面板 Copy 后剪贴板粘贴核对（clipboard_set 与 code_editor
   同一既通路径）；③Vue 面板 Copy 按钮（headless 剪贴板权限不可驱，按钮
   在截图在案）。**外来 WIP 归属**：`D:/autostack/.wt/lang-646/
   foreign-wip-plan022-terminal.patch`（PLAN-022 terminal 虚拟滚动，608 行）
   待属主认领或弃置（§9.3）。
7. **MCP include_* 参数面收敛**：原计划的 include_box/style/events/source
   四 flag 未实现（信封 structure 无 computed 快照，flag 恒 no-op，如实
   去掉）；信封新增 `id`（vnode_N）字段。若 agent 侧需要盒模型/样式入信封，
   后续迭代以 computed 注入实现。
