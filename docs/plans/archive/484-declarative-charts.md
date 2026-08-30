---
plan_id: PLAN-484
status: archived               # drafting → executing → execution_done → reviewed → archived（终态）
feature_name: 声明式 chart 原语（bar-chart 裸名 + 轴/图例自动化）
author: [zcode]
created_at: 2026-08-29
updated_at: 2026-08-29

# /auto-plan:review 结束时填写（2026-08-30 复审通过）：
supersedes_spec_components:
  - "docs/specs/auto-lang/ui/design/chart-components.md: chart 组件族契约 v1（Auto* 前缀四组件、无轴/图例/tooltip、bar/line 单变体）——484 升级为契约 v2"
  - "schema/aura.at + aura/schema.rs: areachart/barchart/linechart/donutchart/chart/chartlegend/charttooltip 七元素（shadcn-vue 发射路线）——484 全部退役删除"
  - "ui_gen/widget/registry.rs: AreaChart/BarChart/LineChart/DonutChart/Chart/ChartTooltip/ChartLegend 七注册——484 退役,裸名让位 official 包"
  - "auto-man/src/vue.rs: chart/chart-area/-bar/-line/-donut 脚手架 + @unovis 可选依赖组——484 退役,资产目录删除"
new_spec_components:
  - "docs/specs/auto-lang/ui/design/chart-components.md: 契约 v2（裸名原语 bar-chart 等/props v2 axis-grid-legend-tooltip-labels-curve-type/nice-ticks 规格与退化域/hover tooltip x 竖带命中与锚点/stacked+monotone/包组件 Init codegen 三坑已知项）"
  - "mouse-area 基础 widget: registry MouseArea(Overlay) + schema mouse-area 元素(iced full/web component) + iced mouse_area 事件转发与 vue div @mouseenter 映射"
touched_goals:
  - "GOAL-007: AutoUI 跨端一致——chart 族同一 .at 源 vue/vm 双端同源渲染（SVG v1 直通）,裸名原语收敛为单一实现"
  - "GOAL-010: 示例应用轨道——charts-gallery 重建（Auto 底座六图卡）+ 024-charts 组件化迁移（806→226 行）"
  - "GOAL-008: App 生成——chart 族经 official 组件包（Plan 435 P4 机制）进入生成面,unovis 外部依赖清零"

affects: [docs/specs/auto-lang/ui, examples/widgets-gallery, examples/ui/024-charts, examples/charts-gallery]
current_step: 6
total_steps: 6
---

# [PLAN-484] 声明式 chart 原语（bar-chart 裸名 + 轴/图例自动化）

## 变更摘要

用户需求（2026-08-29）：chart 的使用体验对齐 ECharts / chart.js——**声明原语（如 `bar-chart`）+ 给定参数与数据，即可自动成图**，不要求用户手写几何。

现状（Plan 437/445 已交付）与差距：

| 维度 | 现状 | 与 ECharts 式体验的差距 |
|---|---|---|
| 使用形态 | `auto-donut-chart (data: .traffic, index: "l", field: "v") {}` —— 已是声明式 | ✅ 基本达标 |
| 命名 | `auto-*` 前缀（§0.6.E-1 政策，避开内置 shadcn 注册名） | ❌ 裸名 `bar-chart` 被内置 registry 的 shadcn chart 族占用（vue 轨 shadcn 发射 / VM 轨 fallback "not implemented"） |
| 多系列 | line/bar/area 已支持 `fields: [...]` + `colors: [...]` | ✅ |
| 坐标轴 | 仅 bar/area 输出 x 轴 `labels str`；y 轴刻度/网格线需用户手搭（024-charts 手写 5 个固定刻度） | ❌ 无 nice-ticks 自动刻度、无网格线 |
| 图例 | 仅 donut 输出 `legend str`；line/bar/area 无 | ❌ |
| 动态数据 | VM 轨"props 播种 → 每渲染帧重放 Init（纯派生幂等）"已落地（437 Phase 2①）；vue 轨 SFC 生成 | ⚠️ 流式数据 + 组件消费的组合未有旗舰示例验证（024-charts 流式走内联几何） |
| 交互（tooltip/crosshair） | shadcn 路线 vue 端有 ChartTooltip；VM 轨 svgdoc 为静态图 | ⚠️ hover tooltip 可由 Stack 命中区叠层实现（本计划纳入）；跟随光标 crosshair 后置（见待澄清 #2） |
| 图类型 | line/bar/area/donut 四类 | bar 缺 stacked、line/area 缺 monotone 平滑——受旧 gallery 形状对拍约束纳入（见待澄清 #4 修订）；pie/scatter 后置 |

结论：**架构无需变更**（沿 437 裁决"引擎给笔，Auto 持笔"，不引引擎内置 chart 控件、不引第三方库）。本计划做三件事：①裸名接线与 shadcn chart 族退役；②轴/图例自动化补进组件族；③024-charts 迁移为组件消费，作为"声明 + 数据 → 自动成图（含流式）"的旗舰验收。

## 目标

**核心定位（用户原话归纳，2026-08-29）**：把"怎么算出 chart"的算法**固化在官方组件内**——用户只做三件事：选类型（原语标签）、配参数（props）、给数据（record 列表）。算法本身（scale/几何/刻度/图例）是 official 包的版本化资产，随包演进，用户代码不动；裸 SVG 原语保留为非常规图表的逃生舱（024-charts 裸写形态继续合法）。

1. 用户写 `bar-chart (data: .monthly, index: "m", fields: ["desktop","mobile"], colors: [...]) {}` 即得完整柱状图（含轴刻度、网格线、图例），vue/VM 双端同源渲染。
2. 官方 chart 原语收敛为**一套实现**（Auto 组件族）：裸名 = 唯一官方名，shadcn chart 族 registry 注册退役（supersede）。
3. 动态数据（流式滑窗）下组件自动重算几何，双端行为一致。

## 架构方案

不新增渲染端机制。chart 原语 = official 组件包（Plan 435 P4 `namespace: "auto"` 机制）中的 Auto 组件；渲染端继续 SVG v1 直通（vue 直通臂 / VM svgdoc 通道）。v2 canvas 桥接维持后置（437 设计决议不变）。

### 裸名方案（核心决议，待用户确认）

official 包组件标签 = `namespace` 前缀 + kebab 名（`AutoDonutChart` → `auto-donut-chart`）。要裸名 `bar-chart`，须让内置 registry 的 shadcn chart 族退役：

- **退役** registry.rs 中 `AreaChart/BarChart/LineChart/DonutChart/Chart` 五个注册（`registry.rs:1994-2018` 一带，Data 类）及其 shadcn props overlay；
- official 包四组件更名去 `Auto` 前缀（`BarChart` → 标签 `bar-chart`）或注册裸名别名；
- `render_support.rs:254` 的 `chart|canvas` fallback 保留（`canvas` 仍占位；`chart` 裸 tag 依旧无引擎实现——组件名解析在 registry/包层，不落到该 fallback）；
- **代价与处置**：shadcn 路线（vue 端 unovis 交互 tooltip）随之退役；`examples/charts-gallery`（`backend: ["vue"]`，shadcn 组合用法先例）迁移为裸名组件消费，或标注 legacy 保留。推荐前者（示例矩阵不留双路线）。

### 轴/图例自动化（组件内 Auto 实现）

- **nice-ticks**：plotters 风格 nice number 算法用 Auto 移植（约 30–60 行），y 轴自动产出 4–5 档刻度值 + 对应网格线路径；x 轴标签抽样（窗口长时等距取 N 个）。
- **发射形态**：沿用 024-charts 手搭形态——刻度文字走 DSL `text` 列（SVG `<text>` 不支持的约束不变，437 §0.6.A），网格线/轴线走 svg `path`。组件 view 内置 y 轴列 + svg，整体仍是一个组件标签。
- **props v2 契约**（向后兼容现有四 props）：
  - `axis: str = "auto"`（`"auto" | "none"`——024-charts 自管刻度时关闭）
  - `grid: bool = true`（水平网格线）
  - `legend: bool = true` + `labels: List<str>`（系列显示名，缺省回落 fields 名）
  - 既有 `data/index/fields/colors` 不变；donut 的 `field` 与 `fields` 并存兼容（donut 单系列）

### 动态数据契约

VM 轨已有"props 播种 → Init 重放（纯派生幂等）"；本计划不新增机制，只验证 vue 轨等价性：SFC 生成需保证父级数据变化触发几何重算（watch/computed 或重渲染重放，与 437 vue 生成器现状对齐）。验收载体 = 024-charts 流式模式改组件消费后双端对拍。

## 需求分析与背景调查

- 来源：用户判断（2026-08-29）——希望 ECharts/chart.js 式声明原语体验；此前 agent 误判"charts 为 vue 轨专属、VM 分派表无 chart 组件"引出本调查。
- 前置交付：Plan 437（官方 chart 组件族 + VM 子组件生命周期七项补全）、Plan 445（024-charts 双端实机 + 流式）、Plan 442 A4（SVG 直通 vue 臂 + VM svgdoc 通道）、Plan 435（组件统一声明 / official 包机制）。
- 硬约束（437 已裁决，本计划沿用）：不引 chart 依赖（plotters-iced 仅支持 iced 0.13 等调研结论）；SVG `<text>` 不支持，轴文字走 DSL text；float 中间量显式声明等 Auto 纪律（§0.6.D）。
- 关键文件：
  - 组件源：`examples/widgets-gallery/src/front/components/{line,bar,area,donut}_chart.at`
  - 包清单：`examples/widgets-gallery/src/front/components/package.at`（namespace: "auto"）
  - 内置注册：`crates/auto-lang/src/ui_gen/widget/registry.rs:1994-2018`（shadcn chart 族五注册）
  - VM fallback：`crates/auto-lang/src/ui/render_support.rs:254`
  - 旗舰示例：`examples/ui/024-charts/src/front/app.at`（806 行，几何内联三份——Init/Reset/.Tick）
  - 契约文档：`docs/specs/auto-lang/ui/design/chart-components.md`（437 新建，本计划修订）

## 详细设计

### M0 决议与契约落库
- 用户确认裸名方案与 charts-gallery 处置（见待澄清事项）。
- 修订 `docs/specs/auto-lang/ui/design/chart-components.md`：props v2 契约表（含默认值语义）、裸名映射表、nice-ticks 行为规格（档数、空数据、单点、全零数据）、退役清单（shadcn 五注册 + ChartTooltip/ChartLegend/ChartCrosshair 资产去留）。

### M1 组件族 props v2
- 四组件统一升级：nice-ticks + 网格线 + 图例 + `axis/grid/legend/labels` props；`labels str`（bar/area 既有）并入 axis 输出。
- **类型扩展（旧 gallery 形状对拍所必需）**：bar 增 `type: "grouped"|"stacked"`（堆叠 = y 域取累计和，系列底边为前系列之和）；line/area 增 `curve: "linear"|"monotone"`（d3 curveMonotoneX 单调三次插值，Hermite→贝塞尔，不过冲数据点）。
- **hover tooltip（决议 2026-08-29 纳入）**：Init 算几何时同步产出每数据点/扇区/柱的包围盒，svg 上叠 Stack + 透明 mouse_area 命中区（registry 已有 Stack/Tooltip 组件族；iced renderer 已有 mouse_area enter/exit 底座，vue gen 已支持 onmouseenter）；hover 显示 DSL 自绘 tooltip（数值 + 系列名），**锚点位置 Init 时算好固定**（柱顶/扇区中心/点上方），事件仅 enter/leave 低频——不做跟随光标。props `tooltip: bool = true`。视觉规格对齐 shadcn 版 ChartSingleTooltip 的内容形态（色点 + 标签 + 数值）。**命中区形态（业界对拍校准，2026-08-29）**：line/area 用"每 x 索引一条全高竖带"（非逐点小圆——大命中目标且与 shadcn ChartSingleTooltip 的 index 语义一致：单 tooltip 出该 x 全系列值），bar/扇区用图元自身包围盒；命中区数量 = 索引数。
- 组件保持"纯派生幂等"纪律：全部输出由 props 在 Init 一次算出，无内部可变状态依赖时序。
- 快速验证：`cargo check -p auto-lang`；widgets-gallery `auto run` + `auto run -r vm` 四路由目检。

### M2 裸名接线与 shadcn chart 族退役
- registry.rs 删除/退役五个 chart 注册及 props overlay（supersede 记录）；
- official 包四组件更名（`BarChart` 等）或裸名别名机制——取 M0 决议结论；全仓 `auto-line-chart`/`auto-bar-chart`/`auto-area-chart`/`auto-donut-chart` 用法迁移（widgets-gallery 四页 + 437 测试钉）；
- `plan437_child_init_tests` 等引用旧名的测试同步。

### M3 charts-gallery 重建（声明式 chart 形式主验证载体，决议 2026-08-29）
- 用户决议：**新建 charts-gallery**——展示的 chart 形状与旧 shadcn 版一致（area/bar/donut/line 及其数据编辑交互形态），但底座从 shadcn-vue + Unovis 换成 Auto 组件族（裸名 `bar-chart` 等）；
- 机制澄清（不改设计，仅记档）：vue 端组件翻译为 TS/SFC（437 生成器现状）；VM 端**无逐图 Rust 实现**——Auto 组件在 VM 执行产出 SVG 子树走 svgdoc，双端零额外实现；
- 载体处置：建议原地替换 `examples/charts-gallery`（名字与端口保留，pac 改 `backend: ["vue","vm"]` 双端），shadcn 版内容随之退役；旧 shadcn 资产（`crates/auto-man/assets/shadcn-ui/chart*`）去留随 M2 一并定；
- 定位与另两载体分工：charts-gallery = 形状目录 + 应用级组合场景；widgets-gallery 四页 = 组件文档/props 说明书；024-charts = 流式动态数据旗舰。

### M4 024-charts 组件化迁移（流式旗舰验收载体）
- `app.at` 退役三份内联几何（Init/Reset/.Tick 各一份），改为四组件消费；流式数据仍由父级滑窗维护（`monthly` 记录列表），几何重算交给组件；
- 面积缩减预期：806 行 → 约 400 行级（数据面板与类型切换逻辑保留）。

### M5 双端验证与 golden
- autoui-verifier 双端一致性：widgets-gallery 四路由 + 重建后 charts-gallery（双端目检 + 几何断言）+ 024-charts（静态 + 流式各 30 ticks，路径串确定性对拍）；
- golden 三件套更新（donut_legend 等既有基线 + 新增 axis/legend 断言）；
- `cargo t`（Category B 局部改动）；触及 docs 生成器时按门禁补 `cargo test -p auto-lang --test docs_gen`。

### M6 复审与沉淀
- 逐项验收核对、spec-impact 元数据、`/auto-plan:review` → merge。

## 测试设计

- **既有回归**：`plan437_child_init_tests`（改名后）、`vue_capabilities`、schema_drift（若契约表落库触发）、gallery_golden。
- **新增断言**：
  - nice-ticks 单元级：组件 Init 输出的刻度串对固定输入确定（在 widgets-gallery 双端对拍中覆盖；组件逻辑在 Auto 层，VM 侧经 svgdoc 几何断言，vue 侧经渲染产物 marker 断言——同 437 验收形态）；
  - 流式组件消费：024-charts Play 30 ticks 后 VTree 路径串与 vue 端产物对拍（autoui-verifier 脚本）。
- **边界用例**：空数据列表、单数据点、全零系列（nice-ticks 退化域）、`fields` 长度 > `colors` 长度（沿用 donut 末色复用政策）。

## 验收标准

1. `bar-chart (data: .monthly, index: "m", fields: ["desktop","mobile"], colors: [...]) {}` 一个标签在 vue 与 VM 双端产出含轴刻度、网格线、图例的完整柱状图（目检 + golden 断言）。
2. 024-charts 流式模式（Play 30 ticks）组件自动重算，双端路径串一致性对拍通过；`app.at` 不再含手写几何。
3. registry 中 shadcn chart 族五注册退役，全仓无 `auto-*-chart` 旧名残留（grep 为零）；`cargo t` 绿。
4. `docs/specs/auto-lang/ui/design/chart-components.md` 与实现一致（props v2 + 裸名 + nice-ticks 规格）。
5. 重建后 charts-gallery 与旧版 chart 形状一致（area/bar/donut/line），pac 声明双端，`auto run` 与 `auto run -r vm` 均可跑且视觉一致（golden 断言），全仓不再依赖 shadcn chart 底座；Auto 实现效果与 shadcn 版基本相当（新旧对拍，用户质量基线）。
6. hover 数据点/扇区/柱时双端均显示 tooltip（数值 + 系列名），位置锚定在目标元素附近；无跟随光标行为（明确不在本计划）。

## 执行步骤

（原子任务在 M0 决议后细化展开；每步完成后追加 [✅ 已完成] 一行证据）

- [x] M0 契约与命名决议（含用户确认剩余待澄清项），修订 chart-components.md [✅ 已完成] 契约 v2 全量落库（裸名映射/props v2 表/nice-ticks 规格/退化域/tooltip 命中区模型含 x 竖带与 donut 图例命中/stacked+monotone 算法规格/shadcn 五注册+资产退役清单），commit 6af3d8bc9
- [x] M1 四组件 props v2（nice-ticks/网格/图例/axis props） [✅ 已完成] MouseArea 基础 widget（registry/view/builder/renderer/snapshot/vnode 六层接线 + 单测）；四组件 v2（nice-ticks 1-2-5 阶梯/固定 y 位网格/图例/labels/≤8 抽样/bar stacked 前缀和/line+area monotone 单调三次插值/hover 竖带+锚点 tooltip）；plan437 e2e 四路由绿（gallery_chart_components_render_geometry）+ mouse_area 单测绿；commit c5f8ea3d4。实证 codegen 坑（已记档 chart-components.md 与组件头注）：包组件 Init 内 prop 字符串比较破坏 codegen→双算双存 view 选边；f-string 含字面量 [] 必须用 {} 插值；条件引用局部变量紧邻使用点声明；带参 msg 声明破坏包子组件编译→handler 裸挂；三大 codegen 坑已正式记档 KNOWN-DEBT-AND-RISKS.md（🟡 三行，含复现路径/绕开/根治线索/回归锚）
- [x] M2 裸名接线 + shadcn chart 族退役 + 全仓改名迁移 [✅ 已完成] registry 七注册/schema 七元素/auto-man 五脚手架+unovis 依赖组全删（-1330 行）；四组件裸名化（LineChart→bar-chart 等折叠解析）+widgets-gallery 页面/app.at/SPEC.md 迁移，全仓 auto-*-chart 零残留；退役路线单测 4 删 1 改（负断言存档）；plan437 e2e 绿；schema_drift/docs_gen/auto-man 全绿；test_charts_gallery_compiles 预期红（M3 重建后转绿并重写断言）；commit 0e0df44f7
- [x] M3 charts-gallery 重建（同形状、Auto 底座、双端） [✅ 已完成] 六图卡（area 四系列/bar grouped/bar stacked/line monotone/donut/area 自定色）换 official 包裸名原语，pac 升双端（render vue + port 4039）；test_charts_gallery_compiles 重写为 Auto 路线断言（包折叠 SFC 引用 + unovis/CurveType 负断言）绿；schema_drift 栅栏绿（mouse-area 入册三表同步、chart 元素退役、kitchen-sink/core.md 再生、DOC_EXCLUDE 折叠形式注记）；commit 7f28b7906
- [x] M4 024-charts 组件化迁移（内联几何退役） [✅ 已完成] app.at 806→226 行，三份内联几何退役改四类裸名原语消费；流式滑窗（.Tick 追点/滑窗/Reset 不变式）语义不变，几何重算交组件 Init 重放；plan484_chart_component_tests 双示例回归入库（组件冒烟 + gallery 裸名渲染 + 遗留几何负断言）；全量 cargo t 3270/3270 绿；commit 6a884149a
- [x] M5 双端一致性验证 + golden 更新 [✅ 已完成] gallery_golden vue 全项目基线复核更新（差异面 = chart 组件/页/kitchen-sink，其余 80 文件 hash 不变）后绿；VM 轨 plan437 四路由 + plan484 双示例 in-process 几何断言绿；ui-iced 全套 --no-fail-fast 复跑（唯一红 = 存量 plan050，master 同红已归属）
- [x] M6 复审（逐项验收 + 遗漏扫描）与归档准备 [✅ 已完成] 六里程碑全勾;scoped 门禁复绿（cargo check ui-iced、plan437/plan484/gallery_golden/schema_drift/docs_gen/auto-man 238 测试）;ui-iced 全套 --no-fail-fast 4040 跑 4037 绿,3 红（plan050×2/code_editor_natives）均 master 存量基线（master 同跑 5 红,子集归属在案）;状态翻转 execution_done,待 /auto-plan:review

## 复审记录

**reviewer**: zcode（/auto-plan:review）· **时间**: 2026-08-30 · **结论**: ✅ PASS → `reviewed`

| 验收标准 | 判定 | 证据 |
|---|---|---|
| 1. bar-chart 裸名声明即得完整柱状图,双端同源 | PASS | widgets-gallery bar-chart 页 vue golden 编译产物含包组件 SFC 引用（gallery_vue_golden 绿）;VM 侧 plan437 e2e marker（`h19` 柱宽 + `8000` 刻度）落图 |
| 2. 024-charts 流式组件自动重算,双端一致;app.at 无手写几何 | PASS | app.at 226 行零手写 SVG（`grep svgdoc` 仅组件产物）;plan484_024_charts_component_smoke（几何落图）+ plan484_024_charts_streaming_recompute（.Tick×30 后 t29 标签 + 重算 path,复审新增）双绿 |
| 3. registry 七注册退役,全仓 auto-*-chart 零残留,cargo t 绿 | PASS | grep auto-*-chart / Auto*Chart 于 examples/crates 均 0;cargo t 3270/3270;cargo tf 3271/3271 |
| 4. chart-components.md 与实现一致 | PASS | M0 契约 v2 逐节对照实现（props v2/裸名/nice-ticks 5 档/退化域/命中区形态） |
| 5. charts-gallery 重建,双端可跑,视觉一致,无 shadcn 依赖 | PASS | test_charts_gallery_compiles（Auto SFC 引用 + unovis/CurveType 负断言）;plan484_charts_gallery_bare_names_render（VM 侧 A/M/h 几何落图）;gallery_vue_golden 更新后绿 |

**遗漏/延后/workaround 扫描**:
- **遗漏（复审补修）**: vue.rs chart 发射死代码（emit_chart_prop/emit_chart_family_attrs/emit_curve_type_prop + CurveType 导入门 + match 臂）M2 时留了早退尸体,与契约"vue.rs 特判臂退役"不符——复审补删（commit 989fc83e9,删后全门禁复绿）。
- **workaround（已记档 DEBT,用户批准）**: 包组件 Init 内 prop 字符串比较/f-string `${}`+字面量 `[]`/带参 msg 声明 三大 codegen 坑 → KNOWN-DEBT-AND-RISKS.md 🟡 三行（含复现/绕开/根治线索/回归锚）。
- **延后（用户批准）**: pie/水平条形/散点图类型、跟随光标 crosshair（v2 canvas 阶段）、hover tooltip 实机目检——各自独立后续计划/复审动作。

**基线归属**: ui-iced 全套 3 红（plan050×2/code_editor_natives）均 master 存量（master 同跑 5 红,本计划子集）。

## 待澄清事项

1. **裸名 vs `auto-` 前缀**（核心决议）：是否按"退役 shadcn chart 族注册、裸名让位 Auto 组件"执行？代价是 shadcn 路线 vue 端交互 tooltip 一并退役（VM 轨本就无此能力）。备选：维持 `auto-*` 命名不动（零改动，但不满足裸名诉求）。
2. ~~交互 tooltip 的 v1 边界~~ **已决议（2026-08-29 用户质询后核实）**：hover tooltip **纳入本计划**（Stack 命中区叠层 + mouse_area enter/leave + 锚点固定 DSL tooltip；Stack/Tooltip 组件族与 iced mouse_area、vue onmouseenter 三件底座均已核实在库）；仅**跟随光标 crosshair** 后置（高频 mousemove 流进 VM 的性能设计需对照 Plan 386 RenderQueue 另立计划）。
3. ~~charts-gallery 处置~~ **已决议（2026-08-29 用户）**：新建 charts-gallery——chart 形状与旧版一致，底座换 Auto 组件族，弃 shadcn 外部依赖；建议原地替换 `examples/charts-gallery` 并升双端（见 M3）。**质量基线（用户追加）**：Auto 实现效果须与 shadcn 版基本相当，以新旧 charts-gallery 视觉对拍为验收口径。
4. ~~图类型扩展~~ **决议修订（2026-08-29 核实旧 gallery 后拆分）**：旧 charts-gallery 实际形状 = area(4系列)/bar-grouped/bar-**stacked**/line-**curve-type:monotone**/donut/area-自定色。受质量基线（形状一致）约束：**stacked + monotone 平滑曲线纳入本计划 M1**（grouped 即现有 AutoBarChart 默认形态；新增 props 沿用 shadcn 旧名：bar `type: "grouped"|"stacked"`、line/area `curve: "linear"|"monotone"`）；pie/水平条形/散点旧 gallery 未用，后置独立计划。
