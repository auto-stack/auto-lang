---
plan_id: PLAN-656
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: universal-scroll-pane
author: [zhaopuming, agent]
created_at: 2026-09-19
updated_at: 2026-09-19

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - docs/specs/widgets/scroll-pane.md
touched_goals: [GOAL-007]      # AutoUI 跨端一致：scroll-pane 双端同语义

affects: [widgets, auto-lang]  # docs/specs/widgets/**、docs/specs/auto-lang/project.md ui 行
current_step: 0
total_steps: 9
---

# [PLAN-656] universal-scroll-pane

## 0. 变更摘要

按 [docs/design/autoui/universal-scroll-architecture.md](../design/autoui/universal-scroll-architecture.md)（2026-09-19 draft）
落地通用滚动架构的 **primitive 本体**（设计文档 Phase A + B + C）：

1. **Phase A 核心语义层**：后端无关的 `ui/scroll/` 模块——`ScrollState / ScrollAxisState /
   ScrollIntent / ScrollSource / ScrollAnchor(预留) / ScrollbarGeometry(纯函数)`，f64 逻辑空间，
   不依赖 iced/Vue。
2. **AutoLang 表面**：新 tag `scroll-pane`（canonical），props `axis: y|x|both`（默认 y）、
   `scrollbar: auto|always|hidden`（默认 auto）、`controller:`、`on-scroll:`；现有
   `scrollable` / `scroll` / `Scroll` 全部保留为 alias，旧代码零改动继续工作。
3. **iced 后端**：`build_scrollable` 升级为 scroll-pane 语义——axis 三值真支持（当前
   `direction` prop 被 iced 端忽略）、scrollbar 策略、controller 程序化控制（复用 Plan 043
   写臂队列机制）、通用路径 `on-scroll` 观察接线（当前只有 autodown 文档路径接双臂）。
4. **Vue 后端**：plain/shadcn 双模式映射 + 与 iced 同形的 `on-scroll` ScrollState 参数 +
   controller（sentinel-ref 模式）。
5. **双端验证**：`examples/capability-tests/p656-scroll-pane/` + autoui-verifier 双端脚本。

**非目标**（设计文档 Phase D/E/F/G，另立计划）：virtual-list 组件、terminal 迁移、
code_editor 自绘滚动条清偿（P626-D4）、ScrollAnchor 运行时消费、nested-scroll 策略 API、
touch/inertia/a11y/RTL、AutoLang 层自定义 ScrollContent 声明语法（设计文档 Q2 开放问题）。

## 1. 目标

### 1.1 用户可见目标

普通用户（设计文档 P1/P2/G 原则）：

```auto
scroll-pane { col { ... } }                  // 纵向，默认 axis: y, scrollbar: auto
scroll-pane(axis: x) { row { ... } }         // 横向（iced 端首次真正生效）
scroll-pane(axis: both) { grid { ... } }     // 双轴独立 x/y 状态
scroll-pane(scrollbar: hidden) { ... }       // 隐藏滚动条仍可滚
```

程序化控制（P4 单向语义，观察与控制分离）：

```auto
let scroll = scroll-controller()
scroll-pane(controller: scroll) { col { ... } }
button("Bottom") { on-click: scroll.to-end() }   // to-start / to-end / scroll-to(offset) / scroll-by(delta)
```

观察（不是控制协议）：

```auto
scroll-pane(on-scroll: |state| { progress = state.progress }) { ... }
```

### 1.2 架构目标

- 滚动状态/输入/viewport/滚动条视觉/内容宿主解耦（设计文档 §2.1）；
- `ScrollPane` 不理解业务内容类型——逻辑 extent（terminal 式自管理内容）通过统一
  hosting API 进入，ScrollPane 只见 `content_extent / viewport_extent / offset / intent`
  （§13 内容透明性验收）；
- 状态单源：普通内容 offset 归 ScrollPane 内部，自管理内容 offset 归宿主（§22.E）；
- 双后端公共语义一致，AutoLang API 不泄露 iced/DOM 细节（§14/§16.4）。

### 1.3 成功样貌

- 新示例/新文档统一使用 `scroll-pane`；旧 `scrollable` 代码无感兼容；
- `direction` 被忽略、通用 tag 无 on-scroll 通路、横向滚动名存实亡三个现状缺口全部关闭；
- 后续 virtual-list / terminal / code-editor 迁移（Phase D/E/F）可在本计划交付的
  `ui/scroll/` 协议 + hosting API 之上直接开工，无需再动 scroll-pane 本体。

## 2. 架构方案

（依赖方向遵循设计文档 §14.1：语义层 → backend adapter，禁止 iced 类型进入协议。）

```text
AutoLang:  scroll-pane(axis/scrollbar/controller/on-scroll) ── alias: scrollable, scroll
                              │ aura_view_builder（tracked/untracked 双臂）
                              ▼
IR:        View::Scrollable { child, axis, scrollbar_policy, controller, offset(写臂), on_scroll(读臂) }
                              │ renderer.rs build_scrollable        │ ui_gen/vue.rs
                              ▼                                     ▼
iced 0.14: iced Scrollable(Direction::{Vertical,Horizontal,Both})   Vue: div.overflow-*（plain）
           scrollbar_style 三策略                                    ScrollArea（shadcn，仅 y/x）
           pending 写臂队列 + Viewport 读臂                          @scroll → ScrollState args
           logical-extent hosting API（generalize terminal 模式）     spacer 占位（为 Phase D 预留）
                              │
                              ▼
核心语义:  ui/scroll/ — ScrollState / ScrollIntent / ScrollSource / ScrollAnchor(预留)
           ScrollbarGeometry 纯函数 / Axis / ScrollbarPolicy / ScrollControllerId
```

关键裁决（执行期可依证据微调，语义变更须升 plan_revision）：

| # | 裁决 | 理由 |
|---|---|---|
| D1 | schema canonical tag 翻转为 `scroll-pane`，aliases = `["scrollable","scroll","Scroll"]` | 设计文档 §19/§20：新文档新 API 统一 scroll-pane；alias 机制保旧代码；下游 ~10 处字符串臂已枚举（T-03），机械可验 |
| D2 | IR 不新增 View 变体，扩展现有 `View::Scrollable` 字段（axis/policy/controller） | 最小搅动；`View::Scrollable` 即 ScrollPane 的 IR 载体，变体重命名属内部美化非本计划必要面 |
| D3 | VNode snapshot keyword 维持 `"scrollable"` 不变 | `015-notes/tests/screenshots/vm_snapshot.txt` 等既有快照稳定性优先 |
| D4 | `direction` prop 保留并映射到 axis（vertical→y, horizontal→x, both→both），不删 | 兼容承诺（AC-02）；schema 文档标注推荐 axis |
| D5 | controller v1 = id 形式薄封装：typed 值持稳定 widget id，方法提交 ScrollIntent → 复用 Plan 043 pending 写臂队列（build 期落盘） | 设计文档 Q3 倾向；避免新运行时 action 通道；时延为一次 build，示例可证 |
| D6 | iced `scrollbar: always` ≈ auto（iced 0.14 无 idle auto-hide，滚动可能时常态显示）| 平台限制如实记录进 spec/文档；hidden 用透明 scrollbar_style 实现 |
| D7 | axis: both 在 Vue shadcn 模式降级 plain `div.overflow-auto`（ScrollArea 仅 v/h） | 双端语义优先于组件形态；视觉策略归 Theme（Q5） |
| D8 | on-scroll 参数为 record：`offset_x/offset_y/viewport_w/viewport_h/content_w/content_h/progress`（活动轴 progress，f64） | 双端同形契约（AC-06）；六测量沿用 Plan 043 Viewport 读臂，autodown 旧三元组通路不动 |

## 3. 技术栈

- Rust：`crates/auto-lang`（`ui/scroll/` 新模块、`ui/view.rs`、`ui/iced/renderer.rs`、
  `ui_gen/vue.rs`、`aura_view_builder.rs`、`aura/schema.rs`）；iced 0.14
  （`iced_widget::scrollable::Direction::Both` 已核实存在于 0.14.2 源码）。
- Schema：`schema/aura.at` element `scroll`（canonical 翻转 + 新 props）。
- 前端：Vue codegen（plain + shadcn ScrollArea 双模式）；playwright（双端验证）。
- 验证：autoui-verifier 技能脚本（`test_vm_mcp.py` / `test_vue_playwright.mjs`）、
  VM MCP `__mcp_scroll` action（`ui/mcp_server.rs:1471-1506` 已有）。

## 4. 需求分析与背景调查

**授权记录**：用户于 2026-09-19 明确指示——依据 universal-scroll-architecture.md 设计
scroll-pane 实施计划，替代现有 scrollable（仅支持有实际高度的内部容器）。本计划为
`/auto-plan:new` 起草，范围 = 设计文档 Phase A+B+C（primitive 本体）；Phase D/E/F/G 另立。
仓库范围：auto-lang 主仓单仓；动作范围：crates/ + schema/ + examples/ + docs/。

**Spec 基线**：`docs/specs/widgets/project.md`（registry/cli/styles 三模块，无滚动条目——
本计划新增 scroll-pane 模块卡即 SD-01）；`docs/specs/auto-lang/project.md` ui 行
（渲染 + ui_gen 职责面）；GOAL-007（AutoUI 跨端一致）为本计划 touched goal。滚动方向
无既有 spec 章节，`docs/specs/widgets/terminal-iced-draw.md` 是 terminal 消费端契约的
先例参照。

**代码现状**（2026-09-19 实测锚点）：

| 事实 | 锚点 |
|---|---|
| `scrollable` 是 canonical `scroll` 的 schema alias，props 仅 class/direction | `schema/aura.at:1061-1075`；`aura/schema.rs:487-496`（Rust fallback 表）、`resolve_tag` `schema.rs:330-360` |
| `direction` prop iced 端忽略（partial） | `ui/render_support.rs:160-163` |
| Plan 043 双臂：写臂 pending 队列（稳定 id + 去抖 0.5px）+ 读臂 Viewport 六测量 → `ScrollMetrics` | `ui/iced/renderer.rs:2362-2400`（写臂）、`:2425-2441`（读臂）、`:2408-2504`（build_scrollable 主体）、update 面排空 `:17302-17312` |
| **通用 tag 路径 offset/on_scroll 恒 None**——双臂只接 autodown 文档路径 | `aura_view_builder.rs:6322-6366`/`2375-2420`（convert_scroll 双胎）、`autodown_scroll_binding` `:3120-3227` |
| 隐式滚动：`overflow-y-auto` 等 style 类 → needs_scroll → View::Scrollable | `aura_view_builder.rs:6290-6314`（untracked）+ tracked 孪生 |
| terminal 已寄宿官方 scrollable（virtual_scroll + 固定网格 extent + 写臂 + 回声抑制）——即 hosting API 要泛化的模式 | `renderer.rs:4345-4413`；`ui/terminal/iced/widget.rs:1059-1063`（自绘条已退役） |
| code_editor 仍自绘滚动条（P626-D4，Phase F 非本计划） | `ui/code_editor/core/render.rs:358-406`；VM 主路径已 hosted `renderer.rs:23300-23314` |
| virtual-list 零实现（设计文档 Phase D） | 全仓 grep 仅设计文档命中 |
| Vue：shadcn→ScrollArea（orientation 仅 v/h）；plain→div.overflow-auto；onscroll 事件名映射已有 | `ui_gen/vue.rs:16496-16502`、`:11878-11909`（attrs）、`:9242/9324`（类）、`:8864`（tag 兜底）、`:15602`（事件）、`ui_gen/widget/registry.rs:227-238,315-343`、`ui_gen/shared/registry.rs:201-218` |
| auto_scroll sentinel-ref + watch/nextTick 模式（Vue 程序化滚动先例） | `vue.rs:2639/3499-3505`、`:7584-7587/7643-7647` |
| 其余 canonical-tag 字符串臂（T-03 清单） | `ui_gen/rust.rs:3555`、`a2ui/export.rs:256`、`ui/gpui/renderer.rs:309`、`ui/desktop_protocol/client_runtime.rs:648/950`、`native_projector.rs:974`、`coverage.rs:208/460`、`snapshot_builder.rs:482-497`、`mcp_server.rs:1471-1506` |
| VNode 快照层 keyword `"scrollable"`、props 仅 offset_y | `ui/vnode.rs:67/164/248`、`vnode_converter.rs:394/553/593` |
| capability-test 先例（Scissor 裁剪/命中验证） | `examples/capability-tests/p515-scroll-overflow/src/front/app.at` |
| gallery scroll demo（vue 生成产物） | `examples/widgets-gallery/gen/front/vue/src/pages/scroll.vue` + demos-registry |

## 5. 详细设计

### 5.1 核心语义模块 `crates/auto-lang/src/ui/scroll/`

纯 Rust、零 backend 依赖（Phase A，设计文档 §8/§9/§12.3）：

```rust
pub enum Axis { X, Y }
pub enum ScrollbarPolicy { Auto, Always, Hidden }

pub struct ScrollAxisState { pub offset: f64, pub viewport_extent: f64, pub content_extent: f64 }
pub struct ScrollState { pub x: Option<ScrollAxisState>, pub y: Option<ScrollAxisState> }
// 不变量：scroll_range = max(content_extent - viewport_extent, 0)；offset ∈ [0, scroll_range]

pub enum ScrollSource { Wheel, Touchpad, Touch, ThumbDrag, RailClick, Keyboard,
                        Programmatic, Sync, FocusReveal, AutoScroll }
pub enum ScrollIntent { ScrollBy{axis,delta,source}, ScrollTo{axis,offset,source},
                        PageBy{axis,pages,source}, ToStart{axis}, ToEnd{axis} }

pub enum ScrollAnchor { AbsoluteOffset, KeepEnd, Item{key: ItemKey, viewport_offset: f64} }  // 预留，无运行时消费

pub struct ScrollControllerId(pub Arc<str>);   // 稳定 widget id 句柄（D5）

pub mod geometry {  // 设计文档 §12.3 逐式实现 + 反向映射
    pub fn scroll_range(s: &ScrollAxisState) -> f64;
    pub fn clamp_offset(s: &ScrollAxisState, offset: f64) -> f64;
    pub fn progress(s: &ScrollAxisState) -> f64;           // offset / scroll_range（0 当 scroll_range=0）
    pub struct Thumb { pub pos: f64, pub extent: f64 }     // rail 坐标系
    pub fn thumb_from_state(s: &ScrollAxisState, rail_extent: f64, min_thumb: f64) -> Thumb;
    pub fn offset_from_thumb_pos(s: &ScrollAxisState, rail: &Thumb, rail_extent: f64, min_thumb: f64) -> f64;
}
```

`ScrollIntent::resolve(&ScrollState) -> ScrollTo` 归一化（By/Page/ToStart/ToEnd 折算成
绝对 offset + clamp），controller 与后续 Phase D/E/F 共用。

### 5.2 语言面（schema）

`schema/aura.at` element 翻转（D1）：

```text
element scroll-pane {
    tag: "scroll-pane"
    aliases: ["scrollable", "scroll", "Scroll"]
    props: [
        class,
        axis: one_of:y,x,both  (default y),
        scrollbar: one_of:auto,always,hidden  (default auto),
        controller: union:scroll_controller,
        on-scroll: union:event_callback,
        direction: one_of:vertical,horizontal,both  (default vertical, legacy 兼容→axis 映射)
    ]
}
```

Rust fallback `ElementDef`（`aura/schema.rs:487-496`）同步：tag 保持 `"scroll"` 内键或改
`"scroll-pane"` 以 alias 表对齐——以 resolve_tag 三级匹配实测为准（T-02 内验证）。
`render_support.rs` 从 partial 升 supported（axis 消费后）。

### 5.3 IR 与 builder（D2/D3/D4）

`View::Scrollable` 增字段：`axis: Axis`、`scrollbar: ScrollbarPolicy`、
`controller: Option<ScrollControllerId>`（`offset/on_scroll/auto_scroll` 原样保留）。
`aura_view_builder.rs` `convert_scroll` 双胎（tracked `:2375-`/untracked `:6322-`，D-GAP
纪律同步两臂）解析新 props：`direction`→axis 映射、`on-scroll` lambda → `ScrollCallback`、
`scroll-controller()` 值 → controller id 绑定（实例化点挂稳定 widget id：
`scroll_pane_{key}`）。VNode props 增 `axis`，keyword 不变（D3）。

### 5.4 iced 后端

`build_scrollable`（`renderer.rs:2408-2504`）扩展：

- axis → `iced::widget::scrollable::Direction::{Vertical,Horizontal,Both{..}}`（0.14 实测支持）；
- scrollbar 策略：auto/always → 现行 `scrollbar_style`（`renderer.rs:2507-2531`）；
  hidden → 新增透明变体（thumb/track 全透明、保留命中与滚动能力）；
- 写臂扩展：pending 队列（`:2362-2400`）接受 controller 提交的 ScrollIntent——按
  controller id 查最近 Viewport 测量缓存（读臂随 on_scroll 或独立缓存登记），经
  `ScrollIntent::resolve` 折算绝对 offset 入队；update 面排空（`:17302-17312`）复用；
- 读臂：`on_scroll` 回调参数从六测量构造 D8 record（offset_x/y、viewport_w/h、
  content_w/h、活动轴 progress）；autodown 旧三元组通路零改动；
- **logical-extent hosting API**（§13）：把 terminal 寄宿模式（`renderer.rs:4345-4413`：
  固定 extent + 写臂 + 回声抑制）泛化为 `ui/scroll/` 内公共函数
  `host_logical_extent(id, extent, on_offset) -> iced Scrollable 配置`，terminal 调用点
  改走该函数（行为等价，terminal 测试守卫）；虚拟列表 Phase D 直接消费。
- `render_dynamic_view`（`:23191-23211`）与泛型 `into_iced`（`:4569-4582`）双渲染臂同步。

### 5.5 Vue 后端（`ui_gen/vue.rs`）

- plain 模式：axis → `overflow-y-auto` / `overflow-x-auto` / `overflow-auto`；
  always → `overflow-scroll`；hidden → overflow-auto + 滚动条隐藏样式
  （`scrollbar-width:none` + `::-webkit-scrollbar{display:none}`）；
- shadcn 模式：y/x → ScrollArea + orientation；both → 降级 plain div（D7）；
  hidden 在 ScrollArea 上叠加同样隐藏样式；
- on-scroll → `@scroll` 处理器读 `scrollTop/scrollLeft/clientHeight/clientWidth/
  scrollHeight/scrollWidth` 构造 **与 iced 同名同形** 的 D8 record（AC-06 契约）；
- controller：sentinel-ref 模式（auto_scroll 先例 `vue.rs:2639`）——controller id → 模板
  ref + 暴露 `to_start/to_end/scroll_to/scroll_by` 方法对象，事件内联调用映射到
  `el.scrollTo({top/left, behavior:'auto'})` 家族；
- spacer 占位技术（§16.3）仅作为 hosting 语义写入 spec 供 Phase D，不在本计划实现面。

### 5.6 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/widgets/scroll-pane.md` | 无 → scroll-pane 模块卡：核心语义（State/Intent/Source/Anchor 预留/geometry）、AutoLang API（axis/scrollbar/controller/on-scroll）、双后端契约（iced 映射表 + D6 限制、Vue 映射表 + D7 降级）、hosting API、兼容承诺（aliases/direction/隐式 overflow）、Phase D-G 边界 | 设计文档 §22 验收标准落成 spec；后续消费端计划的权威依据 | AC-01..AC-08 |
| SD-02 | modify | `docs/specs/widgets/project.md` | 模块清单 3 行 → 增 scroll-pane 行（含 terminal-iced-draw 同款跨目录注记） | 模块清单完整性 | AC-08 |
| SD-03 | modify | `docs/specs/auto-lang/project.md` | ui 行状态注记补 scroll-pane 双端同语义（Plan 656） | ui 模块状态反映新能力 | AC-08 |

INDEX/ledger 派生刷新按 `/auto-plan:merge` 常规流程，不入本表。

## 6. 测试设计

| 层 | 内容 | 命令（预期） |
|---|---|---|
| 核心语义单测 | clamp/range/progress 边界（content<viewport、=、≫）；thumb 正反映射往返恒等；min_thumb clamp 后 travel 映射正确性（设计文档 §12.3 陷阱）；Intent 归一化 | `cargo t scroll`（新模块 #[cfg(test)]，全绿） |
| schema/解析 | scroll-pane 新 props 解析、alias 三拼写等价、direction→axis 映射、未知 prop 报错 | `cargo t schema` / 相关既有套件绿 |
| builder | convert_scroll 双胎产出 View 字段（axis/policy/controller/on_scroll）；隐式 overflow 路径不回归 | `cargo t aura`（作用域滤串） |
| iced 渲染 | build_scrollable axis/policy/style 断言；controller 写臂入队→排空→scroll_to；on_scroll record 构造（扩 `renderer.rs:25815+` 既有套件）；terminal 改走 hosting API 行为等价 | `cargo t iced` |
| Vue codegen | plain/shadcn 双模式 golden：类/attrs/@scroll 参数提取/controller ref 发射/both 降级 | `cargo t vue` |
| docs | schema 变更后 core.md 再生一致 | `cargo test -p auto-lang --test docs_gen` |
| 双端实机 | p656 示例：MCP `__mcp_scroll` 纵横滚动 + 断言 on-scroll state 值；playwright 同场景；六测量/progress 双端同形同值（容差） | autoui-verifier `test_vm_mcp.py` / `test_vue_playwright.mjs` 全绿 + 截图 |
| 门禁 | 零警告/fmt/tf | `cargo check -p auto-lang` 0 warning；`cargo tf` 一次全绿（review 前） |

Category B/C 归类：本计划动 `crates/` Rust + schema → B（scoped + 折叠前 `tf`）；
动 `schema/aura.at` → C（docs_gen 必跑）。不触 aavm/trans/book → 不跑 taa/tt/tb。

## 7. 验收标准

| ID | 标准 | 验证方法 |
|---|---|---|
| AC-01 | `ui/scroll/` 纯语义模块：geometry/state/intent 单测全绿，含 §12.3 min-thumb 往返与 clamp 边界 | `cargo t scroll` 输出全绿 |
| AC-02 | 语言面 + 兼容：`scroll-pane`/`scrollable`/`scroll`/`Scroll` 四拼写解析等价；`direction` 旧 prop 映射 axis；既有隐式 overflow-y-auto 路径、autodown 双臂绑定、terminal virtual_scroll 寄宿、015-notes 等既有快照/测试零回归 | 解析等价单测 + `cargo t iced`/`cargo t vue` + 既有快照 diff 为空（VNode keyword 不变） |
| AC-03 | axis 三值双端真生效：x 横向滚动在 iced 端实际可滚（现状 direction 被忽略→关闭）；both 双轴独立 | p656 示例 VM MCP 横向 scroll 断言 offset_x 变化 + playwright 同场景 |
| AC-04 | scrollbar 策略双端生效：hidden 无可见滚动条但仍可程序化/输入滚动；always/auto 行为差异如实记录（D6） | 截图对比 + controller 在 hidden 态下 to-end 生效断言 |
| AC-05 | controller 四方法（to-start/to-end/scroll-to/scroll-by）双端生效，事件内调用一次 build 内落盘 | p656 按钮驱动 + 双端断言终态 offset |
| AC-06 | on-scroll 观察面双端同形：D8 record 字段集一致，同内容同操作下六测量与 progress 双端一致（px 容差 ±2） | 双端脚本各 dump 一次滚动态，逐字段比对 |
| AC-07 | `examples/capability-tests/p656-scroll-pane/` 双端验证绿，截图与脚本归档 example tests 目录 | autoui-verifier 两脚本 exit 0 |
| AC-08 | 文档与沉淀：core.md 再生含 scroll-pane；widgets-gallery scroll demo 改用 scroll-pane 展示 axis/scrollbar 变体；SD-01..03 spec 增量落盘 | `git diff` 审阅 + docs_gen 绿 |
| AC-09 | 健康门禁：`cargo check -p auto-lang` 零警告、fmt 干净、无遗留 dbg/print；review 前一次 `cargo tf` 全绿 | 命令输出实证记录于复审 |

## 8. 执行步骤

（worktree：`D:/autostack/.wt/lang-656/auto-lang`，branch `plan-656-dev`；计划簿记留 master。）

- **T-01 核心语义模块（Phase A）** 〔新增 `crates/auto-lang/src/ui/scroll/{mod.rs,geometry.rs}`
  + `lib.rs`/`ui/mod.rs` 挂载；纯函数无 backend dep〕
  验证：`cargo check -p auto-lang` 绿 + `cargo t scroll` 全绿 → AC-01
- **T-02 schema 翻转 + 语言面** 〔`schema/aura.at` element 翻转（D1 新 props + aliases）、
  `aura/schema.rs` fallback ElementDef、`render_support.rs` 升 supported；canonical 翻转若
  实测超出场枚举的字符串臂范围（爆炸），回退"scroll canonical + scroll-pane alias"并记
  录（裁决变更不动语义，不升 revision）〕
  验证：`cargo test -p auto-lang --test docs_gen` + schema/resolve 相关套件绿 → AC-02 部分
- **T-03 canonical-tag 字符串臂清扫** 〔§4 表"其余字符串臂"清单逐点改双拼写匹配或映射：
  `ui_gen/rust.rs:3555`、`ui_gen/shared/registry.rs:201-218`、`a2ui/export.rs:256`、
  `ui/gpui/renderer.rs:309`、`ui/desktop_protocol/{client_runtime.rs:648,native_projector.rs:974}`、
  `coverage.rs:208/460`、`snapshot_builder.rs:482-497`、`mcp_server.rs:1471-1506`〕
  验证：`cargo t vue` + `cargo t iced` 作用域绿；gallery gen 产物 diff 仅命名面 → AC-02
- **T-04 IR + builder 双臂** 〔`ui/view.rs` View::Scrollable 增字段、`vnode.rs` props 增 axis
  （keyword 不变）、`aura_view_builder.rs` convert_scroll 双胎 + `scroll-controller()` 值
  构造与 id 绑定、`vnode_converter.rs` 同步〕
  验证：builder/vnode 作用域单测绿 → AC-02/AC-03 前置
- **T-05 iced 后端** 〔`renderer.rs` build_scrollable 扩展（axis/policy/hidden style/双渲染臂）、
  controller 写臂扩展 + Viewport 测量缓存、on-scroll D8 record、hosting API 泛化 + terminal
  调用点改写〕
  验证：`cargo t iced` + terminal 作用域测试绿 → AC-03/04/05/06 iced 侧
- **T-06 Vue 后端** 〔`ui_gen/vue.rs` plain/shadcn 双模式（axis/policy/on-scroll/controller/
  both 降级）+ widget registry 两份同步〕
  验证：`cargo t vue` golden 绿 → AC-03/04/05/06 vue 侧
- **T-07 capability 示例 + 双端实机验证** 〔新增 `examples/capability-tests/p656-scroll-pane/
  src/front/app.at`（纵/横/双轴 + 三策略 + controller 按钮组 + on-scroll 读出）；
  autoui-verifier 脚本 + 截图归档〕
  验证：`test_vm_mcp.py` + `test_vue_playwright.mjs` exit 0 → AC-03..07
- **T-08 文档/gallery/spec 沉淀** 〔`auto docs gen` 再生 core.md；widgets-gallery scroll
  demo 改 scroll-pane；SD-01..03 spec 文件写入〕
  验证：docs_gen 绿 + diff 审阅 → AC-08
- **T-09 健康门禁 + review 交接** 〔零警告/fmt/tf 一次；勾选本清单证据；交 `/auto-plan:review`〕
  验证：命令输出记录 → AC-09

依赖：T-01→T-04→T-05/T-06→T-07；T-02→T-03→(T-05,T-06)；T-08/T-09 收尾。

## 9. 复审记录

- 2026-09-19 /auto-plan:new 起草交接：`stage: new`，plan_revision 1，
  `outcome: pass`（范围=设计文档 Phase A+B+C，授权=用户 2026-09-19 指示），
  `next: work`（用户确认计划后开 worktree 执行）。待决事项见 §10（均不阻塞开工，
  已按倾向落为裁决 D1-D8）。

## 10. 待澄清事项

1. **controller 生效时延**（D5）：v1 走 build 期写臂队列，事件→滚动落盘间隔一次 build；
   若实机验证感知卡顿（AC-05 截图/录屏发现），升级为运行时直发 action——属实现路径
   等价替换，不升 revision，但需在复审记录留证。
2. **`scrollbar: always` iced 限制**（D6）：iced 0.14 无 idle auto-hide，always 与 auto
   视觉等价（滚动可能时常态显示）。是否值得自绘 idle 淡出（设计文档 §12.2 overlay 视觉）
   → 留 Phase G（视觉精化）裁定。
3. **设计文档开放问题 Q2/Q3/Q7**（自定义 ScrollContent 语法 / controller id 形态 / Anchor
   运行时）：本计划按 Q3 倾向落地 id 薄封装；Q2/Q7 明确延后至 Phase D+ 消费端计划。
4. **canonical 翻转回退线**（T-02）：若字符串臂实际爆炸超出枚举清单，回退 alias-only
   方案；此时 docs 面向用户仍主推 scroll-pane 写法，canonical 翻转并入 Phase D 计划。
5. Phase D（virtual-list）/ E（terminal 迁移至 ScrollContent）/ F（code_editor 清偿
   P626-D4）各需独立计划；本计划交付的 `ui/scroll/` 协议与 hosting API 是其直接前置。
