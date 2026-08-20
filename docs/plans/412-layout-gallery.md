# Plan 412 — Layout Gallery（布局专区）+ VM 布局引擎对齐

> **状态（2026-08-20 核查）**: 🟡 Phase 1-2/结构通道 ✅（`4bfd6d27`/`6ee1a5e1`/`f51c8882`——12 个 layout 页全部落地 + `rederive_layout` 全路径 + `grid_row_placements` 分配器 + plan412_tests）。**§6.2/§6.3 视觉+交互验证待桌面会话补跑**（§10.4）：全页双端并排截图与像素测量（≤1px）、scroll/Overlay 交互抽验未执行，§9.2/§9.3 验收标准未闭环。

> 2026-08-14 立项。widgets-gallery 现有 50 页全部展示"组件"，缺"布局"专区。
> 目标：新增 Layout 分组（row/col/center/flex/grid 等，参照 Tailwind 布局能力全集），
> **硬约束：同一份 .at 源码，vue 与 VM 两端行为与 UI 完全一致**（Plan 411 的延续）。
> 本计划先补 VM 布局引擎差距（§2），再落 gallery 页面（§4），最后全页 parity 验证（§6）。

## 0. 调研结论（双端现状矩阵）

### 0.1 语义布局元素（Auto DSL 第一等公民）

| 元素 | vue codegen | VM view_builder | parity |
|---|---|---|---|
| `row` | `flex flex-row`(+gap) | `View::Row`(spacing) | ✅ |
| `col`/`column` | `flex flex-col`(+gap) | `View::Column` | ✅ |
| `center` | `flex flex-col items-center justify-center h-full` | `convert_center` | ✅ |
| `grid` | `div.grid` + `cols`→`grid-cols-N` + `gap`→`gap-N`（**gap 取 int**，与 row/col 的 str 不一致） | `View::Grid`→`build_grid` 手动分行（cell Shrink，轨道非等宽） | ⚠️ 视觉近似 |
| `scroll` | `overflow-auto` | `View::Scrollable` | ✅ |
| `container`/`div` | `<div>` + style 类透传（**CSS 全能力**） | `View::Container`（**flex/grid 类丢失布局语义**，仅背景/边框/尺寸生效） | ❌ 核心差距 |
| `spacer` | `<div>`（无尺寸类时无效果） | `Space` | ⚠️ 需对齐语义 |
| `aside/main/header/nav/section/footer/article` | 语义标签+默认类 | `convert_container` | ✅ |

### 0.2 Tailwind 布局类 × VM 解析器/渲染器现状

**已支持（解析+消费）**：`flex` `flex-col` `flex-row` `flex-1` `items-{start,center,end}`
`justify-{start,center,end,between}` `gap-N`（小数/任意值）`w/h/min-w/min-h/max-w/max-h`
`p/m` 全系 `mx-auto` `hidden`+display 覆盖 `overflow-*` 全系 `shrink-0` `relative` `absolute`(解析,渲染降级) `z-N`(解析)

**已解析、VM 渲染忽略**（`iced_adapter.rs:703+` 注释 "Iced doesn't support grid - store but will be ignored"）：
`grid` `grid-cols-N` `grid-rows-N` `col-span-N` `row-span-N` `col-start` `row-start`

**完全缺失（解析都没有）**：`flex-wrap/wrap-reverse/nowrap` `flex-row-reverse/flex-col-reverse`
`flex-auto/flex-initial/flex-none` `grow/grow-0` `shrink`(非0) `basis-N` `order-N`
`justify-around/evenly` `items-baseline/stretch` `self-*` `place-*` `content-*`
`gap-x-N/gap-y-N` `space-x-N/space-y-N` `inset-N/top/right/bottom/left-N` `fixed/sticky`
`aspect-*` `object-*`

### 0.3 iced 0.14 能力边界（VM 的物理约束）

- ✅ `iced_widget::grid`：原生 Grid widget（`columns(n)`+`spacing`），**无 span API**（逐 cell 顺序排布）
- ✅ Row/Column + spacer 组合可实现 `justify-around/evenly`（build_row 已有 spacer 机制，补 around/evenly 模式）
- ✅ 交叉轴 Fill = `items-stretch`；`float`/`pin`/`responsive` widget 存在（overlay 定位可用 Stack，见 Plan 409 §10 续 5）
- ❌ 无 wrapped_row（**flex-wrap 不可原生实现**）
- ❌ 无绝对定位（absolute/inset 无对应；已有降级先例：主题色板 popup "VM ignores absolute"）
- ❌ 无 per-child 对齐（self-*）、无 baseline、无 order

### 0.4 已暴露的存量 bug（Layout gallery 会直接踩中）

- **现有 `/grid` 页**（pages/grid.at）demo 用 `col (style: "grid grid-cols-3 gap-4")` 写法：
  vue 正常 CSS grid；VM 渲染为纵向堆叠（§0.2 忽略项）——目前两端已经不一致。

## 1. 收录原则（约束推导）

> "理论上 Tailwind 支持的都应该支持" × "vue/VM 完全一致" ⇒

1. **Demo 只收录双端可一致表达的能力**。VM 物理不可实现（wrap/absolute 定位/order/baseline）
   的进"降级矩阵"文档化（§5），不进 demo 页——demo 页两端必须像素级一致。
2. **语义元素优先，style 类为辅，两者都要有 demo**：语义元素（row/col/grid/center/scroll）
   是 Auto 的推荐写法；style 类（grid-cols-N/items-*/flex-1…）是 Tailwind 通路。两套都要覆盖，
   它们天然构成 parity 测试矩阵。
3. **降级必须"显式"**：VM 对不支持的类应渲染成可辨识的降级形态（如 wrap 单行），
   并在 demo 页脚注标注——不静默歪曲。

## 2. Phase 1 — VM 布局引擎补齐（gallery 前置，7 项）

### F1 容器布局语义重派生（最高优先，解决 /grid 存量不一致）

`convert_container`/`convert_container_tracked_ctx`（含 div/aside/header 等 → container 的路径）
按 style 类重派生：

```
style 含 FlexCol   → View::Column（children 递归）
style 含 Flex      → View::Row
style 含 Grid+GridCols(N) → View::Grid { cols: N, gap: gap 类, cells: children }
```

style/背景/边框类保留在结果 View 的 style 上。这使 `col (style: "grid grid-cols-3 gap-4")`
（CSS grid 写法）在 VM 获得真实网格。**响应式前缀剥离已生效**（Plan 411 P0-B），
`md:grid-cols-2` 自动参与。col/row 元素带交叉类（如 col + flex）同样走重派生仲裁
（元素语义优先，类做覆盖）。

### F2 build_grid 等宽轨道 + col-span 行分配器

现状 build_grid 用 Row-chunk、cell Shrink（紧凑但不等宽）。升级：

1. **等宽轨道模式**：每 cell 包 `Fill` 宽（grid-cols 语义 = 等分轨道）；
   保留"cell 自带显式宽度时不覆盖"（w-8 h-8 按钮格场景）。
2. **col-span 支持**：View::Grid cells 增加 `span` 元数据（cell 级 style 读 `col-span-N`）。
   行分配器（CSS auto-placement 简化版）：顺序填充，`当前列 + span > cols` 则换行；
   span cell 宽度 = `span×轨道宽 + (span-1)×gap`（实现：span cell 放入"子 Row[cell + (span-1)×空Fill]"共享外层等分？——不可行，改为**分配器直接产出二维占位表**，渲染为嵌套行列，span cell 跨多列合并）。row-span 第一版不做（降级为 col-span）。

### F3 缺失类解析（class.rs + StyleClass 枚举）

新增：`gap-x-N/gap-y-N`（小数/任意值同 gap）`space-x-N/space-y-N` `justify-around`
`justify-evenly` `items-stretch` `flex-row-reverse` `flex-col-reverse` `flex-auto`
`flex-initial` `flex-none` `grow` `grow-0` `shrink`（非0）`flex-wrap/wrap-reverse/nowrap`
（解析+标记，渲染降级）`self-*`（解析+降级）`order-N`（解析+降级）`inset/top/right/bottom/left-N`
（解析，VM 降级）`fixed/sticky`（解析，VM 降级）

### F4 adapter/渲染映射

- `justify-around`：两端各半格 spacer + 等分（build_row spacer 机制扩展）
- `justify-evenly`：等分 spacer 含两端
- `items-stretch`：交叉轴 Fill
- `gap-x` on Row / `gap-y` on Column → spacing（另一轴忽略）；`space-x/y` 等价 spacing
- `flex-row-reverse/col-reverse`：children 反序（build 期）
- `grow/grow-0/flex-auto`：主轴 Fill 语义细化（flex-auto=basis auto≈Fill；flex-none=Shrink）
- 降级矩阵类：`flex-wrap` 等标记后**保持单行**渲染 + eprintln 一次性提示（开发期可见）

### F5 grid element prop 语义统一（vue 侧）

`grid (cols: N, gap: M)` 的 vue 生成已有；gap prop 取值路径与 row/col 不一致（int vs str）
——Layout 页一律用 `style: "grid-cols-N gap-N"` 写法（Plan 411 gap 决策的延续），
元素 prop 仅在文档中标注为等价捷径。

### F6 spacer 对齐

VM `Space` ↔ vue `<div class="flex-1">`（spacer → div 补 flex-1 默认类），demo 用
`spacer (style: "w-8")` 定宽形式为主。

### F7 CATEGORY_COLOR 扩色

新增 `sky`（#0ea5e9）到三处映射：vue index 卡片色板（vue.rs:1461 附近）、VM
category-section 色点（aura_view_builder dot_bg match）、VM component-card 色板
（border/bg/icon 三元组）。

## 3. 范围裁剪（明确不做 + 理由）

| 能力 | 决定 | 理由 |
|---|---|---|
| flex-wrap 换行 | ❌ demo 不收录；VM 降级单行 | iced 0.14 无 wrap widget；自定义 Widget 超计划范围。**换行需求用 grid-cols 表达**（天然换行，双端一致） |
| absolute/inset 定位 | ❌ demo 不收录 | iced 无绝对定位；分层用 Overlay/Stack（position 页演示 Overlay 版） |
| order-N / self-* / items-baseline | ❌ demo 不收录 | iced 无 per-child 排序/对齐/baseline |
| row-span | 第一版不做 | 分配器二维占位已备，但组合场景少；col-span 先行 |
| aspect-ratio / object-fit | ❌ | 双端均无对应，后续独立项 |
| sticky/fixed | ❌ demo | VM 无定位；header sticky 属应用级布局非组件 demo |

## 4. Phase 2 — Gallery 页面（12 页）

### 4.1 路由与分组

- **路由平铺**（与现有 50 页同构，避免嵌套路由匹配新增面）：`/row` `/col` `/center`
  `/flex` `/alignment` `/spacing` `/sizing` `/scroll` `/position` `/responsive`
  `/grid`（**重写**，迁入 Layout 分组）`/grid-span`
- **app.at**：routes 块 +12（`/grid` 复用）；sidebar 新增 `Layout` 分组，插在
  Overview 之后、Form 之前（布局是最基础概念）；nav-link 图标建议：
  row=`arrow-right` col=`arrow-down` center=`plus` grid=`layout-grid`
  grid-span=`square-stack` flex=`move-horizontal` alignment=`align-center`(缺则用 settings)
  spacing=`space`(缺则 `sidebar`) sizing=`ruler`(缺则 `frame`) scroll=`chevrons-down`
  position=`layers` responsive=`monitor`(缺则 `image`)
  ——lucide_svg 表缺的图标随 F7 一并补
- **index.at**：新 category-section `(name: "Layout", color: "sky", count: 12)` +
  12 张 component-card（置顶，在 Form 之前）

### 4.2 每页 demo 矩阵（preview-card 骨架 + 双写法覆盖）

| 页 | 示例组（每组一个 preview-card） |
|---|---|
| /row | Basic（3 色块）/ gap 阶梯(gap-1/2/4/8) / items-{start,center,end} / justify-{start,center,end,between} / 嵌套 row-in-col |
| /col | Basic / gap / items / justify / 嵌套 col-in-row |
| /center | 纯内容居中 / 双向居中(col+center) / 尺寸受限容器内居中(max-w + center) |
| /flex | flex-1 等分 / flex-none 固定+flex-1 弹性 / grow 对比 / 嵌套伸缩(2 级 flex-1) |
| /alignment | items×justify 3×4 矩阵（12 小格，固定高容器内 3 色块）——一图看清全部组合 |
| /spacing | p-{2,4,8} 阶梯 / m 对照（可视化 margin 用嵌套底色容器）/ gap vs space-x / gap-x,gap-y |
| /sizing | w-{16,32,full} / h-{8,16,full} / min/max-w 演示（max-w 截断长条）/ w-1/2 类分数宽（缺则 Fixed px，见 §2 F3） |
| /scroll | 纵向固定高滚动(8 项) / 横向滚动(6 卡) / 双向 |
| /position | relative 容器内 z-index 叠放（重叠色块 z-10/20/30，VM 用同 z 嵌套序表达）/ Overlay 分层演示（Plan 409 §10 续 5 Stack 语义） |
| /responsive | hidden md:flex 显示切换 / grid-cols-1 md:2 lg:4 / text responsive 阶梯——页面注脚说明 VM 按窗口逻辑宽度实时断点（resize 重建已支持） |
| /grid（重写） | 语义元素版 `grid (cols:3)` / style 版 `style:"grid grid-cols-3 gap-4"`（F1 验收页）/ 响应式列数 / 等宽 vs 内容宽对照 |
| /grid-span | col-span-2 / col-span-3 / 混排画廊（span+常规 cell，F2 验收页） |

### 4.3 demo 编写规约

1. 每页结构沿用组件页骨架：`h1` + 描述 + 若干 `h2`+`preview-card` + `Properties` table
   （布局页的 table 列类/元素属性，如 `Class | Values | Description`）。
2. **色块辅助元素**：demo 通用"占位块" = `col (style: "h-12 w-12 rounded-md bg-{色}-500/40 border border-{色}-500")` 内居中数字——复用 /grid 页现有写法。
3. 同一能力**语义写法与 style 写法成对出现**（各一个 preview-card），Auto/Vue tab
   双代码对照由 preview-card 既有机制自动生成。
4. 页面描述注明降级项（如 /position 页脚注：absolute/wrap 类在 VM 的行为）。

## 5. 降级矩阵（文档化，双端行为对照表）

写入 /position 页描述 + Plan 本节：

| Tailwind 类 | vue | VM |
|---|---|---|
| flex-wrap | 换行 | 单行（不换行） |
| absolute + inset-N | 绝对定位 | 就近布局位（无定位） |
| order-N | 重排 | 源码序 |
| self-* | 单 child 对齐 | 继承容器 items |
| items-baseline | 基线对齐 | 降级 center |
| row-span | 跨行 | 忽略（占 1 行） |
| fixed/sticky | 视口固定 | 就近布局位 |
| aspect-* | 宽高比 | 忽略 |

## 6. Phase 3 — Parity 验证流程（每页执行）

沿用 Plan 411 双通道验收，逐页：

1. **结构断言**：MCP `autoui_snapshot`/`vtree` —— VM 端 Row/Column/Grid 节点数、
   spacing/cols 值与源码预期一致；`autoui_find` 抽查关键 cell。
2. **视觉对照**：vue（浏览器 1440×900，与 VM 窗口 pac.at `window: "1440x900"` 一致）
   vs VM `autoui_screenshot` 并排合成图 + 像素测量（色块 x/y/宽度差 ≤1px 阈值；
   gap 值、轨道等宽、span 跨列宽度逐一测）。
3. **交互抽验**：scroll 页滚到底 / responsive 页记 `__current_route` 外无 state；
   position 页 Overlay 开合。
4. **回归**：`cargo test -p auto-lang --features ui-iced --lib -- plan409 plan411 plan412`
   （新增布局类解析/重派生/分配器单测）+ auto-man 200 测试。
5. Plan 411 的 MCP 工具缺陷仍适用（视觉模型误报教训：以像素测量与 vtree 为准）。

## 7. Phase 4 — 收尾

- widgets-gallery README：50→62 页、新增 Layout 分组说明、降级矩阵链接
- Plan 409 §10 系列 + 411 + 412 的 parity 工作流总结进 docs/guides（可选）
- 全量提交分四个逻辑 commit（F1-F4 引擎 / F5-F7+页面 / 验证记录 / 文档）

## 8. 工作量与风险

| 项 | 规模 | 风险 |
|---|---|---|
| F1 容器重派生 | 中（两臂 ×3 派生 + 测试） | 低——纯增量，不改既有元素路径 |
| F2 等宽轨道+span 分配器 | 大（新算法） | 中——分配器边界（尾行填充、span>cols 钳制）；先单测后集成 |
| F3/F4 类补齐 | 中（~25 类 × 解析+映射） | 低 |
| 12 页 demo | 大但机械 | 低——沿用骨架；注意每页 demo 双写法 |
| 全页 parity | 中 | 视觉回归靠像素测量自动化，别信视觉模型 |

## 9. 验收总标准

1. `/grid` 存量不一致修复（F1 验收）：vue 与 VM 同截图网格行列一致。
2. 12 页全部双端并排截图入 `tmp/`，色块位置差 ≤1px、gap/轨道宽像素级一致。
3. `grid-span` 页 span 跨列宽度 = N 轨道 + (N-1)gap（像素验证）。
4. 降级矩阵类在 VM 无 panic、无静默大变形（单行/就近位）且 eprintln 提示可开。
5. 新旧测试全绿；README/路由/侧边栏/首页卡片四处处登记完整。

## 10. 实施与验证记录(2026-08-14)

### 10.1 实施内容(四个 commit)

| commit | 内容 |
|---|---|
| 1. 引擎(F1-F4/F6 + F7 图标) | class.rs 新类解析(~30 类);iced_adapter(IcedJustify 扩 Around/Evenly、轴 gap、reverse、stretch、降级一次性 eprintln);renderer(justify FillPortion spacer 数学、build_row/column 反序+交叉轴 Fill、build_grid 等宽轨道+槽位 padding 补偿+`grid_row_placements` 分配器);aura_view_builder(`rederive_layout` 重派生 + container/col/row 双胞胎同步 + grid 元素类优先);spacer 默认 w-full h-full;lucide 表 +8 图标 |
| 2. F5/F7+页面 | vue.rs grid gap prop 像素语义(16px→gap-4 / gap-[Npx]);sky 三处色板;app.at 11 新路由+Layout 分组;index.at Layout 12 卡+Display 迁 Grid;12 页 demo;plan412_tests(单测+集成共 20 项) |
| 3. 验证记录 | 本节 |
| 4. 文档 | README 62 页/Layout 分组/降级矩阵指引;计划归档 |

### 10.2 验证结果(结构通道全绿;视觉通道见 10.4)

- **单测**:`cargo test -p auto-lang --features ui-iced --lib -- plan412` → 20 项全过:
  - 解析:gap-x/y、space-x/y、justify-around/evenly、items-stretch、self-*、flex 变体(flex-auto/initial/none、grow、shrink、wrap 系、reverse)、inset/order/fixed/sticky、md: 前缀叠加;
  - 重派生:div/col+grid 类 → `View::Grid{cols:3,gap:16}`、flex-col → Column、flex → Row、交叉仲裁(col+flex→Row / row+flex-col→Column)、grid-cols 类覆盖 prop、多断点取最后声明、无布局类 div 不回归;
  - 分配器:尾行不满、span 换行、span 钳制到 cols、整行 span、混排序列(纯函数 `grid_row_placements` 单测);
  - 集成:12 页全部构建(VM view 树根为 Column);/grid 页 style 写法重派生为真实 3 列 16px 网格 + 响应式 4 列;/grid-span 页 cell 携带 col-span-2/3 元数据;app.at 12 路由登记;12 页 vue SFC 生成含关键类(grid-cols-3、col-span-2、md:grid-cols-2 等)。
- **整站生成**:`auto build --render vue --gen-only` → 62 组件 / 61 页;App.vue 含 Layout 分组全部 nav-link;router/index.ts 含 12 条新路由;index.vue Layout 分组 sky 色板(vue/VM 两端);`grid (cols:3, gap:16)` 生成 `gap-4`(F5 修复后与 VM 的 16px 一致)。
- **回归**:`--lib --skip plan370_015 --skip plan370_store_vm` → 3361 passed / 30 failed,与 master 基线(3341/30)逐项一致——失败均为存量(dstr_tests、vue tests、vm_bridge 等);plan370_015/plan370_store_vm 在 master 同样栈溢出(存量损坏,非本计划引入);vue_capabilities 的 cap_vmodel_fold 同为存量失败。

### 10.3 等宽轨道的实现要点(F2 精确性论证)

等分槽 + gap 按槽位分摊:每槽宽 W/cols(FillPortion),cell(start 列 c、span s)左 padding = c·g/cols、右 padding = (cols−c−s)·g/cols。推导:相邻 cell 间隙恒为 g;span cell 宽 = s·W/cols − (cols−s)·g/cols = s·轨道宽 + (s−1)·gap,与 CSS grid 几何像素级一致(f32 padding,<1px 误差)。尾行补 FillPortion(剩余列) 空槽,保证每行总 portions = cols(否则尾行 cell 被摊宽)。cell 带显式 w-*(如色板 w-8)时整格回退 compact 模式(Plan 402 §13.10 / Plan 411 色板 parity 不回归)。

### 10.4 未完成项(视觉通道)

§6.2 并排截图 + 像素测量、§6.3 交互抽验需交互式运行 VM 窗口(autoui MCP)与浏览器,本轮实施会话无 GUI/MCP 环境,未执行。结构断言(§6.1)已全部通过。建议桌面会话补跑:`auto run`(vm)与 vue dev server 逐页对照 12 页,重点:/grid 行列一致、/grid-span 的 span 宽度 = N 轨道 + (N−1)gap、/alignment 的 around/evenly 分布、/scroll 滚到底。
