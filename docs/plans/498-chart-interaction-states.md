---
plan_id: PLAN-498
status: drafting
feature_name: chart 交互状态机——emphasis 高亮/转折点浮现/legend 点击切换显隐
author: [zcode]
created_at: 2026-08-31
updated_at: 2026-08-31

supersedes_spec_components: []
new_spec_components:
  - "docs/specs/auto-lang/ui/design/chart-components.md: 交互设计节(emphasis 二态模型/hoverIdx 状态机/legend 显隐切换/转折点浮现)"
  - "docs/specs/auto-lang/ui/design/chart-components.md: mouse-area onclick 事件臂(M0 引擎改动登记)"
touched_goals:
  - "GOAL-007: AutoUI 跨端一致——chart 交互双端同源"
affects: [docs/specs/auto-lang/ui, examples/widgets-gallery, examples/charts-gallery, examples/ui/024-charts, crates/auto-lang/src/ui, schema/aura.at]
current_step: 0
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

- [ ] M0 mouse-area on_click 引擎臂(view/extract/vue/aura_view_builder/iced 五处 +
      schema 双源登记 + schema_drift 围栏)
- [ ] M1 line 高亮+转折点(Init 预计算点表 + view 双分支)
- [ ] M2 bar 分组高亮
- [ ] M3 donut 扇区 emphasis 外移
- [ ] M4 legend 点击显隐切换(消费 M0 on_click 臂)
- [ ] M5 双端验证 + golden 更新
- [ ] M6 复审与归档准备

## 复审记录

（复审时填写）

## 待澄清事项

1. VM 轨 mouse-area 的 hover 是否需要 pointer 光标样式(iced 无原生 cursor 桥)?暂不,
   列 v2。
