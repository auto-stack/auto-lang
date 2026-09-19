# 049-canvas-graph — 图元场景样板（PLAN-661）

> canvas 场景契约 v2 能力样板：图元三表族（nodes/edges/labels）+ onhit
> tap 命中 + slider 控制图元参数 + map 括号写，四能力合一。编号顺延
> capability-tests 轨道（048-bp-module-fn 之后）。契约详规见
> docs/specs/auto-lang/ui/design/canvas-scene.md（SD-01）。

## 功能清单

- 图元场景：环形 6 节点（circle）+ 中心 1 矩形（rect）+ 6 辐条边
  （线段）+ 7 标签，坐标由 .at 侧 `math.cos/sin` 循环算好写入三表
  （**上游布局**裁定 R-3——渲染端只收绝对坐标）。
- onhit 命中：点击节点 → handler 收**命中元素 id 字符串载荷**（R-1）；
  未命中区域不派发；v1 命中域 = nodes only。
- slider 控制图元参数：拖动半径（50..110 step 5）→ SetRadius 载荷
  msg → 三表重算重绘（a2r 025 语义基准的三轨一致面）。
- map 括号写：Write 按钮 → `.cfg["scale"] = .radius / 80.0` →
  读回显示（T-01 VM 写臂的就地复现——修复前该语句中止整个 handler）。

## 场景数据契约（canvas v2，schema 描述同源）

- **三表族**（平行串表，B12 风格延续；`<前缀>` = scene prop 值）：
  - `graph_nodes = ["id,x,y,shape,color[,r|w,h]", ...]`
    shape = `circle|rect`；circle 用 r，rect 用 w/h；**x/y 为图元中心**；
    坐标为逻辑坐标（coords extent 同 pen）。
  - `graph_edges = ["x1,y1,x2,y2[,color[,width]]", ...]`
    线段；from/to 语义归上游坐标计算，契约只收绝对坐标（R-3）。
  - `graph_labels = ["x,y,text[,color[,size]]", ...]`
    （宽容解析：畸形项跳过；size 缺省 14。）
- 三表全缺省 = 现行为（strokes-only）逐字节等价（043 消费方零回归）。
- **渲染 = 场景数据契约的纯函数**（Plan 563 裁定重申）：双端各自独立
  绘制（vue `<canvas>` 2D / iced `canvas::Program`），不共享绘制代码；
  extent 缩放（逻辑→px x/y 独立）、circle 半径取 `r*sx` 双端同式。
- **onhit 命中**：tap = pointerdown→pointerup 位移 ≤4px（双端同容差）；
  命中判定 = nodes 声明序**倒序 topmost**；未命中不派发。
  handler 形态 `.NodeTap(id string)`（id 字符串载荷，与 pen 坐标通道
  分立——R-1）。MCP 寻址：`autoui_action press(value=<node_id>)`。

## slider 轨（R-5）

- DSL slider 统一原生 `<input type="range">`（vue）/ iced slider
  （VM）——shadcn Slider 组件路径退役；onchange 载荷 f32
  （`.SetRadius(r float)`，025 语义基准）；MCP `set_value`。
- 双轨断言面见 tests/（VM：snapshot/press/set_value/state 断言；
  vue：DOM range 断言 + fill 设值状态回灌 + 坐标点击命中）。

## map 括号写轨（T-01）

- `.cfg["k"] = v`（字面量键）与 `.cfg[var] = v`（var 键）在 VM 轨
  写后读回正确、写后语句续行（本样板 DoWrite：写 → 读 → 显示）。
- 修复前纪律（078 P-11：VM 轨禁 map 括号写）随修复解除。

## 双端注记

- 画布非方形时逻辑→px x/y 独立缩放（sx≠sy）——环几何在像素面呈各向
  异性，双端**同式畸变**（对拍以结构等价 + 像素量比 ≈ DPR² 口径）。
- 标签文本像素级对齐是既有跨端边界（双端字体不同，非本计划回归）；
  对拍记录：画布区 IoU 0.87 / 交集 RGB 43 / 像素量差 0.8%。

## 测试

- `tests/desktop_mcp.py` — VM 轨（`auto run -r vm` + MCP）：
  三表初始态 / press(id) 命中 / set_value 重算 / map 写读回，11 断言。
- `tests/desktop_vue.mjs` — vue 轨（dev server + Playwright）：
  range 属性面 / 坐标点击命中（含未命中不派发）/ 设值状态回灌 /
  重建后坐标再命中，7 断言。
