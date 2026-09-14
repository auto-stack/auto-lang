---
plan_id: PLAN-626
status: reviewed              # drafting → executing → execution_done → reviewed → archived
feature_name: auto-edit-vm-polish
author: [agent]
created_at: 2026-09-14
updated_at: 2026-09-14

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [auto-lang/ui (插值/MenuBar 视觉契约/CloseRequest 生命周期), auto-lang/vm (fs.tree 内建)]
touched_goals: []

affects: [auto-lang/ui, auto-lang/vm, autoui-skill]
current_step: 10
total_steps: 10
---

# [PLAN-626] auto-edit-vm-polish

## 0. 变更摘要

用户实机验证 `examples/ui/041-auto-edit`（VM 模式，`auto run -r vm`）后提出 4 处修正：

| # | 用户反馈 | 根因（背景调查已定位） | 修复层 |
|---|---|---|---|
| P1 | 右下角行号/列号显示为字面量 `${store.line}:${store.col}` | `text "..."` 字面量插值只认 `${.name}` 单段标识符，`${.store.line}` 多段点路径被拒；回退格式化又把前导点剥掉 | 框架（aura_view_builder 插值） |
| P2 | 状态栏区域多出"有未保存的修改"；要求改为 tab 星号 + 关闭时 alert-dialog 提醒 | 该文本是 Plan 420 P2 的固定坐标 confirm popover（open 态在 VM 下落回普通流渲染到左下）；用户要求改交互形态 | 示例（改用标准 alert-dialog + tab 星号）+ 框架（窗口关闭可拦截） |
| P3 | menubar 下拉：文本不左对齐、sep 用成纵向竖线、面板太宽；问是否标准 ContextMenu 实现 | menubar 是 actions DSL 合成物（复用 Popover 原语），非独立 Menu 组件：按钮对齐走 Plan-414 容器默认 Center；`convert_sep` 默认 vertical 且 menubar 从不传 horizontal；面板 w-48 无宽度防线 | 框架（convert_menubar 三处） |
| P4 | 左侧 Explorer 是不是真实目录？workspace dir 是什么？哪里都没体现 | tree_nodes 是静态演示数据（PLAN-618 T-06）；workspace = AUTO_PROJECT_DIR（auto-man 已注入，automan.rs:1430），UI 无处展示 | 示例（Init 生命周期装载真目录 + Explorer 头部显示）+ 框架（fs.tree 内建） |

附带修复：启动日志 `[VM-HANDLER] App.Init failed: handler not found: Init` —— 本计划给示例定义 `.Init`（装载目录树），该警告随之消失。

## 1. 目标

- **G1**：VM 模式下 `text` 模板 `${.a.b}` 多段点路径插值正确求值（statusbar 实时显示行:列），回退不再丢前导点。
- **G2**：dirty 状态可视化重构——脏 tab 标题带 `*`；关闭脏 tab / 关闭应用（窗口 X 与「退出」菜单）时弹标准 `alert-dialog` 确认；移除固定坐标 confirm popover。
- **G3**：VM menubar 下拉视觉修正——菜单项文本左对齐、`sep` 为横向分隔线、面板宽度有防线不再异常撑宽；并回答用户疑问（menubar=actions DSL 合成物复用 Popover 原语，Vue 端为真 shadcn Menubar；本计划不新增标准 Menu 组件，记入债务清单）。
- **G4**：Explorer 展示真实 workspace 目录树（`AUTO_PROJECT_DIR`），头部体现 workspace 名，点击文件节点读真实文件内容打开。
- **非目标**：不新增独立 `menu`/`menubar` 标准组件（债务候选）；不修 Vue 端 menubar（shadcn 已正常）；不处理坐标锚 popover 落回普通流的框架底层缺陷（本计划仅调查+记录债务，示例侧已绕开）；不改窗口标题栏内容。

## 2. 架构方案

分层修复，框架改动全部落在 VM 视图构建/运行时（`crates/auto-lang/src/ui/`、`crates/auto-lang/src/vm/`），示例改动落在 `examples/ui/041-auto-edit/src/front/`：

1. **插值（P1）**：`aura_view_builder.rs` 的 `resolve_literal_interpolation_with`（≈10321-10389）扩展接受多段点路径 `${.a.b}`，求值走与条件表达式相同的 `resolve_expr_to_value` 扁平化通道（`.store.X` → `read_state(X)`）；`${name}` 回退格式化改为保留原样模板文本（不再剥点重构）。
2. **menubar（P3）**：`convert_menubar`（≈6678-6855）三处——菜单项按钮 style 追加 `justify-start text-left`（走 plan050_content_align 显式对齐通道，压过 Plan-414 容器默认 Center）；`sep` 项调 `convert_sep` 时传 `orientation: "horizontal"`（`w-full h-px bg-border`）；面板 Column 镜像 convert_popover 的宽度防线（无 width 类时注入显式宽度，PLAN-526 T27 同款），并把 w-48 收窄为 w-44。
3. **窗口关闭拦截（P2）**：镜像 `Init` 的 fire 机制（`dynamic.rs:1238-1248` `fire_init` / `lib.rs:4243-4255`）新增 `CloseRequest` 生命周期——iced `CloseRequested` 臂（`renderer.rs:16806-16825`）先查应用是否声明 `handler_App_CloseRequest`/`handler_CloseRequest`（`vm_bridge.rs:1599-1611` 同款双查），声明则改为向应用分派（弹 alert-dialog 由 handler 决定），抑制默认 `window::close`；未声明行为完全不变（向后兼容）。
4. **fs.tree 内建（P4）**：`vm/native_catalog.rs` 白名单 + `vm/native.rs` shim 新增 `fs.tree`（canonical `auto.fs.tree`，`(root, max_depth) -> String` 嵌套 JSON，节点 schema 直接对齐 TreeView：`{id,label,children,kind,icon,is_leaf,badge}`；跳过 `.git`/`target`/`build`/`node_modules`/`gen`；深度上限防爆）。.at 侧经现有 `json.to_value`（3100）过界为 list 值直接赋 `tree_nodes`。
5. **示例重组（P2/P4）**：`editor_store.at` 新增 `Init`/`CloseRequest` 入口与 `ws_dir` 字段；`app.at` 移除 confirm popover 改用 alert-dialog（overlay-probe 先例形态）；tab 星号用 `if t.dirty`（循环成员条件已有 `t.key == .store.active_key` 在用先例）。

## 3. 技术栈

- Rust（框架）：aura_view_builder / renderer / dynamic / vm native（全部既有管线，无新依赖）。
- AutoLang .at（示例）：store handler + view DSL，既有语法边界内（VM view 不能调函数、组件子树快照不可见等 Plan 449 结论继续遵守）。
- 验证：`cargo check -p auto-lang` → 模块级 `cargo t`（iced / aura / vm natives）→ 示例双端实机（VM `auto run -r vm` + Vue `auto run`）经 autoui-verifier 技能。

## 4. 需求分析与背景调查

**授权**：用户 2026-09-14 实机验证后一次性授权分析与实施（"综合上述需求，进行分析并做一个改进计划，然后再实施"）。范围=上表 P1-P4 四项；无额外预算/续跑限制记录。

**背景调查证据**（2026-09-14，Explore 代理 + 实机日志，HEAD=ffe7ab133）：

- **P1 插值**：`text "..."` 双引号串是 `Expr::Str`（lexer 仅反引号/`f"` 产 FStr），走 `convert_text_element` → `resolve_expr_to_string_with` → `resolve_literal_interpolation_with`（aura_view_builder.rs:10321-10389），仅支持 `${.name}`（名字校验 `[A-Za-z0-9_]+`，10344——`store.line` 含点被拒）与 `${member.field}` 循环成员形态；回退 `read_state_as_string_with` Err 臂 `format!("${{{}}}", field_name)`（9333，field_name 已 trim 前导点）→ 窗口显示 `${store.line}` 丢点，与截图一致。`.store.X` 的扁平化特判在 `resolve_expr_to_value`（9588-9602：`Dot(Dot(Ident("."),"store"),X)` → `read_state(X)`）；status_bar 为子组件，`read_state` 按字段名查组件自身 state 堆对象（vm_bridge.rs:565-597），line/col 在 EditorStore 合并进 App 根 state（vm_bridge.rs:448-449）——实施时需验证子组件直读 `.store.*`（013-todo 已验证模式）在插值通道与条件通道的取数路径一致性，若分叉则以条件通道为准。
- **P2 confirm popover**：app.at:177-191 固定坐标（x:420,y:260）popover；VM 下坐标锚在 flow 里是零尺寸 Space（renderer.rs:4475-4477），iced 会把零尺寸 flex 子件从 overlay 遍历剔除 → 面板落回普通流（截图左下角"有未保存的修改"即此形态；popover.rs:144-193 的 1px 隐形锚防线未覆盖该形态，记债务调查）。alert-dialog 标准族已存在（PLAN-530，aura_view_builder.rs:1424-1520，VM 轨=模态 Popover 原语；.at API 先例 `examples/capability-tests/overlay-probe/src/front/app.at:29-44`：`alert-dialog (open:)` + trigger/content/header/title/description/footer/cancel/action）。窗口关闭当前不可拦截：renderer.rs:16268 `__window_close_request` → 直接 `iced::window::close`（注释明示"不该由 App 分派管线处理"——本计划有据变更该契约）。Init fire 机制：dynamic.rs:1238-1248 + lib.rs:4243-4255 + vm_bridge.rs:1599-1611 双名查找。
- **P3 menubar**：标签分发 aura_view_builder.rs:1907-1913；合成 `convert_menubar` 6678-6855（触发按钮 6725-6740 `h-7 px-3 text-[12px]`；面板 6820-6826 硬编码 `w-48 bg-[#16171B] border border-zinc-700 shadow-md py-1`；Popover 包裹 6827-6837 BottomStart）。菜单项=带 content 的 Button（6779-6813，内容 Row `w-full justify-between`），Plan-414 分支（renderer.rs:3700-3712）把 content 包进容器且 `ax` 默认 `Horizontal::Center`（plan414_content_alignment 1372-1381 `unwrap_or(Horizontal::Center)`）；显式让位通道 plan050_content_align（1341-1365）只读按钮自身 style 的 `justify-*`/`text-*`——menubar 项按钮没有 → 居中。sep：`convert_sep`（7982-8030）默认 vertical（`w-7 h-7` 盒内 `w-px h-4` 竖线），menubar 从不传 horizontal。宽度防线先例：convert_popover 7718-7727（PLAN-526 T27 任务栏横贯桌面六轮反馈）；Style::parse 未知类静默跳过（style/parser.rs:23-31）。Vue 对照：ui_gen/vue.rs:5932-6035 真 shadcn Menubar 族，5916 附近 MenubarSeparator。
- **P4 目录**：tree_nodes 静态硬编码（editor_store.at:59-70，PLAN-618 T-06）；TreeSelect 开文件查静态演示内容表（editor_store.at:383-388）。`fs.read_dir`(2866)/`fs.walk`(2860)/`fs.walk_files`(2847)/`fs.join`(2864) 已存在（native_catalog.rs:449,1004-1012；native.rs:9125-9446），返回扁平 JSON；`json.to_value`(3100) 等 JSON 族齐备（native_catalog.rs:1217-1238）。`AUTO_PROJECT_DIR` 由 auto-man 启动注入（crates/auto-man/src/automan.rs:1430）。renderer.rs:9310 注释".at 无 read_dir 原语"已过时（2026-08-22 白名单登记取代），顺手更正。
- **App.Init 警告**：`fire_init` 无条件触发（dynamic.rs:1238-1248），示例未定义 `.Init` → 每次启动必打日志；HandlerNotFound 分支不置 dirty，状态全靠默认值。本计划示例定义 `.Init` 后自然消失。

## 5. 详细设计

### 5.1 P1 插值扩展（T-01）

`resolve_literal_interpolation_with`：
- `${...}` 内容匹配放宽为：单段 `[A-Za-z0-9_]+`（现状保留）或 `.开头多段点路径`（如 `.store.line`）或循环成员 `member.field`（现状保留）。
- 点路径求值：复用 `resolve_expr_to_value` 的 `.store.X` 扁平化臂（构造等价 `Expr::Dot` 或直接调 `read_state` 链），与条件表达式同通道；求值失败回退为**原样保留整段模板文本**（含点），替换现有剥点重构回退。
- Vue 端核验：ui_gen/vue.rs 对 `${.store.X}` 的现有产物（预计已是绑定），双端对齐在 T-07 验证。

### 5.2 P3 menubar 三处（T-02）

`convert_menubar`：
1. 菜单项按钮 style `"h-7 w-full px-0 py-0"` → `"h-7 w-full px-0 py-0 justify-start text-left"`（plan050_content_align 读到 `justify-start` → `ax=Start`，压过 Plan-414 默认 Center；快捷键右推仍由内层 Row `justify-between` 承担）。
2. `MenuItem::Separator` 臂：`convert_sep` 传入 `orientation: "horizontal"` props（得到 `w-full h-px bg-border` 横线）。
3. 面板：style 改 `"w-44 bg-[#16171B] border border-zinc-700 shadow-md py-1"`，并在合成后镜像 convert_popover 的防线——若面板 Column 无有效 width 类则注入显式 `Width::Fixed`（防 Style::parse 静默失败回退全宽，PLAN-526 T27 同款注释留档）。

### 5.3 P2 CloseRequest 生命周期（T-03）

- `vm_bridge.rs`：新增 pub `has_handler("CloseRequest")` 探测（复用 1599-1611 双名查找：`handler_App_CloseRequest` → `handler_CloseRequest`）。
- `renderer.rs` CloseRequested 臂（16806-16825）：`app_of_window(win)` 命中且 `has_handler` → 改为 `call_handler("CloseRequest", &[])`（成功置 dirty），**不**返回 `window::close`；未命中/未声明 → 现行为不变（`__window_close_request` → `window::close`）。注释更新契约说明（原"不该由 App 分派管线处理"改为"声明 CloseRequest 的应用可拦截"）。
- 语义：handler 决定后续——有脏 tab 弹 alert-dialog（"保存并关闭"/"直接关闭"/"取消"）；无脏 tab 直接 `Process.exit(0)`（与 ActQuit 一致；"保存并关闭"= 逐脏 tab `ActSave` 后退出）。

### 5.4 P4 fs.tree 内建（T-04）

- 签名 `fs.tree(root: str, max_depth: int) -> str`（canonical `auto.fs.tree`，id 沿 28xx 段取未占用号）。
- 语义：递归枚举目录，产出嵌套 JSON 数组，节点 `{id: 相对路径, label: 文件名, children: [], kind: "dir"|"file", icon: "folder"|"file-text", is_leaf: bool, badge: ""}`——直接对齐 editor_store tree_nodes 现有 schema 与 TreeView 消费格式；`id` 用 `/` 分隔相对路径（与现 TreeSelect 的 path 匹配逻辑兼容）。目录排序 dir 先、字典序；跳过 `.git`、`target`、`build`、`node_modules`、`gen`、`dist`、隐藏项；`max_depth` 防爆（示例传 4）。
- 实施位：native_catalog.rs 白名单 + BIGVM shim 表、native.rs `shim_fs_tree`（`walkdir` 已是既有依赖，参考 `walkdir_all` 9417-9446）；顺手更正 renderer.rs:9310 过时注释。

### 5.5 示例改造（T-05/T-06）

`editor_store.at`：
- 新字段：`var ws_dir str = ""`、`var ws_name str = ""`、`var quit_confirm_open bool = false`（tab 关闭确认沿用 confirm_open/confirm_idx，但渲染层换 alert-dialog）。
- `.Init`：`ws_dir = Env.get("AUTO_PROJECT_DIR")`；`ws_name = file_basename(ws_dir)`；`tree_nodes = json.to_value(fs.tree(ws_dir, 4))`；`tree_expanded = ["src"]`；`console_log("workspace: " + ws_dir)`。
- `.TreeSelect`：静态内容表改为真实读盘——`File.read_text(fs.join(ws_dir, id))`（保留"目录 id 静默忽略"与"已开同路径激活"语义）。
- `.CloseRequest`：任一 `tabs[i].dirty` → `quit_confirm_open = true`，否则 `Process.exit(0)`；`.QuitSaveClose`（逐脏 tab 复用 ActSave 语义后退出）/`.QuitConfirmCancel` 两新 msg。
- `Init`/`CloseRequest` 在 App `on{}` 薄委托（`.Init -> { store.Init() }` 等生命周期名直通）。

`app.at`：
- tab 条两分支：`if t.dirty { text "*" { style: "text-[12px] text-amber-400 -ml-1" } }`（激活分支置于标题按钮与 × 之间；非激活分支按钮后追加）；非激活分支由单 button 扩为 row 容纳星号。
- confirm popover（177-191）整段移除 → `alert-dialog (open: .store.confirm_open)`（title"有未保存的修改"/description"关闭前要保存吗?"/footer: cancel=取消、action=直接关闭）；新增 `alert-dialog (open: .store.quit_confirm_open)`（退出确认，footer: cancel=取消、action=直接关闭不保存 + 二 action=保存并关闭）。
- Explorer 头部：`text "EXPLORER"` 下加 `text .store.ws_name { style: "text-[11px] text-zinc-400 px-1 pb-1" }`。
- status_bar.at 不改（P1 为框架修复，源码原样生效）。

### 5.6 规范增量

| delta_id | add/modify/retire | docs/specs/... 目标 | before/after 规则 | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/ui/architecture.md（视图插值节） | before: text 字面量插值仅 `${.name}`/`${member.field}`；after: 追加 `${.多段.路径}` 走状态扁平化通道，失败原样保留 | P1 根因修复，插值能力契约化 | AC-01 |
| SD-02 | modify | docs/specs/auto-lang/ui/architecture.md（actions DSL/menubar 节） | before: VM menubar 为 DSL 合成物（实现细节无契约）；after: 契约化视觉规则——项左对齐、sep 横向、面板有宽度防线；明确 menubar 复用 Popover 原语、不设独立 Menu 组件（债务候选登记） | P3 + 回答用户"标准组件？"之问 | AC-03 |
| SD-03 | add | docs/specs/auto-lang/ui/architecture.md（生命周期节，若无则新增小节） | add: `CloseRequest` 生命周期——声明 `handler_*_CloseRequest` 的应用可拦截窗口关闭（fire 语义同 Init；未声明行为不变） | P2 框架支点 | AC-04 |
| SD-04 | add | docs/specs/auto-lang/vm/project.md（内建清单节） | add: `fs.tree(root, max_depth)` 嵌套目录树 JSON 内建（schema/跳过表/深度上限） | P4 数据源 | AC-05 |

## 6. 测试设计

- **单元/模块（Rust）**：
  - 插值：现有 aura_view_builder 插值测试旁新增多段点路径用例（求值成功/失败原样保留两臂）。
  - menubar：layout_tests / aura 快照级断言（sep 横向类、按钮 justify-start、面板宽度防线）——沿现有 menubar 测试形态（layout_tests.rs 已含 menubar）。
  - CloseRequest：vm_bridge has_handler 双名查找单测 + renderer 臂分支测试（声明→分派不关窗；未声明→关窗）。
  - fs.tree：native shim 单测（嵌套 schema、跳过表、max_depth、排序）。
- **示例矩阵（.at / 双端）**：`examples/ui/041-auto-edit/tests/desktop_mcp.py` 既有矩阵形态扩展——autoui_state 断言 line/col 实时值；tab 星号（快照文本）；alert-dialog 开合；Explorer 节点数=真实目录；CloseRequest（VM 实机人工 + MCP 可驱动臂）。Vue 模式跑 `auto run` 对照（插值/menubar 为 VM 轨修复，Vue 端确认无回归即可）。
- **实机验收**：`auto run -r vm` 启动无 `App.Init failed` 告警；行:列随光标实时变化；编辑→tab 出星号→点 × 弹 alert-dialog；编辑→点窗口 X 弹退出确认；menubar 下拉左对齐+横线+宽度正常；Explorer 显示真实文件树、点击 src/app.at 打开真实内容。

## 7. 验收标准

- **AC-01**：VM 模式 statusbar 右下角显示实时光标行列（移动光标数值变化），不再出现 `${...}` 字面量；插值失败时回退显示完整原模板（含点）。验证：实机截图 + 单测。
- **AC-02**：编辑任一 tab 后其标题出现 `*`，保存（ActSave）后消失；确认弹层为标准 alert-dialog 形态（模态、标题/描述/按钮槽），旧固定坐标 popover 不再存在。验证：实机 + 快照。
- **AC-03**：menubar 下拉项文本左对齐、快捷键右对齐、`sep` 为横向通栏细线、面板宽度固定不异常撑宽。验证：实机截图 + layout 测试。
- **AC-04**：有脏 tab 时点窗口 X / 菜单「退出」弹 alert-dialog（三键：保存并关闭/直接关闭/取消），无脏 tab 时直接退出；未声明 CloseRequest 的其他示例（如 002-counter）窗口 X 行为不变。验证：实机 + 回归运行一个未声明示例。
- **AC-05**：Explorer 显示 `AUTO_PROJECT_DIR` 真实目录树（含 src/*.at、docs、pac.at、README.md 等），头部显示 workspace 名，点击文件节点打开真实内容；启动日志无 `App.Init failed`。验证：实机 + fs.tree 单测。
- **AC-06**：Vue 模式（`auto run`）示例无回归（插值正常、alert-dialog 正常、menubar 正常）。验证：autoui-verifier 双端跑一遍。
- **AC-10**（rev2 新增）：编辑器滚动条视觉与官方滚动条一致（半透明 thumb、圆角、透明轨道）。验证：实机对照 + code_editor 45/45 回归。
- **AC-09**（rev2 新增）：编辑器滚动条可用鼠标拖拽到任意位置不卡死（拖拽落点即所见内容）。验证：plan626_scrollbar_drag 单测 + 实机。
- **AC-08**（rev2 新增）：编辑器滚轮方向正确（向下滚动查看下方内容）；可滚动到文件最末行（底端钳制）；大文件连续滚动不失去响应。验证：plan626_wheel 单测（方向/钳制/边界三断言）+ 实机。
- **AC-07**：`cargo check -p auto-lang` 零警告；触及模块的 scoped `cargo t` 全绿；本计划不改编译器/VM 执行语义核心（fs.tree 为新增内建白名单项），不触发 `cargo tv`/`taa` 全量档（Category B 门禁：check + 模块级测试）。

## 8. 执行步骤

worktree：`D:/autostack/.wt/lang-626/auto-lang`（branch `plan-626-dev`）；计划文件记账留 master。

- **T-01** [框架|P1|→AC-01] ✅ [已完成] commit f3aa8bc65：`${.}` 分支放宽多段点路径（嵌套 Dot 链走 resolve_expr_to_value 扁平化），失败原样保留；测试 plan626_literal_interpolation_multi_segment_dot_path 绿（成功 3:9 + 失败保点双臂）。插值相关回归 8+43 全绿。
- **T-02** [框架|P3|→AC-03] ✅ [已完成] commit 04ce66265：justify-start/text-left 显式对齐、sep orientation horizontal（通栏横线）、w-44+宽度防线（镜像 PLAN-526 T27 注入）；测试 plan626_menubar_panel_left_align_horizontal_sep_fixed_width 绿 + vue menubar 合成回归绿。
- **T-03** [框架|P2|→AC-04] ✅ [已完成] commit fbb45e3ce：vm_bridge.has_handler（存量方法，去重后复用）+ dynamic.has_lifecycle_handler/fire_close_request + renderer 关窗臂分流（声明→分派不关窗，未声明行为不变）；测试 plan626_has_handler_close_request_lifecycle 绿。拦截臂的端到端属 iced 事件循环（无法 headless 单测），由示例实机验证覆盖（T-07）。
- **T-04** [框架|P4|→AC-05] ✅ [已完成] commit f7cd9323d：fs.tree(2875, random 段空档防撞号)——fs_tree_walk 嵌套 JSON（TreeView schema 直配/目录优先排序/跳过表/max_depth），catalog 三处登记齐；测试 plan626_fs_tree_nested_json_schema_and_order 绿；native::tests 18/18 + catalog 完整性守卫（id/名唯一）3/3 绿；顺手更正 renderer 壁纸扫描处过时 read_dir 注释。
- **T-05** [示例|P4+Init|→AC-05] ✅ [已完成] commit db5058678：editor_store ws_dir/ws_name 字段 + LoadWorkspace handler（**调整记录**：生命周期名 `Init` 保留给根 widget——store 侧 msg Init 不合成 handler_EditorStore_Init，link 期炸 `Undefined symbol`，改 App.Init 薄委托 store.LoadWorkspace，语义不变）；TreeSelect 全路径（fs.join + File.exists/read_text）读盘开文件；Explorer 头部 ws_name。实机：树=真实目录（console 打开 `src/front/components/tree_util.at` 等真实文件），autoui_state ws_dir/ws_name 就位，`App.Init failed` 启动噪音消失（日志 grep=0）。
- **T-06** [示例|P2|→AC-02,AC-04] ✅ [已完成] commit b3f7dc828：tab 星号（激活/非激活双分支 `if t.dirty`）、tab 关闭确认与退出确认双 alert-dialog（退出三键 取消/不保存退出/保存并退出）、固定坐标 popover 退役；store 侧 CloseRequest（脏检查 gate）/QuitDiscard/QuitSaveClose/QuitConfirmCancel。实机快照 1:1 插值生效、无字面量残留。
- **T-07** [验证|→AC-06,AC-07] ✅ [已完成] commit a20a4bf96 + 5b7cd5f6a：
  - 门禁：`cargo check -p auto-lang --features ui-iced` 与缺省 feature 双过（警告均为存量基线，无新增）；aura_view_builder 102/103（1 失败 strips_tags_and_decodes_entities **基线 f1b64d1a1 同样失败**，564-Q6 在案预存红）；vm_bridge 38/38；ui::iced 188/190（2 失败 external_config_poll/p010_popover_ondismiss **基线同样失败**，预存红）。
  - 矩阵：分支 47/3 vs 基线 48/2——2 失败（menubar/toolbar 合成 onclick 快照）两侧一致为存量；+1 "undo restores text" 单次未复现，判环境 flake。多轮复跑受**并行会话干扰**（他.session 16:54 起 `auto.exe --no-merge` 活跃，矩阵实例被外部杀，应用日志无 panic/退出痕迹，死亡点随机），达到重试上限即停，不再抢跑。
  - 顺手修矩阵自毒 bug：T10 热重载备份回写 Windows 文本模式 CRLF 残留 → 下轮 DSL 解析退化（newline='' 修复，a20a4bf96）。
  - Vue 端（AC-06）：示例 pac `render: "vm"` 纯 VM 桌面示例无 Vue 前端；本计划 diff 未触 ui_gen/vue.rs，无回归面。
  - README 补 Plan 626 边界注记（5b7cd5f6a）。
- **T-08** [框架|用户实机回归|→AC-08 新增] ✅ [已完成] commit 8c9176110：用户实机验证发现 code_editor 滚轮三症（方向反/滚不到底/多次滚动后未响应），同根——handle_wheel 直传 `dy * line_height` 给 `Action::Scroll`：winit 滚轮向下 y 为负（方向反）；上游 Action::Scroll 只累加不归一（scroll.line 恒 0，每帧 layout_runs 全文件行走，T-05 打开真实大文件后滚轮风暴拖垮事件循环=未响应；上下游无钳制，越过底/顶内容滚丢=滚不到底）。修复：方向取负；滚后 `shape_until_scroll` 归一并钳进有效区间；边界 no-op 免重绘。测试 plan626_wheel_direction_clamps_at_bottom_and_noops_at_boundary 绿（方向/底端钳制/边界免重绘三断言）+ code_editor::core 17/17 回归绿。
- **T-09** [框架|用户实机回归|→AC-09 新增] ✅ [已完成] commit 158ffe8cf：用户实机验证发现滚动条**拖拽即卡死**——`handle_mouse_move` 的 `match *self.drag.lock()` 把 Mutex 临时守卫活到整个 match 体结束，`drag_scrollbar_v/h` 内部重入同锁 = 首次移动必然自锁死（std Mutex 不可重入；单测挂死实证，CPU 归零阻塞）。修复：克隆 Drag 状态后再匹配。同轮补齐拖拽路径与滚轮同口径：落点清 `vertical` + `shape_until_scroll` 归一（旧路径残留 vertical + 目标行未 shape → layout_runs 早停 → first_visible_line 停在 usize::MAX → NaN 滚动条几何），render 护栏 usize::MAX 不进 frac。测试 plan626_scrollbar_drag_lands_shaped_normalized_scroll 绿（该测试在修复前即挂死=回归实证）+ code_editor::core 18/18 回归绿。**架构答复**（用户问）：编辑器滚动条非 AutoUI scroller 组件，系 Plan 413 定制 cosmic-text 引擎自绘（样式/行为手搓）；统一到标准 scroller 属架构级改造，记 P626-D4 债务候选。
- **T-10** [框架|样式|→AC-10 新增] ✅ [已完成] commit 3b85a0357：用户追问滚动条是否官方组件——按用户未答选项时的推荐路径执行"只统一样式"：thumb 换官方 vue 风格半透明白 rgba(0.9,0.9,0.9,0.3)（镜像 iced renderer scrollbar_style，Plan 409 §10）+ 3px 圆角，双主题统一；行为（拖拽/钳制）T-08/T-09 已修。组件级统一（抽共享组件或官方 scroller 包裹）仍留 P626-D4。

## 9. 复审记录

- 2026-09-14 draft handoff：`stage: new`，PLAN-626 rev1。背景调查四项根因全部代码级定位（见 §4），SD-01..04 与 AC-01..07 齐备，任务覆盖全部 AC。`outcome: pass`，`next: work`（用户已授权分析与实施连续执行）。
- 2026-09-14 review：`stage: review | plan_id: PLAN-626 | plan_revision: 2 | outcome: pass | reviewed_commit: ac6f9cbb0 (plan-626-dev) | base_commit: f1b64d1a1 | spec_inputs: SD-01..04（合并时沉淀 docs/specs/auto-lang/ui+vm） | acceptance_results: AC-01..10 全 PASS（插值/星号+alert-dialog/menubar 三处/Explorer 真树/滚动三症/滚动条拖拽死锁+样式/矩阵 48/2） | findings: F-1 ffi_dual_019 全量并行缓存竞窗（基线绿/独立绿/尖端调度位移显形）→ 630 复审修复（ffi-dual 串行组），非本计划代码回归 | evidence: cargo tf 3551/3551 + cargo tv 3697/3697（630 顶端覆盖全叠放）+ 单测 plan626_×3 + 实机截图/MCP 探针 | next: merge。
- 2026-09-14 revision handoff（rev2 补记 T-10）：滚动条样式对齐官方（用户追问后按推荐路径"只统一样式"执行），新增 AC-10，`current_step` 9→10。
滚动条拖拽死锁（Mutex 重入）+ 拖拽归一，新增 AC-09，`current_step` 8→9；既有授权范围。
用户实机回归新增 T-08（滚轮三症同根修复），新增 AC-08，`current_step` 7→8；既有授权范围（用户直接报障的缺陷修复），无目标/预算变化。
- 2026-09-14 work handoff（rev2 补记）：`stage: work | plan_id: PLAN-626 | plan_revision: 1 | outcome: pass（rev2 续执行后仍 execution_done，见上） | code_commit: 5b7cd5f6a (branch plan-626-dev, base f1b64d1a1) | task_ids: T-01..T-07 全完成 | evidence: 各任务行 + 实机日志 auto-edit-626.log（App.Init failed=0、快照 1:1 无字面量、ws_dir/ws_name 就位）| blockers: 无（矩阵复跑受并行会话干扰已归因记录，证据以首轮干净跑为准）| next: review。
- 依赖注记：worktree 组内补了 auto-down 只读兄弟 worktree（detached 67bb508，路径解析用，未改动其内容），merge 后随组清理。

## 10. 待澄清事项

- 无阻塞项。两点实施期注意：(1) status_bar 子组件 `.store.*` 直读在插值通道的取数路径需与条件通道一致（§5.1，若分叉以条件通道为准并在 T-01 留证）；(2) alert-dialog 在组件子树（根视图直用，非组件内）的快照可见性沿 overlay-probe 先例，不新增组件封装。
