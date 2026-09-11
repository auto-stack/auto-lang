# 布局件交互原语（hover / 右键）

> **状态**：设计草稿 + 试点落地（PLAN-002 B，2026-09-09）。
> **关联**：PLAN-002（origin PLAN-535）B 项；上游债 = PLAN-526 待澄清③ /
> KNOWN-DEBT-AND-RISKS「布局件级 hover/右键公共基建（wrap_layout_onclick）未做」；
> 前置实现 = Plan 490 G4（布局件 onclick 三节点贯通）。
> **定位**：布局件（row/col/div=Container）的**通用事件与 hover 态**原语。
> 本文件是设计稿 + 试点记录；全示例铺开（§5）另立计划。

## 1. 问题

526 调研（Q6/Q9①，file:line 已核）登记两条 VM 轨缺口，Vue 轨均无此问题：

| # | 症状 | VM 轨现状 | Vue 轨 |
|---|---|---|---|
| Q9① | launcher 网格/结果行无 hover 高亮 | `hover:` 类只有 Button/SVG 两臂消费；row/col 上的 `hover:bg-*` **静默丢弃** | 原生 CSS `:hover`，早已生效 |
| Q6 | 布局件不能挂右键 | 只有 button（402）与 mouse-area（526 T10）两个挂点 | 元素级 `oncontextmenu → @contextmenu` 泛映射（`ui_gen/vue.rs:14470`），任意元素可用 |

两者同源：**布局件只有左键 onclick（Plan 490 G4），没有事件/hover 的通用通道**。
后果是逐点特设——launcher 把行改成 button、桌面空白包 mouse-area，每新增一个
可交互布局件都要再绕一次。测量面：`examples/` + `assets/` 内 `hover:` 出现
481 处、auto-os 侧 88 处，其中落在 row/col/div 上的（a3ui-replica 导航行与目录
div、book-reader 导航行、launcher 网格格、桌面图标格…）此前全部不生效。

## 2. 设计

两条正交能力，共用同一个包装点（`renderer::wrap_layout_events`）：

### 2.1 右键（事件层）

- `View::Row/Column/Container` 增 `on_right_click: Option<M>`（命名对齐
  `View::Button::on_right_click`，402 先例）。
- aura 提取点：`set_layout_events`（原 `set_layout_onclick`）在同一处收
  `onclick`/`click` 与 `oncontextmenu` 双键，tracked/untracked 两路镜像。
- 渲染：`mouse_area(el).on_release(onclick).on_right_press(on_right_click)`
  ——单个 mouse_area 承载左右两键（mouse_area 不捕获未声明事件，内层交互件
  语义不变）；inspect 捕获态自守卫不包（与 490 G4 同规则）。
- 消息桥：`convert_view_messages` 显式接 `on_right_click`（D-GAP 第四例
  教训——新增字段每条桥臂都必须显式接，漏接即静默丢臂）。

### 2.2 hover（样式层）

iced 的 container/row/column **没有 hover 状态回调**（button/svg 有 `Status`），
两条既有路线都不合用：mouse_area 的 on_enter/on_exit 发消息 → 每次 hover 翻转
重建整棵 view 树（大列表悬停性能陷阱）；container 样式闭包在 draw 期求值但拿
不到 hover 态。故新增 `hover_area.rs`：

- **HoverArea**：纯委托包装 widget（PointerArea/table_resize 先例），自持
  `State { hovered, cursor_position, bounds }`（tree::Tag），只观察游标位置、
  不捕获事件；
- **共享标志**：`HoverFlag = Arc<AtomicBool>`，由 `layout_hover_flag(style)`
  在**样式声明了 `hover:` 变体类时**构造（无声明 = None，零开销路径不变），
  一路传进 `build_row/build_column/build_container` → `apply_*_style`；
- **样式二选一**：`layout_style_fn(base, hover, flag)` 返回的闭包读标志，
  在 `build_container_style(base)` 与
  `build_container_style(merged_with_variant(style, Variant::Hover))` 之间
  选择（hover 类覆盖 base，与按钮臂同语义——不是叠加层，`hover:bg-*` 能盖掉
  不透明底色）；
- **翻转开销**：状态变化只 `shell.request_redraw()`（重绘，不重建、不发消息），
  与 iced 原生 button 的 Status 悬停同档；`draw` 每帧把状态回写当前帧标志
  （重建后新闭包捕获新 Arc，视觉不闪断）。

## 3. 语义边界（已登记的差异，非缺陷）

| 面 | 本设计行为 | 浏览器/后续 |
|---|---|---|
| hover 类作用面 | 布局件**自身**（背景/边框/圆角/阴影/文本色） | 同 |
| 子元素文本色级联 | 不级联（`hover:text-*` 只作用于布局件自身文本色） | 浏览器级联到后代 |
| 布局类 hover（padding/border 宽） | 只影响绘制，不触发重排（request_redraw，非 invalidate_layout） | 浏览器会重排 |
| `cursor-pointer` | 已解析、iced 适配器 no-op（`iced_adapter.rs:1209`） | 待独立接线（mouse_area `.interaction`） |
| grid 自身 | **未接**（`View::Grid` 无任何事件槽；试点消费面是格内 col） | 后续项 |
| hover 动画/过渡 | 无（`transition-*` 家族为 KNOWN-DEBT 无动画宿主） | — |

## 4. 试点落地（PLAN-002 B）

| 站点 | 改动 | 证据 |
|---|---|---|
| launcher 网格格 | 既有 `col ... hover:bg-primary/10` 变为生效（无 .at 改动） | `auto-os/apps/028-launcher/src/front/app.at:226` |
| 桌面图标格 | `col` 加 `hover:bg-white/10`；`oncontextmenu` 自 icon button 迁到格 col（格内全域可右键） | `auto-os/shell/desktop.at:73`（pack=pin） |
| 渲染层 | `hover_area.rs` + `wrap_layout_events` + `layout_hover_flag/layout_style_fn` + 三节点双路径臂 | `renderer.rs`、`view.rs`、`aura_view_builder.rs` |
| 测试 | 见 §5 | — |

桌面资产同步：`auto-os/shell/desktop.at`（唯一真相源）→
`scripts/shell-pack-sync.py --sync` → auto-lang `assets/desktop.at`（pin）
→ a2vue 金样再生（`AUTO_LANG_UPDATE_GOLDEN=1`）。

## 5. 回归面与分期

- **试点（本次）**：机制全量生效（渲染层是全局的），验证面限定 launcher +
  桌面 + 既有测试套（`cargo t` 全量失败集与 master 全等；`iced-layout-tests`
  35/35）。
- **全量铺开（另立）**：481 + 88 处 `hover:` 的布局件逐例目检 + 金样对拍 +
  实机 sweep（尤其 a3ui-replica 导航行、book-reader 导航、ui-gallery），
  并把 §3 的差异项按用户裁定补齐（cursor-pointer 接线 / grid 事件槽 /
  hover 文本级联）。
- **风险**：布局件 hover 类一旦生效，任何示例的布局件悬停都会有视觉变化
  ——这是**修正**（对齐 Vue），但需要一轮 sweep 确认无意外（例如依赖"无
  hover"的截图基线）。

## 6. 后续项

1. `View::Grid` 事件槽（cols/gap/cells 之外加 onclick/on_right_click，或
   grid 级 hover）——需要 .at 语法面确认；
2. `cursor-pointer` → `mouse_area.interaction(Interaction::Pointer)`（需保持
   Vue 一致性：浏览器只在 button/a 默认 pointer，div 需显式类）；
3. hover 文本色级联（需要子件样式重算或继承链，成本高于本轮）；
4. 全量铺开计划（§5）。
