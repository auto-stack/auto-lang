# Plan 411 — VM 端视觉对齐 vue（Home + Button 页深度对比）

> **状态**: ✅ COMPLETE(2026-08-22 归档复核)。P0-A/P0-B/P1-A/P1-B/P1-C/P2-A①②③④/P2-B 四项全部落地:响应式前缀类/窗口宽度/active 高亮/toast overlay(08-14/08-15 批);P2-B Button.content 序列化(`f7658c45`)+ autoui_check 对齐与快照过滤(`6fbcbe40`)+ layout 回填(`30fc2d08`);P1-C Inter 三字重内嵌(`d2748a24`)+ P2-A① prism-tomorrow 色板与 P2-A④ 表格细节(`535b291d`)。遗留两项为显式设计决议,登记债务簿:§8.5 gap 兼容分支保留(vue.rs 3 处 + view_builder 8 处,向后兼容决议)+ validator 白名单未加;Inter 与 vue 并排字形截图人工核对未执行(非阻塞,渲染由实机 038/013 回归覆盖)。

> 2026-08-14。对 `examples/widgets-gallery` 同一份 Auto 源码的两种转译产物做
> 逐区域对比（vue dev server @3024 vs `auto run -r vm` iced 原生窗口 @1400×1050），
> 聚焦 Home（`/`）与 Button（`/button`）两页。本文登记差距、根因（已用插桩 +
> AutoUI MCP vtree/snapshot/inspect 验证）与强化方案。
> Plan 409 §10 系列已完成 icon-only button / onClick handler / 主题色 / 路由 /
> preview-card+codeblock 识别等修复，本文是其续篇。

## 0. 对比方法与证据等级

| 手段 | 说明 | 可靠性 |
|---|---|---|
| vue Playwright DOM snapshot | 浏览器端无障碍树（结构事实） | 高 |
| VM `autoui_snapshot` / `autoui_vtree` / `autoui_inspect` | VM 渲染树序列化 | 高（但 vtree 不含 Button.content/Image 子树，见 §4.4） |
| 同视口 1400×1050 并排截图 + 视觉模型 | 布局/颜色/字号差异 | 中（出现过两次误报，见 §1.0，必须与结构数据交叉验证） |
| eprintln 插桩（view_builder / renderer） | 运行时路径取证 | 最高 |

### §1.0 教训：视觉模型误报

对同一张 VM Home 截图，两次视觉分析均声称「卡片无 icon、无描述」；插桩证明
`component-card` 的 Element-arm handler 正常执行（name/desc/icon 提取全对、
`content.is_some()=true` 到达 renderer），高倍放大复核确认卡片**实际渲染了**
icon 方块 + 标题 + 灰色描述。**结论：视觉差异清单必须经 vtree/snapshot 或
插桩交叉验证后才能立项。**

## 1. Home 页（`/`）差距清单

结构上 VM 已与 vue 对齐：badge「v1.0 — 49 Widgets」、h1、双行描述、
Get Started/Browse Widgets、搜索框、6 个 category-section 标题（色点+计数）、
49 张 component-card（icon+标题+描述）、footer。真实差距如下：

| # | 差距 | vue 表现 | VM 表现 | 根因（§3） |
|---|---|---|---|---|
| H1 | 桌面端汉堡菜单泄漏 | `md:hidden` → 隐藏 | 仍渲染（menu icon + Auto UI 行） | R1 |
| H2 | hero 主标题过小 | `lg:text-7xl`=72px | 36px（`text-4xl` 生效，`md:text-5xl lg:text-7xl` 丢弃） | R1 |
| H3 | hero 垂直/水平留白小 | `py-20/24`、`px-6` | py-12(96px)、px-4(16px) | R1 |
| H4 | 描述/正文字号偏小 | `text-base md:text-lg` 等 | 只取基础档 | R1 |
| H5 | 卡片网格列数 2 vs 4 | 1400px → `xl:grid-cols-4` | 2 列 | R2 |
| H6 | header 内边距 px-4 vs px-6 | `px-4 md:px-6` | 16px | R1 |
| H7 | 字体族 | Inter/系统 UI 栈 | iced 默认（Segoe UI） | R5 |
| H8 | 卡片列间距/列对齐 | grid gap-3 | 12px row 间距已对，列数错导致整体观感差异 | R2 |
| H9 | 图标观感 | lucide SVG 矢量 | 已渲染 lucide SVG 染色（✓，本页无差距） | — |

> H9 佐证 Plan 409 §10 续 的 icon 染色修复已生效（badge layers icon fg=#7679f3）。

## 2. Button 页（`/button`）差距清单

结构对齐度高：标题/描述/Installation codeblock（bash 标签+命令）/4 个示例
（Simple、Variants×6、Sizes×3、Events）/Auto-Vue tab + ▼ 折叠 + 代码文本/
Properties 表格 5 列×6 行数据完整/`showClickToast` handler 可执行。差距：

| # | 差距 | vue 表现 | VM 表现 | 根因（§3） |
|---|---|---|---|---|
| B1 | 侧边栏无当前路由高亮 | `router-link` 自动 `[active]`（bg-accent 圆角块） | 当前页与普通项无区别 | R4 |
| B2 | codeblock 语法高亮配色 | Prism 多色（关键字/字符串/包名不同色） | 已有 Rich 高亮但色彩偏少/对比弱 | R6 |
| B3 | codeblock 无 Copy 按钮 | 每块右上 Copy | 无 | R6 |
| B4 | preview-card 折叠按钮文案 | 「Code」文本按钮 | 「▼」符号按钮 | R6 |
| B5 | Events 无 toast 反馈 | vue-sonner 弹 toast | handler 执行（state 翻转）但无 UI | R7 |
| B6 | 表格视觉细节 | 表头字重/行分隔线/单元格内边距 | 数据正确，样式略糙（行分隔弱、列 padding 小） | R6 |
| B7 | 汉堡菜单/响应式（同 H1） | 隐藏 | 泄漏 | R1 |
| B8 | 交互 | tab 切换/折叠/导航均 ✓（`__preview_tabs button-sizes auto` 等 handler 验证通过） | — | — |

## 3. 根因分析（均已实证）

### R1 响应式前缀类未解析（影响面最大）

`StyleClass::parse_single`（`ui/style/class.rs:756`）只匹配**裸类名**：
`"hidden"` → `Hidden`，但 `"md:hidden"`、`"lg:text-7xl"`、`"md:px-6"`、
`"text-base md:text-lg"` 中的带前缀变体全部静默丢弃。VM 窗口按桌面语义，
导致 H1–H4、H6、B7 一整族差距。`is_hidden`（`ui/style/mod.rs:63`）的
`hidden md:flex` display-覆盖逻辑是对的，但前提是前缀类先被解析进来。

### R2 WINDOW_WIDTH 初始化时序 → 响应式列数错误

- `WINDOW_WIDTH` 线程局部默认 **1024.0**（`ui/style/iced_adapter.rs:33`），
  `state.window_size` 也初始化为 1024×768（`iced/renderer.rs:3810`）。
- `set_window_width` 仅在 renderer 渲染期（`renderer.rs:5034`）同步；而
  category-section 的 cols 计算发生在 **view 树构建期**（`aura_view_builder.rs`
  「续 11/14」处，按 `<640/1024/1280` 取 1/2/3/4 列）。
- 窗口实际 1400px，但首次建树用 1024 → cols=2；窗口创建即 1400 不触发
  resize 事件、且 resize 也未必重建 view 树 → 列数永远停在 2。

### R3（已排除）component-card 内容丢失 —— 不存在

见 §1.0。vtree 序列化不含 Button.content（vtree 只有 button/col/container/
input/row/scrollable/text 七种节点），曾误导排查方向。

### R4 侧边栏 active 态

vue 侧 `router-link` 自带 active class（bg-accent）；VM `render_link_button_
with_icon`（nav-link 分支）不读 `__current_route`，无高亮。

### R5 字体族

vue 栈 `Inter, system-ui...`；iced 默认 family。字重映射（font-medium/
semibold）已有，family 未对齐。

### R6 codeblock/preview-card/table 细节

`generate_codeblock_html`（vue.rs）用 Prism 主题色 + Copy + 「Code」按钮；
VM 的 Rich 高亮 lexer（commit 23cd60463）色板简单，无 Copy，折叠按钮用 ▼，
table 行分隔/padding 是独立样式映射。

### R7 toast 无 VM 等价物

`toast()` 是 vue-only（handler_codegen.rs `rewrite_expr` 降级为 false），
VM 无 toast-provider 渲染路径。已有 iced Stack 分层经验（Plan 409 §10 续 5
主题色板 Popup）可复用。

## 4. 强化方案（按优先级）

### P0-A R2 修复：窗口宽度可用性（小改动，立竿见影 H5/H8）

1. `state.window_size` 初始化为 iced `window::Settings` 的实际 size
   （创建处已知 1400×1050），消除 1024 默认值首帧错误。
2. `iced` `WindowResized` 事件 → 更新 `window_size` 后**触发 view 树重建**
   （对 aura 解释器而言：置一个 dirty 标志，update 循环里 rebuild），
   保证用户拖拽窗口后 grid 列数/响应式重新计算。
3. 验证：MCP `autoui_vtree` 里 cards 行每行 4 个；拖小窗口后变 3/2/1。

### P0-B R1 修复：响应式前缀类按桌面语义生效（覆盖 H1–H4、H6、B7）

1. `parse_single` 前增加前缀剥离：`sm:|md:|lg:|xl:|2xl:` → 直接按生效处理
   （VM 即桌面大屏）；`max-sm:` 等反向断点 → 忽略。
2. 同属性冲突按「后写覆盖」：解析结果并入 classes 列表时，响应式变体
   排在基础类之后（Tailwind 生成 CSS 同 specificity 后者胜出）。需要给
   `Style` 合并加一个 dedupe 规则：同类别（如 PaddingX/FontSize/Hidden+display）
   后者替换前者。
3. 风险点：`hidden sm:inline`（=桌面可见）与 `md:hidden`（=桌面隐藏）共存
   于不同元素——`is_hidden` 的 display 覆盖逻辑已覆盖前者，补测试：
   - `"md:hidden -ml-2"` → is_hidden=true
   - `"font-bold text-lg hidden sm:inline"` → is_hidden=false
   - `"text-4xl md:text-5xl lg:text-7xl"` → FontSize=7xl
4. 验证：汉堡菜单消失；hero h1 vtree `font: "72px"`；hero padding 96→160/176。

### P1-A R4：nav-link active 高亮

`render_link_button_with_icon`（或 nav-link 分支）读 `__current_route`：
相等时 style 追加 `bg-accent text-accent-foreground font-medium`（对齐 vue
sidebar active 视觉）。验证：MCP snapshot 当前页项带高亮类 + screenshot。

### P1-B R7：轻量 toast overlay

复用 iced Stack 分层（§10 续 5 的 Popup 模式）：`toast-provider` tag 在
view_builder 转 overlay 层；`toast()` 调用在 VM 降级为向 provider 队列 push
（title+variant），3s 自动消失（iced time subscription）。验证：Events 页
点 Click Me → screenshot 出现 toast，3s 后消失。

### P1-C R5：字体对齐

打包内嵌 Inter regular/medium/semibold（`iced_assets`/`iced_fonts` 或
include_bytes），`text()` 的 family 指向注册名；中文字符回退系统字体。
验证：与 vue 截图并排对比字形。

### P2-A R6：codeblock / preview-card / table 细节

1. Rich 高亮色板对齐 Prism okaidia/one-dark（关键字紫、字符串绿、函数蓝…）。
2. codeblock 右上加 Copy 按钮（VM 内 `arboard`/Win32 clipboard 或降级为
   「已复制 ✓」state 反馈）。
3. 折叠按钮 ▼ → 「Code / Close」文本（或 icon+文字）。
4. table：表头 `font-medium text-muted-foreground`、行 `border-b border-border`
   单元格 `px-4 py-3`。
   验证：Button 页并排截图逐块比对。

### P2-B MCP 工具强化（用户明确依赖 MCP 检查渲染/layout）

1. **vtree 补 layout**：目前 `box.bbox` 全 0（未回填）。在 iced layout 阶段
   把每个 widget 的 bounds 回填进 DebugIdMap（renderer 已有 tree 遍历点），
   `autoui_vtree`/`autoui_inspect` 输出真实 x/y/w/h —— 这是「实时查看每个
   组件的 layout」的关键缺口。
2. **vtree 序列化补全**：Button.content、Image/SVG、Badge、Progress 等节点
   （本次排查被「content 不序列化」误导）。序列化 kind 与 view_builder 的
   View 枚举对齐，避免再次出现 R3 式误判。
3. **autoui_check 与真实 view_builder 对齐**：现报告 `nav-link/aside/scroll/
   icon/header unknown tag`（60 errors）全是假阳性（check 用独立简化分析器）。
   改为读取 view_builder 构建期记录的「fallback 计数器」（Element `_` 兜底
   + `View::Text("<tag/>")` 兜底时记录 tag），报告才可信。
4. **autoui_screenshot 健壮性**：窗口未完成布局（w/h=0）时 wgpu
   `create_texture Dimension X is zero` 直接 panic（本次实测崩过一次）。
   加守卫：size 为 0 时返回错误文本而非建纹理；另建议截图请求先等待一帧。
5. （可选）`autoui_diff`：新工具，输入 vue DOM snapshot 粘贴文本，与 VM
   snapshot 做文本对齐 diff，自动列结构差异——把本次人工对比流程工具化。

## 5. 已完成（本次）

- **vue 侧修复**：`auto-man/src/vue.rs` 增 `ensure_router_file()`；`run_vue_project`
  的「scaffolding-only」与「已存在项目」两条路径补写 `src/router/index.ts`
  （此前增量首次生成时 router 缺失 → vite `Failed to resolve import "./router"`）。
- 对比基建：同视口并排截图脚本（tmp/ 下 `compare-*-sidebyside.png`）、
  MCP 取证命令清单（见 §6）。

## 6. 复现/验证命令

```bash
# vue（基准）
cd examples/widgets-gallery && auto run -r vue -F 3024   # http://localhost:3024/
# vm
auto run -r vm                                            # MCP http://127.0.0.1:9247/mcp
# MCP 取证
curl -s -X POST http://127.0.0.1:9247/mcp -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"autoui_vtree","arguments":{}},"id":1}'
# autoui_action press <vnode_id> 导航；autoui_screenshot name:<名> 截图（先等一帧）
```

## 7. 建议实施顺序

P0-A → P0-B（两族根因覆盖 Home/Button 大部分差距）→ 回归截图对比 →
P1-A/B/C → P2-A/B。每步以「vue 并排截图 + vtree 结构断言」双通道验收。

---

## 8. ✅ 实施记录（2026-08-14 第二批：P0 + P1-A + 截图守卫）

### 8.1 修正后的根因认知（插桩实证推翻/确认 §3 假设）

| 原判定 | 实证结果 |
|---|---|
| R1 响应式前缀未解析 | **部分误判**——`parse_single` 已有前缀剥离（Plan 409 §10 续），`md:hidden`/`md:px-6`/`hidden sm:inline`/padding 阶梯全部正常。真正缺口是 **`text-5xl..text-9xl` 类不存在**（`lg:text-7xl` 剥离成 `text-7xl` 后无变体可解析，静默丢弃 → hero 停在 36px） |
| H1 汉堡菜单泄漏 | **视觉误报**——三次视觉分析把 30px 的 layers 图标（三层叠菱形）认成三条线的汉堡。插桩证明 `md:hidden -ml-2` → `[Hidden]` → `View::Empty`，renderer 从未收到 menu 按钮。附带发现：`autoui_snapshot` 显示的是 AuraNode 源码树，被过滤成 Empty 的节点仍会出现在快照里（误导排查，记入 §4.4） |
| R2 WINDOW_WIDTH 时序 | **部分误判**——resize → `view_dirty` → 重建机制已存在且工作。真实情况：窗口默认 800×600 逻辑（1.75 DPI = 1400×1050 物理截图），cols=2 对 800px 宽是**正确行为**，与 vue 断点阶梯（1/<640, 2/<1024, 3/<1280, 4/≥1280）完全一致。差距源于截图（物理 1400px）与窗口（逻辑 800px）的视口不匹配 |
| R3 卡片丢 desc | **误报**（§1.0 已更正），vtree 不序列化 Button.content 所致 |

### 8.2 实际改动

| 文件 | 改动 |
|---|---|
| `ui/style/class.rs` | +`Text5Xl..Text9Xl`（48/60/72/96/128px）变体 + `text-5xl..text-9xl`/`lg:` 前缀剥离后解析 |
| `ui/style/iced_adapter.rs` | +`IcedFontSize::X5xl..X9xl` + `from_style` 映射 |
| `ui/iced/renderer.rs` | `font_size_to_f32` 与 vtree 序列化两处 +px 值；窗口默认 `800×600`→`1280×800`（3 处 application/初始化 RefCell）；截图守卫：`window_size` 为 0 时回错误而非触发 wgpu `Dimension X is zero` panic |
| `ui/aura_view_builder.rs` | `render_link_button_with_icon`：`to == __current_route` 时 nav-link/component-card 加 `bg-accent text-accent-foreground font-medium rounded-md`（对齐 vue router-link-active） |
| `ui/style/mod.rs` | `plan411_tests` 5 项：md:hidden 过滤 / hidden sm:inline / 字号阶梯 / text-5xl..9xl 解析 |
| `crates/auto-man/src/vue.rs` | （第一批）`ensure_router_file` 修 vue 增量生成缺 router |

### 8.3 验证（1280×800 同逻辑视口）

| 项 | 修复前 | 修复后 | 证据 |
|---|---|---|---|
| hero "Auto UI" | vtree `font:"36px"` | `font:"72px"` | vtree；glyph 块高 51 vs vue 56 逻辑 px |
| 卡片网格 | 2 列 | **4 列，x 位置与 vue 几乎一致**（328/546/777/1008 vs 355/588/819/1048） | 像素扫描 y=770 |
| 侧边栏 active | 无 | Home/Button 当前项圆角胶囊高亮（`bg:#818cf8 fg:#fff`） | 视觉 + vtree style |
| 汉堡菜单 | （本就正确） | 不渲染 | 插桩 Empty + 无 menu 按钮到达 renderer |
| 截图健壮性 | 0 尺寸 panic | 返回错误文本 | 代码路径 |
| 测试 | — | plan411 5 绿 + plan409 回归 4 绿 | cargo test |

### 8.4 未实施（后续批次）

P1-B toast overlay、P1-C Inter 字体、P2-A codeblock 高亮色板/Copy/「Code」按钮/表格分隔线、P2-B MCP 工具四项强化（vtree layout 回填、Button.content 序列化、autoui_check 对齐真实 builder、快照过滤 Empty 节点）。

### 8.5 ✅ gap 属性废弃 → 统一 Tailwind style 路径（第三批，同日）

**背景**：`gap: "2"` 属性是 AI 生成漂移产物（DSL 原设计只有 Tailwind style 串），
且双端支持各半残（vue 只认字符串、VM 只认 int → 本节开头用户报告的
「Button 页 row (gap: "2") 间距不生效」即由此而来）。

**决策**（讨论定案）：废弃 gap 属性，间距统一走 `style: "gap-N"`；
默认 gap = 0（对齐 Tailwind Preflight 的零重置语义）。
VM 侧无需新代码——Style 解析器已支持 `gap-0..12`、小数（`gap-1.5`→6px，
f32×4 精确）、任意值（`gap-[8px]`），`effective_spacing()` 渲染期以 style gap
优先于 legacy spacing 字段；vue 侧 style 串天然透传成 class。

**codemod**（84+1 处、37 文件，残留 0）：
- `row (gap: "2")` → `row (style: "gap-2")`
- `row (gap: "2", style: "S")` → `row (style: "gap-2 S")`（style 已含
  `gap-*` 时只删属性，如 card.at 的 `gap: "1", style: "grid gap-1"`）
- `stdlib/.../DataTable.at` 的 body 内裸属性 `row { gap: "2" }` → 标准形式
- **不动** `examples/a3ui-replica`：其 `gap: 12` 是声明式组件树的数据字段，
  非视图 DSL 属性

**验证**：vue 端 DOM computed gap 权威值——Variants 行 8px、Sizes 行 8px、
Events 行 12px（= gap-2/gap-2/gap-3）；VM 端同视口像素实测 Sizes 行按钮间隙
8-9 逻辑 px ✓（vtree 的 `spacing:0` 是 legacy 字段，渲染期被 style gap 覆盖，
属正常）。vue 增量重生成 32 文件无错；回归 38 测试全绿（gap/plan409/plan411）。

**遗留**：vue codegen 的 gap 属性分支（`vue.rs` 5277/7072/7119 三处，含
「无 gap 默认 gap-4」兜底）与 VM view_builder 的 8 处 gap 提取暂时保留作
向后兼容，未在本批拆除；后续按 §4 方案拆分支 + validator 属性白名单防 AI
再次写出 gap 属性。

### 8.6 ✅ pac.at 窗口尺寸声明（第四批，同日）

**需求**：VM 窗口启动尺寸由项目 Auto 代码声明（此前为 renderer 硬编码 1280×800）。

**实现**（沿用 front_port 的「pac.at → env → 运行时」注入模式）：

| 层 | 文件 | 改动 |
|---|---|---|
| 声明 | `examples/widgets-gallery/pac.at` | `window: "1440x900"`（逻辑像素；iced 按系统 DPI 缩放为物理尺寸） |
| 解析 | `auto-man/src/pac.rs` | `Pac.window: Option<(f32,f32)>`，接受 `WxH`/`W×H`，校验 200–7680/4320 范围 |
| 注入 | `auto/src/main.rs` | run 命令打印 `VM window size: 1440x900 (from pac.at)` 并设 `AUTO_VM_WINDOW` |
| 消费 | `auto-lang/ui/iced/renderer.rs` | `pub fn startup_window_size() -> iced::Size`（env 优先，非法回退 1280×800），替换 3 处 `.window_size(...)` + 2 处初始 `window_size` RefCell |

**验证**：启动日志出现注入行；截图 2520×1575 物理 = 1440×900 逻辑 ✓；卡片网格 4 列（328/553/793/1031 逻辑 x）；plan411 5 测试 + auto-man 200 测试全绿。
