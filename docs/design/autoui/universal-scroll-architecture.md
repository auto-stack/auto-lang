# AutoUI Universal Scroll Architecture

> **定性**：AutoUI 通用滚动架构设计（替代原 `universal-scrollbar.md`）。  
> **状态**：draft（2026-09-19）——架构与公共语义基线；具体实现计划另立。  
> **目标**：统一普通布局、虚拟列表、自管理 buffer 与高级复合组件的滚动模型；在 AutoLang 表面保持最小 API，同时允许 iced、Vue 与未来 ArkTS / Android / iOS 等后端复用同一语义层。

> **关联**：
> - roadmap「虚拟滚动容器（scrollable 虚拟化）」条目（PLAN-019 冒烟期方向共识）
> - auto-term PLAN-022（终端虚拟滚动消费端）
> - vm-frame-budget §5.3 D-3（for 列表虚拟化）
> - Design 20 AutoUI 分离架构
> - Design 29 Style & Theme
> - Plan 043 scrollable 双臂（`offset` 写入 + `on_scroll` 读出）
> - KNOWN-DEBT：P626-D4（code_editor 自绘滚动条 vs 标准 scroller 长期统一）
> - auto-down PLAN-051 延后项「统一滚动条 widget」

---

## 1. 结论先行

AutoUI 需要的不是一个孤立的 `Scrollbar` 组件，而是一套统一的 **Scroll Architecture**。

对普通 AutoLang 用户而言，滚动仍然应该非常简单：

```auto
scroll-pane {
    col {
        ...
    }
}
```

而具有天然 viewport 语义的高级组件，例如：

```auto
code-editor(...)
terminal-pane(...)
virtual-list(...) { ... }
```

应当直接可用，不要求用户额外套一层 `scroll-pane`。它们在组件内部复用统一的 `ScrollPane` 基础设施：

```text
CodeEditor
   └─ ScrollPane
       └─ CodeBuffer

TerminalPane
   └─ ScrollPane
       └─ TerminalBuffer

VirtualList
   └─ ScrollPane
       └─ VirtualListContent
```

因此：

> **`scroll-pane` 是滚动 viewport primitive；高级组件可以将其封装起来。**

框架层统一，用户层保持最简。

---

## 2. 问题陈述

### 2.1 当前滚动能力存在多套实现

现有 AutoUI / 消费端中，滚动能力分散在不同层：

| 实现 | 形态 | 主要局限 |
|---|---|---|
| iced 官方 scrollable | 真容器滚动 | 依赖实际 child layout extent；不能自然表达虚拟内容 / 自管理 buffer |
| code_editor 自绘 | 编辑器内部自行滚动/绘制 | 与系统级 scroll 行为、视觉、命中规则分裂 |
| terminal 历史实现 | terminal 自绘滚动条或专用绑定 | 与终端 buffer 强耦合，难复用 |
| auto-down custom scrollbar | 外置条 + 事件 | 演示级，尚未形成通用协议 |
| Vue ScrollArea / CSS | 浏览器原生滚动体系 | 与 iced 语义层没有统一契约 |

原始问题看似是“做一套统一滚动条”，实际更深：

> **需要把滚动状态、滚动输入、viewport、滚动条视觉与内容宿主解耦。**

### 2.2 必须统一的内容类型

新架构必须同时支持：

1. **普通真实布局**：如 `col` / `row` / 普通 AutoUI component；
2. **虚拟内容**：如 `virtual-list`，只物化可见区域；
3. **自管理 buffer**：如 terminal / editor，大量内容不以完整 UI 树存在；
4. **自绘或逻辑画布**：如 canvas / spreadsheet / timeline；
5. **未来自定义组件**：用户自行实现大型内容源。

关键原则：这些差异是实现分类，而不是 AutoLang API 的 `mode` 分类。

---

## 3. 设计原则

### P1. 用户看到的是“可滚动 viewport”，不是 Adapter

普通用户只需要：

```auto
scroll-pane {
    child
}
```

不得要求普通用户声明：

```text
mode: virtual
mode: terminal
adapter: xxx
logical-height: xxx
```

这些属于内部实现。

### P2. 高级组件应封装滚动 primitive

如果一个组件天然具有 viewport 语义，它应该自己包含 `ScrollPane`：

- `code-editor`
- `terminal-pane`
- `virtual-list`
- `tree-view`
- `data-grid`
- `log-view`
- `pdf-view`
- `spreadsheet`

用户不需要额外包装。

### P3. ScrollPane 不理解业务内容类型

`ScrollPane` 不应该知道 child 是：

```text
VirtualList / Terminal / CodeEditor / Canvas / ordinary Col
```

它只与统一的 `ScrollContent` 能力交互。

### P4. State down, Intent up

滚动协议采用单向语义：

```text
Host / ScrollContent
        │
        │ ScrollState
        ▼
     ScrollPane
        │
        │ ScrollIntent
        ▼
Host / ScrollContent
```

避免使用“双向 offset 绑定 + suppress echo”作为架构核心。

### P5. 后端无关

AutoUI Scroll Architecture 不依赖 iced、DOM、CSS、ArkUI、Compose 或 UIKit 的具体滚动模型。

后端只负责把统一 Scroll 语义映射到本地能力。

---

## 4. 总体架构

```text
┌────────────────────────────────────────────┐
│            AutoUI High-level UI            │
│                                            │
│  scroll-pane     code-editor   terminal    │
│       │               │            │       │
│       │          internal       internal   │
│       │          ScrollPane     ScrollPane  │
└───────┼───────────────┼────────────┼───────┘
        │               │            │
        ▼               ▼            ▼
┌────────────────────────────────────────────┐
│                 ScrollPane                 │
│                                            │
│  viewport / clipping / input / scrollbar  │
│  nested-scroll / controller / a11y        │
└──────────────────────┬─────────────────────┘
                       │
                       │ ScrollContent protocol
                       ▼
┌────────────────────────────────────────────┐
│                ScrollContent               │
│                                            │
│ LayoutScrollContent                       │
│ VirtualListContent                        │
│ CodeBuffer                                │
│ TerminalBuffer                            │
│ Canvas / custom content                   │
└──────────────────────┬─────────────────────┘
                       │
                       ▼
┌────────────────────────────────────────────┐
│              Scroll Model / Host           │
│                                            │
│ ScrollState / ScrollIntent / Anchor        │
└──────────────────────┬─────────────────────┘
                       │
         ┌─────────────┼─────────────┐
         ▼             ▼             ▼
       iced           Vue        Future backend
                               ArkTS / Compose / iOS
```

---

## 5. AutoLang 公共 API

### 5.1 普通内容

```auto
scroll-pane {
    col {
        text("A")
        text("B")
        ...
    }
}
```

默认：

```text
axis: y
scrollbar: auto
```

等价于：

```auto
scroll-pane(
    axis: y,
    scrollbar: auto
) {
    col { ... }
}
```

### 5.2 横向滚动

```auto
scroll-pane(axis: x) {
    row {
        ...
    }
}
```

### 5.3 双轴滚动

```auto
scroll-pane(axis: both) {
    canvas(...)
}
```

内部不应把 `both` 实现成一个单值 offset，而应维护独立的 x/y axis state。

### 5.4 Scrollbar 可见策略

```auto
scroll-pane(scrollbar: auto)   { ... }
scroll-pane(scrollbar: always) { ... }
scroll-pane(scrollbar: hidden) { ... }
```

v1 建议仅公开这三种策略；宽度、颜色、圆角、hover 动画等归 Theme。

### 5.5 程序化控制

需要外部控制时，使用 typed controller，而不是重新引入双向 offset binding：

```auto
let scroll = scroll-controller()

scroll-pane(controller: scroll) {
    col { ... }
}

button("Bottom") {
    on-click: scroll.to-end()
}
```

控制能力可包括：

```text
scroll.to-start()
scroll.to-end()
scroll.scroll-to(offset)
scroll.scroll-by(delta)
scroll.reveal(target)
```

### 5.6 观察滚动

外部监听是观察接口，不是控制协议：

```auto
scroll-pane(
    on-scroll: |state| {
        progress = state.progress
    }
) {
    article
}
```

应明确区分：

```text
on-scroll              -> observe
on-scroll-intent       -> internal control protocol
```

---

## 6. 高级组件的使用方式

### 6.1 CodeEditor

用户：

```auto
code-editor(
    document: doc
)
```

内部概念结构：

```auto
component code-editor(...) {
    view {
        row {
            gutter(...)

            scroll-pane {
                code-buffer(...)
            }
        }
    }
}
```

职责分层：

```text
CodeEditor
  selection / cursor / IME / syntax / search / commands

ScrollPane
  viewport / clip / scrollbar / scroll input / nested scroll

CodeBuffer
  logical extent / visible lines / virtualization / text offset mapping
```

### 6.2 TerminalPane

用户：

```auto
terminal-pane(session: term)
```

内部：

```auto
component terminal-pane(session) {
    view {
        scroll-pane {
            terminal-buffer(session)
        }
    }
}
```

`TerminalBuffer` 把统一 `ScrollIntent` 映射到 terminal engine 的 `display_offset` / mouse-reporting policy。

### 6.3 VirtualList

用户直接：

```auto
virtual-list(items: messages) {
    |message|
    message-row(message)
}
```

内部：

```auto
component virtual-list(...) {
    view {
        scroll-pane {
            virtual-list-content(...) {
                ...
            }
        }
    }
}
```

其中：

- `VirtualList` 是高级可滚动组件；
- `VirtualListContent` 实现 `ScrollContent`；
- visible range / overscan / materialization 属于 virtual-list，不属于 ScrollPane。

---

## 7. ScrollContent Capability

### 7.1 定义

`ScrollContent` 是 ScrollPane 与直接内容节点之间的能力协议。

它不是用户通常需要直接操作的 Adapter。

概念上：

```text
ScrollContent {
    state() -> ScrollState
    apply(intent: ScrollIntent)
}
```

### 7.2 两种 child

从 ScrollPane 角度，child 只有两类：

```text
                      Child
                        │
            ┌───────────┴───────────┐
            │                       │
  no ScrollContent capability    has ScrollContent
            │                       │
            ▼                       ▼
  implicit LayoutScrollContent   delegate to child
```

这是语言层真正需要的分类。

### 7.3 普通布局：隐式 ScrollContent

```auto
scroll-pane {
    col { ... }
}
```

`col` 不需要实现 `ScrollContent`。

ScrollPane 自动产生：

```text
LayoutScrollContent
  content_extent = measured layout extent
  offset = ScrollPane internal state
```

### 7.4 内建特殊内容

以下内容可直接实现 `ScrollContent`：

```text
CodeBuffer
TerminalBuffer
VirtualListContent
CanvasContent
SpreadsheetContent
```

### 7.5 自定义 ScrollContent

高级 AutoLang component 可以声明能力，例如概念语法：

```auto
component huge-log-content(logs) {
    scroll-content {
        state: scroll-state
        on-scroll-intent: handle-scroll
    }

    ...
}
```

其宿主高级组件仍可把 ScrollPane 封装在内部。

---

## 8. 核心 Scroll Model

### 8.1 ScrollState

建议逻辑滚动空间使用 `f64`，渲染几何进入具体 backend 时再转换为 `f32` / native number。

```rust
struct ScrollAxisState {
    offset: f64,
    viewport_extent: f64,
    content_extent: f64,
}
```

二维场景：

```rust
struct ScrollState {
    x: Option<ScrollAxisState>,
    y: Option<ScrollAxisState>,
}
```

约束：

```text
scroll_range = max(content_extent - viewport_extent, 0)
offset = clamp(offset, 0, scroll_range)
```

### 8.2 ScrollIntent

建议统一滚动意图，而不是直接双向写 offset：

```rust
enum ScrollIntent {
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

    ToStart { axis: Axis },
    ToEnd   { axis: Axis },
}
```

### 8.3 ScrollSource

```rust
enum ScrollSource {
    Wheel,
    Touchpad,
    Touch,
    ThumbDrag,
    RailClick,
    Keyboard,
    Programmatic,
    Sync,
    FocusReveal,
    AutoScroll,
}
```

宿主可以根据 source 采用不同策略。

例如 Terminal：

```text
Touchpad   -> 累积亚行 delta
ThumbDrag  -> 直接映射 display_offset
Wheel      -> 按行滚动或交给 PTY mouse-reporting policy
Programmatic -> 输入/输出引发贴底或 reveal
```

### 8.4 ScrollController

`ScrollController` 是对 ScrollPane / Scroll Model 的程序控制句柄。

它不是状态源；状态仍归对应 ScrollContent / ScrollModel 所有。

---

## 9. Anchor 与动态内容

仅有 absolute offset 不足以覆盖聊天流、日志、终端历史等动态内容。

例如在顶部 prepend 数据时：

```text
旧内容顶部插入 100 项
content_extent 增加
```

如果只保持数值 offset 不变，视觉位置会跳。

因此架构应预留 `ScrollAnchor`：

```rust
enum ScrollAnchor {
    AbsoluteOffset,
    KeepEnd,
    Item {
        key: ItemKey,
        viewport_offset: f64,
    },
}
```

典型语义：

```text
chat at bottom + new message   -> KeepEnd
chat reading history + append  -> preserve current anchor
prepend history                -> preserve visible item position
terminal at bottom             -> follow new output
terminal scrolled up           -> do not steal viewport
```

Anchor 属于 Scroll Model / 内容宿主，不属于 Scrollbar。

---

## 10. ScrollPane 输入职责

### 10.1 ScrollPane 负责的输入

ScrollPane 作为 viewport，应统一处理或分派：

```text
mouse wheel
touchpad
touch pan
keyboard scrolling
scrollbar thumb drag
rail click
nested-scroll propagation
programmatic controller
```

但具体消费策略可委托 ScrollContent。

### 10.2 Terminal 等特殊输入策略

Terminal 的 alternate screen / mouse reporting 等业务策略仍属于 terminal 层。

ScrollPane 只产生统一输入意图；TerminalBuffer 决定：

```text
ScrollIntent::Wheel
    ├─ scroll history
    └─ send PTY mouse event
```

---

## 11. Nested Scroll 与 Scroll Chaining

允许：

```auto
scroll-pane {
    col {
        header

        code-editor(...)

        footer
    }
}
```

这里外层 ScrollPane 与 CodeEditor 内部 ScrollPane 是两个合法 viewport。

基本规则：

```text
inner can consume
    -> inner consumes

inner reaches edge
    -> according to chain policy
       propagate remaining delta to parent
```

未来可公开：

```text
chain: auto
chain: contain
chain: parent
```

v1 可先固定合理默认行为，不急于暴露完整 API。

---

## 12. Scrollbar 只是 ScrollPane 的视觉 primitive

### 12.1 定位

`Scrollbar` 不再是整个架构的中心。

它只负责：

```text
rail
thumb
hover
drag
pointer capture
visual feedback
```

内容、virtualization、anchor、wheel policy 都不属于 Scrollbar。

### 12.2 视觉宽度与命中宽度分离

建议保持现代 overlay 视觉：

```text
idle visual thumb    ~3px
hover/drag visual    ~5-6px
interaction gutter   显著宽于视觉宽度
```

具体值由 Theme token 决定。

不得令 3px 视觉宽度同时成为 hit-test width。

### 12.3 Thumb 几何

```text
scroll_range = max(content_extent - viewport_extent, 0)
thumb_ratio  = viewport_extent / content_extent
raw_thumb    = rail_extent * thumb_ratio
thumb_extent = clamp(raw_thumb, min_thumb_extent, rail_extent)
thumb_travel = rail_extent - thumb_extent
thumb_pos    = offset / scroll_range * thumb_travel
```

反向拖动：

```text
offset = thumb_pos / thumb_travel * scroll_range
```

`min_thumb_extent` clamp 后必须使用 `thumb_travel` 做映射，不能继续直接按 content 比例映射。

### 12.4 Pointer Capture

拖拽必须：

```text
pointer down thumb
    -> capture pointer

pointer move
    -> continue drag even outside rail

pointer up / cancel / blur / destroy
    -> release capture
```

内部可使用简单状态机：

```text
Idle
  ↕
Hover
  ↓ pointer down
Dragging { grab_offset }
  ↓ pointer up/cancel
Idle
```

---

## 13. Virtualization 的边界

ScrollPane 不负责 item virtualization。

它只理解：

```text
logical content extent
viewport extent
current offset
ScrollIntent
```

VirtualList 自己负责：

```text
item count
item extent / estimated extent
visible range
overscan
materialization
key / recycling
variable-height correction
```

因此：

```text
ScrollPane cannot distinguish:

ordinary layout:  content_extent = 12,000 px
virtual list:     content_extent = 12,000,000 px
terminal buffer: content_extent = rows × cell_height
```

这是架构成功的重要验收条件。

---

## 14. Backend 抽象

### 14.1 核心原则

Scroll Architecture 必须位于 AutoUI 语义层，而不是某个 backend 之上。

正确依赖方向：

```text
AutoLang / AutoUI Scroll Semantics
               │
               ▼
       Backend Scroll Adapter
          │             │
          ▼             ▼
        iced            Vue
                          │
                 future backends
```

不得设计成：

```text
iced::Scrollable
      ↓
AutoUI ScrollPane wrapper
```

否则后续 backend 会被 iced 的行为模型绑死。

### 14.2 Backend 需要实现的能力面

抽象上 backend 需要提供：

```text
viewport clipping
layout / measure
input normalization
pointer capture
native or custom scrolling
scrollbar rendering / styling
scroll offset application
scroll state observation
nested-scroll bridge
accessibility semantics
```

具体 backend 可以选择：

1. **Native-backed**：尽量使用平台原生 scroll container；
2. **Hybrid**：原生 viewport + AutoUI ScrollModel；
3. **Custom**：AutoUI 自己维护 offset / clipping / scrollbar。

架构不要求所有 backend 使用同一种物理实现。

---

## 15. iced Backend

### 15.1 v1 策略

iced 目前已有成熟 `scrollable`，因此 v1 可优先复用其：

```text
viewport
wheel / pointer behavior
scrollbar hit-testing
scroll_to mechanism
```

但必须通过 AutoUI backend adapter 封装，不能让 iced 类型进入 AutoLang Scroll 协议。

### 15.2 普通布局

```text
ScrollPane
   -> iced Scrollable
   -> real child layout
```

由 measured content extent 形成 implicit `LayoutScrollContent`。

### 15.3 自管理内容

对于 Terminal / VirtualList / CodeBuffer：

- ScrollModel 的语义状态由内容宿主持有；
- iced scrollable 可以承担 viewport / input / scrollbar；
- 必要时使用 logical/virtual extent bridge；
- 具体 host 把 `ScrollIntent` 映射到自身 offset；
- 不允许 iced offset 反客为主成为业务状态源。

### 15.4 实现风险

需要在正式计划中验证：

- iced 是否能自然表达 virtual extent；
- 自定义 content extent / placeholder 的布局成本；
- programmatic scroll_to 与用户输入的同步时序；
- nested scroll 行为；
- pointer capture 与 overlay scrollbar 可定制能力；
- 双轴滚动限制。

如果 iced 原生 scrollable 无法满足部分能力，可在 backend 内部逐项替换，不影响 AutoUI 公共语义。

---

## 16. Vue Backend

Vue / Web 后端通常更容易落地，因为浏览器已经提供完整滚动基础设施。

### 16.1 普通内容

可映射为：

```text
ScrollPane
   -> overflow: auto / scroll
```

浏览器承担：

```text
viewport clipping
wheel / touchpad / touch
scroll offset
scrollbar（可原生或 CSS 定制）
nested scrolling
```

### 16.2 ScrollState

可从 DOM scroll container 映射：

```text
offset          <- scrollTop / scrollLeft
viewport_extent <- clientHeight / clientWidth
content_extent  <- scrollHeight / scrollWidth
```

### 16.3 自管理 / 虚拟内容

VirtualList / Terminal 仍应遵循统一 ScrollContent 协议。

Vue backend 可采用：

- spacer / virtual canvas 提供 logical extent；
- DOM `scrollTop` 作为 viewport input；
- virtual content 根据 ScrollState materialize visible range；
- terminal/code-buffer 映射逻辑 offset。

### 16.4 不应泄露 DOM 特性

即使 Vue 实现容易，也不能把以下内容变成 AutoUI 核心语义：

```text
scrollTop
scrollHeight
CSS overflow
DOM event names
```

它们只存在于 Vue backend adapter 内。

---

## 17. 未来 Backend：ArkTS / Android / iOS

本架构原则上不受未来 backend 增加影响。

未来 backend 只需要实现同一 Scroll backend contract。

### 17.1 HarmonyOS / ArkTS

可以把 ScrollPane 映射到 ArkUI 对应 scroll/container 能力；AutoUI 仍提供统一：

```text
ScrollState
ScrollIntent
ScrollController
ScrollContent
```

平台特有 fling / edge effect / accessibility 由 backend 适配。

### 17.2 Android / Jetpack Compose

Compose 可使用平台 scroll state / lazy container / nested scroll 能力作为实现材料，但它们属于 backend。

`virtual-list` 是否映射 LazyColumn，或继续使用 AutoUI 自己的 virtualization，可由 backend 与组件实现策略决定，不改变 AutoLang API。

### 17.3 iOS

无论最终通过 UIKit / SwiftUI 或其他 bridge，仍然只需要把原生 scrolling 与 AutoUI ScrollState / Intent 做映射。

### 17.4 必须提前抽象掉的平台差异

为了保证未来 backend 真正可插拔，核心协议不能假设：

```text
只有 mouse wheel
只有 pixel scroll
只有 vertical axis
scrollbar 永远可见
scrollbar 一定由框架绘制
存在 DOM
存在 iced widget tree
程序滚动一定同步完成
```

必须允许 backend 处理：

```text
touch / fling / inertia
platform overscroll / edge effects
nested scroll
native scrollbar policies
RTL
accessibility
fractional offsets
asynchronous animation
safe-area / overlay behavior
```

这些是 backend policy / capability，不应污染上层内容协议。

---

## 18. Backend Capability 与 Feature Negotiation

不同平台原生 scroll 能力不完全一致，因此 backend 可以声明 capability：

```text
native_scrollbar
custom_scrollbar
pointer_capture
inertial_scroll
nested_scroll
animated_programmatic_scroll
bidirectional_scroll
rtl_scroll
overscroll_effect
```

AutoUI ScrollPane 根据 capability 选择：

```text
native implementation
hybrid implementation
custom fallback
```

这与 child 的 `ScrollContent capability` 是两层不同的能力协商：

```text
Child capability       -> 内容如何滚
Backend capability     -> 平台如何实现滚动
```

两者不要混在一起。

---

## 19. 推荐内部类型

建议核心命名：

```text
AutoLang primitive
    scroll-pane

Legacy compatibility alias
    scrollable

Runtime component
    ScrollPane

Content capability
    ScrollContent

Read-only state
    ScrollState / ScrollAxisState

Input command
    ScrollIntent

Input origin
    ScrollSource

Programmatic handle
    ScrollController

Dynamic positioning
    ScrollAnchor

Visual primitive
    Scrollbar

Ordinary-child wrapper
    LayoutScrollContent

Backend boundary
    ScrollBackend / backend-specific adapter
```

其中 `scrollable` 可保留兼容，但新文档、新实现与新 API 推荐统一使用 `scroll-pane`。

---

## 20. 与旧 `scrollable` 的兼容

### 20.1 兼容目标

旧代码：

```auto
scrollable {
    col { ... }
}
```

应继续工作。

可以把它解释为：

```auto
scroll-pane {
    col { ... }
}
```

### 20.2 迁移原则

- v1 不强制全仓改名；
- runtime 内部优先统一到 ScrollPane；
- 新功能只在新架构上演进；
- `scrollable` 最终可以长期作为 alias，或在未来版本再决定弃用策略。

---

## 21. 建议落地路径

### Phase A — Core semantics

实现纯 AutoUI 语义层：

```text
ScrollState
ScrollIntent
ScrollSource
ScrollController
ScrollContent capability
axis/range/clamp
Scrollbar geometry
```

不绑定 iced / Vue。

### Phase B — iced ScrollPane

基于现有 `build_scrollable` / `scrollbar_style` / offset 双臂能力重构：

```text
legacy scrollable
      ↓
AutoUI ScrollPane semantics
      ↓
iced backend adapter
```

先完成普通 LayoutScrollContent。

### Phase C — Vue ScrollPane

把统一语义映射到 DOM/CSS scroll container。

完成与 iced 一致的 AutoLang API 与事件语义。

### Phase D — VirtualList

实现：

```text
VirtualList
   └─ ScrollPane
       └─ VirtualListContent
```

v1 先固定 item extent；variable-height / estimate correction 后续。

### Phase E — Terminal

把现有 terminal 虚拟滚动逻辑迁移为：

```text
TerminalPane
   └─ ScrollPane
       └─ TerminalBuffer : ScrollContent
```

保留 terminal engine 为 display offset 单源。

### Phase F — CodeEditor

清偿 code_editor 自绘 scroll debt：

```text
CodeEditor
   └─ ScrollPane
       └─ CodeBuffer : ScrollContent
```

### Phase G — Anchor / nested / mobile quality

完善：

```text
ScrollAnchor
nested scroll / chaining
touch / inertia
accessibility
RTL
animated programmatic scrolling
```

---

## 22. 验收标准

### A. API 一致性

以下写法成立：

```auto
scroll-pane { col { ... } }
code-editor(...)
terminal-pane(...)
virtual-list(...) { ... }
```

用户不需要知道 Adapter / virtual mode / terminal mode。

### B. 组件复用

CodeEditor、Terminal、VirtualList 不再各自实现独立 scrollbar / viewport 输入体系，而是复用 ScrollPane。

### C. 内容透明性

ScrollPane 本身不能区分：

```text
ordinary layout
virtual list
terminal buffer
code buffer
custom canvas
```

差异只能通过 ScrollContent capability 表达。

### D. Backend 透明性

同一个 AutoLang：

```auto
scroll-pane { ... }
```

可由 iced 与 Vue 产生各自实现，而公共语义一致。

未来 ArkTS / Compose / iOS backend 不需要修改 AutoLang Scroll API。

### E. 状态单源

自管理内容的业务 scroll state 必须留在宿主，不允许 Scrollbar / backend native offset 无意中成为第二状态源。

### F. Virtualization 独立

ScrollPane 不包含 list item virtualization 逻辑。

### G. 普通内容零负担

普通 `col / row` 不需要实现 ScrollContent；ScrollPane 自动生成 LayoutScrollContent。

---

## 23. 开放问题

| # | 问题 | 当前倾向 |
|---|---|---|
| Q1 | `scrollable` 是否长期作为 alias？ | 是；新实现和文档使用 `scroll-pane` |
| Q2 | AutoLang 自定义 `ScrollContent` 的具体声明语法？ | 使用 capability block；不暴露 Adapter 对象 |
| Q3 | ScrollController 是显式对象还是 widget id 命令？ | 倾向 typed controller；可保留 id 形式薄封装 |
| Q4 | iced v1 是否完全依赖原生 scrollable？ | 优先复用；能力不足时只替换 backend 内部实现 |
| Q5 | Vue scrollbar 默认原生还是 AutoUI overlay？ | 语义统一；视觉策略由 Theme/backend 决定 |
| Q6 | VirtualList v1 是否支持变高？ | 否；固定高度先落地 |
| Q7 | Anchor 是否 v1 必须？ | Core 预留数据模型；可在 chat/terminal 迁移阶段实现 |
| Q8 | nested scroll policy 是否 v1 暴露？ | 默认策略先实现；高级属性后续公开 |
| Q9 | 双轴滚动与 RTL 如何进入首版？ | 数据模型首版支持；完整平台行为可分阶段实现 |
| Q10 | native scrollbar 与 AutoUI custom scrollbar 如何统一主题？ | 由 Backend capability + Theme policy 决定 |

---

## 24. 最终设计判断

这一架构的核心不是“发明一个更强的 scrollbar”，而是建立 AutoUI 的统一滚动语义层：

```text
ScrollPane
+ ScrollContent
+ ScrollState
+ ScrollIntent
+ ScrollController
+ ScrollAnchor
+ Scrollbar
+ Backend Adapter
```

对用户来说，复杂度被压缩为两种体验：

### 普通任意内容需要滚动

```auto
scroll-pane {
    col { ... }
}
```

### 天然可滚动的高级组件

```auto
code-editor(...)
terminal-pane(...)
virtual-list(...) { ... }
```

高级组件内部自行复用 ScrollPane。

这样既可以兼容今天的 iced / Vue 双后端，也为未来 ArkTS、Android/Jetpack Compose、iOS 以及其他 backend 保留稳定的抽象边界。

> **AutoLang API 保持稳定；内容模型和平台实现都可以独立演进。**

这应当成为 AutoUI 滚动系统的长期架构基线。
