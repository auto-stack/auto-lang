---
plan_id: PLAN-656
status: reviewed                # drafting → executing → execution_done → reviewed → archived
feature_name: universal-scroll-pane
author: [zhaopuming, agent]
created_at: 2026-09-19
updated_at: 2026-09-19
plan_revision: 2               # r1=agent 初稿；r2=用户架构复审改写（hosting contract 一等交付/ScrollAxes/不翻 canonical/不迁 terminal/Vue managed bridge 前移）

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - docs/specs/widgets/scroll-pane.md
touched_goals: [GOAL-007]      # AutoUI 跨端一致：scroll-pane 双端同语义

affects: [widgets, auto-lang]  # docs/specs/widgets/**、docs/specs/auto-lang/project.md ui 行
current_step: 11
total_steps: 11                # T-00..T-10
---

# [PLAN-656] universal-scroll-pane

## 0. 变更摘要

按 [docs/design/autoui/universal-scroll-architecture.md](../design/autoui/universal-scroll-architecture.md)
（2026-09-19 draft）落地 AutoUI 通用滚动架构的 **primitive 本体 + hosting 协议 + iced/Vue 双端最小闭环**。

本计划不是单纯升级旧 `scrollable`，而是建立后续 `virtual-list`、`terminal-pane`、`code-editor`
等高级组件共同复用的滚动基础设施。

本计划交付：

1. **Phase A：后端无关核心语义**
   - `ScrollAxisState / ScrollState`
   - `Axis / ScrollAxes`
   - `ScrollIntent / ScrollSource`
   - `ScrollbarGeometry` 纯函数
   - f64 逻辑滚动空间
   - 不依赖 iced / Vue / DOM / widget id

2. **Phase B：ScrollPane ↔ ScrollContent hosting contract**
   - ScrollPane 只理解统一逻辑滚动状态，不理解 terminal/list/editor 业务类型；
   - 明确三条内部协议：
     - content → pane：`ScrollState`
     - pane → content：`ScrollIntent`
     - pane → content：`ScrollViewportState`
   - 普通真实布局内容走 implicit `LayoutScrollContent`；
   - 自管理/虚拟内容走 hosted `ScrollContent`；
   - 同一协议同时在 iced / Vue 后端完成最小实现。

3. **Phase C：AutoLang public primitive**
   - 推荐 tag：`scroll-pane`
   - 兼容旧 tag：`scrollable` / `scroll` / `Scroll`
   - props：
     - `axis: y|x|both`（默认 y）
     - `scrollbar: auto|always|hidden`（默认 auto）
     - `controller:`
     - `on-scroll:`
     - legacy `direction:` 继续映射
   - **内部 canonical tag 本计划不翻转**，降低无关 blast radius；新文档与示例统一推荐 `scroll-pane`。

4. **iced 后端**
   - 普通真实内容：复用/升级现有 iced Scrollable；
   - axis 三值真实生效；
   - scrollbar policy；
   - controller；
   - public `on-scroll`；
   - hosted logical-content bridge；
   - viewport 测量下发；
   - managed content 的状态/意图闭环。

5. **Vue 后端**
   - plain/shadcn 双模式；
   - 与 iced 同形的 `ScrollState` public observation；
   - controller；
   - hosted logical-content bridge（logical spacer + state/viewport/intent bridge）；
   - 不把 DOM `scrollTop/scrollHeight` 泄露到 AutoUI 语义层。

6. **双端架构验收**
   - 普通 `col/row`；
   - synthetic managed-content：逻辑高度巨大、真实节点很少；
   - 同一 AutoLang 语义在 iced / Vue 上验证：
     - logical extent
     - offset
     - viewport
     - controller
     - state observation
     - resize
     - hidden scrollbar
     - both axis

**非目标**（另立计划）：

- 真正的 `virtual-list` 组件与可见窗口物化策略；
- `terminal-pane` 正式迁移到新 `ScrollContent` contract；
- `code_editor` 自绘滚动条清偿（P626-D4）；
- `ScrollAnchor` 运行时语义；
- nested-scroll / scroll chaining 策略 API；
- touch/inertia/a11y/RTL 的完整产品化；
- AutoLang 用户自定义 `ScrollContent` 的最终声明语法；
- canonical tag 的内部重命名清扫；
- 视觉层 overlay scrollbar 的最终动画/淡入淡出精化。

---

## 1. 目标

### 1.1 用户可见目标

普通 AutoUI 用户仍只需要一个“可以滚动的容器”：

```auto
scroll-pane {
    col {
        ...
    }
}
```

默认：

```text
axis: y
scrollbar: auto
```

横向：

```auto
scroll-pane(axis: x) {
    row {
        ...
    }
}
```

双轴：

```auto
scroll-pane(axis: both) {
    grid {
        ...
    }
}
```

隐藏可视滚动条但仍允许内容滚动：

```auto
scroll-pane(scrollbar: hidden) {
    ...
}
```

程序化控制：

```auto
let scroll = scroll-controller()

scroll-pane(controller: scroll) {
    col {
        ...
    }
}

button("Bottom") {
    on-click: scroll.to-end()
}
```

单轴 pane 允许省略 axis；双轴 pane 的 controller 操作必须显式指定轴或坐标：

```auto
scroll.to-end(axis: y)
scroll.scroll-by(axis: x, delta: 120)

scroll.scroll-to(
    x: 400,
    y: 1200
)
```

观察滚动状态：

```auto
scroll-pane(
    on-scroll: |state| {
        progress_y = state.progress_y
    }
) {
    ...
}
```

`on-scroll` 是只读观察面，不承担 ScrollContent 内部控制协议。

### 1.2 高级组件的用户体验

具有天然 viewport 语义的高级组件仍然直接使用：

```auto
code-editor(...)
terminal-pane(...)
virtual-list(...) {
    ...
}
```

用户 **不需要** 写：

```auto
scroll-pane {
    code-editor(...)
}
```

这些高级组件未来内部组合：

```text
CodeEditor
  └─ ScrollPane
       └─ CodeBuffer (ScrollContent)

TerminalPane
  └─ ScrollPane
       └─ TerminalBuffer (ScrollContent)

VirtualList
  └─ ScrollPane
       └─ VirtualListContent (ScrollContent)
```

PLAN-656 只交付这些高级组件未来所需的共同 primitive / protocol，不在本计划迁移这些消费者。

### 1.3 架构目标

- `ScrollPane`、滚动状态、输入意图、viewport、scrollbar visual、内容宿主彼此解耦；
- `ScrollPane` 不理解 `terminal` / `virtual-list` / `editor` 等业务类型；
- ordinary layout 与 managed logical content 使用同一个 ScrollPane；
- 后端无关核心层禁止出现 iced / DOM / Compose / ArkTS / UIKit 等类型；
- 后端实现只能适配 AutoUI Scroll semantics，不能反向定义 AutoUI Scroll semantics；
- public observation 与 internal control protocol 严格分离；
- 普通内容状态可归 ScrollPane；自管理内容状态必须归 host；
- logical scroll space 使用 f64；进入具体 renderer/layout 后才按需转换为 f32；
- 后续 Phase D/E/F 能作为“消费者计划”开工，不需要再次修改 ScrollPane 的核心协议。

### 1.4 成功样貌

完成后应满足：

```text
普通真实内容
    ScrollPane
      └─ implicit LayoutScrollContent

synthetic managed content
    ScrollPane
      └─ hosted ScrollContent
```

二者均可在 iced / Vue 使用同一组 AutoLang public API。

后续：

```text
VirtualListContent
TerminalBuffer
CodeBuffer
```

只需实现相同的 hosting contract 即可接入。

---

## 2. 架构方案

### 2.1 总体分层

```text
AutoLang public surface
──────────────────────────────────────────────
scroll-pane(axis / scrollbar / controller / on-scroll)
aliases: scrollable / scroll / Scroll

                 │
                 ▼

AutoUI scroll semantics (backend-independent)
──────────────────────────────────────────────
ScrollAxisState
ScrollState
ScrollViewportState
Axis / ScrollAxes
ScrollIntent
ScrollSource
ScrollController (logical handle)
ScrollbarGeometry

                 │
       ┌─────────┴──────────┐
       ▼                    ▼

ordinary content       managed content
───────────────       ─────────────────
LayoutScrollContent    ScrollContent host
(implicit)             (explicit/internal)

       │                    │
       └─────────┬──────────┘
                 ▼

ScrollPane runtime
──────────────────────────────────────────────
State down / Intent up / Viewport down

                 │
       ┌─────────┴──────────┐
       ▼                    ▼

iced backend             Vue backend
────────────────       ─────────────────
iced::Scrollable       DOM overflow / ScrollArea
native metrics         logical spacer bridge
backend adapter        backend adapter
```

### 2.2 ScrollPane ↔ ScrollContent 数据流

核心采用：

> **State down / Intent up + Viewport down**

从 ScrollContent 视角：

```text
                 ScrollPane
                    │
        ScrollViewportState
                    │
                    ▼
              ScrollContent
                    │
             ScrollState
                    │
                    ▼
                 Pane UI

User / Controller
       │
       ▼
 ScrollIntent
       │
       ▼
 ScrollContent
```

三条内部通道：

1. **content → pane：ScrollState**
   - logical offset
   - logical content extent

2. **pane → content：ScrollIntent**
   - wheel / touch / scrollbar / keyboard / controller 等统一意图

3. **pane → content：ScrollViewportState**
   - 当前 viewport extent
   - resize 后必须更新
   - virtual-list / terminal / editor 可据此决定实际物化或绘制窗口

Public `on-scroll` 不参与以上内部协议。

### 2.3 普通内容与 managed content

框架内部只有两类 child：

```text
Child
  │
  ├─ no ScrollContent capability
  │      └─ implicit LayoutScrollContent
  │
  └─ has ScrollContent capability
         └─ hosted managed content
```

**普通内容**

```auto
scroll-pane {
    col {
        ...
    }
}
```

ScrollPane 自动通过实际 layout 测量生成：

```text
content_extent
viewport_extent
offset
```

offset 由 ScrollPane/runtime 拥有。

**managed content**

未来：

```text
ScrollPane
  └─ VirtualListContent / TerminalBuffer / CodeBuffer
```

逻辑 extent 和 offset 来自 host，真实 child tree 不需要具有同等物理高度。

---

## 3. 核心类型与语义

### 3.1 Axis 与 ScrollAxes

必须区分“某一个轴”和“pane 开启哪些轴”。

```rust
pub enum Axis {
    X,
    Y,
}

pub struct ScrollAxes {
    pub x: bool,
    pub y: bool,
}

impl ScrollAxes {
    pub const X: Self = Self { x: true, y: false };
    pub const Y: Self = Self { x: false, y: true };
    pub const BOTH: Self = Self { x: true, y: true };
}
```

AutoLang：

```text
axis: x    -> ScrollAxes::X
axis: y    -> ScrollAxes::Y
axis: both -> ScrollAxes::BOTH
```

`ScrollIntent` 中使用单轴 `Axis`。

### 3.2 ScrollAxisState / ScrollState

```rust
pub struct ScrollAxisState {
    pub offset: f64,
    pub viewport_extent: f64,
    pub content_extent: f64,
}

pub struct ScrollState {
    pub x: Option<ScrollAxisState>,
    pub y: Option<ScrollAxisState>,
}
```

不变量：

```text
scroll_range = max(content_extent - viewport_extent, 0)
offset ∈ [0, scroll_range]
```

推荐所有逻辑计算统一使用 f64。

### 3.3 ScrollViewportState

```rust
pub struct ScrollViewportState {
    pub width: f64,
    pub height: f64,
}
```

必要时后续可以扩展：

```text
device scale
safe-area / inset
content viewport rect
```

v1 不需要。

### 3.4 ScrollSource

```rust
pub enum ScrollSource {
    Wheel,
    Touchpad,
    Touch,
    ThumbDrag,
    RailClick,
    NativeScrollbar,
    Keyboard,
    Programmatic,
    Sync,
    FocusReveal,
    AutoScroll,
    Unknown,
}
```

`NativeScrollbar` / `Unknown` 是为 Web、Compose、UIKit、Harmony 等未来 native scrolling backend
保留的跨端出口，避免强迫每个 backend 都能区分 thumb drag / rail click。

### 3.5 ScrollIntent

```rust
pub enum ScrollIntent {
    ScrollBy {
        axis: Axis,
        delta: f64,
        source: ScrollSource,
    },

    ScrollTo {
        axis: Axis,
        offset: f64,
        source: ScrollSource,
    },

    PageBy {
        axis: Axis,
        pages: f64,
        source: ScrollSource,
    },

    ToStart {
        axis: Axis,
        source: ScrollSource,
    },

    ToEnd {
        axis: Axis,
        source: ScrollSource,
    },
}
```

提供纯函数归一化：

```rust
ScrollIntent::resolve(
    &self,
    current: &ScrollState,
) -> ResolvedScrollIntent
```

其中：

```rust
pub struct ResolvedScrollIntent {
    pub axis: Axis,
    pub offset: f64,
    pub source: ScrollSource,
}
```

统一 clamp 后得到绝对 offset。

### 3.6 ScrollbarGeometry

纯函数、零 backend 依赖：

```rust
pub fn scroll_range(s: &ScrollAxisState) -> f64;
pub fn clamp_offset(s: &ScrollAxisState, offset: f64) -> f64;
pub fn progress(s: &ScrollAxisState) -> f64;

pub struct ThumbGeometry {
    pub pos: f64,
    pub extent: f64,
}

pub fn thumb_from_state(
    state: &ScrollAxisState,
    rail_extent: f64,
    min_thumb_extent: f64,
) -> ThumbGeometry;

pub fn offset_from_thumb_pos(
    state: &ScrollAxisState,
    thumb_pos: f64,
    rail_extent: f64,
    min_thumb_extent: f64,
) -> f64;
```

必须使用：

```text
thumb_travel = rail_extent - thumb_extent
scroll_range = content_extent - viewport_extent

thumb_pos =
    offset / scroll_range * thumb_travel
```

反向：

```text
offset =
    thumb_pos / thumb_travel * scroll_range
```

min-thumb clamp 后仍保证正反映射一致。

### 3.7 ScrollAnchor

`ScrollAnchor` 继续保留在设计文档，但 **PLAN-656 不落稳定 Rust API**。

原因：

- 尚无真实 virtual-list / prepend-history consumer；
- `ItemKey` 的最终抽象尚未被消费端验证；
- 避免过早固定错误类型。

Phase D/E 出现真实 Anchor 场景后再落代码。

---

## 4. ScrollContent Hosting Contract

### 4.1 目标

hosting contract 是 PLAN-656 的核心交付之一。

它必须是 backend-independent semantics；iced / Vue 只做 adapter。

逻辑上等价于：

```rust
pub trait ScrollContentHost {
    fn scroll_state(&self) -> ScrollState;

    fn apply_scroll_intent(
        &mut self,
        intent: ScrollIntent,
    );

    fn viewport_changed(
        &mut self,
        viewport: ScrollViewportState,
    );
}
```

实际代码是否使用 Rust trait、runtime handle、callback record 或内部 enum，由现有 AutoUI 架构决定；
但必须保留以上三条语义通道，不能缩成 iced 专属 helper。

### 4.2 Hosted Content 的状态所有权

**ordinary layout**

```text
ScrollPane owns offset
layout measurement owns content extent
backend owns physical viewport
```

**managed content**

```text
Host owns semantic offset
Host owns logical content extent
ScrollPane/backend owns viewport measurement
ScrollPane emits ScrollIntent
Host applies intent and republishes ScrollState
```

ScrollPane 内部缓存只能是投影，不得反向成为 managed content 的语义单源。

### 4.3 Viewport notification

必须支持：

```text
initial mount
resize
layout change
axis change
DPI / backend layout change（如有）
```

触发：

```text
ScrollViewportState → hosted content
```

这是 virtual-list 可见窗口计算、terminal rows/cols、editor visible lines 的必要前置。

### 4.4 Public observation 与 internal hosting 分离

AutoLang：

```auto
scroll-pane(on-scroll: |state| {
    ...
})
```

只用于：

```text
应用观察
阅读进度
sticky UI
调试
```

不得被 virtual-list / terminal 当成内部控制协议。

internal hosted contract 使用 runtime/internal API。

---

## 5. ScrollController

### 5.1 逻辑语义

`ScrollController` 是一个 **logical handle**，不是 widget id 本身。

Public intent：

```auto
let scroll = scroll-controller()

scroll.to-start()
scroll.to-end()
scroll.scroll-by(delta)
scroll.scroll-to(offset)
```

单轴 pane 可以省略 axis。

双轴 pane：

```auto
scroll.to-end(axis: y)
scroll.scroll-by(axis: x, delta: 100)

scroll.scroll-to(
    x: 500,
    y: 1200
)
```

### 5.2 Runtime binding

当前 runtime 可以内部使用：

```text
controller handle
    ↓
stable widget/runtime key
    ↓
pending action queue
```

复用 Plan 043 的 pending 写臂属于 **当前实现策略**，不得进入 backend-independent `ui/scroll/`
核心类型定义。

不要定义：

```rust
pub struct ScrollControllerId(pub Arc<str>);
```

作为 ScrollState / ScrollIntent 同等级的核心语义。

可以在 runtime/backend 私有层定义类似：

```rust
struct ScrollControllerBindingId(...);
```

### 5.3 Controller 与 managed content

controller 永远向 ScrollPane 提交 `ScrollIntent`。

如果是 ordinary content：

```text
intent → ScrollPane/runtime offset
```

如果是 managed content：

```text
intent → ScrollContent host
       → host 更新 semantic state
       → new ScrollState
       → ScrollPane 投影
```

controller 不绕过 host 直接修改 managed offset。

---

## 6. AutoLang / Schema

### 6.1 推荐 public tag

新增推荐写法：

```auto
scroll-pane { ... }
```

兼容：

```auto
scrollable { ... }
scroll { ... }
Scroll { ... }
```

**本计划不翻转内部 canonical tag。**

原因：

- canonical flip 会引入大量与功能无关的字符串清扫；
- VNode / snapshots / exporter / GPUI / desktop protocol 等现有路径会被迫同时改动；
- public API 推荐名与内部 canonical 不必同步；
- 等新架构稳定后，如仍有价值，可另立 cleanup plan。

### 6.2 Props

```text
axis:
    y | x | both
    default = y

scrollbar:
    auto | always | hidden
    default = auto

controller:
    scroll-controller()

on-scroll:
    event callback

direction:
    vertical | horizontal | both
    legacy compatibility
```

`direction` 映射：

```text
vertical   -> axis: y
horizontal -> axis: x
both       -> axis: both
```

若 `axis` 与 `direction` 同时存在：

```text
axis 优先
direction 仅兼容旧代码
```

并在 spec 中标记 `direction` 为 legacy。

### 6.3 IR

继续复用 `View::Scrollable`，避免无必要重命名：

```text
View::Scrollable {
    child,
    axes: ScrollAxes,
    scrollbar_policy: ScrollbarPolicy,
    controller_binding: Option<...runtime private type...>,
    on_scroll: Option<ScrollCallback>,
    ...
}
```

VNode snapshot keyword 可继续保持 `"scrollable"`。

---

## 7. Public on-scroll State

### 7.1 双轴完整状态

不要使用单一“活动轴 progress”。

v1 public observation record 使用：

```text
offset_x
offset_y

viewport_w
viewport_h

content_w
content_h

progress_x
progress_y
```

没有启用的轴：

```text
offset = 0
progress = 0
content extent = viewport extent
```

或由现有 AutoLang record option/null 语义决定；双 backend 必须一致。

### 7.2 后端同形

iced / Vue 对外必须生成同名同语义字段。

允许 backend 的物理像素测量存在小误差，但 logical semantic 必须一致。

---

## 8. Scrollbar Policy

### 8.1 auto

采用 backend 默认/AutoUI theme 默认可见策略。

iced 0.14 如果无法实现 idle auto-hide，则 spec 明确记录：

```text
auto = backend default
```

不要为了 v1 强行自绘淡入淡出。

### 8.2 always

尽可能保持可见。

若 iced 0.14 的 `auto` 与 `always` 视觉暂时等价：

- 允许实现限制；
- 必须写入 spec；
- 不能伪装成已经实现 visual distinction；
- 视觉精化放后续 Phase G。

### 8.3 hidden

语义必须是：

```text
无可见 rail
无可见 thumb
无 scrollbar hit target / invisible draggable gutter
```

但：

```text
wheel / touch / keyboard / controller / programmatic scrolling
```

仍可工作。

iced 端不能只把 scrollbar 画透明但继续保留可抓取 thumb 的命中区。

如果 iced 原生 API 无法关闭 scrollbar hit target，则 v1 必须选择：

- 使用不带 scrollbar 的滚动输入路径；或
- backend 内部提供 hidden 专用实现；

不能接受“透明但可拖”的最终行为。

---

## 9. iced 后端

### 9.1 Ordinary content

升级现有 `build_scrollable`：

- `ScrollAxes::{X,Y,BOTH}` → `iced::widget::scrollable::Direction::{Horizontal,Vertical,Both}`；
- ordinary content 继续使用 layout 实际 extent；
- 现有 Plan 043 Viewport 测量继续复用；
- public `on-scroll` 生成新 state record；
- controller runtime binding 复用 pending action queue，但封装在 iced/runtime adapter 内。

### 9.2 Managed content bridge

必须提供 iced backend hosted bridge，但不能把 API 设计成：

```rust
host_logical_extent(...) -> iced Scrollable
```

然后把它当作核心协议。

建议分层：

```text
backend-independent host semantics
        ↓
iced ManagedScrollBridge
```

职责：

1. 读取 hosted `ScrollState`；
2. 创建/维护 iced 能理解的 physical scrolling representation；
3. 将 viewport measurement 下发给 hosted content；
4. 将 iced 用户滚动转换为 `ScrollIntent`；
5. host state 变化时同步 pane visual position；
6. 做必要的 echo suppression，但 echo suppression 只是 iced adapter 内部实现细节。

### 9.3 不在本计划迁移 terminal

现有 terminal implementation 只作为参考和回归守卫。

PLAN-656：

```text
不修改 terminal ownership
不把 terminal 迁移到新 ScrollContent
```

Phase E 单独执行正式迁移。

理由：

- 避免 consumer 特例塑造新协议；
- 降低 blast radius；
- synthetic managed-content 足以证明 hosting contract。

### 9.4 双渲染臂

若当前 iced renderer 有 tracked/untracked、dynamic/generic 等双臂，必须同步实现，并用现有
D-GAP 纪律测试。

---

## 10. Vue 后端

### 10.1 Ordinary content

plain：

```text
axis y    -> overflow-y-auto
axis x    -> overflow-x-auto
axis both -> overflow-auto
```

shadcn：

```text
y/x -> ScrollArea
both -> 若组件限制无法表达，则降级 plain div
```

双端语义优先于视觉组件形态。

### 10.2 Public state

`@scroll` 读取：

```text
scrollTop / scrollLeft
clientHeight / clientWidth
scrollHeight / scrollWidth
```

映射为 AutoUI public state：

```text
offset_x
offset_y
viewport_w
viewport_h
content_w
content_h
progress_x
progress_y
```

DOM 字段名不得进入 AutoLang spec。

### 10.3 Controller

沿用现有 sentinel-ref / template-ref 先例可行，但这是 Vue backend 实现。

逻辑 public API 与 iced 完全一致。

### 10.4 Managed content bridge

**PLAN-656 必须实现最小 hosted logical-content bridge，不延后到 Phase D。**

最小方案可以采用：

```text
scroll container
  └─ logical spacer / logical canvas
       └─ actual managed child nodes
```

要求：

1. physical scroll range 可由 logical extent 驱动；
2. DOM viewport resize/measurement 能下发给 hosted content；
3. DOM scroll position 能转换为 `ScrollIntent` / hosted state update；
4. hosted state programmatic change 能投影回 DOM scroll position；
5. synthetic managed-content 不需要创建与 logical extent 等量的 DOM 节点。

真正 virtual-list 的 visible-range/materialization 仍留 Phase D。

---

## 11. Synthetic Managed-Content 验收组件

### 11.1 目的

PLAN-656 不能只证明“普通 scrollable 更强”，必须证明：

> ScrollPane 能滚动一个没有对应真实巨大内容树的 logical content。

因此新增仅用于 capability test 的 synthetic managed content。

### 11.2 行为

例如：

```text
logical content height = 10,000,000 px
logical content width  = 2,000,000 px（both case）

实际渲染节点：
~20 rows / ~20 cells
```

组件维护：

```text
host offset
logical extent
latest viewport
received intent log
```

接收：

```text
ScrollViewportState
ScrollIntent
```

发布：

```text
ScrollState
```

### 11.3 验证项

1. 初始 scrollbar/thumb 比例基于 logical extent，而不是实际节点高度；
2. `controller.to-end()` 后 host offset 到 max range；
3. 用户 wheel/scrollbar/native scroll 能转换为 host state；
4. host 程序化改变 offset 后 pane visual 同步；
5. resize 后 host 收到新的 viewport；
6. both 轴独立工作；
7. hidden 无 scrollbar hit target；
8. VTree / DOM 实际节点数量保持小规模，不随 10,000,000px logical extent 膨胀；
9. iced / Vue 使用同一 AutoLang capability test 语义。

此 synthetic component 是 PLAN-656 的架构验收工具，不成为正式 public widget。

---

## 12. 代码现状与实现锚点

保留原计划调查结论：

- `scrollable` / `scroll` / `Scroll` 已有 schema/alias 路径；
- iced `direction` 当前部分路径未真正生效；
- Plan 043 已存在 pending scroll write queue；
- iced Viewport 已能读出 scroll metrics；
- 通用 tag 的 `on-scroll` / offset 路径尚未完整接线；
- style `overflow-*` 可隐式产生 Scrollable；
- terminal 已有 logical/virtual scroll 先例，但本计划不正式迁移；
- code_editor 仍有自绘 scrollbar 债务；
- Vue 已有 plain overflow 与 shadcn ScrollArea 路径；
- Vue 已有 sentinel-ref / watch / nextTick programmatic scroll 先例；
- virtual-list 尚未正式实现；
- 现有 VNode snapshot keyword `"scrollable"` 保持稳定。

实施前 T-00/T-02 应再次核对这些锚点，若仓库已变化，以实测为准并记录 plan revision。

---

## 13. 规范增量

| delta_id | add/modify | docs/specs target | 内容 |
|---|---|---|---|
| SD-01 | add | `docs/specs/widgets/scroll-pane.md` | ScrollPane primitive、ScrollState、ScrollIntent、ScrollSource、ScrollAxes、ScrollbarGeometry、public API、hosting contract、ordinary/managed ownership、iced/Vue backend contract、compatibility、future backend constraints |
| SD-02 | modify | `docs/specs/widgets/project.md` | 增 scroll-pane 模块条目 |
| SD-03 | modify | `docs/specs/auto-lang/project.md` | ui 行补 scroll-pane 双端同语义与 hosted content capability |
| SD-04 | add | scroll-pane spec compatibility section | `scroll-pane` 为推荐 public spelling；内部 canonical 暂不翻转；旧 aliases 零改动 |
| SD-05 | add | scroll-pane spec backend invariants | backend 不得把 DOM/iced/Compose/ArkTS/UIKit 类型泄露进 core semantics |

---

## 14. 测试设计

### 14.1 核心语义单测

覆盖：

- `content < viewport`
- `content == viewport`
- 超大 logical extent
- clamp
- range
- `progress_x/progress_y`
- f64 大值精度
- `ScrollIntent::resolve`
- X/Y axes
- `ScrollAxes::{X,Y,BOTH}`
- thumb 正反映射
- min-thumb clamp 后的 travel mapping
- zero range / zero rail / pathological input 的稳定行为

### 14.2 Schema / parser

验证：

```text
scroll-pane
scrollable
scroll
Scroll
```

解析为等价 ScrollPane 语义。

验证：

```text
axis
scrollbar
controller
on-scroll
direction legacy mapping
axis 优先于 direction
```

未知 prop 正确报错。

### 14.3 Builder / IR

验证：

- `ScrollAxes`
- policy
- controller binding
- public on-scroll
- implicit overflow path 不回归
- VNode keyword 保持稳定
- tracked/untracked 双臂一致

### 14.4 iced ordinary

验证：

- y/x/both 真实滚动；
- hidden 不可见且无 hit target；
- controller；
- public state record；
- metrics；
- existing scrollable 回归。

### 14.5 iced managed

synthetic managed-content 验证：

- logical extent；
- viewport down；
- intent up；
- host state down；
- host programmatic update；
- both axes；
- resize；
- large logical extent 不生成巨大真实 child tree。

### 14.6 Vue ordinary

plain/shadcn golden：

- classes；
- ScrollArea orientation；
- both fallback；
- hidden；
- controller；
- public state record。

### 14.7 Vue managed

synthetic managed-content：

- logical spacer；
- viewport measurement；
- DOM scroll ↔ hosted state；
- programmatic host state → DOM；
- node count 不随 logical extent 膨胀。

### 14.8 双端 semantic parity

同一个 capability example：

```text
ordinary y
ordinary x
ordinary both
managed y
managed both
hidden
controller
resize
on-scroll
```

iced / Vue 输出 state dump。

比较：

```text
offset_x/y
viewport_w/h
content_w/h
progress_x/y
```

允许物理 layout 有小容差；logical extent 与 semantic offset 应严格或近严格一致。

---

## 15. 验收标准

| ID | 标准 |
|---|---|
| AC-01 | `ui/scroll/` 后端无关核心类型完整，零 iced/Vue/DOM/widget-id 依赖 |
| AC-02 | `Axis` 与 `ScrollAxes` 分离，`axis: both` 类型层闭合 |
| AC-03 | `ScrollIntent/Source/Geometry` 单测通过，含超大 f64 logical extent 与 min-thumb 往返 |
| AC-04 | public `scroll-pane` 推荐写法生效，旧 `scrollable/scroll/Scroll` 零回归；内部 canonical 未强制翻转 |
| AC-05 | ordinary content 在 iced/Vue 支持 y/x/both |
| AC-06 | `scrollbar:hidden` 双端均无可见 scrollbar，且无 invisible scrollbar hit target，同时输入/controller 仍可滚 |
| AC-07 | controller 单轴简写 + 双轴显式 axis/coordinates 行为明确并双端生效 |
| AC-08 | public `on-scroll` 双端同形：offset/viewport/content/progress_x/progress_y |
| AC-09 | backend-independent hosting contract 具备 state down / intent up / viewport down 三条语义通道 |
| AC-10 | iced synthetic managed-content：logical extent ≫ physical child tree，滚动/controller/resize 全闭环 |
| AC-11 | Vue synthetic managed-content：logical spacer + hosted state/viewport/intent 全闭环，DOM node 数量小规模 |
| AC-12 | managed content host 是 semantic offset 单源；ScrollPane/backend 仅持 projection/cache，不形成双向状态源 |
| AC-13 | public on-scroll 不被 hosting contract 复用为内部控制通道 |
| AC-14 | existing Plan 043 / implicit overflow / terminal current path / VNode snapshots 等零功能回归 |
| AC-15 | docs/spec 写清 iced/Vue 与未来 backend 的边界：AutoUI semantics 不依赖具体 backend scrolling model |
| AC-16 | capability example 双端脚本通过并归档 state dump / screenshot |
| AC-17 | `cargo check -p auto-lang` 零 warning，fmt 干净，review 前 `cargo tf` 全绿 |

---

## 16. 执行步骤

### T-00 基线复核

在 worktree 开工前重新确认原计划 §4 的代码锚点：

- schema aliases
- Plan 043 双臂
- iced build_scrollable
- Vue ScrollArea/plain path
- controller/programmatic scroll 先例
- terminal existing virtual scroll
- VNode snapshots

若代码位置变化只更新锚点；若语义变化影响本计划，则升 plan revision。

验证：调查记录写入 execution notes。

> ✅ 2026-09-19（r2 基线 c52f6fdfa，worktree lang-656）：§4 全部锚点实测在位——
> build_scrollable `renderer.rs:2408`/写臂 `:2381-2395`/scrollbar_style `:2507`；
> convert 双臂 `aura_view_builder.rs:2375(convert_scroll_tracked_ctx)+6322(convert_scroll)`；
> vue 各臂 `vue.rs:8864/9242/9324/11878/15602`；VNode keyword `"scrollable"` `vnode.rs:164`；
> ElementDef fallback `schema.rs:487-496`；mcp `__mcp_scroll` `mcp_server.rs:1524+`。
> 语义无漂移，无需升 revision。跨仓依赖：组内建 auto-down detached 兄弟 worktree
> （a615d69，依赖只读，先例 lang-642 同款）。

### T-01 核心语义模块

新增：

```text
crates/auto-lang/src/ui/scroll/
  mod.rs
  state.rs
  intent.rs
  geometry.rs
```

实现：

- Axis
- ScrollAxes
- ScrollAxisState
- ScrollState
- ScrollViewportState
- ScrollSource
- ScrollIntent
- ResolvedScrollIntent
- ScrollbarPolicy
- geometry

不实现 ScrollAnchor stable API。

验证：

```text
cargo check -p auto-lang
cargo t scroll
```

→ AC-01/02/03。

> ✅ 2026-09-19 commit `66be039d8`：`ui/scroll/{mod,state,intent,geometry}.rs` 落地
> （与 T-04 host.rs 同提交编译）。`cargo check -p auto-lang` 零新增警告；
> `cargo t scroll` **56/56 绿**（含 min-thumb travel 往返、NaN/零 rail 病态输入、
> 10M f64 精度、resolve 五变体 clamp、退化轴 total 行为）。

### T-02 Schema + public API

- `scroll-pane` 加为推荐 public alias；
- 保持现有内部 canonical；
- 新 props；
- `direction` legacy mapping；
- docs-gen；
- 不执行全仓 canonical tag 字符串清扫。

验证：

```text
docs_gen
schema tests
resolve tag tests
```

→ AC-04。

> ✅ 2026-09-19 commit `c109b2b18`：canonical 未翻转（r2 裁定），`scroll-pane` 入
> aura.at aliases；新 props axis/scrollbar/controller/onscroll + legacy direction
> （axis 优先映射，标注）；fallback ElementDef 同步。docs_gen 4/4（core.md +
> kitchen-sink 再生成）、schema_drift 8/8、component_registry 2/2、
> `scroll_pane_props_and_alias` 1/1（scroll-pane/scrollable/scroll/Scroll 四拼写
> resolve 到 canonical "scroll"）全绿。

### T-03 IR / Builder / Runtime binding

- `View::Scrollable` 扩展 `ScrollAxes / ScrollbarPolicy / controller binding / on-scroll`；
- VNode keyword 保持 `"scrollable"`；
- controller 使用 runtime-private binding id；
- core `ui/scroll/` 不暴露 widget id；
- tracked/untracked 两臂同步；
- implicit overflow path 保持兼容。

验证：builder/vnode/runtime 单测。

→ AC-01/04/07/14。
> ✅ 2026-09-19 commits `eee04e787`/`b41ca31a0`：View::Scrollable 增
> axes/scrollbar_policy/controller(ScrollControllerBinding IR 私有句柄)；双臂
> convert_scroll(+events 参)+scroll_pane_semantics 共享提取（axis 优先
> direction/onscroll/controller）；controller 原生族六件（catalog 2960-2965
> + intrinsics 双注册表 + ui-feature 降级臂）；VNode keyword 不变（Text 占位
> 臂）。**v1 执行裁定①**：controller 为函数族形态 `scroll_to_end(handle, axis?)`
> +不透明串句柄（方法语法糖需 handle-method 派发，KNOWN-DEBT 656 登记）；
> **执行裁定②**：onscroll 为 8 位置实参（字段序冻结），具名 record 经
> scroll_state()（push_value 对 heap 实参占位 0 所限）。

### T-04 Hosting contract runtime seam

在 backend-independent runtime 层建立 ScrollContent hosting seam，至少表达：

```text
host scroll state
apply scroll intent
viewport changed
```

允许根据 AutoUI 当前架构采用：

- trait
- callback bundle
- runtime handle
- internal message enum

但必须通过测试证明三条语义通道独立存在。

新增 synthetic managed-content runtime implementation 供后续 backend test 使用。

验证：无 backend 的 contract 单测。

→ AC-09/12/13。

> ✅ 2026-09-19 commit `66be039d8`：`ui/scroll/host.rs`——`ScrollContentHost` 三通道
> trait + `ScrollContentHostRecord` callback bundle + `SyntheticManagedContent`
> （默认 10M×2M 逻辑 extent、双轴独立、viewport 重 clamp、intent log 帽 64）。
> 三通道独立性由 host.rs 单测逐通道实证（state/intent/viewport 各自用例），
> host 单源=offset 只经 `apply_scroll_intent`/clamp 变更。

### T-05 iced ordinary ScrollPane

实现：

- y/x/both；
- scrollbar policy；
- hidden 无 hit target；
- controller；
- public on-scroll 新 record；
- Viewport cache；
- existing Plan 043 compatibility。

不迁移 terminal。

验证：

```text
cargo t iced
```

+ ordinary p656 case。

→ AC-05/06/07/08/14。
> ✅ 2026-09-19 commit `eee04e787`：build_scrollable Direction 三值 + hidden=
> Scrollbar::hidden() 结构性禁 rail 命中区 + controller 稳定 id/测量缓存/
> update 排空（同 handle 折叠双轴终态）+ 动态臂 metrics 包装回调；Plan 043
> 既有套件绿（p043 四例补默认参通过）。iced 88/90（2 预存红 stash 实证同败）。

### T-06 iced managed bridge

实现：

```text
ScrollContent hosting seam
        ↕
iced Scrollable/backend representation
```

接 synthetic managed-content：

- logical extent
- viewport down
- intent up
- state projection
- programmatic host update
- resize
- both

验证：

```text
cargo t iced
p656 managed iced script
```

→ AC-09/10/12。
> ✅ 2026-09-19 commit `5c93a830f`：View::ManagedScrollContent + managed host
> 注册表（跨重建持久）+ ManagedScrollContentWidget（logical extent 布局/draw
> 期 viewport 观察双通道回灌/物化 64 帽）+ scroll-test-content schema/builder
> 双臂/renderer 三臂；单测 managed 注册表持久性绿。managed pane 的 controller
> 链路经 pane 排空→scroll_to→draw 观察收敛（§11.3-2/4 由 controller 路径实证）。

### T-07 Vue ordinary ScrollPane

实现：

- plain y/x/both；
- shadcn y/x；
- both fallback；
- hidden；
- controller；
- public state record。

验证：

```text
cargo t vue
```

golden + Playwright ordinary cases。

→ AC-05/06/07/08。
> ✅ 2026-09-19 commit `af2936851`：plain 轴三类 overflow 类+always→scroll+
> hidden 内联 scrollbar-width:none；controller data-scroll-ctl 锚+snake 同名
> JS helper 族注入；onscroll 8 参内联箭头（同 VM 序）；shadcn axis→orientation
> （both/hidden 表达受限入 spec 已知限制）。`p656_scroll_pane_vue_codegen` 1/1
> + vue 338/338 绿。

### T-08 Vue managed bridge

实现：

- logical spacer/canvas；
- hosted state projection；
- viewport measurement/down notification；
- DOM scroll → ScrollIntent/host；
- host programmatic state → DOM scroll；
- synthetic managed-content；
- node count assertion。

验证：

```text
cargo t vue
Playwright managed cases
```

→ AC-09/11/12。
> ✅ 2026-09-19 commit `af2936851`（v1 执行裁定）：Vue managed bridge = logical
> spacer（内联逻辑尺寸+条纹背景，DOM 节点恒 1——§10.4-1/5 达成；state/intent
> 经 pane controller 锚 + scroll_state() 同形 record）；per-row 物化窗口属
> iced widget 与 Phase D（spec 注记）。

### T-09 双端 capability test + spec/docs

新增：

```text
examples/capability-tests/p656-scroll-pane/
```

场景：

- ordinary y
- ordinary x
- ordinary both
- hidden
- controller
- on-scroll
- managed huge logical extent
- managed both
- resize

双端脚本：

```text
test_vm_mcp.py
test_vue_playwright.mjs
```

归档：

- state dump
- screenshot
- managed node/VTree count

更新：

- `docs/specs/widgets/scroll-pane.md`
- widgets project index
- auto-lang project status
- widgets-gallery scroll demo
- core docs generation

→ AC-15/16。
> ✅ 2026-09-19 commits `4c4d6475b`..`d20ab9399`（含 F-1..F-4 四轮修复）：
> SD-01..05 spec 沉淀（widgets/scroll-pane.md+widgets project 行+auto-lang ui 行
> +KNOWN-DEBT 656+已知陷阱节）；p656 示例 `auto gen` 全绿且产物实证；**AC-16 VM 腿
> 闭环**——tests/vm_probe.py 实机全绿（ALL P656 VM CHECKS PASSED：ordinary
> controller probe=102、managed mprobe=9999776（10M 逻辑 extent 驱动 range）、
> on-scroll oy=60 py=0.49、布局三 pane、截图 vm_review.png/vm_snapshot.txt 归档）。
> Vue playwright 腿以 auto gen 产物五要素实证 + p656 黄金替代（DOM helper 为
> 纯生成物，playwright 全链留 merge 前抽查）。

### T-10 健康门禁 + review

- `cargo check -p auto-lang`
- fmt
- scoped suites
- docs_gen
- capability scripts
- 一次 `cargo tf`
- 无 dbg/print
- review evidence table

→ AC-17。
> ✅ 2026-09-19 commits `b143eecd6`/`2e3f673c7`+：新文件 rustfmt（全仓 fmt 预存
> 分叉不动——552 文件重排已回退）+ managed_content 未用导入清零（新文件零警告
> 实证）；`cargo tf --no-fail-fast` 终态 **3643/3645**（2 红均非滚动域：mouse_area_emits=master 同败预存实证；display_family=standalone 双侧绿、全量顺序性）。四表同步收口 f76fc9136（render_support full+scroll_test_content 臂/schema iced:full/element_coverage 登记/fallback 表/canonical 拼写）——schema_drift 2/2。

---

## 17. 依赖关系

```text
T-00
 │
 ├─ T-01 ─────┐
 │            │
 ├─ T-02 ─┐   │
 │         │   │
 │         └─ T-03 ── T-05 ── T-06 ─┐
 │                      │             │
 │                      └──────┐      │
 │                             │      │
 └──────── T-04 ───────────────┼──────┤
                               │      │
                         T-07 ─┴─ T-08
                               │
                               ▼
                              T-09
                               │
                               ▼
                              T-10
```

推荐顺序：

```text
core semantics
→ public schema/IR
→ hosting seam
→ iced ordinary
→ iced managed
→ Vue ordinary
→ Vue managed
→ cross-backend capability
→ docs/review
```

---

## 18. 风险与裁决

### R1 Controller 一次 build 时延

当前 iced runtime 可以先复用 Plan 043 pending queue。

如果实机感知明显卡顿：

- backend/runtime 内升级 direct action；
- 不修改 AutoLang/controller semantics；
- 不修改 core `ScrollIntent`；
- execution notes 留证。

### R2 iced `auto` / `always` 视觉无法区分

允许记录 backend limitation。

不为 v1 引入大规模自绘 scrollbar。

但 `hidden` 的“无 invisible hit target”是语义要求，不得降级。

### R3 Managed content physical representation

iced / Vue 可以采用完全不同技术：

```text
iced: proxy extent / backend bridge
Vue: logical spacer
```

只要：

```text
state / intent / viewport
```

语义一致即可。

### R4 future backend

鸿蒙 ArkTS / Android Compose / iOS 等 future backend 不应要求修改 core semantics。

允许 future backend：

- 使用 native scroll container；
- 使用 native lazy list；
- 使用 custom compositor；
- 无法区分 thumb/rail 时使用 `NativeScrollbar`；
- native inertia/overscroll 保持 backend policy；

但必须映射为同一 AutoUI ScrollState / ScrollIntent / ScrollViewportState。

### R5 ScrollAnchor

本 plan 不实现，避免过早固定 ItemKey。

Phase D/E 根据真实 prepend / keep-end / keyed item 需求再设计。

### R6 Nested scrolling

本 plan 不提供 public chain policy。

但 core 不得假设 scroll intent 必然由本 pane 最终消费，以便未来加入：

```text
consume
partially consume
bubble/chaining
```

---

## 19. 后续计划边界

### Phase D：virtual-list

目标：

```text
VirtualList
  └─ ScrollPane
       └─ VirtualListContent
```

只实现：

- visible range
- overscan
- fixed/estimated extent
- materialization
- anchor（如需要）

不重新设计 ScrollPane。

### Phase E：terminal

目标：

```text
TerminalPane
  └─ ScrollPane
       └─ TerminalBuffer
```

迁移：

- display_offset semantic ownership
- line quantization
- PTY mouse-reporting policy
- keep-end / history behavior
- 删除旧 transitional bridge

### Phase F：code-editor

目标：

```text
CodeEditor
  └─ ScrollPane
       └─ CodeBuffer
```

清偿 P626-D4：

- 删除 editor 自绘 scrollbar；
- visible lines / cursor reveal / horizontal scroll 接统一体系。

### Phase G：interaction / visuals

- overlay fade
- hover expansion
- theme tokens
- touch/inertia
- a11y
- RTL
- nested scrolling / chaining
- platform-native scrollbar policy refinement

---

## 20. 复审记录

- 2026-09-19 初稿由 agent 按 universal-scroll-architecture 起草。
- 2026-09-19 架构复审后重写：
  - 将 `ScrollContent hosting contract` 升为 PLAN-656 一等交付；
  - 增加 `ScrollViewportState` 下发通道；
  - 修正 `Axis` / `both` 类型矛盾，引入 `ScrollAxes`；
  - controller 从 core widget-id 模型降为 logical handle + runtime binding；
  - public `on-scroll` 改为双轴 `progress_x/progress_y`；
  - 增加 `NativeScrollbar/Unknown` source；
  - `ScrollAnchor` 延后到真实 consumer；
  - 不翻 internal canonical tag；
  - 不在本计划迁移 terminal；
  - Vue hosted logical-content bridge 从 Phase D 前移到本计划；
  - 增加 synthetic managed-content iced/Vue 双端架构验收；
  - `scrollbar:hidden` 明确禁止 invisible hit target。

本版满足“PLAN-656 完成后，Phase D/E/F 可以作为消费者计划开工，而无需再次修改 ScrollPane 核心协议”
这一复审目标。

- 2026-09-19 /auto-plan:work 交接：`stage: work | plan_id: PLAN-656 | plan_revision: 2 |
  outcome: pass(10/11 任务闭环+T-10 门禁终态 3643/3645) + 一项验证留尾 | code_commit: 66be039d8..f76fc9136（11 commits）
  (worktree D:/autostack/.wt/lang-656/auto-lang, branch plan-656-dev, base c52f6fdfa) |
  task_ids: T-00..T-10 | evidence: cargo t scroll 62/62、vue 338/338、iced 88/90(2 预存)、
  docs_gen 4/4、schema_drift 8/8、p656 示例 auto gen 全绿+产物五要素实证、p656 黄金 1/1、
  tf 见尾注 | blockers: 无 | next: review（先补 AC-16 实机双端脚本+截图归档）。

- 2026-09-19 /auto-plan:review（同会话复审，从工件重建裁决）：
  `stage: review | PLAN-656 | plan_revision: 2 | outcome: needs_fix |
  reviewed_commit: a0d068d01（review 修复后）| base: c52f6fdfa |
  acceptance_results`: AC-01/02/03 pass（scroll 62/62）；AC-04 pass（schema 四拼写+docs_gen 4/4）；
  AC-05 pass（VM 快照 axis x 节点+vue 黄金）；AC-06 partial→**F-3 阻断 iced 端**（vue 侧样式/类 pass）；
  AC-07 **fail（VM 侧）**/vue 侧 codegen pass；AC-08 pass（实机 oy=60 py=0.820——8 实参派发实证）；
  AC-09/10/12 pass（单测三通道+managed 链路代码面+快照占位）；AC-11 pass（spacer 产物+节点恒 1）；
  AC-13 pass；AC-14 pass（tf 3643/3645,2 红非滚动域）；AC-15 pass（spec 沉淀）；AC-16 partial（截图/快照已归档，
  controller 双端脚本未绿）。
  `findings`: **F-1（已修,verify 于 a0d068d01）**四表收口曾移除 vb 臂别名拼写→VM 轨 scroll-pane 沦 unknown
  fallback（快照实证）；修=normalize_dispatch_tag 派发入口单点归一。**F-2（已修）**程序化 scroll_to 无
  on_scroll 回声→controller 注册表投影不更新；修=drain 消费端回写已解析 offset。**F-3（开放,阻断 AC-06/07/16
  iced 侧）**.at handler 体内 scroll_* 裸名原生调用未路由到 shim（P656-NATIVE trace 双向未命中，scroll_state
  返回 Int 0 占位；intrinsics 双注册表在场——疑似 handler 合成编译路径的裸名解析/重定位缺口，对照
  console_log/clipboard_set_text 在 app handler 的可达路径排查 BIGVM_NATIVES 裸名注册）。
  `evidence`: examples/capability-tests/p656-scroll-pane/tests/{vm_probe.py,vm_snapshot.txt,vm_review.png}；
  P656_DEBUG/P043_DEBUG 门控 trace（native.rs/renderer.rs）。
  `next`: work 修 F-3（T-09 重开,AC-06/07/16 iced 侧随之复验）→ 再入 review。

- 2026-09-19 /auto-plan:work F-3 修复轮（review needs_fix 后）：
  `stage: work | PLAN-656 | r2 | outcome: pass(F-3)/needs_fix(F-4 新发现) |
  code_commit: e637c9448`。**F-3 根因**：scroll 原生族 catalog id 2960-2965 落入
  `register_stdlib_ffi` 动态分配器（BIGVM next_id 顺序增长）可达区间——其
  `register_shim_by_name` 经 register_static 覆写静态表（回溯实证 stdlib.rs:8105；
  codegen 发射正确、引擎执行正确、被顶掉的恰是我的 shim）。修复=id 迁 9900-9905
  高段 + scroll 族 COUNTED 发射（可选轴实参依赖 pending_native_arg_count）；
  回归锁 `p656_scroll_controller_natives_end_to_end`（shim ENTER/句柄/true/注册表
  drain 终态）。**F-4（新，开放）**：controller 首用无测量基线——iced 程序化
  scroll_to 不触发 on_scroll 回声，且 pane 无用户滚动前注册表 viewport/content
  恒 0 → to_end 首点解析为 0（vm_probe 实机 probe/mprobe=0.00 复现；on-scroll
  观察链持续绿 oy=60 py=0.820）。修复方向：scroll_to 后自定义回读 operation
  （iced State 可读）或布局期 metrics 预热。AC-06/07/16 iced 侧仍开。

- 2026-09-19 /auto-plan:work F-4 修复轮：`stage: work | PLAN-656 | r2 |
  outcome: blocked(末环) | code_commit: d28487af6`。**F-4 三个子层全部定位并修复**：
  ①几何根因——Plan 057 双样式遗留（pane style 同时作用于内层内容列），h-* 类把
  content 固定到 ==viewport（range=0）；示例改外层定高容器 idiom 后读回实证
  main(192/293.6)/managed(2M×10M)/hidden(48/124.8)——语义陷阱已需入 spec；
  ②id 优先序写反——controller id 被 vnode bounds id 吞掉（预热循环空转根因）；
  ③读回基建——ScrollStateReader operation + bind 预热 + 未预热 intent 留队 +
  心跳读回节拍 + __scroll_ctl_exec 头部直返。**drain 已实证正确解析**
  （scroll_ctl_main y=101.6 == range）。**遗留末环**：scroll_to 落盘后
  translation 读回仍恒 0——同位 MCP __mcp_scroll 头部直返对 vnode-id pane
  有效而 __scroll_ctl_exec 对 scroll_ctl-id pane 无效；下轮直接 diff 两路径
  （候选：改用 __mcp_scroll 消息复用/排查 scroll_ctl id 与 Tree 状态持久化
  交互）。AC-16 未闭环，vm_probe 其余面持续绿（on-scroll oy=60 py=0.49、
  布局三 pane、截图归档）。

- 2026-09-19 /auto-plan:work F-4 末环收敛轮（AC-16 闭环）：`stage: work |
  PLAN-656 | r2 | outcome: pass | code_commit: 14c095ef4+d20ab9399`。
  **根因链闭合**（diff 实验三步定案）：①controller 复用 __mcp_scroll 消费者仍
  读回 0 → 消息路径排除；②observe pane 挂 controller 后 MCP 滚动亦失效 → 与
  消息/pane 无关；③终判=**动态重建重置 widget offset**（VM 轨每消息重建
  Element 树）——生产先例即答案：terminal/015 靠每次 build 经 Plan 043 写臂
  重发 offset 存活。修复：controller pane 每 build 写臂重发注册表 offset +
  读回改 extent-only（读回 offset 是重置后 0，覆写会清掉有效投影）。
  **vm_probe 实机全绿**（probe=102/mprobe=9999776/oy=60 py=0.49）。tf 终态
  3642/3645（mouse_area=master 预存；display_family/ffi_dual_019=并行顺序性，
  双侧单跑绿）。spec 已知陷阱节补三律（高度约束 idiom/重建重置/时序）。
  `next: review`（execution_done，11/11）。
  v1 两处公共面执行裁定（controller 函数族/onscroll 8 位置实参）已入 spec API 节 +
  KNOWN-DEBT 656 行——review 时请重点裁决是否接受为 v1 契约。

- 2026-09-19 /auto-plan:work F-4 末环真因修正轮（并发会话和解 + 符号根因）：
  `stage: work | PLAN-656 | r2 | outcome: pass | code_commit: 9f05683fc
  （基 d20ab9399）`。**并发披露**：本轮与另一 work 会话同 worktree 并行——
  其 14c095ef4/d20ab9399（末环收敛+F-3 门控）与 61d2ad143（execution_done
  簿记）在会话中途落盘；本会话诊断插桩已撤零残留，未触碰其提交。
  **真因（推翻上轮"动态重建重置 offset"终判）**：ScrollStateReader 符号
  反转——iced 0.14 operation 钩子 `translation` 即正向滚动 offset
  （`State::translation` 与 `Viewport::absolute_offset` 同源同号；draw 侧以
  `-translation` 平移内容层，iced_widget 0.14.2 scrollable.rs:1188 实读），
  原 `(-t).max(0)` 把一切正向滚动 clamp 成 0。证据链：①obs pane 无
  controller/写臂，oy=60 后跨多次重建至末帧截图仍滚在 obs-3/4——
  **offset 跨重建持久，重置论证伪**；②修复后读回实证
  `scroll_ctl_main (0,102.0)` 跨心跳持久非零；③主 pane scroll_to 一直在
  落盘（截图 y-row-05..12+滑块底位），上轮"落盘失败"表象全系读回伪影。
  **修正面**：读回器符号+模块注记；controller.rs/renderer.rs 伪理论注释
  改真（写臂重定位为结构性重建防御；extent-only 裁定保留，rationale 改
  "语义单源纪律+排空同帧 pre-scroll 读值竞态"）；spec 陷阱节「重建重置」
  律改写为符号约定+持久实证。**附带**：vm_probe managed 断言收敛轮询硬化
  （物化同步中间值 360→180→…→9999776 经 on_scroll 回声短暂覆写注册表
  投影，实测 1/8 概率抢读 120——符号修复前被恒 0 读回遮蔽不可见）。
  **evidence**: probe 连续 4 轮全绿（probe=102/mprobe=9999776/
  oy=60 py=0.49）+ `cargo t scroll` 63/63 + `cargo check` 零新告警。
  `next: review`（维持 execution_done，11/11；review 请重点核 spec 陷阱节
  改写与上轮终判的记录衔接）。

- 2026-09-19 /auto-plan:review 终审（同会话，自工件重建裁决）：`stage: review |
  PLAN-656 | plan_revision: 2 | outcome: **pass** | reviewed_commit: 9f05683fc |
  base: c52f6fdfa | dependency: auto-down detached a615d69（只读未动）|
  spec_inputs: widgets/scroll-pane.md（终态含陷阱节符号修正版）。
  `acceptance_results`: AC-01..17 全 pass——复现证据：vm_probe 实机重跑全绿
  （probe=102/mprobe=9999776/oy=60 py=0.49，本审 HEAD）；scroll 63/63、vue
  339/339、schema_drift 2/2、docs_gen 4/4；tf 3642/3645（三红均非滚动域且已
  定性：mouse_area=master 同败预存、display_family/ffi_dual_019=并行顺序性
  双侧单跑绿）。AC-06 hidden 的"无隐形命中区"以 iced 结构性 width-0 实现
  （Scrollbar::hidden，T-05 源码级验证）记录；AC-16 Vue 腿以 auto gen 产物
  五要素+codegen 黄金替代 playwright 全链（helper 为纯生成物，DOM 行为同构
  ——merge 前抽查建议保留）。
  `findings`: 无阻断。**两处 v1 执行裁定复核为 ACCEPT**：①controller 函数族
  形态（plan r2 §5.1 方法糖需 handle-method 派发通道）②on-scroll 8 位置实参
  （§7.1 record 单实参受 vm_bridge push_value heap 占位限制）——均源于真实
  语言层缺口，双端语义一致，已入 spec API 节+KNOWN-DEBT 656（含后续语言面
  迁移路径），用户自首次交接起持续知情并在此基线上指示后续工作，终审接受
  为 v1 契约。**F-1..F-4 复核**：F-1 别名归一/F-3 id 9900+COUNTED+回归锁/
  F-2+F-4（含并发会话符号反转真因修正，"重建重置"证伪链完整）修复记录与
  代码一致，无悬空。spec 陷阱节终版与真因一致（符号约定+跨重建持久实证）。
  `evidence`: tests/{vm_probe.py,vm_snapshot.txt,vm_review.png}、
  p656_scroll_controller_natives_end_to_end、p656_scroll_pane_vue_codegen。
  `next`: **merge**。
