# [PLAN-661] canvas 场景契约扩容（图元+命中）+ AutoUI Slider 三轨补全 + VM map 括号写修复

---
plan_id: PLAN-661
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: canvas-graph-scene-slider
author: [zcode]
created_at: 2026-09-19
updated_at: 2026-09-19
plan_revision: 1
current_step: 0
total_steps: 6

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [auto-lang/ui/design/canvas-scene.md]
touched_goals: [GOAL-007, GOAL-008]   # 引用 docs/specs/goals.md 的 GOAL-NNN
affects: [auto-lang/ui, auto-lang/vm] # 受影响的 specs 路径
---

## 0. 变更摘要

把 AutoUI 既有的两块半成品能力补齐到可消费状态，并顺修一个阻塞下游的
VM 写通道缺陷（三项同源于 auto-down PLAN-078 终裁，2026-09-19 用户裁定
Q-1 路径 C'）：

1. **VM map 括号写修复**：`m[k]=v` 写臂补 map 路由（codegen 发
   `auto.hashmap.set`，与读臂对称），消除「RuntimeError("Invalid array
   ID") 静默吞 handler」缺陷。graph store 以 map 存 settings/坐标的
   jade 消费流的先决项。
2. **AutoUI Slider 三轨补全（不新造组件）**：`View::Slider.on_change`
   的 `fn(f32)->M` 指针 newtype 化（`SliderChangeHandler`，镜像
   ScrollCallback 先例，顺除 map_msg panic 占位存量炸弹）→ 补 aura
   builder 臂（schema ElementDef + convert_slider + 快照 actions）→
   MCP `set_value` 动作闭环 → vue 轨统一 `<input type="range">` 响应式
   发射。a2r 轨语义基准（025 fixture）保持不变。
3. **canvas 场景契约扩容（笔笔画 → 图元场景）**：`CanvasScene` 在
   strokes 之外增 nodes（圆/矩形）/edges（线段）/labels（文本）三表，
   双端各自独立绘制（纯函数既有裁定不动）；新增 `onhit` 命中契约
   （tap → 元素 id 载荷上报，MCP `press` 可达）。

交付物 = 编译器/AutoUI 侧能力 + capability-tests 双轨样板证明契约端到端。
jade-garden graph_view 真渲染消费 = jade/auto-down 侧后续计划（§10 留
指针），不在本计划范围。

## 1. 目标

### 目标（Goals）

- G-1 VM 写通道对称：map 括号写（`m[k]=v`，字面量键与 var 键同形）与
  读臂行为对称——写后读回正确、写后语句继续执行；vue 轨零回归。
- G-2 Slider 三轨对齐：同一 DSL `slider` 词位（025 fixture 语义基准：
  value 绑 float 字段、onchange 载荷 msg、step）在 a2r/VM-aura/vue 三轨
  行为等价——快照可见、动作可驱动、状态可回写断言。
- G-3 canvas 图元场景：nodes/edges/labels 平行串表族驱动，双端渲染保持
  「场景数据契约的纯函数」（view.rs:1007-1013 既有裁定）；命中测试
  tap→元素 id 上报，`autoui_action` press 可达。
- G-4 既有面零回归：043-canvas-paint（pen 三件套/strokes）、slider a2r
  golden 语义、SET_ELEM 数组路径、input/checkbox 全部不动或等价迁移。

### 非目标（Non-Goals）

- 交互式缩放/平移（viewport 字段与事件回路）——cytoscape 对位首批子集
  = 节点/边/标签/tap（078 终裁原文），缩放平移后置（§10 登记）。
- fcose 力导布局或任何 VM 侧自算布局——布局是场景数据，由上游（jade
  graph store）下发坐标（R-3 裁定）。
- edges/labels 的命中上报——v1 命中域 = nodes only（tap 打开页语义）。
- jade-garden graph_view 真渲染装配、component-gallery 第三单元转正、
  desktop 图谱页——消费侧行动，归 jade/auto-down 侧后续计划。
- eraser/挖除语义升级、canvas 子元素快照树（v1 canvas 单节点 +
  actions 自描述）。
- shadcn-vue Slider 组件族的视觉升级（Plan 408 组件路径退役让位原生
  range，见 R-5）。

## 2. 架构方案

### 2.1 轨道地图（现状 → 目标）

| 轨道 | map 括号写 | slider | canvas 图元 |
|---|---|---|---|
| **VM/aura**（aura_view_builder → View\<DynamicMessage\> → iced renderer + snapshot + MCP） | 写臂缺路由 → **修**（T-01） | schema 无条目 + builder 无臂 + 快照 actions 空 + MCP set_value 空转 → **补**（T-02/03） | 契约仅 strokes → **扩**（T-05） |
| **vue**（ui_gen/vue.rs SFC 发射） | 已正确（探针④ SFC 实文） | 裸 `<input>` 无 type="range" + 组件路径值不同步 → **统一原生 range**（T-04） | generate_canvas_html 双表渲染 → 扩三表（T-05） |
| **a2r**（ui_gen/rust.rs 原生 Rust 发射） | 不适用 | 已完整（PLAN-025，语义基准）→ 仅 handler 形态适配 + golden 刷新（T-02） | 不适用（a2r 轨无 canvas 面） |
| **VM 编译器/引擎**（codegen/engine/native） | 读臂有路由、写臂无、native 已注册未消费 → T-01 | 不适用 | 不适用 |

关键事实（2026-09-19 实勘，master 857235623；078 基线 b69c7344c 后行号
漂移已逐项复核）：

- iced 渲染层 **Slider 臂已存在**（renderer.rs:5050 `iced::widget::slider`
  + step + inspect-capture 静态读出）——VM 轨缺口只在视图树构建侧与
  快照/动作面，不在渲染层。
- `View::Slider.on_change: fn(f32)->M`（view.rs:796）是 a2r 专用形态；
  view.rs:327-330 明文记录 fn 指针**不能**跨 map_msg /
  DynamicMessage→IcedMessage 转换，且 map_msg Slider 臂现状即
  `|_| panic!("map_msg not supported for Slider views")` 占位
  （view.rs:2330-2340）——newtype 化修的是存量缺陷，非预防性改造。
- 载荷回调 newtype 先例三件在案：`PointerMoveHandler`（Plan 499）、
  `ScrollCallback`（Plan 043 T1）、`ColResizeCallback`——slider 照抄
  形态即可（R-4）。
- 507 Stage 5 落地的 `aura/element_coverage.rs` 元素登记表 +
  schema_drift 双向围栏：**新增 schema ElementDef 必须同步登记
  coverage**，否则日常门禁红（T-03 内置动作项）。

### 2.2 起草裁定（078 四项开放问题，本节裁决）

- **R-1 命中事件契约**：canvas 新增独立 `onhit: MsgRef` prop，handler
  收**命中元素 id 字符串载荷**（`.OnNodeTap(id string)` 范式，与
  `.SetVol(v float)` 载荷绑定同通道）。不复用 pen press 上报（坐标通道
  与标识通道语义不同）；不新造 onnodeclick 词位（onhit 面向全部图元，
  nodes-only 命中域是 v1 实现取舍而非契约边界）。MCP：canvas UiNode
  actions 挂 `press`（value=元素 id），引擎侧 id→图元中心坐标→合成
  pen start/end + onhit 派发——`autoui_action` press 可达（需求原文）。
- **R-2 缩放/平移**：v1 不进契约（无 viewport 字段、无交互回路）。场景
  表绝对坐标；上游重算坐标即重绘（状态驱动天然支持）。schema 描述预留
  扩展注记，债登记 §10。
- **R-3 布局归属**：场景坐标由上游下发（`<前缀>_nodes` 表自带 x/y）。
  「渲染=场景数据契约的纯函数」裁定决定布局是数据不是渲染；环形/网格
  布局在 .at 侧循环+数学生成（jade 消费侧职责）。map 写修复（T-01）
  正是 graph store 以 map 存 settings/坐标的先决。
- **R-4 slider aura 事件模型**：`SliderChangeHandler<M>` newtype
  （`Arc<dyn Fn(f32) -> M + Send + Sync>` + Clone/Debug），镜像
  ScrollCallback。a2r 发射面同步适配（fixture .at 源不变——语义基准
  保持；golden 输出形态刷新）。map_msg panic 占位消除。
- **R-5 vue slider 双路径统一**：DSL `slider` 统一发射原生
  `<input type="range">`（value 响应式绑定 + @input 载荷派发 msg），
  shadcn-vue Slider 组件路径（Plan 408 `:default-value` 数组、值不同步
  旧限）退役让位。理由：与 a2r/VM 语义对齐最短路径、零组件依赖；
  map_tag 既有断言（`map_tag("slider", true)=="Slider"`）随裁定更新。

### 2.3 范围切分裁定

本计划 = 单期承载编译器/AutoUI 侧全部（map 写 + slider 三轨 + canvas
图元），不拆两期。理由：三块共享同一验证基建（capability 样板 + 双轨
对拍 + MCP 驱动），拆期重复建设；总量中等（6 任务）。jade 消费侧
（graph_view 真渲染/gallery 单元/desktop 装配）另立计划——078 判定档
§5 Q-2 已把宿主面单元记 DEBT 随独立计划跟踪，本计划交付其全部上游
依赖并在 §10 留交接指针。

## 3. 技术栈

- Rust workspace（crates/auto-lang）：ui/view.rs（View 契约）、
  ui/aura_view_builder.rs、ui/snapshot_builder.rs、ui/iced/renderer.rs、
  ui/mcp_server.rs、ui_gen/vue.rs、ui_gen/rust.rs、aura/schema.rs、
  aura/element_coverage.rs、vm/codegen.rs、vm/engine.rs、vm/native.rs。
- 验证：`cargo check -p auto-lang`、`cargo t iced|ui|vm_codegen`（按步
  作用域）、`cargo tv`（VM 语料 golden）、`cargo test -p auto-lang
  --test docs_gen`（schema 描述改动）、autoui-verifier 技能脚本
  （test_vm_mcp.py / test_vue_playwright.mjs）。
- 样板载体：examples/capability-tests/（新样板 0xx-canvas-graph）。

## 4. 需求分析与背景调查

### 4.1 授权与来源（已给的许可）

- 来源裁定：auto-down PLAN-078 T-00 终裁（2026-09-19 用户裁定）——
  Q-1 路径 C'（canvas 契约已存在，扩容图元+命中，独立立项归 auto-lang
  侧 auto-plan:new）；Q-3（map 括号写=VM 实现问题，复现包 handoff
  auto-lang 修复）；Q-4（slider=补全不新造，随独立计划）。终裁全文：
  auto-down plan-078-dev 分支 `docs/plans/attachments/
  078-graph-view-ruling.md` §5/§6（078 merge 后随该仓 master 可读）。
- 本会话授权边界：**立项起草**（用户明示「起草完成交 handoff 即可，
  执行另行授权」）。执行/建 worktree 待用户后续授权 `/auto-plan:work`。
- 预算/自动续跑限制：无声明。

### 4.2 证据基础（2026-09-19 本仓 master 857235623 实勘复核）

078 T-00 调查基线 b69c7344c（078 期间零新增补丁声明），其后 660 系列
合入致行号漂移——下表全部按当前 master 复核（✓=逐行目验）：

| # | 事实 | 证据（当前行号） |
|---|---|---|
| E-1 | canvas 纯函数裁定+View::Canvas 契约 | view.rs:1007-1025（scene 唯一事实源/双端独立绘制/共享 schema 契约；pen 三件套 PointerMoveHandler） |
| E-2 | CanvasScene 仅 strokes | view.rs:1031-1039（CanvasStroke points/color/width/eraser）、view.rs:1044-1046 |
| E-3 | scene 双表读取 | aura_view_builder.rs:11797/11831/11854-11857（`<前缀>_pts`/`<前缀>_meta` read_state_as_vec） |
| E-4 | canvas schema 条目 | aura/schema.rs:2405-2418（ElementDef：scene/coords/clear/onpen 三件套） |
| E-5 | 双端绘制臂 | renderer.rs:4809-4814+7627（CanvasPainter canvas::Program）；vue.rs:6645+（generate_canvas_html，Plan 563 臂） |
| E-6 | slider a2r 完整 | rust.rs:3701-3742（PLAN-025 T-03，View::slider 发射）；tests/fixtures/025-native-input/slider.at（SetVol(float)/.vol float/step 1.0）✓ |
| E-7 | View::Slider fn 指针缺陷 | view.rs:792-799（on_change: fn(f32)->M）、1806（构造器）、2330-2340（map_msg **panic 占位**）、327-330（ScrollCallback 注记点名 Slider fn 指针不可跨消息映射） |
| E-8 | aura slider 全缺 | aura_view_builder.rs 零 "slider" 臂（input 2075/4013、checkbox 2181/4095 在案可参照）；aura/schema.rs 无 slider ElementDef（仅 :33 分类注释提及） |
| E-9 | 快照/动作空转 | snapshot_builder.rs:348-358（Slider kind 有 min/max/value/step 提取，actions=vec![]——「fn not directly extractable」）；mcp_server.rs:696（set_value 已声明）、3095（`target.kind != "Slider"` 门）、3124+（handler 从 target.actions 按名查找——空 actions → "No handler found"） |
| E-10 | iced 渲染臂已在 | renderer.rs:5050-5071（iced::widget::slider + step + inspect-capture 静态读出） |
| E-11 | vue 双路径 | vue.rs:9061（map_tag slider→裸 input，无 type="range"）；vue.rs:11741 generate_shadcn_attrs 内 12826-12857（组件路径 :default-value 数组、值不同步，Plan 408） |
| E-12 | 拖拽 machinery 先例 | PLAN-617 progress+onseek（vue.rs:5433-5450 声明门控臂；PointerMoveHandler on_seek 复用范式，view.rs:812-815） |
| E-13 | map 写双臂不对称 | codegen.rs:6942 邻域（Expr::Index 赋值臂无条件发 SET_ELEM）vs codegen.rs:3405-3412（读臂 auto.hashmap.get 路由）；engine.rs:5475（SET_ELEM 仅 ListData downcast）；native.rs:294-295（auto.hashmap.set→NATIVE_HASHMAP_INSERT_STR 已注册未消费） |
| E-14 | 复现包 | auto-down plan-078-dev 分支 docs/plans/attachments/078-vm-map-write-repro/（README 根因+probe/pac.at+drive.mjs 实录：`RuntimeError("Invalid array ID: 4000000")`，写后语句全不执行）✓ 已读 |
| E-15 | coverage 围栏 | aura/element_coverage.rs 元素登记表 + schema_drift 双向同步（507 Stage 5，overview.md:320 记述）——新增 ElementDef 须同步登记 |
| E-16 | spec 台账锚 | docs/specs/auto-lang/ui/plans.md:166（563 归档行：scene 双表契约+IoU 0.9793/RGB 3.0 对拍锁定）；chart-components.md:13-16（563 修订头） |

### 4.3 消费方影响盘点

- canvas 现有消费方：043-canvas-paint（pen 样板，strokes 主用户）、
  charts/svg 组件（借 canvas 底座）。扩容为**加法**（三新表缺省空 →
  现行为逐字节等价），零迁移成本。
- slider 现有消费方：a2r 轨 025 fixture（golden 刷新）；vue 轨既有
  发射形态变化（裸 input→range input）——examples 内 slider 使用处
  随 T-04 视觉回归。
- map 写修复消费方：jade graph store（settings/坐标 map 深写）+ 任何
  `.at` 侧 map 括号写作者。修复前纪律（078 P-11：VM 轨禁 map 括号写）
  在修复后解除。

## 5. 详细设计

### 5.1 T-01 VM map 括号写（方案 i：codegen 路由，与读臂对称）

- `vm/codegen.rs` `Expr::Index` 赋值臂（~6942）：镜像读臂（~3405）的
  map 判定（编译期容器型信息——读臂现行判法实勘后照抄），对 map 目标
  发 `CALL_NAT auto.hashmap.set`（栈序按 NATIVE_HASHMAP_INSERT_STR
  约定：map_id, key, value——执行时以 native.rs 实现为准对齐）。
- engine 侧 `SET_ELEM` 不加臂（方案 ii 弃——读侧已走 native 路由，写侧
  对称即可；engine 臂留作纯数组路径不动）。
- 赋值表达式结果语义（表达式值=所写值）与点号写/全量重赋一致。
- 回归用例：078 复现包 probe（DoIt 后 count=1 + readback="2"，字面量键
  与 var 键两形态）收编为仓内测试（vm 语料或单测，随 T-01 落位）。

### 5.2 T-02 SliderChangeHandler newtype（a2r 适配 + 存量炸弹清除）

- `ui/view.rs`：`pub struct SliderChangeHandler<M>`（`Arc<dyn Fn(f32)
  -> M + Send + Sync>`，Clone/Debug/new/call——逐行照抄 ScrollCallback
  形态）。`View::Slider.on_change` 改 `Option<SliderChangeHandler<M>>`
  （无 onchange = 无动作，a2r 现行「占位零参闭合」发射改为 None——
  语义等价且更净；构造器签名随改）。
- map_msg 臂（view.rs:2330）：panic 占位 → `on_change.map(|h|
  h.map_msg(...))` 组合（ScrollCallback 同款）。
- `ui/iced/renderer.rs:5050` 臂：`on_change` 从 fn 指针调用改
  `Option` 分派（None → iced slider 传 no-op 闭包；inspect-capture
  静态读出分支保留）。
- `ui_gen/rust.rs:3701-3742`：发射 `SliderChangeHandler::new(闭包)`
  形态（fn 指针字面量 → 闭包包装；载荷变体构造逻辑不变）；golden
  刷新（`test_slider_codegen_arm_fixture` rust.rs:8269 等），fixture
  .at 源零改动。
- `ui/snapshot_builder.rs:348`：actions 注册所需信息（handler 对应
  msg 名）不在闭包内——由 T-03 的 aura 臂从 events map 旁路供给
  （快照挂法对齐 input/checkbox 现行通道，见 5.3）。

### 5.3 T-03 aura slider 臂 + 快照 + MCP set_value 闭环

- `aura/schema.rs`：`elements.insert("slider", ElementDef{...})`
  （props：value StateRef、min/max/step Number、onchange MsgRef、
  disabled、class；描述注明 a2r 语义基准 025 fixture）——挂载点在
  input（:550）/checkbox（:566）邻域，分类入 :33 表单族。
- `aura/element_coverage.rs`：同步登记（按 507 Tier 分级归位——若判
  Tier3 not-yet 则 schema_drift 同步绿即达标，投影器臂非本任务项）。
- `ui/aura_view_builder.rs`：`"slider" => self.convert_slider(...)`
  双分发位（1976 canvas 位邻域 + 3709 位邻域，对齐 input 2075/4013
  双臂形态）；`convert_slider`：value 经 read_state 提 f32（float 字段，
  int 容差降级）、min/max/step 数值提取（缺省 0/100/None，对齐 vue 轨
  现行缺省）、onchange 经 `event_to_message_with` 载荷绑定（`.SetVol(v
  float)` 同款）构造 `SliderChangeHandler`、class/style 预设对齐
  checkbox 臂形态。
- `ui/snapshot_builder.rs`：Slider 节点 actions 挂 `set_value`
  （handler 名取 onchange msg 名——aura 臂在构造 View 时以并行旁路
  （handler 名内嵌于 SliderChangeHandler 的 Debug 载荷或 snapshot
  上下文）供给；执行时对齐 input "type"/checkbox "toggle" 的现行挂法
  取最短路径）。
- `ui/mcp_server.rs`：SetValue 动作分发补 f32 载荷构造 → handler
  `call(v)` 出 DynamicMessage → ActionMessage 注入 iced loop（现状
  门 3095 与 handler 查找 3124+ 均已在，缺的是 Slider actions 非空后
  的载荷分支——执行时核对 input_value 构造处补 SetValue 数值分支）。
- 闭环语义（AC-02 验收面）：`set_value(75)` → `.SetVol(75.0)` msg →
  model `vol=75.0` → 快照 Slider value=75 可断言。

### 5.4 T-04 vue slider 统一原生 range

- `ui_gen/vue.rs:9061` map_tag：slider 双词位统一 → `input`（保留）。
- 属性发射（generate 属性区，slider 分支重写）：`type="range"` +
  `min`/`max`/`step` + value 响应式绑定（`:value` 状态绑定或 v-model
  形态，随仓内 input 两向绑定既有惯例 448 六件取一致）+ `@input`/
  `onchange` 载荷派发（f32 载荷 msg，对齐 a2r 语义）+ disabled + class。
- shadcn 组件路径（generate_shadcn_attrs 12826-12857 + map_tag
  self_closing=true 组件形态）：DSL slider 不再走组件路径（退役让位，
  R-5）；断言 `map_tag("slider", true)=="Slider"`（vue.rs:20671）随
  裁定更新，has_component("slider")（21259）登记面同步清理。
- DOM 验收形态（AC-03）：`<input type="range" min max step>` 在场 +
  拖动/设值后状态与显示同步（playwright 断言）。

### 5.5 T-05 canvas 图元场景契约 v2 + onhit 命中

- **契约**（平行串表族，B12 风格延续；`<前缀>` = scene prop 值）：
  - `<前缀>_nodes`：`"id,x,y,shape,color[,r|w,h]"`（shape=`circle|rect`；
    circle 用 r，rect 用 w/h；坐标为逻辑坐标——coords extent 同 pen）
  - `<前缀>_edges`：`"x1,y1,x2,y2[,color[,width]]"`（线段；from/to
    语义归上游坐标计算，契约只收绝对坐标——R-3）
  - `<前缀>_labels`：`"x,y,text[,color[,size]]"`
  - 三表全缺省 = 现行为（strokes-only）逐字节等价（E-3 消费方零回归）。
- `ui/view.rs`：`CanvasScene` 增 `nodes: Vec<CanvasNode>` /
  `edges: Vec<CanvasEdge>` / `labels: Vec<CanvasLabel>`（Default 空）；
  `View::Canvas` 增 `on_hit: Option<PayloadHandler<M>>`（**String 载荷**
  handler——若仓内无 String 载荷先例 newtype 则随本任务新增
  `ElementHitHandler<M>`，形态同 PointerMoveHandler）。
- `ui/aura_view_builder.rs`：`extract_canvas_scene`（11834）扩三表解析
  （宽容解析——畸形项跳过并 debug 日志，对齐 pts/meta 现行容错）；
  `convert_canvas` 增 `onhit: MsgRef` 事件提取（events map "onhit"）。
- `ui/iced/renderer.rs`：CanvasPainter（7627）`draw` 扩图元绘制
  （circle=Path::circle+fill、rect=Path::rectangle+fill、edge=Line 帧、
  label=text——iced canvas 原语齐备，extent 缩放复用 strokes 现行
  换算）；tap 命中：PenArea/Program 事件面在 pen end 时若
  `移动距离≤容差 && 命中 node` → `on_hit.call(id)`（命中判定：声明序
  倒序 topmost，circle 含 r、rect 含 w/h，坐标经 extent 换算同 pen）。
- `ui_gen/vue.rs`：`generate_canvas_html`（4992）redraw 映射扩三表
  同语义绘制（canvas 2D arc/fillRect/strokeText/fillText）+ 指针
  click 包装命中判定同语义（双端各自实现，共享的是契约不是代码）。
- `ui/snapshot_builder.rs:132`：Canvas 节点 actions 挂 `press`
  （value=元素 id；描述注明命中派发语义）+ props 暴露 nodes 计数/ids
  （MCP 寻址面）。
- `ui/mcp_server.rs`：canvas `press(value=id)` 分发——id→图元中心
  坐标→合成 pen start/end（复用 1507 pen 合成通道形态）+ onhit 派发。
- `aura/schema.rs:2405` canvas ElementDef：props 增 `onhit`（MsgRef，
  「handler receives hit element id (string)」）；description 增图元
  三表与命中语义（docs_gen 门禁覆盖面）。

### 5.6 T-06 能力样板 + 双轨验证 + spec 沉淀

- 新样板 `examples/capability-tests/0NN-canvas-graph/`（NN 顺延现序；
  .at：nodes/edges/labels 表（环形布局上游算好坐标写入表）+ onhit 打开
  页/msg 显示命中 id + slider 控制图元参数（半径/边宽——两能力合一样
  板）+ map 括号写驱动一参数（T-01 语义就地复现）+ SPEC.md 契约成文）。
- 双轨验证：VM MCP（snapshot → press(id) → set_value → 状态断言）+
  vue playwright（DOM range input + canvas 坐标点击命中）——复用
  autoui-verifier 脚本，不写 ad-hoc。
- 图元渲染对拍：VM 截图 vs vue 截图（IoU/RGB 距口径同 563，图元几何
  简单可收紧阈值——详设时定，基线 ≥563 口径）。
- spec 沉淀：SD-01..04 + plans.md 台账行 + specs.json upsert +
  `python scripts/spec-index.py` 再生（merge 期动作，任务内预演）。
- KNOWN-DEBT 登记：缩放/平移后置、edges/labels 命中后置、jade 消费侧
  交接指针（§10）。

### 5.7 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | auto-lang/ui/design/canvas-scene.md | before：canvas 契约详规散于 chart-components.md 563 修订头+schema 描述；after：独立详规——图元三表契约（nodes/edges/labels 平行串表族+坐标系）+onhit 命中语义（nodes-only/声明序 topmost/容差 tap）+渲染纯函数裁定重申+布局上游归属（R-3）+非目标（缩放平移/边命中） | 图元场景是独立能力面，563 笔笔画详规不足以承载扩容语义 | AC-05/06/07 |
| SD-02 | modify | auto-lang/ui/design/chart-components.md | before：:13-16「canvas v2 桥接已由 PLAN-563 落地（scene 前缀双表契约）」；after：增注「图元场景扩容由 PLAN-661 落地（三表族+onhit），详规转 design/canvas-scene.md」 | 563 修订头的契约表述需指向 v2 详规，避免双源 | AC-05/08 |
| SD-03 | modify | auto-lang/ui/overview.md | before：:73 form/payload 覆盖集 v1.8 列 slider（实际 VM 轨缺臂）；after：slider 三轨补全注记（VM/aura 臂+vue range+MCP set_value 闭环，PLAN-661） | 覆盖集表述与实现同步 | AC-02/03/08 |
| SD-04 | modify | aura schema 详规载体（crates/auto-lang/src/aura/schema.rs 条目描述，docs_gen 锚） | before：canvas 条目仅笔笔画语义、无 slider 条目；after：canvas 描述增图元三表+onhit；新增 slider ElementDef（props/语义=a2r 025 基准） | schema 描述是 DSL 侧单一事实源，docs_gen 门禁锚 | AC-02/05/08 |

## 6. 测试设计

- **单测（随各任务）**：map 写路由 golden（字面量/var 键、写后语句续行）；
  SliderChangeHandler Clone/map_msg 组合（panic 占位消除回归）；
  CanvasScene 三表解析宽容（畸形项/缺省空）；命中判定纯函数单测
  （circle/rect 含边、topmost 序）。
- **语料/golden**：`cargo tv`（VM .at 语料——T-01 后全量；预存红基线
  照 568 资源表口径辨识）；a2r slider golden 刷新（025 fixture 源不变）。
- **集成（双轨）**：autoui-verifier——VM：snapshot（Slider kind/actions、
  Canvas press+nodes）→ set_value → press(id) → 状态断言；vue：
  playwright DOM 断言（type="range"、canvas 命中点击）。
- **对拍**：0NN-canvas-graph 双端截图 IoU/RGB（≥563 口径）。
- **门禁分级（Category B）**：开发期 `cargo check -p auto-lang` +
  `cargo t iced`/`cargo t ui`（按步作用域）；schema 描述改动 →
  `cargo test -p auto-lang --test docs_gen`；review/fold 前 `cargo tf`
  （+VM 改动已含 tv 全量）。不触 aavm 域（零 `taa` 触发）。

## 7. 验收标准

- **AC-01（map 括号写）**：078 复现形态（字面量键+var 键）在 VM 轨
  `auto run -r vm` 执行——DoIt 后 count=1、readback="2"、写后语句续行；
  vue 轨零回归；`cargo tv` 无新增红。验证：复现包 drive.mjs 同形脚本
  或样板内 MCP 断言 + `cargo tv`。
- **AC-02（slider VM/aura 闭环）**：DSL slider（025 fixture 语义）经
  `auto run -r vm` —— autoui_snapshot 出 kind=Slider 节点（props
  min/max/value/step + actions 含 set_value）；`autoui_action
  set_value=75` 后快照 value=75 且 model 字段回写可断言（msg 载荷
  `.SetVol(75.0)` 形态）。
- **AC-03（slider vue 轨）**：生成的 SFC 含 `<input type="range">` 且
  min/max/step/value 绑定在场；playwright 设值/拖动后状态与派生显示
  同步（载荷 msg 派发实证）。验证：test_vue_playwright.mjs 形态脚本。
- **AC-04（slider a2r 轨）**：`test_slider_codegen_arm_fixture` golden
  刷新后绿；fixture .at 源零改动（语义基准保持）；map_msg slider
  组合单测绿（panic 占位消除）。
- **AC-05（canvas 图元场景）**：0NN-canvas-graph 三表驱动双端渲染
  （快照 nodes 计数/ids 暴露 + 截图对拍 ≥563 口径）；043-canvas-paint
  双轨零回归（strokes 兼容，`cargo tv` + 既有样板冒烟）。
- **AC-06（命中契约）**：VM `autoui_action press(value=<node_id>)` →
  onhit msg（元素 id 载荷）派发 → 状态可见（样板 msg 面断言）；vue
  playwright 坐标点击节点中心同语义断言；未命中区域点击不派发。
- **AC-07（样板成文）**：examples/capability-tests/0NN-canvas-graph/
  SPEC.md 契约成文（三表+onhit+slider+map 写四语义各一节），DOC_EXCLUDE
  归类能力样板轨道（563 同款）。
- **AC-08（沉淀与围栏）**：SD-01..04 落地；element_coverage/schema_drift
  同步绿（`cargo t` 日常档携带）；plans.md 台账行 + specs.json upsert +
  spec-index 再生；KNOWN-DEBT 登记三后置项+消费侧指针。

## 8. 执行步骤

> 全部在 worktree `D:/autostack/.wt/lang-661/auto-lang`（Plan 529 布局）
> 执行；plan 簿记留主检出。依赖：T-01 独立；T-02 → T-03；T-04 独立；
> T-05 独立（可与 T-01..04 并行）；T-06 收口（依赖全部）。

- **T-01 VM map 括号写路由**
  文件：`vm/codegen.rs`（Expr::Index 赋值臂 ~6942；读臂 ~3405 参照）、
  `vm/native.rs`（仅核对栈约定，不改）。
  动作：index-assign 臂识别 map 容器 → 发 `CALL_NAT auto.hashmap.set`
  （栈序按 native 实现）；078 复现包 probe 收编仓内回归（vm 语料）。
  验证：`cargo check -p auto-lang`；新回归绿；`cargo tv`。
  → AC-01。
- **T-02 SliderChangeHandler newtype**
  文件：`ui/view.rs`（新 type+Slider 变体+map_msg 臂+构造器）、
  `ui/iced/renderer.rs:5050` 臂、`ui_gen/rust.rs:3701-3742` 发射+golden。
  动作：如 §5.2；a2r golden 刷新（fixture .at 源零改动）。
  验证：`cargo t iced`；`cargo test -p auto-lang` 滑窗单测；
  golden diff 审（语义等价——载荷/step/value 不变，仅 handler 包装形态）。
  → AC-04。
- **T-03 aura slider 臂 + 快照 + MCP 闭环**
  文件：`aura/schema.rs`（slider ElementDef）、
  `aura/element_coverage.rs`（登记）、`ui/aura_view_builder.rs`
  （convert_slider 双臂）、`ui/snapshot_builder.rs:348`（actions）、
  `ui/mcp_server.rs`（SetValue 载荷分支）。
  动作：如 §5.3；快照挂法对齐 input/checkbox 现行通道。
  验证：`cargo t ui`；VM 冒烟（025 fixture 形态 .at 走 autoui-verifier
  VM 脚本：snapshot→set_value→断言）。
  → AC-02。
- **T-04 vue slider 原生 range 统一**
  文件：`ui_gen/vue.rs`（map_tag 9061、属性发射分支、shadcn 路径退役、
  断言 20671/21259 更新）。
  动作：如 §5.4。
  验证：`cargo t vue`（若该滤串无别名单测则滑窗 `cargo test -p
  auto-lang vue_`）；playwright DOM 断言（临时 fixture 或并入 T-06 样板）。
  → AC-03。
- **T-05 canvas 图元场景 + onhit**
  文件：`ui/view.rs`（CanvasScene 扩+on_hit handler）、
  `ui/aura_view_builder.rs`（extract_canvas_scene 三表+onhit 提取）、
  `ui/iced/renderer.rs`（CanvasPainter 绘制+tap 命中）、
  `ui_gen/vue.rs`（generate_canvas_html 三表+命中）、
  `ui/snapshot_builder.rs:132`（press action+nodes 暴露）、
  `ui/mcp_server.rs`（canvas press(id) 分发）、
  `aura/schema.rs:2405`（canvas 条目扩）。
  动作：如 §5.5。
  验证：`cargo t iced` + `cargo t ui`；单测（解析/命中纯函数）；
  `cargo test -p auto-lang --test docs_gen`（schema 描述改动）。
  → AC-05/06。
- **T-06 样板 + 双轨验证 + 沉淀**
  文件：`examples/capability-tests/0NN-canvas-graph/`（新）、
  `docs/specs/auto-lang/ui/design/canvas-scene.md`（新）、
  `docs/specs/auto-lang/ui/design/chart-components.md`、
  `docs/specs/auto-lang/ui/overview.md`、
  `docs/plans/KNOWN-DEBT-AND-RISKS.md`。
  动作：如 §5.6；spec 沉淀预演（正式 upsert 归 merge 技能）。
  验证：双轨脚本全绿（VM MCP + vue playwright）+ 截图对拍 +
  `cargo tf`（fold 前全量）。
  → AC-07/08。

## 9. 复审记录

- 2026-09-19（draft handoff，/auto-plan:new）：
  - stage: new；plan_revision: 1。
  - 实勘记录：078 转引 file:line 全部按当前 master 857235623 复核
    （E-1..E-16，行号漂移已修正——view.rs CanvasScene 1031→1044 等）；
    复现包与终裁档原文已读（auto-down worktree down-078 在位）。
  - 起草裁定四项：R-1 onhit 独立契约（id 载荷+press 可达）、R-2 缩放
    平移 v1 非目标、R-3 布局上游下发、R-4 SliderChangeHandler newtype
    （+R-5 vue 原生 range 统一）——依据 §2.2/§4.2 证据，无待用户裁决
    阻塞项。
  - outcome: pass（authorization 边界=起草；执行待用户 `/auto-plan:work`
    授权）。
  - next: work（T-01 起步或 T-05 并行开臂均可）。

## 10. 待澄清事项

- **已裁（起草期，见 §2.2）**：078 四项开放问题全部裁决（R-1..R-4），
  另裁 vue 双路径统一（R-5）。无阻塞性待裁项。
- 执行期小裁量（不阻塞，执行者按证据取最短路径并在任务记录注明）：
  slider handler Option 形态的构造器签名细节；快照 actions 挂法对齐
  input 现行通道的具体机制；图元对拍阈值收紧幅度。
- **后置债（T-06 登记 KNOWN-DEBT）**：缩放/平移（viewport 契约+交互
  回路）；edges/labels 命中上报；fcose 力导近似（上游布局侧）。
- **消费侧交接（jade/auto-down 后续计划，非本仓）**：jade-garden
  graph_view 真渲染（环形/网格布局 .at 侧生成→三表灌入→onhit 打开页；
  VM twin 断言域从壳+过滤派生计数投影扩至真渲染面）；auto-down
  component-gallery 第三单元转正；desktop 图谱页装配。上游依赖=本计划
  AC-01/02/05/06 全绿。
- 078 复现包路径依赖：auto-down plan-078-dev 分支（078 merge 前从
  worktree `D:/autostack/.wt/down-078/auto-down` 取；merge 后随该仓
  master）——T-01 收编仓内后本依赖解除。
