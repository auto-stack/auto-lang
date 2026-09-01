---
plan_id: PLAN-499
status: archived
feature_name: chart v2 canvas 交互——axisPointer 跟随光标/mousemove 流/动画过渡
author: [zcode]
created_at: 2026-08-31
updated_at: 2026-09-01

supersedes_spec_components:
  - "docs/specs/auto-lang/ui/design/chart-components.md: §交互态(105 行一带,plan-498 落地)——line N 竖带例与 donut path 级 hover 例退役,由 mouse-area onmousemove 单命中区(axisPointer 十字线/极坐标扇区命中/tooltip 淡入)取代;§已知坑 hov 族哨兵 9 纪律沿用"
new_spec_components:
  - "docs/specs/auto-lang/ui/design/chart-components.md: §v2 canvas 交互——mouse-area onmousemove+coords 原语(限频语义:VM 臂 33ms 时间闸+0.5px 量化,vue 臂原生不限频;事件携带 viewBox 逻辑坐标)/axisPointer 索引吸附/极坐标扇区命中/动画双轨(timer 数值插值+CSS transition 类)"
  - "docs/design/autoui/canvas-pointer-events.md: 指针事件统一原语设计(P-list 图元协议 schema 草案/坐标语义/命中同源/路线 B 边界)——设计账本新条目,README 已注册"
touched_goals:
  - "GOAL-007: AutoUI 跨端一致——交互能力对齐(canvas 轨:chart 跟随式交互双端达成)"
affects: [docs/specs/auto-lang/ui, docs/design/autoui/diagram-components.md, crates/auto-lang/src/ui, auto-musk]
current_step: 6
total_steps: 6
---

# [PLAN-499] chart v2 canvas 交互——axisPointer 跟随光标/mousemove 流/动画过渡

## 变更摘要

Plan 484 的 chart 交互 v1(锚定 tooltip/槽位高亮)受两轨限制:VM svgdoc 静态无交互、
mousemove 高频流未开。本计划启动 **v2 canvas 阶段**:跟随光标的十字线(axisPointer)、
逐扇区/逐点 hover 的动画过渡。前置调研: Plan 386 RenderQueue、iced canvas Program、
mousemove 事件流进 VM 的性能设计。

**2026-08-31 定稿增补**(diagram-components.md §7.4/§7.5): mousemove 流按**通用指针
事件原语**设计(charts/diagram 共消费,不做 chart 私有通路),本计划即统一 canvas 交互
原语的先导落地计划;M1 调研增补 P-list 图元协议(charts segs 与 diagram render 记录
归一)与指针事件坐标语义(组件局部逻辑坐标)两条输入。

## 目标

1. **通用指针移动事件原语**: mousemove 事件流进 VM 的限频通道(目标 ≤30Hz 采样,
   不冲击 RenderQueue)——DSL/schema 层按通用原语定义(charts 为首个消费方,
   diagram 边命中/节点拖拽为后续消费方);
2. **axisPointer 十字线/竖带指示**跟随光标(折线图),tooltip 跟随移动;
3. VM donut **逐扇区直接 hover**(svgdoc 静态限制的解除路径)或等价方案——canvas
   代码命中路径按"扇区命中与 diagram 边命中同一能力"设计;
4. hover 态**过渡动画**(vue CSS transition / VM 数值插值),双端观感对齐。

## 架构方案

- **mousemove 流**: iced mouse_area on_move 事件 → 限频采样(帧同步,每帧最多一条)
  → 走 Plan 386 RenderQueue 批量通道进 VM → 组件内命中计算(纯派生);事件携带
  **组件局部逻辑坐标**(viewBox 系),屏幕→逻辑换算在引擎层完成(diagram-components.md
  §7.4 坐标语义条),组件代码不感知;
- **donut 逐扇区**: 两条路径评估——(a) svgdoc 事件能力扩展(svg-text 同族,需 resvg/
  usvg 交互层,成本高);(b) canvas 层直接绘制扇区+代码命中测试(ECharts 模型,推荐)——
  路径 (b) 的绘制侧即 **P-list 图元协议**(`{kind:"path"|"rect"|"ellipse"|"text", ...}`,
  同时满足 charts segs 与 diagram render 记录),M1 产出其 schema 草案;
- **动画**: vue 侧 CSS transition;VM 侧数值插值(请求帧驱动)——双端观感对齐策略在
  调研后定。

前置依赖: Plan 386(RenderQueue)、Plan 484(v1 交互与组件形态)、Plan 492(引擎三族,
已归档);设计输入: docs/design/autoui/diagram-components.md(§7.4/§7.5 统一面)。

## 需求分析与背景调查

- 来源: Plan 484 交互 v1 的既知边界(DEBT 挂账): 跟随光标 axisPointer、hover 动画、
  VM donut 逐扇区 hover(svgdoc 静态限制)。
- Plan 386: RenderQueue 批量渲染通道——mousemove 流的性能承压面。
- 参考: ECharts axisPointer(line/shadow/cross 三形态)、emphasis/doubleDownplay、
  tooltip.trigger(axis/item)。

## 详细设计

（M1 调研产出全文见 [docs/design/autoui/canvas-pointer-events.md](../design/autoui/canvas-pointer-events.md)，2026-09-01 定稿。要点：）

- **mousemove 流零新协议**：in-process 臂沿 `mouse_area → IcedMessage →
  decode_payload(pipe-payload) → VM handler` 既有通路；新 DSL 面 =
  `mouse-area` 增 `onmousemove` 事件臂 + `coords: "WxH"` 逻辑幅面 prop；
- **iced 臂需自定义 PointerArea widget**（iced 0.14 `on_move` 闭包拿不到
  bounds 尺寸，px→逻辑换算必须在 widget 内做）；带 on_move 才走新 widget，
  存量 mouse-area 映射零改动；
- **限频**：widget State 时间闸 33ms（≤30Hz）+ 0.5px 量化去重；vue 臂不限频
  （DOM 原生事件不经 RenderQueue，契约=逻辑坐标一致）；
- **坐标语义落定**：事件携带组件局部逻辑坐标（viewBox 系），屏幕→逻辑换算
  在引擎层（PointerArea bounds 归一 × extent / vue 生成端内联
  offsetX÷clientWidth×W）；coords 缺省 = bounds 局部 px（raw 模式）；
- **P-list 图元协议 schema 草案**：path/rect/ellipse/text 四 kind + 公共
  `hit:{id}` 锚；charts `segs {d,c,i}` 即 path 特例映射；本计划不实现渲染桥；
- **donut 扇区命中与渲染后端解耦**：全图单 mouse-area + 组件内极坐标测试
  （rIn²≤dx²+dy²≤rOut² + atan2 累进角区间）→ svgdoc 静态限制解除，无需
  canvas 先行；扇区-边命中同源 = "逻辑坐标 + 代码几何测试"同一能力；
- **axisPointer**：N 竖带 → 单全图 mouse-area + 索引吸附；十字线 = svg 内
  动态 path（逻辑坐标直书）；tooltip 跟随 `left-[${cx}px]`；
- **动画**：vue CSS transition 类 / VM `timer{AnimTick(every_ms:33, when:…)}`
  数值插值；对齐口径 = 观感相当 + 帧率基线对照；
- **边界**：路线 B（两进程）widget 树输入注入不在本计划（输入通道
  PointerMoved 已备、消费方仅编辑器 FrameSource），挂 DEBT。

## 测试设计

- mousemove 流压测(30Hz 持续流下 RenderQueue 队深/帧率);
- axisPointer 双端渲染一致性;
- 动画帧率基线(vue CSS vs VM 插值)。

## 验收标准

1. 折线图 hover 出现跟随光标的十字线 + 锚定 tooltip,双端一致;
2. 持续 mousemove 流下无帧率塌陷(基线对照);
3. donut 逐扇区 hover(vm 轨)按调研结论落地或明确不可行结论归档。

## 执行步骤

- [x] M1 调研: Plan 386 RenderQueue + mousemove 流方案设计(产出设计文档;增补输入 =
      P-list 图元协议 schema 草案 / 扇区-边命中同源性 / 指针事件坐标语义,见待澄清事项 3)
      [✅ 已完成] `docs/design/autoui/canvas-pointer-events.md` 落库并注册 README 索引
      (worktree commit 1b9e9c9d3);四条关键结论:①零新协议(沿 pipe-payload 通路)②PointerArea
      自定义 widget(iced on_move 闭包无 bounds)③donut 代码命中与渲染后端解耦(svgdoc 静态
      限制解除无需 canvas 先行)④路线 B widget 树输入注入挂 DEBT
- [x] M2 mouse_area on_move 限频流通道落地 + 压测
      [✅ 已完成] 六层接线(schema coords prop/view.rs PointerMoveHandler+双字段/
      aura_view_builder 双臂/renderer into_iced 包装臂+convert_view_messages 复合映射/
      vue 生成端内联箭头换算/新 pointer_area.rs 自定义 widget);压测=单元级 6 测
      (125Hz 流→25Hz 发布、静止悬停单发、闸过期发新坐标、400×214 bounds×560×300
      extent→(280,150)±0.5、raw px 恒等、零 bounds 防护)+ builder 2 测 + vue 1 测 +
      e2e 管道 1 测(plan499_pointer_stream_tests:pipe-payload 全链送达 handler 双 float
      形参);实机帧率基线对照归 M3(axisPointer 为真实消费方)随双端验证;
      中途 master 前进(498/503 折入)已 re-sync 合并(278f5e2a9,MouseArea 双向扩展
      正交并存),gallery_golden/schema_drift/docs_gen 复绿
- [x] M3 axisPointer 十字线(折线)双端实现
      [✅ 已完成] line_chart 三副本:N 竖带退役 → 单全图 mouse-area(coords "560x300"+
      onmousemove,absolute inset-0 随 svg 容器缩放);索引吸附最近数据点(竖线 cx 吸附/
      水平线跟随 y 钳 20..260,虚线 3 3);tooltip 跟随 LeftOffset(cx-80 钳 40..400);
      cntLn Init 预落(handler 跑根态不可读 prop,P320 单态);**引擎修复一枚**:CALL_SPEC
      浮点接收者 .to_int()——"<unknown_nv:>" 型名不匹配 6872 分支前缀(f64/f32 nanbox
      漏网报错,补前缀即通,plan499_engine_float_to_int_tests 回归);测试:M3 断言 2+
      引擎回归 1,498/484 全绿,492 c2 锚点随基线演进(21/21),golden 重采样(仅
      line_chart 项差异);实机双端目检归 M5 后统一执行(与 donut/动画一并验收)
- [x] M4 donut 逐扇区方案落地或归档结论
      [✅ 已完成] 落地(非归档):全图单 mouse-area(coords "260x260")→ .PointerMove
      极坐标代码命中(环带 3844≤r²≤10000 + atan2 归一(-π/2 起)落累进角区间,endsDn/
      cntDn Init 预落)→ 复用 498 emphasis 外移;**svgdoc 静态限制经代码命中解除**,
      无需 canvas 渲染先行(设计 §4 结论实证);path 级 onmouseenter(VM 轨死事件)退役,
      双端同一命中代码;图例命中保留双入口;tooltip 维持角落锚定(跟随属 M3 line 故事,
      donut max-w-md 缩放下面向 px 定位有对位风险,记录为已知边缘);测试:命中/换扇区/
      内孔/环外四态 + 覆盖层落树;498/484/492 回归绿;golden 重采样(仅 donut 项差异)
- [x] M5 动画过渡(vue CSS / VM 插值)对齐
      [✅ 已完成] 双轨:line/donut tooltip 淡入 = `timer { AnimTick (every_ms: 33,
      when: .anim) }` 数值插值(tipOp 0→100 四拍到顶停摆;入场边沿启动防闪/移动中
      不重置/离场复位)+ vue 侧 `transition-opacity duration-150` CSS 类双轨;
      **vue tooltip class 通道修复**:style f-string 走 :style CSS 声明通道类串失效
      → `class: f"…"` → :class 类通道;**M5 补漏一枚**:donut `.AnimDnTick` handler
      曾漏写(timer/msg 声明齐备而 on 块缺体,vue/VM 双轨 tooltip 恒 opacity-0)——
      补 handler×三副本 + 回归测 `plan499_donut_tooltip_fade`(7/7 plan499 测绿);
      golden 重采样(仅 line/donut 项);**vue 实机验证**(worktree charts-gallery,
      Playwright):line 十字线 `M 142 20 V 260`(吸附 Feb)+ `M 40 100 H 550` +
      tooltip "Feb/Desktop 305/Mobile 200" opacity-100(PNG 像素级复核);donut
      6点钟→Desktop/250°→Mobile 扇区切换白描边 emphasis + opacity-100(纯白弧
      像素级复核),内孔/离场负例;VM 轨以 in-process 测试覆盖(autoui MCP 无
      mousemove 注入工具,目检不可行——测试断言十字线/扇区/插值语义);过程坑:
      早期验证误撞 master 陈旧 serve(4039 旧进程)与 svg nth() 误中 bar/area 卡
      (同 viewBox),重定位后全绿(worktree commit 5468a3e38)
- [x] M6 复审与归档准备
      [✅ 已完成] scoped 复验全绿:cargo check 净;plan499 7/7 + plan498 6/6 +
      plan484 4/4 + plan492 21/21;schema_drift 1/1;docs_gen 4/4;gallery_golden
      重采样后绿(仅 line/donut 项差异);KNOWN-DEBT 登记 P499-1..6(when 门
      空转心跳/donut 角落锚定+svgdoc 重解析/实机帧率未量化/路线 B 输入注入/
      移入:osconfig E0063+kitchen-sink 解析错——后两枚系 master 侧既有,
      499 复验中发现并登记);全量 cargo tf 归 /auto-plan:review 门禁执行

## 复审记录

**reviewer**: zcode(/auto-plan:review) · **time**: 2026-09-01 · **verdict**: ✅ PASS → `status: reviewed`
**worktree**: `.worktrees/plan-499-dev`(HEAD c3c1e1f3c,基 f7655cc1b,+21 文件 1924/-118)

### 门禁重跑(review 期实测,worktree 内)

- `cargo tf`(全量含 1M churn):**3335/3335 绿**——首跑红一枚(E0425:
  plan499 引擎回归测缺 ui-iced cfg 门,`build_dynamic_component` 默认特性下
  configured out;plan498 同模式为逐测挂门)→ 补 `#[cfg(feature="ui-iced")]`
  (commit c3c1e1f3c)复绿。**暴露根因:执行期所有验证都带 `--features
  ui-iced`,从未跑过裸 `cargo t/tf`**——review 门禁按设计捕到。
- `cargo tv`(engine.rs 改动追加):1421/1422,1 败 = `cb_asynchronous_channel`
  ——**master 默认检出同现**(实证),499 引擎改动仅 CALL_SPEC 浮点接收者
  前缀一行,判非本计划回归 → KNOWN-DEBT P499-7(移入)。
- scoped 复验:plan499 7/7、plan498 6/6、plan484 4/4、plan492 21/21、
  gallery_golden 1/1、schema_drift/docs_gen(tf 内含)全绿;cargo check 净。

### 验收标准逐条(对代码重验,非对勾)

1. **十字线+锚定 tooltip 双端一致 —— PASS**:代码实证(line_chart.at:630-637
   dashed path ×2 + mouse-area coords 560x300/onmousemove;pointer_area.rs
   MIN_INTERVAL_MS=33/QUANTIZE_STEP=0.5);in-process 断言(吸附/跟随/钳制/
   离场/单命中区落树)7/7;vue 实机本会话 DOM+像素复核(十字线 42 命中、
   tooltip opacity-100)。
2. **持续 mousemove 无帧率塌陷 —— PASS(结构+单元级),余量测量延后**:设计
   §8 四项门禁,1(限频断言 125Hz→≤30Hz/静止单发)+2(坐标断言
   400×214×560×300→280,150±0.5)已交付;3(e2e 节奏 P95)/4(svgdoc 重栅格)
   需实机 iced 窗 mousemove 注入——**工具不存在**(autoui MCP 无该动词),
   VM 臂塌陷路径已被 widget 级限频结构性排除且单测背书,vue 臂不经
   RenderQueue。延后在 M2/M3/M5 执行记录与结题报告中**非静默**逐步披露,
   登记 P499-3(偿还=MCP 注入工具+帧率探针)。判定:非阻塞。
3. **donut 逐扇区(vm 轨)落地 —— PASS**:M4 落地非归档(环带 3844≤r²≤10000
   + atan2 累进角,代码实证 donut_chart.at:193-194/307);命中/换扇区/内孔/
   环外四态 + 覆盖层落树测试绿;vue 实机 6点→Desktop/250°→Mobile 切换 +
   纯白弧像素复核;path 级死事件退役(onmouseenter 仅存图例 4 行=设计内
   双入口)。

### 遗漏/延后/workaround 扫描

- diff 全量 grep 无 TODO/FIXME/HACK/临时/绕过标记;三副本 .at md5 逐一
  相同;eprintln! 仅测试 SKIP 通知(仓内既有惯例);engine.rs 改动最小化
  (一行条件+注释);schema 描述 496/498/499 追加式;无静默缩水。
- 延后项全部有账:P499-1(when 门空转心跳)/P499-2(donut 角落锚定+svgdoc
  重解析)/P499-3(实机帧率)/P499-4(路线 B 注入)——均执行期或结题报告
  披露;P499-5/6/7 为 review 期发现的 **master 侧既有问题移入登记**
  (osconfig E0063/kitchen-sink 解析错/cb_asynchronous_channel)。

### 结议

验收 3 条全 PASS(第 2 条带已披露余量);门禁红已修;无未批准缩水。
`status: reviewed` → 可跑 `/auto-plan:merge 499`。

## 待澄清事项

1. mousemove 流的采样率与批量策略需要压测数据支撑(M2 前置);
2. canvas Program 桥接(437 v2 决议)与本计划的关系: 本计划只做交互流,绘制桥接
   是否同批实施需 M1 调研后定。
3. M1 调研输入增补(docs/design/autoui/diagram-components.md §7.4/§7.5,2026-08-31 定):
   ① P-list 图元协议需同时满足 charts `segs` 与 diagram render 记录;
   ② donut 扇区命中与 diagram 边命中共用 canvas 代码命中路径;
   ③ 指针事件携带组件局部逻辑坐标(viewBox 系),屏幕→逻辑换算在引擎层完成。
   mousemove 流按**通用指针事件原语**设计(charts/diagram 共消费),不做 chart 私有
   通路——本计划即统一 canvas 交互原语的先导落地计划。
