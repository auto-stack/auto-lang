# diagram 组件族契约 v1（flow-diagram，裸名）

> 来源：plan-502（2026-09-01 归档，Phase 1 = flow-diagram v1）；需求级设计全文见
> [docs/design/autoui/diagram-components.md](../../../../design/autoui/diagram-components.md)
> （2026-08-31 定稿，本文件只沉淀**已落地**的 v1 契约面）。姊妹篇
> [chart-components.md](chart-components.md)（plan-437/484/498/499）。
> 载体：`examples/widgets-gallery/src/front/components/flow_diagram.at`
> + 页面 `pages/flow-diagram.at`（Diagrams 分组）。

## 范围

Phase 1 仅 **flow-diagram**（有向分层图）。sequence/state/class/er/mindmap/gantt/timeline
归 Phase 2a/2b（设计文档 §5 有契约草案，未落地不入本契约）。

## 数据轨 props（record schema）

```
node  = { id: str, label: str = id, shape: str = "rect", group: str = "", parent: str = "",
          icon: str = "", style: str = "", event: str = "", guard: str = "",
          x: float = -1, y: float = -1 }
edge  = { from: str, to: str, label: str = "", line: str = "solid",
          head: str = "arrow", tail: str = "none", kind: str = "", cardinality: str = "",
          activate: bool = false }
```

- 组件专有 prop：`direction: str = "td"`（`"lr"` 经转置同管线）。
- `x/y ≥ 0` = 用户钉住位（Phase 1 恒 -1，可编辑画布前向预留，布局跳过其坐标求解）。
- `group` v1 **平铺忽略**（Phase 2a 递归超节点）；`parent` 同批。
- DSL 静态糖（`node/edge/group` 词汇）归 Phase 3，到位前示例以数据轨书写。

## 布局：Sugiyama-lite（纯 Auto，"几何 = Auto 代码"）

rank（DFS 灰→灰回边剥离开环 → DAG 最长路径分层）→ order（barycenter 重心法双向
2 轮降交叉，stamp 去重平行边）→ coord（层内等距 + 父居中，min-gap 左打包）；
`direction td/lr` 对坐标转置复用同管线。全 float 纪律（中间量显式 float）。
验收口径：与 dagre **目检相当**（交叉数/间距层级），非像素对齐；双跑确定性有测试钉。

## 渲染：v1 SVG 轨（charts 484 同通路）

- Init 把布局+几何写进段记录（`nodesDg/edgesDg/glyphsDg` 分桶），view `for` 循环发射
  svg `rect/path/ellipse`——vue 轨 svg 子树直通 / VM 轨 svgdoc 序列化。
- **边路由 v1**：节点 bbox 边界交点直线段（轴满偏消除除法尾差）；正交三段路由备选。
- **head/tail 字形**：arrow 三角 / diamond 四点 / circle 双弧实心小多边形，Init 手算
  （方向单位向量；donut 弧先例）；`cf-*` v1 别名 circle；`line: "dash"`/thick 同槽。
- **标签机制（M1 对照定案 2026-09-01）：svg `text` 直通**——vue 轨 `in_svg_subtree`
  上下文分流（子树内 text 直通 SVG `<text>`、位置参数文本为元素内容；子树外
  text→span 不变）+ VM 轨 svgdoc `text` 序列化臂（resvg 原生栅格化）。节点标签
  text-anchor middle，边标签边中点偏移。overlay（DSL text 绝对定位）经对照实证
  局限（vue 响应式缩放脱钩/VM 落位降级/shadcn class 退化）**退守 tooltip 专用**，
  不用于节点/边标签。"svg 无 `<text>`" 引擎约束自此解除（chart 页自身未动，
  y 刻度标签仍 DSL text 列）。

## 交互（498 三段式模板）

`hoverNode: str` 状态机（单行 handler HoverNode/NodeOut + 哨兵 999 无悬停值）→
emphasis 平面二态（hover 节点蓝框加粗/其余降透明）+ 锚定 tooltip（节点 bbox 左上，
overlay 保留场景）。命中 = 节点 bbox 绝对定位 mouse-area 兄弟层（svg 兄弟层不经
svgdoc 序列化，484 增量先例；coords = viewBox 逻辑幅面）。select/focus 模型归
Phase 2a（依赖 498 M0 on_click 臂）。

## 引擎依赖面（502 落地的改动）

svg `text` 直通两处（vue.rs `in_svg_subtree` 分流 + aura_view_builder svgdoc text 臂）
+ vue 轨 svg 子树内多语句 for 包装 `<template>`（div 破坏 SVG 命名空间缺陷修复）。
顺带修复与 diagram 无关的四件（SET_ELEM 栈标签/I32_TO_F32 位再转换/record 键 `to`
上下文化/link 位置参数）见 plan-502 执行期发现 4-7。

## 显式非目标（v1）

group 嵌套/递归超节点、focus 选择模型（Phase 2a）；DSL 静态糖（Phase 3）；
边 hover/节点拖拽/pan-zoom（Phase 4/5）；其余七类图（Phase 2a/2b）；
canvas v2 迁移（与 charts 共用 499 判据集，设计文档 §7.4）。
