# canvas 图元场景契约 v2（三表族 + onhit 命中）

> 来源：PLAN-661（2026-09-19）。canvas 契约 v1（Plan 563 笔笔画双表）的
> 图元场景扩容详规——v1 详规此前散于 chart-components.md 563 修订头 +
> schema 描述，本档独立成规后为 canvas 场景契约单一详规源。
> 样板：`examples/capability-tests/049-canvas-graph/`（四能力合一）+
> `043-canvas-paint`（笔笔画面，v1 兼容回归锚）。

## 范围

canvas 元素的图元场景数据契约（nodes/edges/labels 三表族）、onhit 命中
契约、双端渲染规约、布局归属。笔笔画契约（`<前缀>_pts/_meta` 双表 +
pen 三件套）见 563 详规（schema 描述 + 043 SPEC），本档不重复。

## 场景数据契约（平行串表族，B12 风格；`<前缀>` = scene prop 值）

| 表 | 项格式 | 说明 |
|---|---|---|
| `<前缀>_nodes` | `id,x,y,shape,color[,r\|w,h]` | shape=`circle\|rect`；circle 用 r，rect 用 w/h；**x/y 为图元中心**；逻辑坐标（coords extent 值域） |
| `<前缀>_edges` | `x1,y1,x2,y2[,color[,width]]` | 线段；契约只收**绝对坐标**（from/to 语义归上游，见布局归属） |
| `<前缀>_labels` | `x,y,text[,color[,size]]` | 宽容解析；size 缺省 14 |

- 解析宽容：畸形项跳过（pts/meta 现行容错同款）；三表全缺省 =
  strokes-only 现行为**逐字节等价**（043 消费方零回归）。
- 缺省色：node `#3b82f6` / edge `#9ca3b8` / label `#111827`；缺省尺寸
  r=16 / w=h=40 / width=2。

## 渲染规约（纯函数裁定重申）

- **渲染 = 场景数据契约的纯函数**（Plan 563 裁定）：双端各自独立绘制
  （vue `<canvas>` 2D / iced `canvas::Program`），不共享绘制代码；共享
  的是数据契约与本映射规约。
- 逻辑→px 按 extent **x/y 独立线性缩放**（sx≠sy 画布非方形时几何呈
  各向异性，双端同式）；circle 绘制半径取 `r*sx`（双端同式）。
- 绘制顺序：edges → nodes → labels（声明序，后声明者覆盖）。

## onhit 命中契约（R-1）

- DSL：`onhit: .MsgHandler`——独立事件 prop；handler 收**命中元素 id
  字符串载荷**（`.OnNodeTap(id string)` 范式，与 `.SetVol(v float)`
  载荷绑定同通道）。
- tap 判定：pointerdown→pointerup 位移 ≤4px（逻辑 px，双端同容差）；
  拖拽笔画不触发。
- 命中域：**nodes only**（v1 实现取舍而非契约边界；edges/labels 命中
  上报后置，KNOWN-DEBT）。判定 = 声明序**倒序 topmost**；circle 含 r、
  rect 以中心 ±w/2、±h/2；未命中不派发。
- MCP：canvas UiNode actions 挂 `press`（value=元素 id）；快照 props
  暴露 nodes 计数与 ids（寻址面）。`autoui_action press(value=id)` 与
  真实 tap 消息同构直达（`event␟s␟id` 编码）。
- 不复用 pen press 上报（坐标通道与标识通道语义不同）；不新造
  onnodeclick 词位。

## 布局归属（R-3）

场景坐标由**上游下发**（三表自带 x/y）。「渲染=纯函数」裁定决定布局
是数据不是渲染：环形/网格/fcose 力导布局在 .at 侧（或 jade graph
store 侧）循环+数学生成后写入三表；VM/渲染端不自算布局。map 括号写
（PLAN-661 T-01：`m[k]=v` VM 写臂修复）是 graph store 以 map 存
settings/坐标消费流的先决。

## 非目标 / 后置债

- 交互式缩放/平移（viewport 字段与事件回路）——cytoscape 对位首批子集
  = 节点/边/标签/tap（078 终裁）；缩放平移后置（KNOWN-DEBT）。
- edges/labels 命中上报（v1 命中域 nodes only）。
- eraser/挖除语义升级、canvas 子元素快照树（v1 canvas 单节点 + actions
  自描述）。

## 对拍口径（PLAN-661 实录）

- 结构等价：节点计数/辐条/中心块一致；各向异性畸变双端同构；画布区
  对齐后像素量比 ≈ DPR²。
- 定量（049 初态实录）：画布区 300×300 对齐 IoU 0.8677 / 交集 RGB 平均
  距 42.95 / 像素量差 0.8%。**标签文本像素级对齐是既有跨端边界**
  （双端字体不同）——几何像素（节点+边+中心块）结构完全一致，文本
  差异主导 IoU 缺口；后续样板收紧阈值时以几何像素子集口径为准。
