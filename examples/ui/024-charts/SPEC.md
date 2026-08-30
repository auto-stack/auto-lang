# 024-charts — regeneration SPEC

> Purpose: 图表工坊——编辑数据系列、切换图表类型、观察几何重算的交互式
> demo。AutoOS 系统监视器/图表应用的孵化形态（Design 21 App 轨道填洞 ①）。
> **Frontend-only, in-memory data, no backend.** 几何模式复用 widgets-gallery
> Charts 分组的已验证实现（Plan 437 §0.6.E/F/G/H）。

## Functional spec (regenerate from this, no code)

单视图图表工坊：

- **Layout**: 顶部标题栏；主体为左右行布局——左侧数据面板（固定宽），
  右侧图表区（自适应）。
- **数据**: 月度 x 三系列（Desktop / Mobile / Tablet），6 个月份 record
  列表（gallery 契约形状 `{ m, desktop, mobile, tablet }`），内存态、
  M1 不可编辑数值（M2 流式追加）。
- **类型切换**: 四个按钮（Line / Bar / Area / Donut），当前类型高亮；
  切换零重算（几何在 Init 一次算全四类，view 按类型分支渲染）。
- **系列开关**: 每系列一行——色块 + 名称 + checkbox（可见性）+ 统计串
  （last/max）。关闭系列即从图上隐藏该系列（line/bar/area）。
- **几何**（全部 Auto 内联计算，Init 一次）：
  - line: linear scale，每系列一条 M/L path；
  - bar: band scale（slot = 内宽/月数，槽内三等分），每系列一条合成
    path（每柱 M/h/v/h-Z 子路径）；
  - area: 线 path + 下探底边闭合 Z 的填充 path（fill-opacity 0.25），
  - donut: **末月构成**（最后一月三系列占比切片），d3-arc 风格外弧 A +
    内弧回程 Z，大弧标志 span>π，循环累进角度（math.cos/sin 直调——
    §0.6.H 根因已修的落地验证）。
- **M1 简化（记录在案）**: 刻度对全部三系列取 max（隐藏系列不触发
  重算/rescale）；donut 忽略可见性（plan §6：donut 限静态分布展示）。

## Data model

```
monthly: List of { m str, desktop int, mobile int, tablet int }
chartType str ∈ {"line","bar","area","donut"}
dVisible/mVisible/tVisible bool
每类型每系列一条 path str（lineD/lineM/lineT、barD/…、areaD/…、donutD/…）
labels str（月份拼接）、donutLegend str、stats{D,M,T} str
```

## Architecture notes (what the AI should know)

- Widget tier: `row/col/button/checkbox/text/svg+path/line`（SVG 直通，
  Plan 442 A4）；无 tabs 受控绑定——类型切换用按钮行 + 条件样式
  （tabs 的 value 受控绑定未验证，M1 不冒险）。
- 几何纪律（437 §0.6.D）：float 中间量显式声明 `float`；int→float 经
  存储强转或与 float 字面量混合；`math.*` 可直调（含循环体内）。
- 系列编辑（M2+）走整表重建（append/filter 模式，013 先例），避免就地改。
- 几何代码故意三份内联（Init 一次算全）而非按需重算——handler 间无法
  互相调用、模块级 fn 不进 vue SFC（§0.6.E-3），M1 取"零重算"形态。

## Known deferrals (do NOT try to add these — they need unbuilt features)

- **双模式展示**（已落地）: **静态模式** = 初始/Reset 展示种子月度数据
  （Jan–Jun 三系列，`seed` 为不变式源）；**流式模式** = `.Tick`（model
  `interval int = 400` 作周期 + `running` str 门控，handler 内自决——
  Plan 402 语义）每 tick 追加一个数据点（纯整数算术锯齿波
  `150+(tickN*37)%120` 族——f64 值入 record 的比较链未验证，整数保平安）
  并按 `windowLen`（8/12/20 可调）滑出旧点；line/bar/area/stats 重算，
  donut 保持静态构成。Play/Pause/Reset 三键：Play 启动流式、Pause 停、
  **Reset 重建 monthly=seed 并全量重算（回到静态展示，tickN 归零）**。
- **数值编辑**（系列数值增删改）: 后续引入（整表重建模式，013 先例）。
- **组件化渲染函数**（LineChart 等）: 待 Plan 435 统一声明体系合入
  后由 437 收口，M1 页面内联形态。
- **vm 实机/desktop_mcp/golden**: M3 辖区。
