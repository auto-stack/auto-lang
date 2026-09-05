---
plan_id: PLAN-563
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: canvas-element
author: [zhaopuming]
created_at: 2026-09-05
updated_at: 2026-09-05

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]       # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
total_steps: 10
---

# [PLAN-563] canvas 基础组件——状态驱动画布 + 笔画事件面（vue/iced 双端独立实现）

## 变更摘要

`schema/aura.at` 的 `canvas` 元素自 P1 抽取起一直是占位（props TBD、
backends 全 none）。但**平台能力两端正齐备且仓内已实证**（2026-09-05 调研，
用户纠偏确认）：

- iced：`broker_surface.rs:43` `DrawListPainter` 已 `impl
  iced::widget::canvas::Program`（Frame/Path 均在用）；`renderer.rs:6112`
  ToastPeekPainter 同证；
- vue：浏览器原生 `<canvas>` 2D；
- 事件：抽象视图层 `MouseArea`（`view.rs:760` / `renderer.rs:3984`）已有
  `on_click / on_release / on_move / on_context_menu`——iced mouse_area
  按下/抬起/移动全通；vue 侧有 window/document 级事件修饰符机制
  （`ui_gen/vue.rs:604`）与 mouse-area onmousemove 换算先例（499 M2）。

缺的只是 **AURA 面**：schema 元素定义、状态驱动场景模型、笔画事件语义、
双端生成/渲染。本计划补齐这一层：`canvas` 元素 = **从状态渲染的画布**
（strokes 列表 → 双端各自画）+ **笔画事件三件套**（press 门控在底层）。
经典 MSPaint（Plan 553 待澄清①指向的消费 app）与白板/签名板/截图标注/
打砖块类游戏全部以此为地基。

**双端独立性**（用户明确要求）：vue 与 iced 各自独立实现渲染（vue =
`<canvas>` 2D context；iced = `canvas::Program`），不共享绘制代码；共享的
只有 **场景数据契约**（strokes 状态 → 渲染的映射规约，写入 schema 描述与
SPEC，双端对拍锁定）。

## 目标

1. **schema 定案**：`canvas` 元素从占位升正式——props（`scene`、`coords`、
   `clear`）+ 事件（`onpenstart/onpenmove/onpenend`）+ 双 backend 声明。
2. **抽象视图层**：`AbstractView::Canvas` 变体 + aura_view_builder 解析。
3. **iced 实现**：canvas::Program 渲染器（strokes → Frame/Path stroke），
   mouse_area 按压门控接线（press 起笔 / move 走笔 / release 收笔）。
4. **vue 实现**：`<canvas>` 生成 + 2D 绘制 effect（strokes → 2D API）+
   mousedown/mousemove/mouseup 事件生成（逻辑坐标换算，499 M2 同型）。
5. **首个冒烟 app**：`examples/capability-tests/031-canvas-paint/`（能力
   样板轨道——正式消费 app"经典 Paint"另立计划，不混入本计划）。
6. 双端行为对拍：同 strokes 状态 → 双端截图一致性（445/537 对拍惯例）。

## 架构方案

### 场景模型：状态驱动（Model A，chart 同范式）

```
.at 作者面（声明式，全状态驱动，无命令式绘图 API）：

canvas {
    scene: .strokes          // 画布内容 = model 状态（唯一事实源）
    coords: "400x300"        // 逻辑坐标系（onpen* 事件 x/y 值域，499 同型）
    onpenstart: .PenStart    // (x, y) float 参数
    onpenmove: .PenMove      // 仅 pen-down 期间派发（门控在底层）
    onpenend: .PenEnd        // 收笔（含坐标）
    style: "w-full h-64 border rounded-lg"
}

// strokes 状态形态（B12 规避：平行字符串列表，028 先例）：
// .stroke_pts  = ["x1,y2|x2,y2|...", ...]   每笔画点列（| 分隔）
// .stroke_meta = ["#111827,3,0", ...]       颜色,线宽,eraser 位
```

- 渲染 = 纯函数：`strokes 状态 → 双端各自的绘制`。双端不共享代码，共享
  **映射规约**（点列解析、坐标 → 像素、线帽/拐角、clear 色）。
- MCP 可测性：pen 事件只改状态 → autoui_state 断言笔画/点数（553 套件
  同型），不依赖像素读取。
- 序列化免费：strokes 本就是字符串状态 → storage 存取零额外面。

### 事件面：笔画三件套（press 门控在底层）

- `onpenstart(x,y)`：mouse_area on_press（iced）/ mousedown（vue）触发，
  逻辑坐标换算（`coords` 声明值域；499 引擎侧 screen→logical + ≤30Hz
  节流（VM 臂）同款复用）。
- `onpenmove(x,y)`：**仅按下期间派发**——门控由底层维护（iced mouse_area
  on_move + 按下态；vue document 级 mousemove 监听 + buttons 位），
  .at 作者无感。
- `onpenend(x,y)`：release（含移出画布后的收笔——vue 用 window 级
  mouseup，`onmousemove.window` 机制先例 481/499）。
- 与既有 `hit-region`（hover 向 onmousemove）关系：**并存不合并**——
  hit-region 保持 hover 语义（chart tooltip 面，已冻结）；canvas 的 pen
  三件套是新的成组语义。schema 描述里互相引用。

### 双端实现路径（各自独立）

- **iced**：`renderer.rs` 新增 `Canvas` 视图臂 → 包 `iced::widget::canvas`
  + 自有 `Program` 实现（`broker_surface.rs` DrawListPainter 形态参考，
  但**独立实现**：直接 strokes→Path stroke，不经 DrawOp 线协议——
  `desktop_protocol::DrawOp` v1 只有 quad/text/scissor 无路径 op，canvas
  元素进程内渲染不经过它；远程宿主画布化登记远期，见待澄清③）。
  事件：canvas 外包 mouse_area（on_press/on_move/on_release）。
- **vue**：`ui_gen/vue.rs` 新增元素臂 → `<canvas ref>` + watch/effect 里
  2D 绘制（clear + 逐笔画 polyline，lineCap/lineJoin round）+ 事件三件
  （mousedown / document mousemove（buttons&1 门控）/ document mouseup），
  坐标换算含 devicePixelRatio。

### 与 chart/svg 家族的关系

chart（437/ADR-19）与 diagram（502）走"派生几何 → svg 节点族"路线，
适合**结构化图形**；canvas 元素补的是**自由笔迹/高频点列**生态位——
T1 调研将核对 chart 在 iced 端的实际渲染路径（几何在 aura_view_builder
派生），确认二者不重叠、schema 描述互引。

## 需求分析与背景调查
（从 docs/specs/overview.md 与相关 module spec 取材）

- **GOAL-007**（AutoUI 跨端一致）双端同源契约；**GOAL-008**（声明式 UI
  表达力）新原语；**GOAL-010**（AutoOS 默认应用集）经典 Paint 的地基。
- **来源**：Plan 553 待澄清①（真画布原语另立）+ 用户 2026-09-05 纠偏
  （"浏览器原生 canvas；iced 也有 canvas 组件"）——调研证实：平台齐备，
  缺的是 AURA 面；553 的像素形态绕行不再是唯一选择。
- **仓内证据**（2026-09-05 grep）：
  - `broker_surface.rs:43-160`：canvas::Program + Frame/Path/Text 实证；
  - `renderer.rs:3984`：MouseArea on_click/on_release/on_move/on_context_menu
    抽象面已通（iced mouse_area）；
  - `ui_gen/vue.rs:604/696/3741`：`.window`/`.document` 事件修饰符机制；
    `:5914` 499 M2 mouse-area onmousemove 内联换算先例；
  - `view.rs:760`：hit-region 抽象（hover 面，chart 向，与本计划并存）；
  - `schema/aura.at:1152`：canvas 占位条目（本计划的改造对象）；
  - `desktop_protocol/message.rs:76`：DrawOp v1 = quad/text/scissor
    （无路径——canvas 不经此线协议）。
- **范式先例**：chart 系列=状态→渲染（437）；30Hz 节流逻辑坐标流（499）；
  B12 平行字符串列表（028）；双端截图对拍（445 charts 19/19、537）。

## 详细设计

### 1. schema（`schema/aura.at` canvas 条目重写）

```
element canvas {
    tag: "canvas"
    category: "media"          // T1 复核归类（chart 家族相邻）
    tier: <T1 定：与 chart 同档>
    backends: { web: "native", iced: "native", gpui: "unknown" }
    props: [
        scene（绑定点列状态）, coords（"WxH" 逻辑值域）, clear（背景色，可选）
    ]
    events: [ onpenstart, onpenmove, onpenend ]   // (x, y) float
    allows_children: false      // 纯状态驱动，无声明式子节点（v1）
    description: 状态驱动画布……
}
```

### 2. 抽象视图（`view.rs` + `aura_view_builder.rs`）

- `AbstractView::Canvas { scene: 绑定表达式, coords: (w, h), clear,
  on_pen_start/on_pen_move/on_pen_end: Option<MsgRef>, style }`；
- builder 解析 props/事件绑定（hit-region onmousemove 解析同型）。

### 3. iced 端（`renderer.rs`）

- `Canvas` 臂 → `mouse_area(canvas(Program)).on_press/on_move/on_release`；
- Program 持 strokes 快照（视图脏位联动重建 Geometry，Cache 复用——
  broker_surface Geometry cache 同型）；点列解析 → `Path::new` +
  `frame.stroke(&path, Stroke{width, color, line_cap: Round, line_join: Round})`。
- 逻辑坐标换算：组件盒尺寸 → coords 值域线性映射（499 换算同型）；
  VM 臂 move 节流 ≤30Hz。
- eraser 位 v1 = 背景色笔画（不挖除）——规约写明。

### 4. vue 端（`ui_gen/vue.rs`）

- 元素臂：`<canvas :ref + class>` + `onVnodeMounted/Updated` 钩子里
  effect 重绘（清屏 + 逐笔画 `ctx.beginPath/moveTo/lineTo/stroke`，
  lineCap/lineJoin round，devicePixelRatio 缩放）；
- 事件：`@mousedown` → onpenstart（换算坐标）；document mousemove（
  `buttons & 1` 门控）→ onpenmove；document mouseup → onpenend（
  `.window` 修饰符机制承载，481/499 先例）；同样 30Hz 节流换算。
- scene 变更重绘：watch scene 引用 + 深比较点列长度（T1 定具体机制：
  状态脏位 vs deep watch——vue 侧已有同类机制则复用）。

### 5. 冒烟样板（`examples/capability-tests/031-canvas-paint/`）

极简自由画板：pen 三事件追加 `stroke_pts`/`stroke_meta`，颜色/线宽选择，
undo（弹尾笔）、清空、storage 存取。**放 capability-tests 轨道**（552 迁
探针同目录；正式"经典 Paint"应用待本计划 review 后另立 plan 填 032+
空号）。

## 测试设计

1. **单元（Rust）**：点列解析/坐标换算纯函数；`build_dynamic_component`
   编译含 canvas 的 .at 冒烟（shell pack 同型 pack 测试）。
2. **layout/渲染测试**：Canvas 视图臂进 iced 渲染树冒烟
   （`layout_tests.rs` 惯例）。
3. **vue 生成测试**：ts_adapter/vue.rs 元素臂产物含 `<canvas`、三个事件
   绑定与换算调用（既有生成测试形态）。
4. **MCP 状态断言**（样板 app）：penstart/penmove/penend 派发 →
   strokes 状态点数/笔画数断言（VM 轨；553 套件 harness 复用——注意其
   stdout DEVNULL 与 vnode-id refresh 教训）。
5. **双端截图对拍**：固定 strokes 状态（手写播种）→ vue/iced 截图
   一致性（445/537 对拍惯例；容差策略 T1 定）。
6. **门禁分级**：Category B（UI 模块改动）——`cargo check -p auto-lang` +
   `cargo t`（局部：canvas/render 相关模块名）+ 生成测试；复审是否跑
   `cargo tf` 由 reviewer 裁定（schema/渲染器核心面，倾向跑）。

## 验收标准

1. schema canvas 条目正式化（props/events/backends 齐，docs_gen 绿）。
2. 双端渲染：同一 .at 样板在 `auto run`（vue）与 `auto run -r vm`（iced）
   均可自由绘画，笔迹视觉一致（对拍通过）。
3. 笔画事件三件套双端行为一致：按下起笔、按住走笔（含快速拖动不丢段，
   ≤30Hz 节流内）、抬起/移出收笔。
4. 样板 app MCP 断言绿；strokes 状态可 storage 持久化。
5. 既有回归零破坏：`cargo t` 局部绿；schema_drift/docs_gen 绿（schema 改动）。
6. SPEC/设计注记：场景数据契约与双端映射规约成文（schema description +
   capability-tests 样板 SPEC）。

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- [ ] **T1 调研定案**
  核对四件事并写回本节：a) chart/diagram 在 iced 端实际渲染路径
  （确认与 canvas 生态位不重叠）；b) vue 侧 scene 变更重绘机制选型
  （状态脏位 vs deep watch——找既有机制复用）；c) canvas 元素 tier/category
  归档（schema 邻近条目对照）；d) iced mouse_area on_move 在按下态的
  语义确认（移出组件后 move/release 是否仍达——决定 vue window 级
  对齐面）。
  验证：调研笔记 scratch/p563/probes.md
- [ ] **T2 schema 正式化**
  `schema/aura.at:1152` canvas 条目按详细设计 §1 重写；docs_gen 校验。
  验证：`cargo test -p auto-lang --test schema_drift --test docs_gen`
- [ ] **T3 抽象视图层**
  `crates/auto-lang/src/ui/view.rs` 增 `Canvas` 变体；
  `aura_view_builder.rs` 解析 props/事件（hit-region onmousemove 同型）。
  验证：`cargo check -p auto-lang --features ui-iced`
- [ ] **T4 iced 渲染臂**
  `crates/auto-lang/src/ui/iced/renderer.rs`：mouse_area+canvas(Program)
  包装、strokes→Path stroke、坐标换算、move 节流。
  验证：`cargo t`（渲染相关局部）+ 最小 .at 手动 vm 冒烟
- [ ] **T5 iced 事件接线**
  on_press/on_move/on_release → onpenstart/move/end 消息派发（含移出
  收笔语义，T1d 结论）。
  验证：vm 冒烟手绘 + MCP state 断言探针
- [ ] **T6 vue 生成臂**
  `crates/auto-lang/src/ui_gen/vue.rs`：`<canvas>` 元素臂 + 重绘 effect
  （T1b 机制）+ 事件三件生成（.window 修饰符承载）。
  验证：`auto build` 产物 grep `<canvas` + 生成单测
- [ ] **T7 样板 app**
  `examples/capability-tests/031-canvas-paint/`：极简画板（pen 三事件 +
  颜色/线宽 + undo/清空/storage）+ SPEC.md（场景数据契约成文）。
  验证：`auto run` + `auto run -r vm` 双端手绘
- [ ] **T8 MCP 状态断言**
  样板 `tests/desktop_mcp.py`（pen 事件 → strokes 状态断言；harness 复用
  553 教训：DEVNULL/id-refresh）。
  验证：`python tests/desktop_mcp.py` 双轨
- [ ] **T9 双端截图对拍**
  播种固定 strokes → vue（playwright）/iced（autoui_screenshot）截图对拍
  （445/537 惯例；容差 T1 定）。
  验证：对拍脚本输出 PASS 归档 scratch/p563/
- [ ] **T10 文档与终检**
  schema description 定稿；README/docs（capability-tests 目录行）；
  `cargo check` 零警告 + 局部测试绿 + schema_drift/docs_gen 绿。
  验证：终检命令清单全绿

## 复审记录

## 待澄清事项

1. **后续消费 app 队列**（本计划只交地基）：经典 MSPaint（032+ 空号正式
   应用轨道）、白板、签名板、截图标注、557 打砖块类——各自另立 plan。
2. scene 的 v2 形态（shapes 家族：rect/circle/text 图元、eraser 真挖除、
   变换）远期——v1 只做笔画点列，契约预留 `stroke_meta` 第三位扩展。
3. **远程宿主画布化**：`desktop_protocol::DrawOp` v1 无路径 op（quad/text/
   scissor）——Smithay 宿主线（509）远期若要远程渲染 canvas，需 DrawOp
   增 Path 变体（线格式追加式，TextStyled/Scissor 同例）；本计划进程内
   渲染不经过该协议。
4. gpui 后端：`backends.gpui: "unknown"` 保持，待该线启动再定。
