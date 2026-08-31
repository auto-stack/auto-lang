# chart 组件族契约 v2（LineChart / BarChart / AreaChart / DonutChart，裸名）

> 来源：plan-437（2026-08-28 归档，v1 = AutoLineChart 等 Auto 前缀四组件）；
> **plan-484 修订（2026-08-29，本版）**：裸名化 + props v2（轴/图例/tooltip/类型扩展）。
> 官方 Auto 实现的 chart 组件族——几何逻辑全在 Auto（"引擎给笔，Auto 持笔"），
> 渲染端 v1 = SVG 子树直通（vue 轨 442 A4 直通臂 / VM 轨 svgdoc 通道），
> v2 canvas Program 桥接后置。载体：`examples/widgets-gallery/src/front/components/
> {line,bar,area,donut}_chart.at`（widget 名去 Auto 前缀，见"命名与退役"）。

## 范围

四类图（line/bar/area/donut）的组件契约、几何算法与双轨渲染路径、hover tooltip
命中区模型。vue 端 shadcn/unovis 第三路线**随 484 退役**（见"命名与退役"）。

## 原则

- **核心定位（484）**：算法固化在官方组件内——用户只选类型（原语标签）、配参数
  （props）、给数据（record 列表）；算法是 official 包的版本化资产，随包演进，
  用户代码不动。裸 SVG 原语保留为非常规图表的逃生舱。
- **推荐写法（484 复审后用户裁定）**："可辨明 node 身份的属性入括号，其余入 body"——身份属性（`index`/`type`/`curve`/`id`）留在括号，数据与长属性（`data`/`fields`/`colors`/`labels` 及布尔开关）写进 body。解析器原生支持 body 内联 props（`identifier:` 即 prop、`on*` 归事件、其余为子节点），括号/body 两形态解析后等价。示例：

      bar-chart (index: "quarter", type: "stacked") {
          data: .quarterlySales
          fields: ["product", "service"]
          labels: ["Product", "Service"]
      }

- **数据形状定案**（437 Phase 0，不变）：匿名 record 字面量列表 + 字符串字段名
  （`data: [{ m: "Jan", v: 30 }, …]`），`index` = x 标签字段，`fields` = 数值系列字段。
- **几何 = Auto 代码**：linear/band scale + path generator 全部用 Auto 写
  （float 纪律：中间量显式声明 float；int→float 经存储强转；局部变量不得与 model
  字段同名）。
- **段记录打包**：Init 把几何结果写进 model 的 `segs List`（`{d,c}` 或 `{a,l,c}`），
  view 用 `for seg in .segs` 循环发射 SVG——路径与颜色同源，双轨一致。
- **纯派生幂等**（ADR-19 约束）：Init 每渲染帧重放，组件全部输出（几何/刻度/图例/
  tooltip 锚点）由 props 一次算出；hover 状态（`hoverIdx`）是唯一可变 model 字段，
  由 enter/leave 单行 handler 维护。

## 命名与退役（484 核心决议）

- **裸名让位 Auto 组件**：official 包四组件更名 `LineChart/BarChart/AreaChart/
  DonutChart`，标签 `line-chart/bar-chart/area-chart/donut-chart`；`auto-*` 旧名
  全仓退役（widgets-gallery 四页、024-charts、437 测试钉同步迁移）。
- **registry 退役**：内置 `AreaChart/BarChart/LineChart/DonutChart/Chart` 五注册
  （shadcn 发射路线）及其 schema/aura.at 七元素 props overlay、
  `emit_chart_family_attrs`、CurveType 映射全部退役；`render_support.rs` 的
  `chart|canvas` fallback 保留（`chart` 裸 tag 依旧无引擎实现——组件解析在包层）。
- **shadcn 资产退役**：`crates/auto-man/assets/shadcn-ui/chart*`（ChartTooltip/
  ChartLegend/ChartCrosshair/ChartSingleTooltip 与 chart-area/bar/donut/line）
  随路线退役删除；unovis npm 依赖声明同步移除。
- **质量基线（484 用户）**：Auto 实现效果须与 shadcn 版基本相当——以新旧
  charts-gallery 视觉对拍为验收口径。

## 组件契约 v2

视口约定（不变）：`viewBox "0 0 560 300"`，绘图区内边距 left=40 / right=550 /
top=20 / baseline=260。

| 组件 | props（widget-parens） | 几何 |
|---|---|---|
| LineChart | `data: List`（必填）, `index: str = "m"`, `fields: List<str> = ["v"]`, `colors: List<str> = 四色默认`, `axis: str = "auto"`, `grid: bool = true`, `legend: bool = true`, `labels: List<str> = []`, `tooltip: bool = true`, `curve: str = "linear"` | linear scale + 每系列一条 path；curve="monotone" 时 d3 curveMonotoneX |
| BarChart | 同上（无 curve），增 `type: str = "grouped"` | band scale（槽=510/n）+ type="stacked" 时系列底边=前系列累计和、y 域取累计最大 |
| AreaChart | 同 LineChart | 折线下探底边闭合 Z（fill 0.25 叠加）+ 顶边描边；curve 同 LineChart |
| DonutChart | `data: List`, `index: str = "l"`, `field: str = "v"`（单系列语义，不设 fields）, `colors`, `legend`, `labels`, `tooltip` | d3-arc：角度按占比循环累进，外弧 A + 内弧回程 Z，laf = span>π |

默认四色调色板（不变）：`["#2563eb", "#16a34a", "#d97706", "#dc2626"]`；
colors 不足时复用末色（`ci >= ccount → ccount - 1`）。

### 轴与刻度（`axis: "auto" | "none"`，默认 auto；`grid: bool = true`）

- **nice-ticks 算法**（d3-scale/plotters 风格，整数域）：
  `step = 10^floor(log10(vmax/4)) × {1,2,5}` 取最小使 `4×step ≥ vmax`；
  `top = ceil(vmax/step)×step`；刻度 = 0, step, 2·step, …, top（恒 5 档内）。
  y 像素映射 `y = 260 - v/top × 240`。
- **退化域**：data 为空列表或 vmax==0 时 `top=4, step=1`（刻度 0..4），网格照发，
  绘图区空白不报错。单点数据正常出刻度（域按该点值）。
- **发射形态**：y 刻度标签 = DSL `text` 列（svg `<text>` 不支持，约束不变），
  左侧 w-10 列组件内置发射；网格线 = svg `path`（水平线，色 #e2e8f0）；
  x 轴标签 = `index` 字段值等距抽样 ≤8 个，DSL `text` 行置于图下。
- `axis: "none"` 时组件不发射刻度/网格/标签（024-charts 自管刻度的旧形态出口）。

### 图例（`legend: bool = true`；`labels: List<str>`）

- 每系列一行/项：色块（w-3 h-3 rounded-sm）+ 显示名（`labels[i]` 缺省回落
  `fields[i]` 字段名）。
- bar/line/area 图例横排于图下；donut 图例含百分比（整数运算
  `vi * 100 / total`，浮点尾差坑不变）。

### hover tooltip（`tooltip: bool = true`；484 纳入）

- **命中区模型**（业界对拍定案）：Init 产出每命中区的屏幕包围盒；
  line/area 用 **x 索引全高竖带**（宽=槽宽，非逐点小圆——大命中目标且与
  shadcn ChartSingleTooltip 的 index 语义一致：单 tooltip 出该 x 全系列值）；
  bar 用柱包围盒（stacked 为每段）；donut 命中区 = **图例行**（扇区弧形无法
  用矩形近似命中，hover 图例行 → tooltip 锚定扇区中心，位置 Init 已知）。
- **发射形态**：svg 子树叠 `Stack` + 透明 `mouse-area` 命中区（484 新增基础
  widget：VM = iced mouse_area on_enter/on_exit；vue = div @mouseenter/@mouseleave，
  无视觉、仅事件转发）。tooltip 本体 = DSL 自绘 col（bg-popover border rounded
  px-2 py-1 text-xs；行 = 色点 + 系列名 + 值），**锚点 Init 算好固定**（竖带顶
  /柱顶/扇区中心），事件仅 enter/leave 低频——**不做跟随光标**（crosshair 随
  v2 canvas 桥接另立计划，需 mousemove 流性能设计对照 Plan 386）。
- hover 状态（tooltip）：`hovered str` 门控 + tipTitle/tipBody 槽位（484 形态）。
- 形态对齐 shadcn ChartSingleTooltip：色点 + 标签 + 数值。

### 交互态（plan-498 落地：emphasis 二态 / 转折点 / legend 显隐）

- **emphasis/downplay 二态**（ECharts emphasis 模型,组件层实现,引擎仅 mouse-area
  增 on_click 臂）：line/area 图例行 mouse-area 悬停 → `.HoverSeries(k)/.SeriesOut`
  写悬停态字段——高亮系列线宽 2→3+opacity 1、其余 downplay 0.25（area fill 三态
  0.45/0.25/0.08）；line 高亮系列浮现**转折点圆圈**（r=3,坐标 Init 预计算槽位
  pts0..3）。bar 命中竖带 `.Hover(i)` 顺带分组态：该组柱描边 1.5、其余组
  fill-opacity 0.3。donut 图例行/扇区 `.Hover(i)`：该扇区沿中角**外移 12px**
  （Init 预计算平移路径 d/e 双形态,弧径不变）+ 白描边 2px。
- **legend 点击显隐**：图例 mouse-area `onclick: .Toggle(k)` 翻 vis 族布尔——隐藏
  系列几何与 emphasis 全跳过（隐藏优先于悬停）,图例项落 opacity-40（可再点复原）。
- **悬停态字段命名纪律**：图族专属 hovLn/hovAr/hovBr/hovDn + visLn/visAr/visBr/
  visDn0..3；**无悬停哨兵 = 9（越界正值）,勿用 -1**——view 条件对负数字面量比较
  恒假（引擎缺陷 P498-1）;同名字段在 VM 单态架构下跨组件串扰,图族更名解耦
  （P498-2,同族多实例仍联动,vue 轨无此现象）。
- **mouse-area 事件面**（M0 引擎臂,schema 双源已登记）：onmouseenter/onmouseleave/
  ondblclick（496）/onclick（498 → iced on_press / vue @click）。
- **双端表现差异**：vue 走 CSS transition 淡入（组件 style 块 path transition
  0.15s）,VM 直接切换——状态与几何完全同源,仅增强动画差异。

### 类型扩展（旧 gallery 形状对拍所必需）

- bar `type: "stacked"`：y 域输入 = 各 index 全系列累计和的最大值（再经 nice-ticks）；
  系列 i 柱底 y = 系列 0..i-1 值之和映射，柱顶 = 累计和映射；每段独立包围盒命中。
- line/area `curve: "monotone"`：d3 curveMonotoneX（Fritsch-Carlson 单调三次插值，
  不过冲数据点）：secant 斜率 → 切线 m[i]（端点 0、相邻异号取 0）→ 每段
  `C (x_i+h/3, y_i+m_i·h/3) (x_{i+1}-h/3, y_{i+1}-m_{i+1}·h/3) (x_{i+1}, y_{i+1})`。
  确定性断言：固定输入对照 d3 参考实现手算控制点。

## 渲染端

- **v1（现行）**：SVG 子树直通——vue 轨 svg/path 直通臂；VM 轨 `serialize_svg_children`
  序列化 svgdoc（支持 Element/Conditional/ForLoop，动态属性经 resolve_expr_to_string
  的 Index 臂取 `seg["d"]`）。**484 增量**：svg 兄弟层的 Stack/mouse-area 为常规
  widget（非 svgdoc 内容），不经序列化路径。
- **v2（后置，设计决议不变）**：canvas Program 桥接（Auto 图元列表 → iced canvas），
  服务 .Tick 高频流式重绘 + crosshair；触发条件 = svgdoc 序列化性能不达标。

## 显式非目标

- 不引第三方 chart 依赖（437 调研结论不变）。
- 不做跟随光标的 crosshair / mousemove 流交互（v2 canvas 阶段另立计划）。
- pie/水平条形/散点等新图类型后置独立计划（旧 gallery 未用，484 不做）。
- 多系列不做自动调色板生成（colors 不足复用末色）。

## 已知坑（复用必读）

- VM 轨子组件 Init 在渲染期补发、每帧重放（ADR-19）：纯派生幂等安全；**副作用型
  Init（计数/一次性请求）会随脏重建重复执行**——组件 Init 保持纯派生；hover 的
  enter/leave handler 是事件驱动（非渲染期），不违背幂等。
- svgdoc 里静态网格 path 与动态 seg path 的属性序不同（HashMap 序），断言用值片段
  勿用整串。
- 图例百分比用整数运算（`vi * 100 / total`），浮点除会产生尾差。
- nice-ticks 的 log10 经由 float；vmax 为 int 时先存储强转 float（float 纪律）。

### 484 时代三大 codegen 坑——492 已定案并摘除（2026-08-30）

Plan 484 执行期记档的三个"包组件 codegen 坑"（Init 内 prop 字符串比较破坏
codegen / f-string 含字面量 `[]` 时 `${}` 插值破坏编译 / 带参 msg 声明破坏整包
编译）经 Plan 492 全链复现实验**全部定性为误归因**：三条链（lexer/parser/单 VM
合成/vue SFC）在 master 上均正常，484 现场的真凶是**裸 prop 名出现在赋值 RHS 位
触发 undefined variable 解析错，包文件被 per-file try-parse 静默整文件丢弃**
（诊断面缺陷，492 M5 已修——装载层逐条告警 + 合成层可取走诊断集合）。

直接写法（492 M6 起组件已回归）：

- Init 内 prop 比较直用**点前缀**形态（`if .curve == "monotone"`、
  `if .type == "stacked"`）——props 在运行期播种进统一 state，`.prop` 即读；
  **裸名形态**（`if curve ==`）在 IF 条件位可过，但出现在赋值 RHS 位
  （`.x = curve`）会被语义检查拒——一律用点前缀。
- f-string 的 `${}` 插值与字面量 `[]` 可任意组合（`f"w-[${slot}px] h-full"`）；
  `{x}` 花括号形态是**纯字面量**，从不插值（484 的"绕开形态"实为无害垃圾类）。
- 带参 msg 声明（`msg { Init, Hover(int) }`）与裸挂带参 handler 双轨均正常。
- 刻度/图例的槽位字段（yTick0..4/legendColor·Text0..3）是 43956041e 的 R006
  规避（text 内容引用循环变量记录字段的渲染缺口，vue 侧 492 M3 已修引号剥离；
  双轨完全直写化待后续——现状保留槽位形态）。
- 组件源内的旧"实证纪律"注释已随直接写法更新；三副本
  （charts-gallery / 024-charts / widgets-gallery）同步。
