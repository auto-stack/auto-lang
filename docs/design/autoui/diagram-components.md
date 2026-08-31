# Diagram 组件家族与 DSL 设计（flow / sequence / class / state / er / mindmap / gantt / timeline）

> 状态：**设计定稿待评审**（2026-08-31，用户需求驱动 + Mermaid/D2 官方文档调研）。
> 定位：需求级设计，归位 `autoui/`；chart 家族的姊妹篇——对标契约
> [chart-components.md](../../specs/auto-lang/ui/design/chart-components.md)（plan-437/484）。
> 关联：Plan 498（chart 交互状态机，drafting）、Plan 499（chart v2 canvas 交互，drafting）、
> Plan 484（chart 组件化与 v1 交互）、Plan 492（引擎三族与 codegen 定案）、Plan 386（RenderQueue）、
> Plan 496（mouse-area ondblclick 先例）。
> 本文 §7 给出与 498/499 的**统一交互与渲染模型**——结论：可统一，统一面在
> 交互状态机模式、图元记录 schema、canvas v2 桥与 mousemove 流、动画双轨四处。

---

## 1. 背景与问题

chart 家族（line/bar/area/donut）已覆盖**数据编码类**图形（数据 → 视觉通道映射）。
AutoUI 还缺**语义拓扑类**图形——节点是名词、边是动词的结构表达（流程图、时序图、
ER 图……），即 Mermaid/D2 所在的领域。本文回答四个问题：

1. 哪些图类型属于 diagram 家族（与 charts 划界）；
2. Mermaid 与 D2 的声明语法各如何设计、取什么（调研已核对官方文档）；
3. AutoUI 的 diagram DSL 长什么样（与既有语法风格适配）；
4. 与 charts 侧 Plan 498（交互状态机）/499（canvas v2）如何统一。

**核心立场（延续 437/484）**：算法固化在官方组件内（"引擎给笔，Auto 持笔"），
不引第三方 diagram 渲染依赖——mermaid.js/d2 只能在 vue 轨跑，内嵌即破坏
`auto run -r vm` 双端一致性（autoui-verifier 硬约束）。布局与几何全部用 Auto 写。

## 2. 类型盘点与家族划界

| 类别 | 类型 | 声明核心 | 归属 |
|---|---|---|---|
| 结构关系类 | 流程图、时序图、类图、状态机图、ER 图、架构图/C4、思维导图/组织树、网络拓扑 | 节点 + 边 + 容器嵌套 | **本文 diagram 家族** |
| 时间进程类 | 甘特图、时间线、用户旅程、Git 提交图 | 时间轴 + 事件/任务 | diagram 家族（时序子组） |
| 数据编码类 | 饼/环、柱/折线/面积、四象限、桑基、雷达、块图 | 数据 → 视觉通道 | **charts 家族**（已有四类；四象限/桑基/雷达作为 charts 后续扩展） |

划界比 Mermaid 更干净：Mermaid 把 xychart/pie 也算 diagram，但它们本质是数据编码。
**Diagram 家族专指语义拓扑**。架构图/C4/网络拓扑不单设组件——`group` 嵌套 +
`shape: "person"/"cloud"` + `icon` prop 即覆盖（D2 同款路线）。

## 3. 业界调研结论：Mermaid vs D2

（调研基于 mermaid.js.org 与 d2lang.com 官方文档及两仓文档源码，示例已逐字核对。）

| 维度 | Mermaid | D2 | 本文取舍 |
|---|---|---|---|
| 分派机制 | 首行关键字切换 18+ 种子语法（一国一语法） | 单一对象模型；图类型是 `shape` 属性取值 | **采 D2**：与统一 AST 契合 |
| 结构编码 | 行内标点（`A{x}`、`||--o{`、`-.->`），密度高解析脆 | 键值 + 缩进（`x.shape: circle`），LSP/重构友好 | **采 D2**：ID 与 label 分离 |
| 边语法 | 线型×箭头×长度矩阵 | `-- -> <- <->` 四元组 + 箭头形状属性化（`target-arrowhead`） | **采 D2**：`->` 统一，修饰进 props |
| 类型覆盖 | 时序类全（gantt/timeline/journey/gitGraph） | 结构图深（sql_table 列级连线/grid/near），时序类缺 | **两者都要**：结构类采 D2 模型，时序类采 Mermaid 关键字 |
| 组合性 | 弱（一码一图，classDef 复用） | layers/scenarios/steps + link 导航 + vars + classes | **不需要新语法**——AutoUI model 状态天然是场景切换（§4.4） |

## 4. DSL 语法设计

### 4.1 设计原则

1. **单一对象模型 + 家族组件分派**：所有 diagram 共享 `node/edge/group` 三原语；
   每类型一个独立 kebab-case 组件（对齐 `line-chart/bar-chart` 家族模式，props 按类型特化）。
2. **只复用 Auto 已有语法原语**：`->`（`on { .Init -> {} }` 已用箭头）、`{ }` 块嵌套
   （view 同款）、`key: value` props、`name type` 声明序（model 同款）、`"""` 多行字符串
   （Plan 169，导入轨用）。
3. **ID 与 label 分离**（D2 教训）：id 可被 edge 引用与事件携带，label 仅展示。
4. **静态糖 ↔ 动态数据双轨**：声明行是糖，desugar 成 record List 走 props——与
   `donut-chart (data: .traffic)` 完全同构，ADR-19 重播种链路原样复用。
5. **交互是一等公民**（对 Mermaid/D2 的差异化）：diagram 是活组件不是图片——节点
   hover/选中、状态机实时高亮（§7）。
6. **不做符号动物园**：`<|--`、`||--o{`、`-.->` 全部语义化进 props
   （`kind: "extends"`、`cardinality: "1..*"`、`line: "dash"`）。

### 4.2 通用三原语（全家族共享）

```
node <id> "<label>" (shape: "...", style: "...")     // 形状是枚举 prop,非标点
group <id> "<label>" { ... }                         // 容器/子图,块嵌套即包含
edge <id路径> -> <id路径> (label: "...", line: "...", head: "...", tail: "...")
```

- **路径引用**：`edge cache.redis -> api`（D2 式点路径，与 Auto `.` 成员访问一致）；
- **边链**：`edge a -> b -> c` desugar 为两条边；
- **未声明即引用**：`edge` 引用未声明的 id 时自动创建默认 `rect` 节点（label = id，
  Mermaid 先例）；防拼写事故的 lint 告警归 Phase 3 糖语法同批。
- **边 props 词汇**（一次定义全家族通用）：

| prop | 取值 | 对标 |
|---|---|---|
| `label` | str | Mermaid `-->|text|` / D2 `: label` |
| `line` | `"solid" \| "dash" \| "thick"` | Mermaid `---/-.-/===` 三族，只留三档 |
| `head` / `tail` | `"arrow" \| "none" \| "diamond" \| "circle" \| "cross" \| "cf-one" \| "cf-many"` | D2 `source/target-arrowhead`；双向 = `tail: "arrow"` |
| `kind` | `"extends" \| "implements" \| "composition" \| "aggregation" \| "dependency" \| "association"` | 类图/ER 语义关系，renderer 映射 UML 字形 |
| `cardinality` | `"1" \| "0..1" \| "1..*" \| "*"` | ER/UML 基数 |

**形状词汇**（v1）：`rect, round, stadium, diamond, hexagon, cylinder, circle, cloud,
person, text, initial, final`（后两个为状态图伪节点）。

### 4.3 逐类型语法

**① flow-diagram（流程图；架构图/C4/拓扑同构表达）**

```
flow-diagram (direction: "td") {
    node start "开始" (shape: "stadium")
    node check "就绪?" (shape: "diamond")
    group cache "缓存层" {
        node redis "Redis" (shape: "cylinder")
    }
    edge start -> check
    edge check -> run (label: "yes")
    edge retry -> check (line: "dash")
    edge run -> cache.redis
}
```

**② sequence-diagram（时序图；消息即边——D2 洞见，碎片保留 Mermaid 关键字）**

```
sequence-diagram (title: "下单流程") {
    actor user "用户"
    participant db "订单库" (shape: "cylinder")

    edge user -> api (label: "POST /orders")
    edge api -> db (label: "查询库存", activate: true)
    edge db -> api (label: "ok", line: "dash")
    loop "直到成功" {
        edge api -> api (label: "重试")
    }
    alt "有库存" {
        edge api -> user (label: "201 Created")
    } else "缺货" {
        edge api -> user (label: "409")
    }
    note (over: [user, api]) "网关无状态"
}
```

`activate: true` 替代 Mermaid `+/-` 记号；`loop/alt/else/opt/par/note` 为时序专用
块关键字（时序语义强，值得专用词汇）。

**③ class-diagram（类图；成员行与 Auto model 声明序一致）**

```
class-diagram {
    class Animal (abstract: true) {
        + age int
        + is_mammal () bool
    }
    class Duck : Animal {
        - beak_color str
        + swim () void
    }
    edge Order -> Duck (kind: "association", cardinality: "1..*")
}
```

`class Duck : Animal` 的 `:` 继承声明 desugar 为 `edge Duck -> Animal (kind: "extends")`——
与显式 edge 写法等价，二者可混用（同一条边去重）。

**④ state-diagram（状态机；`active` 绑定真实应用状态是杀手级用法）**

```
state-diagram (active: .phase) {
    state idle "空闲"
    state running "运行中" {
        state step1 "下载中"          // 复合状态 = 块嵌套
    }
    edge initial -> idle
    edge idle -> running (event: "start")
    edge running -> idle (event: "stop", guard: "count > 3")
    edge running -> final (event: "done")
}
```

**⑤ er-diagram（ER 图；基数走 props，支持 D2 式列级端点）**

```
er-diagram {
    entity Customer {
        id int (pk)
        name str
        email str (unique)
    }
    entity Order {
        id int (pk)
        customer_id int (fk)
    }
    edge Customer -> Order (cardinality: "1-*", label: "places")
    edge Customer.id -> Order.customer_id (line: "dash")
}
```

**⑥ mindmap（思维导图/组织树；纯嵌套即结构，无 edge）**

```
mindmap {
    node root "发布规划" (shape: "circle") {
        node a "调研"
        node b "设计" {
            node b1 "DSL 语法"
        }
    }
}
```

**⑦ gantt / timeline（时序类）**

```
gantt (title: "发布计划") {
    section "设计" {
        task analysis "需求分析" (start: "2026-09-01", duration: "10d")
        task dsl "DSL 设计" (after: analysis, duration: "5d", crit: true)
    }
    milestone v1 "v1.0 发布" (date: "2026-10-01")
}

timeline (title: "平台简史") {
    event "2002" "LinkedIn"
    event "2004" "Facebook" "Google"
}
```

任务 id 可被 `after:` 引用（走 id 引用，非 Mermaid 裸字符串）。

### 4.4 双轨：静态糖 ↔ 动态数据

所有糖 desugar 为 record List props，同一组件天然支持数据驱动：

```
// 静态轨:手写声明(文档、演示)
flow-diagram { node/edge 糖 }

// 动态轨:模型驱动(实时拓扑、日志回放、状态机高亮)
flow-diagram {
    nodes: .topoNodes      // List<{ id, label, shape, group }>
    edges: .topoEdges      // List<{ from, to, label, line }>
}
```

D2 的 layers/scenarios/steps **不需要新语法**：场景 = model 里一个 `str`，数据 prop
按 `.scenario` 派生不同 nodes/edges（props 重播种 → Init 重放，ADR-19 链路复用）。
时序图回放日志、甘特图绑任务表、状态图绑 `.phase`，都走动态轨。

### 4.5 语法决策与既有先例映射

| 设计决策 | Auto 既有先例 |
|---|---|
| `edge a -> b (label: ...)` | `on { .Init -> { } }` 箭头语法 |
| `group x "y" { ... }` 块嵌套 | view 的 `col (style: ...) { ... }` |
| `node start "开始" (shape: ...)` 位置文本 + props 括号 | `h1 "标题"` / `circle (cx, cy, r)` |
| 成员行 `+ age int` | model 的 `legendColor0 str = ""` 声明序 |
| `nodes: .list` 数据轨 | `line-chart { data: .monthly }`（484 推荐写法:身份属性入括号、数据入 body） |
| 组件命名 `flow-diagram` | `donut-chart` kebab-case 消费 |
| 导入 `src: """..."""` | `"""` 多行字符串（Plan 169） |
| 事件/绑定 props | `onmouseenter: .Hover(0)`、`checked: .dVisible` |

## 5. 组件契约 v1

视口起步 `viewBox "0 0 560 300"`，随节点数自适应（charts 惯例）。
**record schema（糖与数据轨共同目标形态）**：

```
node  = { id: str, label: str = id, shape: str = "rect", group: str = "", parent: str = "",
          icon: str = "", style: str = "", event: str = "", guard: str = "",
          x: float = -1, y: float = -1 }
edge  = { from: str, to: str, label: str = "", line: str = "solid",
          head: str = "arrow", tail: str = "none", kind: str = "", cardinality: str = "",
          activate: bool = false }
```

`x/y ≥ 0` = 用户钉住位置（布局引擎跳过该节点坐标求解，仅以其为约束布局其余节点）——
可编辑画布（§8 Phase 5）的前向兼容预留，v1 恒为 -1（全自动布局）。

| 组件 | 专有 props | 布局 |
|---|---|---|
| flow-diagram | `direction: str = "td"`，`nodes`/`edges` | 分层布局（§6.1）；group 递归先内后外 |
| sequence-diagram | `actors`(nodes)、`messages`(edges，列表序即消息序)、`fragments: List<{kind, label, start, end}>`（边索引区间）、`notes: List<{text, over: List<str>}>` | 固定泳道（无布局引擎，类比 band scale） |
| state-diagram | `nodes`（`shape: "initial"/"final"` 伪节点、`parent` 复合态）、`edges`（`event/guard`） | 分层布局（初始态为根） |
| class-diagram | `classes: List<{id, members: List<str>}>`、`edges`（`kind`） | 分层布局 |
| er-diagram | `entities: List<{id, fields: List<{name, type, key}>}>`、`edges`（`cardinality`、端点 `"a.b"` 列级） | 分层布局 |
| mindmap | `nodes`（`parent` 层级） | 整洁树布局（§6.1） |
| gantt | `tasks: List<{id, label, start, duration, end, deps, section, crit, milestone}>` | 时间轴（无布局引擎） |
| timeline | `events: List<{when: str, texts: List<str>}>` | 时间轴 |

公共交互 props（§7）：`tooltip: bool = true`、`onselect`/`onhover`（带参 id，Phase 1.5 起）、
`selected: str`/`active: str`（受控绑定）。默认色板沿用四色 + 语义中性灰。

## 6. 渲染架构

### 6.1 v1：SVG 轨（与 charts 484 同路径）

- **布局 = Auto 代码**（对齐"几何 = Auto 代码"原则，nice-ticks 先例）：
  - **分层布局 Sugiyama-lite**：rank（最长路径分层）→ order（重心法 2–4 轮降交叉）→
    coord（层内等距 + 父居中一趟）；`direction td/lr` 经转置；group 先递归布局内部，
    再以整体尺寸作超节点参与外层。全 float 纪律（中间量显式 float）。
  - **整洁树 Reingold-Tilford-lite**（mindmap/组织树）。
  - 验收口径：与 dagre 目检相当（交叉数与间距层级），非像素对齐。
- **图元发射**：Init 把布局+几何写进 model 段记录（`segs List`），view `for seg in .segs`
  循环发射 svg `path/rect/ellipse`——路径与颜色同源，双轨一致（charts 同款）。
- **箭头**：三角形 path，Init 算端点角度；`head/tail` 字形（diamond/circle/cf-*）为
  小多边形，全部手算（donut 弧先例）。
- **边路由 v1**：节点 bbox 边界交点直线段（正交三段路由备选，不阻塞 v1）。
- **标签发射（关键约束）**：svg `<text>` 不支持（chart-components 既有约束）——
  节点/边标签 = **DSL text 绝对定位 overlay**（donut tooltip 同机制：
  `text { style: f"absolute left-[${x}px] top-[${y}px]" }`，f-string px 类 492 定案后
  直写合法）。大图 DOM 膨胀风险见 §10；canvas v2 一并解决。
  **备选（Phase 1 对照评估）**：svg 直通标签集增 `text`——vue 轨直通平凡，VM 轨
  svgdoc 经 resvg 原生支持文本栅格化；若双端验证通过则取代 overlay 为首选
  （随 svg 缩放、零 DOM 膨胀、规避 VM 轨动态 arbitrary 值风险）。

### 6.2 v2：canvas 轨（与 Plan 499 统一，见 §7.4）

canvas Program 桥接（437 v2 决议"Auto 图元列表 → iced canvas"）+ mousemove 流 +
代码命中。diagram 迁移条件与 charts 共用同一判据集（§7.4）。

### 6.3 兼容轨（可选，后置）

`src: """mermaid 文本"""` 不调 mermaid.js 渲染，而是实现**Mermaid 文本 → Auto
records 转译器**（文本 → 结构化数据 → 原生渲染），保双端一致。仅覆盖常见子集，
作为存量文档迁移入口，非长期路线。

## 7. 交互模型与 Plan 498/499 的统一设计

**结论先行：可统一。** 统一面在①交互状态机模式、②图元记录 schema、③canvas v2 桥
与 mousemove 流、④动画双轨；布局算法、DSL 糖、数据形状三处不统一（也无必要）。

### 7.1 统一的五层抽象（chart 与 diagram 同构）

| 层 | charts（437/484/498/499） | diagrams（本文） | 统一度 |
|---|---|---|---|
| 声明层 | 无糖（数据 props 足够） | node/edge/group 糖（Phase 3） | 不统一，不冲突 |
| 数据层 | `data` 单 record 列表 | `nodes`+`edges` 双 record 列表 | 风格统一，schema 各异 |
| 几何层 | Init 纯派生（scale/nice-ticks/arc） | Init 纯派生（布局引擎） | **同一模式**（ADR-19 幂等重放） |
| 图元层 | `segs List`（`{d,c}` 等） | render 段记录（`{kind,d,c,...}`） | **统一 schema 候选（P-list，§7.4）** |
| 渲染+交互层 | v1 svg 直通 + mouse-area 命中；v2 canvas 桥 | 同左 | **完全共享基础设施** |

### 7.2 交互状态机：498 三段式的泛化

498 的"Init 纯派生 + 最小状态写入 + view 状态投影"三段式原样成为 diagram 的模板，
状态族对照：

| 状态族 | 498（charts） | diagram v1 | 关系 |
|---|---|---|---|
| hover | `hoverSeries/hoverIdx`（enter/leave 单行 handler） | `hoverNode str`（同款 enter/leave） | 直接复用模式 |
| emphasis | 系列/分组/扇区 emphasis + 其余 downplay（二态） | **focus 模型**：本节点 + 邻接边/点 emphasis，二度外 downplay（ECharts graph focus） | 498 平面二态的邻接扩展，语义向前兼容 |
| visibility | `visible0..3` legend 点击切换系列显隐 | group collapse/expand（点击组标题收起子图，view 条件跳过） | 同一"离散 toggle + 条件跳过"模式 |
| select | 无（498 不做） | `selectedNode`（click，受控 `selected:` prop 双轨） | diagram 新增（依赖 §7.6 引擎臂） |

### 7.3 命中区模型统一

- charts：竖带（x 索引全高）/ 柱 bbox / 图例行（donut 弧形无法矩形近似）；
- diagram：**节点 bbox**（布局后 Init 已知）→ **绝对定位 mouse-area 兄弟层**
  （svg 兄弟层 mouse-area 是常规 widget，不经 svgdoc 序列化——484 增量），与
  donut tooltip overlay 同机制；
- **边命中 v1 不做**（细长斜线无法矩形近似，与 donut 扇区同因）→ v2 canvas 代码
  命中统一解决：**499 的 donut 逐扇区命中（路径 b）与 diagram 边命中是同一能力**
  （is-point-in-arc / point-to-polyline 距离测试），落地一次两家族受益。

### 7.4 canvas v2（499）的统一面

- **图元列表协议（P-list）**：charts 的 `segs` 与 diagram 的 render 记录归一为
  `{ kind: "path"|"rect"|"ellipse"|"text", geom/d, fill/stroke tokens }`——
  437 v2 决议的"Auto 图元列表 → iced canvas Program"桥**只实现一次**，两家族消费。
  建议作为 499 M1 调研的增补输入（见 §7.5）。
  **P-list 是内部桥接协议而非终点**：未来通用 `canvas` DSL 容器（可编辑画布 /
  a2ui-composer 方向，§8 Phase 5）以其为基底，组件层不感知。
- **触发判据扩展**：499 现判据"svgdoc 序列化性能不达标"；diagram 增补两条——
  高频 `.Tick` 流式重绘（charts/diagram 共有）、**大规模节点图**（初判 ≥80 节点，
  阈值待压测定）。
- **mousemove 限频流**（≤30Hz，Plan 386 RenderQueue 批量通道）：axisPointer 十字线
  与 diagram 边跟随命中/节点拖拽**共用同一通道**。
- **坐标语义**：mousemove 等指针事件携带**组件局部逻辑坐标**（viewBox 坐标系），
  屏幕 → 逻辑的换算在引擎层完成；未来 canvas 容器的 pan/zoom transform 是其上
  一层映射，组件代码不感知。499 M1 增补输入③。
- **动画**（499 M5 双轨策略：vue CSS transition / VM 数值插值）：直接覆盖 diagram
  布局动画（数据变化 → 节点位置插值）与 focus 过渡。
- **对 diagram 的特殊意义**：canvas 轨不只是性能，还是**文本发射的正解**——svg 轨
  无 `<text>`，节点标签 v1 只能 DSL text 绝对定位 overlay（大图 DOM 膨胀）；canvas
  Program 原生文本绘制一举解决。

### 7.5 对 498/499 的影响与边界

- **对 498：无冲突，纯受益**。diagram 以 498 状态机为直接模板；498 无需任何改动。
- **对 499：已直接修订 499 计划**（drafting 期改动成本最低），M1 调研输入增补三条：
  ① P-list 图元协议需同时满足 charts `segs` 与 diagram render 记录（§7.4 schema）；
  ② donut 扇区命中与 diagram 边命中共用 canvas 代码命中路径；
  ③ 指针事件坐标语义（§7.4 坐标语义条）。mousemove 流按**通用指针事件原语**设计
  （charts/diagram 共消费），不做 chart 私有通路——499 即统一 canvas 交互原语的
  先导落地计划。
- **不统一处（也无必要）**：布局算法（scale vs 图布局引擎）、DSL 糖（diagram 独有
  Phase 3）、数据形状（单表 vs 双表）。统一是"共享模式与基础设施"，**非合并组件**——
  chart 家族契约（chart-components.md）不动。

### 7.6 引擎层缺口：mouse-area 无 onclick

`View::MouseArea` 现有 `on_enter/on_exit/on_double_click`（496 M5 增 dblclick），
**无 on_click**（`crates/auto-lang/src/ui/view.rs:524`）。该缺口同时命中：

1. **498 M4**：legend 点击切换显隐写的是"`onclick` 翻转"，但图例命中原语 mouse-area
   无 onclick 臂——498 落地时必然撞上（用 button 伪装或补引擎臂二选一）；
2. **diagram select**：节点选中交互同依赖。

**建议**：`View::MouseArea` 增 `on_click` 字段（vue `@click` / iced mouse_area
`on_click`，496 ondblclick 同款先例，小引擎改动）——**一次扩展，两计划受益**。
时序建议放在 diagram Phase 1.5（或 498 M4 实施时 whichever 先行，另一计划消费）。

## 8. 落地路线（plan 拆分建议；编号以 `new-plan.sh` 取号为准）

| 阶段 | 内容 | 引擎改动 | 依赖 |
|---|---|---|---|
| **Phase 1** | flow-diagram 组件 + 分层布局 + v1 SVG 渲染 + hover tooltip/emphasis（498 模式）+ gallery/024 示例 | **零** | 484/492 既有通路 |
| **Phase 1.5** | mouse-area `on_click` 臂（§7.6）+ diagram select/受控 `selected:` | 小（View::MouseArea + 双端映射） | 496 先例；与 498 M4 协同 |
| **Phase 2a** | sequence/state/er/class（结构类其余，state 含 `active:` 实时绑定） | 零 | Phase 1 布局核 |
| **Phase 2b** | mindmap/gantt/timeline（树/时序类） | 零 | — |
| **Phase 3** | DSL 糖（编译器：组件消费块内 `node/edge/group` 词汇，desugar 到数据轨）+ 双轨等价性测试 | parser/view 层 | Phase 1/2 组件就位 |
| **Phase 4** | canvas v2 迁移（P-list/边命中/布局动画）+ mermaid 文本转译兼容轨（可选） | 跟随 499 | **499 落地** |
| **Phase 5** | 交互编辑画布：节点拖拽/连线/框选/pan-zoom（通用 `canvas` 容器原语 + 带坐标指针事件，坐标语义见 §7.4）；远景 = a2ui-composer 式界面编辑器 | 大（DSL 原语 + 双端 canvas） | Phase 4 + 499 指针事件流 |

## 9. 显式非目标

- 不引 mermaid.js/d2/任何第三方 diagram 渲染依赖（双端一致性，437 结论同款）。
- v1 不做：边 hover、节点拖拽、pan/zoom 视口导航（归 Phase 5 canvas 容器）、
  布局引擎切换（elk 端口）、journey/C4 专用语法/gitGraph/kanban、图标库托管
  （icon 只收本地/URL 路径）。
- 不改 chart 家族现有契约；统一指共享模式与基础设施（§7.5）。
- 自动布局不承诺 dagre 像素级对齐（目检相当口径）。

## 10. 已知风险与开放问题

1. **标签发射**：svg 无 `<text>` → 绝对定位 text overlay。动态 `left-[${x}px]` 类在
   vue 轨 Tailwind arbitrary value 无虞；**VM/iced 轨对动态 arbitrary 值的解析支持面
   需核验**（donut tooltip 只用了固定值 `top-[20px]` 先例）——Phase 1 首个验证项，
   回退方案 = R006 式槽位字段。备选：svg 直通标签集增 `text`（§6.1），与 overlay
   同批对照评估，通过则升级为首选。
2. **布局算法质量**：Sugiyama-lite 在稠密图上交叉数可观；接受目检相当口径，
   `layout` prop 预留 `"auto"` 以外的将来值。
3. **gantt 日期**：`"YYYY-MM-DD"` str 手工 split 解析，无时区/时长日历语义
   （excludes weekends 等 Mermaid 能力不做）。
4. **糖语法（Phase 3）**：组件消费块新增元素词汇需 parser 扩展；糖/数据双轨等价性
   用确定性断言（固定输入对照 desugar 输出）。
5. **大图性能阈值**：≥80 节点初判无压测支撑，进 499 canvas 判据后以实测修正。
6. **开放问题**：sequence 的 span/激活条与 fragment 区间的渲染细节（D2 用嵌套对象
   表达，Mermaid 用 +/- 记号，本文 `activate: bool` 简化——多段激活/嵌套激活 v2 再议）。
