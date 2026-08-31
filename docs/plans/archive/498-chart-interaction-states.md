---
plan_id: PLAN-498
status: archived
feature_name: chart 交互状态机——emphasis 高亮/转折点浮现/legend 点击切换显隐
author: [zcode]
created_at: 2026-08-31
updated_at: 2026-09-01

supersedes_spec_components:
  - "docs/specs/auto-lang/ui/design/chart-components.md: 484 hover 单态节(hoverIdx int = -1 锚定 tooltip)——被 emphasis 二态/legend 显隐/转折点浮现交互模型取代"
new_spec_components:
  - "docs/specs/auto-lang/ui/design/chart-components.md: 交互设计节(emphasis 二态模型/悬停态字段 hovLn·hovAr·hovBr·hovDn 无悬停哨兵 9/legend 显隐 vis 族/转折点浮现;规约文本沉淀时须同步哨兵与字段族名,勿沿用 -1 旧例)"
  - "docs/specs/auto-lang/ui/design/chart-components.md: mouse-area onclick 事件臂(M0 引擎改动登记——iced on_press/vue @click,diagram select 共同前置)"
  - "schema/aura.at + aura/schema.rs: element mouse-area events 表增 onclick(顺带补登 496 漏写的 ondblclick)"
touched_goals:
  - "GOAL-007: AutoUI 跨端一致——chart 交互(emphasis/显隐)双端同源"
affects: [docs/specs/auto-lang/ui, examples/widgets-gallery, examples/charts-gallery, examples/ui/024-charts, crates/auto-lang/src/ui, schema/aura.at]
current_step: 7
total_steps: 7
---

# [PLAN-498] chart 交互状态机——emphasis 高亮/转折点浮现/legend 点击切换显隐

## 变更摘要

Plan 484 交付了四类 chart 原语的静态渲染 + hover tooltip(锚定式)。本计划补齐**交互态**:
悬停高亮(emphasis/downplay 二态)、转折点浮现、legend 点击切换系列显隐。设计已定稿
(484 会话交互设计节,chart-components.md)。主体为组件层实现,走
"hoverIdx/visible 状态 + Init 预计算 + view 条件样式分支"的既有通路;**唯一引擎改动 =
M0: `View::MouseArea` 增 `on_click` 臂**(496 ondblclick 同款先例;diagram-components.md
§7.6 登记的缺口——legend 点击与 diagram select 共同依赖,一次扩展两计划受益)。

## 目标

1. hover 系列分组/折线/扇区 → **emphasis 高亮**:该系列视觉增强(线宽 2→3、opacity 1),
   其余系列 downplay 弱化(opacity 0.25);
2. line: 高亮系列浮现**转折点圆点**(Init 预计算每点坐标,hover 时仅渲染该系列的);
3. donut: 悬停扇区(图例行触发)→ 该扇区沿中角**外移 offset** + 描边(ECharts emphasis.scale);
4. **legend 点击切换系列显隐**(visible0..3 状态数组 + onclick 翻转 + view 条件跳过该系列全部元素);
5. 双端一致: vue(CSS transition 淡入增强)/ VM(直接切换),状态与几何完全同源;
6. **M0 前置**: mouse-area `on_click` 臂双端落地(vue `@click` / iced `on_press`),
   schema 双源(`aura/schema.rs` + `schema/aura.at`)同步登记——M4 与 diagram select
   的共同前置(diagram-components.md §7.6)。

## 架构方案

引擎改动仅 M0 一处(mouse-area `on_click` 臂,496 ondblclick 先例的逐点复刻,落点见
详细设计 M0)。交互状态机 = "Init 纯派生 + 最小状态写入 + view 状态投影"——与既有 hover
tooltip 同款三段式:
- Init: 预计算每系列的 常驻样式/高亮样式 对(如 stroke-width、opacity、点半径);
- 事件: `.HoverSeries(k)` / `.HoverOut` 写 `hoverSeries int = -1`;`.ToggleSeries(k)` 翻转
  `visibleK bool`;
- view: 按 hoverSeries/visible 状态选 每系列的高亮/常驻样式分支。

约束: 交互样式一律走**静态 class 或状态槽位字符串**(禁止 f-string `${}` 含字面量 `[]`、
禁止 Index 值直赋——沿 484 实证的包组件纪律,见 chart-components.md 已知坑)。

## 需求分析与背景调查

- 来源: Plan 484 归档后用户目检反馈(2026-08-31)——hover tooltip 工作后,参照 ECharts
  交互模型提出高亮/转折点/legend 联动需求。
- ECharts 交互四要素对照: ① hit-test(命中) → 已有(竖带/图例行 mouse-area);② emphasis/
  downplay → 本计划核心;③ tooltip → 已有(锚定式);④ axisPointer 跟随光标 → v2 canvas
  (DEBT 挂账,需 mousemove 流);⑤ legend 点击切换显隐 → 本计划。
- 现有基础: hover 状态机已在四组件运行(hovered/tipTitle/tipBody + 竖带命中区);donut 图例
  mouse-area 已带 literal index 事件(`.Hover(0..3)`)。
- 关键文件: `examples/{widgets-gallery,charts-gallery,ui/024-charts}/src/front/components/
  {line,bar,area,donut}_chart.at`(三副本同步);契约 `docs/specs/auto-lang/ui/design/
  chart-components.md`。

## 详细设计

### M0 mouse-area `on_click` 引擎臂(496 ondblclick 同款先例)
- `ui/view.rs:534`: `View::MouseArea` 增 `on_click: Option<M>` 字段;`view.rs:1386`
  递归 `map` 臂同步;
- `aura/extract.rs:67`: onclick 别名解析覆盖 mouse-area 上下文(实施时核验既有别名表
  是否已含 onclick,button 等元素的 onclick 是通用通路);
- `ui_gen/vue.rs:12925`: mouse-area div 事件映射 `onclick → "click"`(生成 `@click`);
  同族断言参考 `vue.rs:23316`(`@dblclick`),新增 `@click` 生成断言;
- `ui/aura_view_builder.rs:7229/7277`: tracked 与 untracked 两条 convert 臂增
  `aura_events_get_base(events, "onclick")` 抽取;
- `ui/iced/renderer.rs:3514/5122`: 两条 lowering 臂接 iced `mouse_area.on_press`;
  事件电路检查段(19747/19900)同步补 `on_click` 臂;
- schema 双源登记: `aura/schema.rs:2304` mouse-area 描述更新 + `schema/aura.at:519`
  events 列表增 `"onclick"`(顺带补登 496 漏写的 `"ondblclick"`);跑
  `cargo test -p auto-lang --test schema_drift` 围栏(Category C)。

### M1 line 高亮 + 转折点
- model: `hoverSeries int = -1`、每系列点坐标表(Init 已有 ysM,落成模型字段点表);
- view: 每系列 path 双分支(hoverSeries==k ? stroke-width 3+opacity 1 : 2+0.85);
  hoverSeries==k 时渲染该系列点圆(`circle (cx: ..., cy: ..., r: "3")`,坐标 Init 预计算);
- 非 hover 系列降 opacity 0.25(downplay)。

### M2 bar 分组高亮
- 同构: hover 分组 → 该组柱 fill 不变 + 描边 stroke 1.5,其余组 opacity 0.3。

### M3 donut 扇区 emphasis
- 悬停扇区(图例触发)→ 该扇区 path 沿中角外移 12px(Init 预计算外移向量
  (cos(mid)*12, sin(mid)*12)),描边 2px;其余不变。

### M4 legend 点击切换显隐
- `visible0..3 bool` 状态 + onclick 翻转;隐藏系列的 segs/图例项条件跳过;
  与 emphasis 正交(隐藏优先于悬停)。

## 测试设计

- M0 引擎臂: `cargo test -p auto-lang --test schema_drift`(Category C) + vue `@click`
  生成断言 + VM 轨点击冒烟;改动面含 `crates/` Rust 源码 → 门禁 Category B
  (`cargo check -p auto-lang` + `cargo t ui` 局部模块);
- plan484 冒烟扩展: 悬停态断言(hoverSeries 切换后高亮样式落图);
- 双端一致性: charts-gallery + 024-charts 目检(用户验收);
- 回归: plan437 e2e + gallery golden(组件改动→基线更新)。

## 验收标准

0. M0: mouse-area `on_click` 双端可用(vue `@click` / VM iced `on_press`),schema 双源
   登记,schema_drift 围栏绿;
1. hover 系列/分组/图例行 → emphasis 高亮生效,其余 downplay,双端一致;
2. line 高亮时转折点圆点浮现;
3. legend 点击切换该系列显隐;
4. 全量门禁绿(cargo t + plan437/plan484 + golden)。

## 执行步骤

- [x] M0 mouse-area on_click 引擎臂(view/extract/vue/aura_view_builder/iced 五处 +
      schema 双源登记 + schema_drift 围栏)
      [✅ 已完成] view.rs 枚举+map 臂/aura_view_builder 两 convert 臂/iced renderer 两 lowering 臂(on_press)+t496_walk 单击计数;extract.rs 核验 onclick 已在通用别名表(无需改);vue base_event_to_dom 已含 onclick(新增 @click 生成断言);schema 双源登记(aura.at events 并补登 496 漏写的 ondblclick);schema_drift 绿+plan498 VM 轨冒烟绿+cargo t ui 780/780(worktree commit 9d44dc36b)
- [x] M1 line 高亮+转折点(Init 预计算点表 + view 双分支)
      [✅ 已完成] line+area(同族):图例 mouse-area 触发 HoverSeries/SeriesOut;转折点 pts0..3 Init 预计算、path0..3 槽位化弃 segs;三分支(3px+1 / 2px+0.85 / 2px+0.25;area fill 0.45/0.25/0.08);哨兵 -1→9(负数字面量 view 比较缺陷,见待澄清#2);plan498 M1 断言 3 绿+plan484/plan492(两锚点随基线更新)全绿;三副本同步(commit 9249f35d4)
- [x] M2 bar 分组高亮
      [✅ 已完成] 几何按(组×系列)拆记录 {d,c,g} 入 segs0..3 槽位列表;.Hover(i) 顺带写 hoverGroup(哨兵 9):该组描边 1.5/其余组 0.3 三分支;plan498 M2 断言绿+plan484/plan492 回归绿;三副本同步
- [x] M3 donut 扇区 emphasis 外移
      [✅ 已完成] Init 预计算中角外移 12px 路径 e0..3;.Hover(i) 顺带 hoverSeries(哨兵 9);悬停扇区白描边 2px;段槽位化(d/e 双形态)弃 segs;plan498 M3 断言绿+plan484/plan492 回归绿;三副本同步
- [x] M4 legend 点击显隐切换(消费 M0 on_click 臂)
      [✅ 已完成] 四图族 mouse-area onclick→.Toggle(k) 翻 visLn/visAr/visBr/visDn0..3;隐藏系列几何跳过+图例项 opacity-40,隐藏优先于悬停;实证 VM 单态架构(P320)同名字段跨组件串扰→字段名图族专属解耦(见待澄清#3);plan498 全 6 测试绿+plan484/plan492 回归绿;三副本同步
- [x] M5 双端验证 + golden 更新
      [✅ 已完成] gallery_vue_golden + docs/components/core.md 再生成(schema 改动联动,docs_gen 绿);VM 轨重建 auto.exe 截图核验(六图卡+图例正常);vue 轨 Playwright 实机断言全 PASS(legend 点击显隐 46→44→46 往返+opacity-40 恰 1+悬停 stroke-width=3);cargo t 3329/3329 绿(vue_capabilities×5/ui_snapshots×3 失败经 master 对拍为既有红,非本计划回归);commit e22e89bcf
- [x] M6 复审与归档准备
      [✅ 已完成] 验收 0-4 对照证据入复审记录;健康检查:新文件 rustfmt 归整、无新告警、无遗留调试输出;发现项两条(负数字面量比较缺陷/VM 单态串扰)已挂待澄清#2#3;scoped 复验(plan498×6/plan484×4/plan492×21/schema_drift)全绿

## 复审记录

### 正式复审(/auto-plan:review,2026-09-01,reviewer: zcode)

**复审基线**:worktree `.worktrees/plan-498-dev`(branch plan-498-dev,7 commits,
9d44dc36b..7c53c8164),diff = 24 files(+1983/-343),与计划 affects 面一致;三副本
(charts-gallery/024-charts/widgets-gallery)4/4 逐字节同步复核通过。

**验收标准逐项重验**(verify, don't trust——全部本会话重跑):

| # | 标准 | 判定 | 证据 |
|---|------|------|------|
| 0 | M0 on_click 双端可用 + schema 双源 + schema_drift 绿 | ✅ pass | view.rs:541/1401、aura_view_builder.rs:7275/7326、renderer.rs:3514/3531(on_press)/5138 五臂代码核读;schema/aura.at:526 events 四项;`test_a2vue_mouse_area_onclick` 绿;`plan498_mouse_area_onclick_arm_lands` 绿;schema_drift 绿(cargo tf 内) |
| 1 | emphasis/downplay 双端一致 | ✅ pass | VM:plan498 line/area/bar/donut 4 测试(svgdoc 属性断言)6/6 绿;vue:Playwright 实机复验(hover Mobile → stroke-width=3 落 DOM)PASS |
| 2 | line 转折点浮现 | ✅ pass | `plan498_line_emphasis_and_turning_points`:r=3 圆圈随 HoverSeries 浮现/随 SeriesOut 消失断言绿 |
| 3 | legend 点击切换显隐 | ✅ pass | VM:`plan498_legend_toggle_visibility`(几何跳过+opacity-40+复原)绿;vue:Playwright 复验 46→44→46 路径往返+opacity-40 恰 1 项 PASS |
| 4 | 全量门禁绿 | ✅ pass | **cargo tf 3330/3330**(含 1M churn 档,本计划唯一全量点);cargo tv 3468/3470,2 失败与 master 对拍完全一致(cookbook cb_asynchronous_channel/cb_devtools_log_error 既有红);plan437 2/2、plan484 4/4、plan492 21/21、gallery_golden 1/1、docs_gen 4/4 复验绿 |

**遗漏/延后/workaround 猎查**:

- 遗漏:无。M0 五处落点 + 事件电路段(t496_walk clk)逐一对码;测试设计四项全数落地;
  diff 中零 TODO/FIXME/HACK;三副本同步复核。
- 延后:axisPointer → v2 为计划内既定(非执行期私拆);VM 光标样式 → v2 为计划内待澄清#1。
- Workaround(两处,均已挂账非静默):
  1. **P498-1 候选**:view 条件对负数 int 字面量比较恒假 → 哨兵 9 规避(待澄清#2)。
  2. **P498-2 候选**:VM 单态架构(P320)同名字段跨组件串扰 → 图族专属字段名解耦;
     同族多实例仍联动,与 vue 轨存在行为差异(待澄清#3)。

**复审附加发现**(非阻断,merge/后续关注):

1. 规约文本分歧:chart-components.md:102 仍写 `hoverIdx int = -1`(484 遗留)——实现
   为 hov 族字段+哨兵 9;merge 沉淀交互设计节时须以实现为准(new_spec_components
   已注明"勿沿用 -1 旧例")。
2. VM 实机像素级点击未执行(快照层折叠 mouse-area,MCP press 无法定位):电路覆盖 =
   单测(view 树臂+convert_view_messages 存活+handler 派发)+ renderer on_press lowering
   代码核读 + 496 同族原语(iced mouse_area on_double_click)生产已证;残余风险低。
3. 双端目检(用户验收)按测试设计归用户,尚待执行。
4. vue_capabilities×5 / ui_snapshots×3 既有红(master 同集,不在 tf 门禁面内)与本案无关。

**结论**:验收 0-4 全 pass,无未批准延后;两处 workaround 已记录挂账。
→ **status: reviewed**,可入 `/auto-plan:merge`。

---

### M6 执行侧自检准备笔记(2026-08-31,execution 会话)

### 验收标准对照(execution 侧证据)

- **0. M0 on_click 双端可用 + schema 双源**:✅ view/aura_view_builder(两 convert 臂)/iced
  renderer(on_press lowering + convert_view_messages 显式臂)/map 臂全落地;vue 经通用
  base_event_to_dom(既有 onclick→click)生成 @click,新增生成断言
  `test_a2vue_mouse_area_onclick` 绿;schema 双源登记(schema.rs 描述 + aura.at events
  增 onclick 并补登 496 漏写的 ondblclick);schema_drift 绿;VM 轨冒烟
  `plan498_mouse_area_onclick_arm_lands` 绿(工作树重建 auto.exe 后另经实机截图核验)。
- **1. emphasis/downplay 双端一致**:✅ line/area(bar/donut 见下)——VM 轨
  `plan498_line_emphasis_and_turning_points` / `plan498_area_emphasis`(svgdoc 内联
  属性断言);vue 轨 Playwright 实机(hover Mobile → stroke-width=3 落 DOM)。
- **2. line 转折点浮现**:✅ hoverSeries==k 时 r=3 圆圈浮现/离焦消失,单测断言往返。
- **3. legend 点击切换显隐**:✅ VM 轨 `plan498_legend_toggle_visibility`(M0 电路 →
  Toggle → 几何跳过 + opacity-40 + 复原);vue 轨 Playwright(46→44→46 路径往返,
  opacity-40 恰 1 项)。
- **4. 全量门禁**:✅ cargo t 3329/3329 绿;plan484 4/4、plan492 21/21、plan437
  child-init 2/2、gallery golden + docs_gen 再生成后绿;vue_capabilities×5 +
  ui_snapshots×3 失败经 master 对拍确认为**既有红**(与本案无关,cargo tf 全量门禁
  归 /auto-plan:review)。

### 执行侧发现(供复审/merge 关注)

1. **负数字面量 view 比较缺陷**(待澄清#2):哨兵 9 规避,组件注释 + KNOWN-DEBT 候选。
2. **VM 单态架构字段名串扰**(待澄清#3):图族专属字段名(hovLn/hovAr/hovBr/hovDn +
   visLn/visAr/visBr/visDn)解耦;同族多实例仍联动(vue 无此现象),跨轨差异挂账。
3. **基线更新**:gallery_vue_golden(四图组件 + kitchen-sink 消费页)、
   docs/components/core.md(mouse-area 条目)、plan492 两锚点(c2 msg 前缀匹配、
   金丝雀改健康态相对比较)——均为本案改动的正当联动。
4. 段记录槽位化(line path0..3/pts0..3、area a/l、donut d/e、bar segs0..3)弃 .segs
   列表——M1 详细设计的"落成模型字段点表"实现形态;≤4 系列契约与图例槽位既有纪律一致。

### 工作树提交

M0 9d44dc36b / M1 9249f35d4 / M2 25907608e / M3 1cd094fe0 / M4 c7cab80d8 / M5 e22e89bcf /
M6 7c53c8164——branch plan-498-dev(7 commits)。

## 待澄清事项

1. VM 轨 mouse-area 的 hover 是否需要 pointer 光标样式(iced 无原生 cursor 桥)?暂不,
   列 v2。
2. 【M1 实证引擎缺陷,KNOWN-DEBT 候选】view 条件对负数 int 字面量比较恒假:`if .v == -1`
   在初始 model 值 -1 时也走 else(写入侧正常,read_state 确为 Int(-1);疑条件串渲染
   形态导致 rhs 解析偏差)。plan498 以越界哨兵 9 规避(hoverSeries/hoverGroup 无悬停=9);
   偿还方向:eval_condition_with 对负数字面量的 rhs 归一。
3. 【M4 实证架构限制,KNOWN-DEBT 候选】VM 单态架构(Plan 320 单 VM 单根状态)下,
   子组件同名字段经根状态跨组件共享:对 LineChart.Toggle(0) 一次派发,gallery 六个图例
   项同时落 opacity-40(line/area/donut 的 visible0 联动)。组件侧以图族专属字段名
   (hovLn/hovAr/hovBr/hovDn + visLn/visAr/visBr/visDn)解耦四族;同族多实例仍共享
   (两个 line-chart 实例联动)——vue 轨实例隔离无此现象,跨轨行为差异挂账。偿还方向:
   子组件状态按实例隔离(需架构级改动)。

## spec-sync 回写记录（merge,2026-09-01）

- 账本:`.autoos/specs.json` 六段 upsert P498-1..6(reports/goals/architecture/
  designs/tests/reviews,幂等 id);
- module:`docs/specs/auto-lang/ui/overview.md` 组件线补交互态现状;
  `plans.md` 追加 498 行;`design/chart-components.md` 新增「交互态」节
  (emphasis 二态/转折点/legend 显隐/悬停字段命名纪律〔哨兵 9,勿用 -1〕/
  mouse-area 事件面/双端表现差异),废弃 484 hoverIdx -1 旧例;
- 全局:`docs/specs/goals.md` GOAL-007 补 498 引用;`scripts/spec-index.py` 再生;
- 债务:P498-1(负数字面量 view 比较)/P498-2(VM 单态同名字段串扰)入
  KNOWN-DEBT-AND-RISKS.md;
- 折入:master 合并 plan-498-dev(7 commits)+ gallery golden 融合再生成
  (503 stella × 498 交互同存基线),main cargo t 3333/3333 绿。
