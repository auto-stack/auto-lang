---
plan_id: PLAN-502
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: diagram Phase 1——flow-diagram v1(分层布局 + SVG 渲染 + hover 交互)
author: [zcode]
created_at: 2026-08-31
updated_at: 2026-08-31

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - "docs/specs/auto-lang/ui/design/diagram-components.md: flow-diagram 组件契约节(数据轨 props/布局/hover 交互)"
touched_goals:
  - "GOAL-007: AutoUI 跨端一致——diagram 渲染双端同源"

affects: [docs/specs/auto-lang/ui, docs/design/autoui/diagram-components.md, examples/widgets-gallery, examples/ui]
current_step: 0
total_steps: 7
---

# [PLAN-502] diagram Phase 1——flow-diagram v1(分层布局 + SVG 渲染 + hover 交互)

## 变更摘要

依据设计文档 [diagram-components.md](../design/autoui/diagram-components.md) §8 Phase 1
立项：交付 diagram 家族首个组件 **flow-diagram**——Sugiyama-lite 分层布局(纯 Auto)
+ v1 SVG 渲染(484 charts 同通路) + hover tooltip/emphasis(498 状态机模板)。

主体**零引擎改动**；唯一待定引擎改动 = svg `text` 直通(M1 标签发射对照验证若胜出,
落一处小引擎改动,单列步骤)。显式排除: DSL 糖(Phase 3)、节点 select(Phase 1.5,依赖
498 M0 on_click 臂)、边 hover/节点拖拽/pan-zoom(Phase 4/5)、其余图类型(Phase 2a/2b)。

## 目标

1. **M1 标签发射对照**(Phase 1 首个验证项,设计文档 §6.1/§10.1): svg `text` 直通
   (vue 轨平凡 / VM 轨 svgdoc 经 resvg 原生支持) vs DSL text 绝对定位 overlay
   (VM 轨动态 arbitrary 值支持面待核验)——产出决策记录,胜者成为全 diagram 家族
   的标签机制;
2. flow-diagram 用户态组件: 数据轨 props `nodes`/`edges`(record schema 见设计文档
   §5,含 `x/y` 钉住位预留,Phase 1 恒 -1)、`direction: "td"`;
3. **Sugiyama-lite 分层布局**(纯 Auto): rank(最长路径分层) → order(重心法 2–4 轮
   降交叉) → coord(层内等距 + 父居中一趟);`direction td/lr` 经转置;
4. 图元发射 + 边路由 v1: Init 把布局+几何写进 render 段记录,view `for seg in .segs`
   循环发射 svg `path/rect/ellipse`;边 = 节点 bbox 边界交点直线段;箭头/head/tail
   字形(arrow 三角形、diamond/circle/cf-* 小多边形)Init 手算(donut 弧先例);
5. hover 交互(498 三段式模板): `hoverNode str` 状态 + emphasis 高亮(平面二态)
   + 锚定 tooltip;命中 = 节点 bbox 绝对定位 mouse-area 兄弟层(svg 兄弟层不经
   svgdoc 序列化,484 增量先例);
6. 示例页 + 双端(vue/vm)目检一致。

## 架构方案

- **纯用户态组件**(对齐 charts 484: "引擎给笔,Auto 持笔",几何 = Auto 代码,Init 纯
  派生,ADR-19 重播种链路复用);组件落位官方组件包机制,消费方式
  `use { package: official from "components" }`;
- **零引擎改动起步**: 渲染走既有 svg 子树直通(vue)/ svgdoc 序列化(VM)双通路;
  hover 命中走 mouse-area 兄弟层;均无需引擎新臂;
- M1 若 svg `text` 直通胜出 → 一处小引擎改动(vue.rs svg 直通集 + aura.at/svg shape
  tag 集 + aura_view_builder 序列化臂),门禁随之升 Category B/C;
- 与 498 的关系: hover 状态机直接抄 498 三段式模板(Init 纯派生 + 最小状态写入 +
  view 状态投影);select 交互依赖 498 M0 的 on_click 臂,归 Phase 1.5 不在本计划。

## 需求分析与背景调查

- 设计文档: docs/design/autoui/diagram-components.md(2026-08-31 定稿)——§4 DSL 语法
  (本计划只用数据轨)、§5 record schema、§6.1 v1 SVG 轨与布局算法、§7.2/§7.3 交互与
  命中区模型、§10.1 标签发射风险;
- charts 484 先例: 组件化机制(`ui_gen/widget/component_registry.rs`,Builtin > Local
  > Package)、Init 几何派生(1-2-5 nice step / path d 串)、svgdoc 双端通路、
  `vm/tests_chart_geometry.rs` 端到端几何对拍;
- 已知约束: svg 无 `<text>`(M1 对照验证的由来);svgdoc 静态 → VM 轨元素级事件不可用
  (mouse-area 兄弟层命中);f-string px 类 492 定案后直写合法(overlay 轨先例 =
  donut tooltip);
- 静态糖(`node/edge/group` DSL 词汇)不在本计划——Phase 3 parser 扩展;本计划只交付
  数据轨 + 组件,静态糖到位前示例以数据轨书写。

## 详细设计

### M1 标签发射对照验证(决策点)
- svg `text` 直通原型: vue 轨 svg 直通标签集加 `text`(vue.rs `map_tag` 同族,平凡);
  VM 轨 svgdoc 序列化加 `text` 元素(aura_view_builder serialize_svg_element 同族,
  resvg 栅格化);双端渲染冒烟;
- overlay 对照: 动态 `left-[${x}px]` arbitrary 值在 VM/iced 轨解析支持面核验
  (donut tooltip 仅有固定值 `top-[20px]` 先例);
- 产出: 决策记录(回填设计文档 §6.1 标签发射条),胜者进 M5。

### M2 flow-diagram 组件骨架
- props: `nodes`/`edges` List(record schema §5: id/label/shape/group/x/y…)、
  `direction: str = "td"`;
- Init 管线骨架: props 解析 → 布局占位(等距网格) → render 段记录;view svg 发射
  冒烟(矩形节点 + 直线边),双端可见。

### M3 Sugiyama-lite 布局核
- rank/order/coord 三趟 + direction 转置;全 float 纪律(中间量显式 float);
- 几何对拍测试(参照 `vm/tests_chart_geometry.rs`): 固定小图 → 断言层位/交叉数/
  坐标确定性。

### M4 边路由 + 箭头字形
- bbox 边界交点直线段;head/tail 字形(arrow/diamond/circle/cf-one/cf-many)Init
  预计算端点角度与小多边形 path。

### M5 标签 + hover 交互
- 标签按 M1 决策落地(节点/边标签);
- `hoverNode str = ""` 状态机 + emphasis 双分支(线宽/opacity)+ 锚定 tooltip;
  节点 bbox 绝对定位 mouse-area 兄弟层命中。

### M6 示例与双端验证
- 示例页(widgets-gallery 新页或 examples/ui/ 独立示例,对齐 024-charts 结构);
  vue/vm 双端目检;gallery golden 基线。

### M7 复审与归档准备

## 测试设计

- 布局几何端到端(参照 `vm/tests_chart_geometry.rs`): records → rank/order/coord →
  确定性断言;
- 门禁分级: 纯 .at 组件步骤 = Category A;若 M1 动引擎 → 该步升 Category B
  (`cargo check -p auto-lang` + `cargo t ui` 局部模块) + Category C
  (`cargo test -p auto-lang --test schema_drift`,svg shape tag 集变更触发);
- 双端一致性: 示例目检(用户验收) + gallery golden(基线更新)。

## 验收标准

1. M1 决策记录落地,标签机制双端可用(节点/边标签正确渲染);
2. flow-diagram 静态图渲染: 分层布局目检与 dagre 相当口径(设计文档 §6.1 验收口径,
   非像素对齐),`direction td/lr` 正确;
3. hover 节点 → emphasis 高亮 + tooltip,双端一致;
4. head/tail 箭头字形正确(arrow/diamond/circle/cf-* 至少 arrow 一档);
5. 门禁绿(cargo t + golden;若动引擎 + schema_drift)。

## 执行步骤

- [ ] M1 标签发射对照验证(svg text 直通原型 vs overlay;产出决策记录)
- [ ] M2 flow-diagram 组件骨架(数据轨 props + Init 管线 + svg 冒烟)
- [ ] M3 Sugiyama-lite 布局核(rank/order/coord + 几何对拍测试)
- [ ] M4 边路由 + 箭头字形(bbox 交点直线 + head/tail 小多边形)
- [ ] M5 标签 + hover 交互(hoverNode 状态机 + emphasis + tooltip)
- [ ] M6 示例页 + 双端验证 + golden 基线
- [ ] M7 复审与归档准备

## 复审记录

（复审时填写）

## 待澄清事项

1. **group 嵌套布局是否进 Phase 1**: 设计 §6.1 为"递归先内后外、超节点参与外层";
   若 M3 复杂度超预期,降级为平铺节点先行,group 归 Phase 2a 同批——M3 开工时定;
2. **emphasis 取平面二态还是邻接 focus 模型**(设计 §7.2): 倾向 Phase 1 先平面二态
   (498 同款),focus 模型(邻接 emphasis + 二度外 downplay)归 Phase 2a;
3. **示例落位**: widgets-gallery 新页 vs examples/ui/ 独立示例(0xx 编号待分配)——
   M6 开工时定。
