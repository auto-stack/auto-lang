# scroll-pane — AutoUI 通用滚动架构（PLAN-656）

> **Status**: active
> 路径：`crates/auto-lang/src/ui/scroll/`（核心语义 + hosting + controller 存储）；
> iced adapter `ui/iced/renderer.rs` + `ui/iced/managed_content.rs`；
> Vue adapter `ui_gen/vue.rs`。
> 设计基线：[docs/design/autoui/universal-scroll-architecture.md](../../design/autoui/universal-scroll-architecture.md)（Phase A+B+C 落地）。

## 目标与范围

- 统一滚动语义层：普通真实布局（implicit LayoutScrollContent）与自管理逻辑内容
  （hosted `ScrollContent`）使用同一 `scroll-pane` primitive；
- 后端无关核心类型，iced / Vue 只做 adapter；DOM `scrollTop/scrollHeight`、
  iced `Viewport` 等平台词汇不进入 AutoLang 语义层；
- Phase D（virtual-list）/ E（terminal 迁移）/ F（code_editor 清偿 P626-D4）/
  G（anchor/nested/touch/a11y）为后续消费者计划，不修改本协议。

## 核心类型（`ui/scroll/`，零 backend 依赖）

| 类型 | 职责 |
|---|---|
| `Axis` / `ScrollAxes::{X,Y,BOTH}` | 单轴 vs pane 开启轴集合（`axis: both` 类型层闭合） |
| `ScrollAxisState { offset, viewport_extent, content_extent }` | 单轴状态，f64 逻辑空间；`scroll_range = max(content−viewport, 0)`，`offset ∈ [0, range]` |
| `ScrollState { x: Option<_>, y: Option<_> }` | 双轴状态；未启用轴投影退化态（offset=0） |
| `ScrollViewportState { width, height }` | pane → hosted content 的 viewport 下发通道 |
| `ScrollIntent { ScrollBy/ScrollTo/PageBy/ToStart/ToEnd }` | 统一滚动意图 + `ScrollSource`（含 `NativeScrollbar`/`Unknown` native backend 出口） |
| `ResolvedScrollIntent` | `ScrollIntent::resolve(&ScrollState)` → clamp 后绝对 offset（NaN 折 0，total） |
| `ScrollbarPolicy::{Auto,Always,Hidden}` | 可见策略；宽度/颜色/动画归 Theme |
| `geometry::*` | thumb 正反映射（min-thumb clamp 后按 `thumb_travel` 映射，往返恒等） |
| `ScrollContentHost` / `ScrollContentHostRecord` | hosting contract 三通道 trait / callback bundle |
| `SyntheticManagedContent` | 架构验收工具（**非 public widget**），默认 10M×2M 逻辑 extent |

**刻意不落地**：`ScrollAnchor`（无真实 consumer 前不固定 `ItemKey`）；
core 级 `ScrollControllerId`（controller 是 logical handle，binding 属 runtime 层）。

## hosting contract（三条语义通道）

```text
content → pane : scroll_state()            （State down）
pane → content : apply_scroll_intent()     （Intent up）
pane → content : viewport_changed()        （Viewport down）
```

状态所有权：ordinary content 的 offset 归 ScrollPane/runtime；managed content 的
semantic offset 与 logical extent 归 host 单源，pane/backend 只持投影缓存。
public `on-scroll` 是观察面，不得被 virtual-list/terminal 当内部控制协议。

## AutoLang API

```auto
scroll-pane { col { ... } }                    // axis: y（默认），scrollbar: auto
scroll-pane(axis: x) { row { ... } }
scroll-pane(axis: both) { ... }
scroll-pane(scrollbar: auto|always|hidden) { ... }
scroll-pane(controller: <handle>) { ... }
scroll-pane(onscroll: |ox, oy, vw, vh, cw, ch, px, py| { ... }) { ... }
```

- 推荐拼写 `scroll-pane`；`scrollable` / `scroll` / `Scroll` 为 alias（内部
  canonical tag 未翻转——r2 裁定，canonical 清扫另立）。
- legacy `direction: vertical|horizontal|both` 保留映射 axis，`axis` 优先。
- **controller（v1 执行裁定）**：句柄为不透明非空串。`scroll_controller()`
  原生族（`scroll_to_start/to_end/scroll_by/scroll_to/scroll_state`）以函数
  形态提供；.at 侧可直接以字符串状态字段作柄（`controller: .sc` +
  `scroll_to_end(.sc)`）。计划 r2 §5.1 的方法语法糖（`scroll.to_end(axis: y)`）
  需语言面 handle-method 派发 + kwargs，属后续语言面计划（见 KNOWN-DEBT 登记）。
- **on-scroll（v1 执行裁定）**：8 个位置实参，顺序冻结 = record 字段序
  `offset_x, offset_y, viewport_w, viewport_h, content_w, content_h,
  progress_x, progress_y`（VM handler 桥对 heap 实参是占位 0，record 对象
  实参不可达）；具名 record 经 `scroll_state(handle)` native
  （GenericInstanceData，`s.progress_y` 具名可读）。双端同序。

## Backend contract

### iced（`ui/iced/renderer.rs`）

- `axes` → `Direction::{Vertical, Horizontal, Both}`；
- `scrollbar: hidden` → `Scrollbar::hidden()`（width 0，结构性无 rail 命中区——
  非视觉透明；wheel/keyboard/controller 滚动保留）；
- `auto`/`always` 视觉等价（iced 0.14 无 idle auto-hide）——**已知限制**，
  overlay 淡出属 Phase G；
- controller：句柄派生稳定 id（`scroll_ctl_<key>`）+ 测量缓存（on_scroll 读臂）
  + update 期 intent 排空（同 handle 折叠双轴终态，一次 scroll_to）；
- managed bridge：`ManagedScrollContentWidget` layout 声明逻辑 extent，
  draw 期观察 viewport 回灌 host（terminal virtual_scroll 同款）。

### Vue（`ui_gen/vue.rs`）

- plain 模式：`overflow-y/x/-auto`（always → `*-scroll`）；hidden → 内联
  `scrollbar-width:none`（滚动能力保留）；controller → `data-scroll-ctl` 锚 +
  注入 snake 同名 JS helper 族；onscroll → 8 位置实参内联箭头（同 VM 序）；
  **PLAN-692**：带 `overflow-*:auto` 语义类的普通容器在 plain 装配点自动挂
  `.ash-scroll` 类（原生滚动条统一皮肤），与 ScrollArea 同视觉参数；
- shadcn 模式：axis y/x → ScrollArea orientation；**both/hidden 表达受限**
  （语义完整形态在 plain 模式——已知限制，后续 plain 降级或 ScrollArea 定制另立小改）；
  **PLAN-692**：`size` prop（uint px，缺省 8）→ 根元素内联 `--sb-size` CSS 变量；
  ScrollBar.vue thumb 厚度 = `--sb-size`（经 reka 内联 `width/height:
  var(--reka-scroll-area-thumb-{width,height})` 的未定义厚度侧分层接管，长度
  变量不触碰）；rail 宽 = size+6px（hover 空间）；thumb hover 加宽 +2px/侧 +
  `cursor:pointer`，按住 `active:` 主色高亮；`.ash-scroll`（原生路径）同参：
  轨宽 size+4px、thumb 视觉宽 = size（2px 透明 border + background-clip，
  hover 时 border 归零加宽）、`:active` 主色。native 侧 thumb cursor 为浏览器
  平台限制（不可定制），"同一套"五参数=宽/色/圆角/hover/active；
- managed bridge v1：`scroll-test-content` = logical spacer（内联逻辑尺寸，
  DOM 节点恒 1，不随逻辑 extent 膨胀）；物化窗口属 iced widget 与 Phase D。

### iced（PLAN-692 兼容注记）

`size` prop 为 vue 臂视觉面；iced 臂 `renderer.rs` scrollbar 样式为固定宽度
主题样式，本计划不接 `size`（VM 窗忽略该 prop，schema 兼容零破坏）——后续
如需 iced thumb 宽度可从 `scrollable::Scrollbar` 宽度字段接线（另立小改）。

### 未来 backend

不得把 DOM/iced/Compose/ArkTS/UIKit 类型泄露进 core semantics；无法区分
thumb/rail 的平台用 `ScrollSource::NativeScrollbar`；native inertia/overscroll
保持 backend policy；core 不假设 intent 必然由本 pane 消费（为 nested scroll 预留）。

## 已知语义陷阱与滚动位保持（review F-4 实证 + 末环符号修正）

- **pane 高度约束 idiom**：Plan 057 双样式遗留使 pane 的 style 同时作用于
  内层内容列——pane 自身的 `h-*`/max-h 类会把 content 固定到 == viewport
  （range=0，永不滚）。正确 idiom：外层 `container (style: "h-48")` 定高，
  pane 自身只带视觉类（`border rounded h-full w-full`）。旧 scrollable 同样
  受此影响。
- **滚动位保持与读回符号**：iced 滚动 offset 由 Tree 状态跨重建保持
  （实证：无 controller/写臂的观察 pane 跨多次重建仍保持滚动位）。
  controller pane 另经 Plan 043 写臂每 build 重发注册表 offset
  （terminal/015-notes 生产同款先例；同值去抖，结构性重建防御）。
  `ScrollStateReader` 读回只补 viewport/content 基线（extent-only），
  不覆写 offset——offset 语义单源在注册表（写臂回填/on_scroll 回声/
  用户滚动），且排空同帧的 pre-scroll 读值回写会短暂覆盖命令值投影。
  注意 iced 0.14 符号约定：operation 钩子的 `translation` **即正向滚动
  offset**（与 `Viewport::absolute_offset` 同源同号；draw 侧以
  `-translation` 平移内容层）——F-4 追查期曾按"内容平移取负"解读，
  任何正向 offset 被 clamp 成 0，是"读回恒 0/重建重置"误判的来源。
- **controller 时序**：intent 排空经 `__mcp_scroll` 回环消息在 update 头部
  直返落盘（MCP 同款机制）；新绑定 handle 下一 tick 读回预热；未预热
  intent 留队延迟解析（首用动作晚一拍而非解析为 0）。

## 兼容承诺

- `scrollable`/`scroll`/`Scroll` 旧代码零改动（alias + legacy direction）；
- 隐式 `overflow-y-auto` style 类路径、Plan 043 双臂（autodown 文档绑定）、
  terminal `virtual_scroll` 寄宿、VNode snapshot keyword `"scrollable"` 均不变；
- 新功能只在新架构上演进。

## 验证面

- 单测：`cargo t scroll`（state/intent/geometry/controller/host 全绿）；
- codegen 黄金：`p656_scroll_pane_vue_codegen`；
- capability 示例：`examples/capability-tests/p656-scroll-pane/`（双端同源，
  ordinary y/x/both + hidden + controller + on-scroll + managed 10M 逻辑 extent）。
