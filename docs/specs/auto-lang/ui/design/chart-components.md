# chart 组件族契约（AutoLineChart / AutoBarChart / AutoAreaChart / AutoDonutChart）

> 来源：plan-437（2026-08-28 归档）。官方 Auto 实现的 chart 组件族——几何逻辑全在
> Auto（"引擎给笔，Auto 持笔"），渲染端 v1 = SVG 子树直通（vue 轨 442 A4 直通臂 /
> VM 轨 svgdoc 通道），v2 canvas Program 桥接后置。载体：`examples/widgets-gallery/
> src/front/components/{line,bar,area,donut}_chart.at`。

## 范围

四类图（line/bar/area/donut）的组件契约、几何算法与双轨渲染路径；vue 端 shadcn/unovis
第三路线的声明契约（schema/aura.at 七元素）。

## 原则

- **数据形状定案**（Phase 0）：匿名 record 字面量列表 + 字符串字段名
  （`data: [{ m: "Jan", v: 30 }, …]`），`index` = x 标签字段，`fields` = 数值系列字段。
- **几何 = Auto 代码**：linear/band scale + path generator 全部用 Auto 写
  （float 纪律：中间量显式声明 float；int→float 经存储强转；局部变量不得与 model
  字段同名）。
- **段记录打包**：Init 把几何结果写进 model 的 `segs List`（`{d,c}` 或 `{a,l,c}`），
  view 用 `for seg in .segs` 循环发射 SVG——路径与颜色同源，双轨一致。
- **命名政策**：Auto 前缀（AutoLineChart…）避开内置 shadcn LineChart 等折叠名
  （§0.6.E-1 撞名教训）。

## 组件契约

| 组件 | props（widget-parens） | 几何 |
|---|---|---|
| AutoLineChart | `data, index="m", fields=["v"], colors=["#2563eb"]` | linear scale + 每系列一条 M/L path |
| AutoBarChart | `data, index="q", fields=["v"], colors=[…]` | band scale（槽=510/n，槽内 60% 均分系列）+ 每柱 MhvhZ 合单 path |
| AutoAreaChart | `data, index="m", fields=["v"], colors=[…]` | 折线下探底边闭合 Z（fill 0.25 叠加）+ 顶边描边 path |
| AutoDonutChart | `data, index="l", field="v", colors=四色默认` | d3-arc：角度按占比循环累进，外弧 A + 内弧回程 Z，laf = span>π，math.cos/sin 直调 |

colors 不足时复用末色（`ci >= ccount → ccount - 1`）。

## vue 端第三路线契约（schema/aura.at）

七元素 props 全量落库（437 Phase 1，发射遍历声明）：areachart 17 / barchart 17 /
linechart 16 / donutchart 13 / chart(config) 2 / charttooltip 3 / chartlegend 1 项；
npm 声明 `@unovis/vue`（运行时依赖由 auto-man PLAN-457 OPTIONAL_DEPS 管控）。
链路：schema 声明 → registry overlay 填 vue BackendMapping.props →
`emit_chart_family_attrs` spec 驱动发射。转换层特例：curve-type 字符串值 → CurveType
枚举映射（未知回退 MonotoneX）；custom-tooltip 组件引用恒走绑定。

## 渲染端

- **v1（现行）**：SVG 子树直通——vue 轨 svg/path 直通臂；VM 轨 `serialize_svg_children`
  序列化 svgdoc（支持 Element/Conditional/ForLoop，动态属性经 resolve_expr_to_string
  的 Index 臂取 `seg["d"]`）。
- **v2（后置，设计决议）**：canvas Program 桥接（Auto 图元列表 → iced canvas），
  服务 .Tick 高频流式重绘；触发条件 = svgdoc 序列化性能不达标。

## 显式非目标

- 不引第三方 chart 依赖（plotters-iced 只支持 iced 0.13；逻辑进 Rust 与"chart 为
  Auto 组件"冲突）。
- v1 不做 ChartTooltip/ChartLegend 的 Auto 实现（声明契约已落库，实现随需）。
- 不做事件（schema 声明 "No events in v1"）。
- 多系列 v1 同色由调用方经 colors 指定，不做自动调色板生成。

## 已知坑（复用必读）

- VM 轨子组件 Init 在渲染期补发、每帧重放（ADR-19）：纯派生幂等安全；**副作用型
  Init（计数/一次性请求）会随脏重建重复执行**——组件 Init 保持纯派生。
- svgdoc 里静态网格 path 与动态 seg path 的属性序不同（HashMap 序），断言用值片段
  勿用整串。
- 图例百分比用整数运算（`vi * 100 / total`），浮点除会产生尾差。
