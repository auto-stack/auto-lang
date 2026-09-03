---
plan_id: PLAN-526
status: reviewed              # 复审通过 2026-09-03（cargo tf 3397/3399，唯二红=在案存量 530/531 同款归因；见复审记录） drafting → executing → execution_done → reviewed → archived
feature_name: desktop-shell-ux-fixes
author: [zhaopuming]
created_at: 2026-09-03
updated_at: 2026-09-03

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []

affects: [ui/session, ui/iced, shell.at, desktop.at, settings.at, 028-launcher]
current_step: 39
total_steps: 39
---

# [PLAN-526] 桌面壳 UX 十题修复：窗口三键/resize/焦点环/fit、任务栏样式统一、右键菜单、壁纸选择、launcher 焦点、关机确认

## 变更摘要

VM 虚拟桌面（`cargo run -p auto-lang --features ui-iced --example ui_desktop`）实机验收
收集的 10 个 UX 问题集中清偿（用户 2026-09-03 截图反馈，四波次）：

- **W1 窗口 chrome 与几何（Q1-Q4）**：标题栏 mac 红黄绿灯 → `× / – / ⬛` 三键
  （关闭/最小化到任务栏/最大化还原，行内垂直居中）+ resize 把手光标反馈与
  clamp + 聚焦描边四边完整 + boot 开窗补 fit 适配（计算器空白根治）。
- **W2 任务栏与右键菜单（Q5/Q6/Q7/Q10-UI）**：任务栏底色与标题栏统一 alpha
  口径；非 start 键去 accent 紫；窗口 icon 去内嵌 ×改右键菜单；icon 外框统一
  start 尺寸/本体放大两号；桌面空白与任务栏右键菜单；托盘加关机键（确认后退出）。
- **W3 launcher 与热键（Q9/Q10-键表）**：launcher 条目 hover 高亮 + Pointer 光标 +
  召唤后搜索框自动焦点；删除 Escape 退出桌面的全局热键。
- **W4 壁纸选择（Q8）**：设置面板壁纸卡支持磁盘图片目录缩略图选择 +
  `set_wallpaper` 总线动词热切换（storage 持久化）。

**验收口径**：10 题逐条实机复验（ui_desktop 实机 + 截图），配套单测
（session 状态机/shell_at/desktop_command/desktop_protocol）全绿；
**全改不碰 wrap_layout_onclick 公共路径**（布局件右键/hover 基建延后，见待澄清⑤）。

**搭车清账**：KNOWN-DEBT P503-1（虚拟窗 min/max 视觉位无动词）由 W1 清偿。

## 目标

1. 标题栏三键 `× / – / ⬛`（非 mac 圆点）：行内在标题栏垂直居中；分别实现
   关闭、最小化到任务栏（任务栏 icon 点击恢复）、最大化/还原；resize 拖拽
   有光标反馈且不越出桌面区。
2. 聚焦窗口 accent 描边四边完整（标题栏 + 主面板左/右/下全部可见）。
3. boot 开窗（ui_desktop 直挂组件）按注册表 `window: "fit"` 声明适配内容
   尺寸——计算器窗口不再有大片空白。
4. 任务栏视觉：底色与窗口标题栏一致（同 alpha 口径）；仅 start 键保留
   accent；icon 外框统一 start 方格尺寸、icon 本体放大两号；运行中窗口
   icon 无内嵌 ×（右键菜单替代）。
5. 桌面空白处与任务栏均可右键弹出上下文菜单（desktop.at 条件面板模式）。
6. launcher：条目 hover 高亮 + Pointer 光标；召唤后搜索框自动聚焦可直打字。
7. Escape 不再退出桌面；任务栏右下角关机键 → 确认面板 → 退出。
8. 设置面板壁纸卡：可选磁盘图片（目录扫描缩略图 + 手输路径），保存即
   热切换、storage 持久（如 `D:\Down\stella-os\wallpapers` 下的图）。

## 架构方案

改动面三层（与 503/518 同型）：

1. **状态与消息层**（`crates/auto-lang/src/ui/session.rs`）：
   `WmCommand` 增 `Minimize(Wid)` / `ToggleMaximize(Wid)`；`VWinState` 增
   `minimized: Cell<bool>`、`maximized: Cell<bool>`、`restore_rect:
   RefCell<Option<iced::Rectangle>>`（替换现"rect≥98% 桌面即视为最大化"的
   派生判定，virtual_window.rs:184-186）；`DesktopCommand` 增 `Shutdown`、
   `SetWallpaper(String)` 两总线动词（parse 表 + enum）。
   `HotkeyTable::builtin()` 删 `ExitDesktop ← Escape` 映射（session.rs:3351）。
2. **渲染/执行层**（`crates/auto-lang/src/ui/iced/renderer.rs` +
   `iced/virtual_window.rs`）：三键图标化与居中；聚焦环 overlay 层；resize
   把手 cursor 与命中区加宽；`WmCommand`/`DesktopCommand` 新执行臂
   （Minimize/Maximize/Shutdown/SetWallpaper）；boot 开窗补 fit armed；
   summon_launcher 焦点 Id 稳定化；MouseArea 件 cursor/右键事件臂。
3. **壳面 DSL 层**（`crates/auto-lang/assets/shell.at`、`desktop.at`、
   `settings.at`、`examples/ui/028-launcher/src/front/app.at`）：任务栏样式
   类改造、右键菜单条件面板、关机确认面板、壁纸缩略图网格、launcher 行
   button 化。

数据流不变：壳面 .at → `__desktop_cmd` 总线 → `execute_desktop_commands`
（renderer.rs:8141）执行臂；窗口操作 → `DesktopMessage::Wm(WmCommand)` →
session 状态机。壁纸热切换复用 518 G1 set_theme 全链先例
（storage_host_publish + 字段回读 + snapshot::invalidate_all）。

## 技术栈

Rust（iced 0.14.2 / auto-lang ui 模块）+ AutoUI DSL（shell.at/desktop.at/
settings.at/launcher app.at）+ 既有测试设施（nextest 过滤器、desktop_protocol
ui-iced 测试组、autoui-verifier 实机脚本）。

## 需求分析与背景调查

用户实机反馈（2026-09-03，ui_desktop 窗口模式截图×2）列出 10 题。三路并行
代码调研结论（file:line 均已核实）：

| # | 问题 | 根因 | 位置 |
|---|---|---|---|
| Q1 | 窗口无法 resize，"放大"无效 | 链路完整但：把手透明无光标/无 hover 反馈、边命中区仅 6px；"放大"=绿点是纯装饰（黄绿点传 None，WmCommand 无 Min/Max 变体，KNOWN-DEBT P503-1）；Resize 臂无 max clamp | virtual_window.rs:224-242,129-130,267-271; renderer.rs:12940-12956; session.rs:926-956 |
| Q2 | 三圆点飘在顶边、mac 风格 | traffic_light 容器只给 left padding 未设对齐（iced 默认 Vertical::Top 钉顶边）；红=Close 有效、黄绿=装饰 | virtual_window.rs:124-144（TITLEBAR_H=36 line 31） |
| Q3 | 聚焦描边只围标题栏上侧左右 | 描边挂在 win_box 整框 border，但子树 body 铺满 bounds 把 1px 边框盖掉：client_area 自带高不透明底色盖住左/右/下，标题栏区无自身底色故透出 | virtual_window.rs:197-211,162-175; iced container draw 先画框后画子 |
| Q4 | 计算器窗口固定大小大量空白 | boot 路径 include 的是 app.at 不读 pac.at，`window:"fit"` 丢失、fit_pending 未 armed、registry_id 未回填（launcher 路径才有 fit） | renderer.rs:10177-10188; session.rs:1983-1998; app_registry.rs:51-53 |
| Q5 | 任务栏底色不一致/全紫/icon 带 × | ①taskbar `bg-card` alpha=1.0 vs 标题栏 Surface×t_alpha（off 档 0.95）；②shell.at 图标按钮无颜色类→button 默认 preset `bg-primary text-primary-foreground` 全紫；③窗口条目是 row{icon+×按钮} | shell.at:100,105,115,151,156-159,133-136; aura_view_builder.rs:5453-5463; virtual_window.rs:51-63 |
| Q6 | 桌面/任务栏无右键菜单 | button 有 oncontextmenu（desktop.at:87 图标右键先例）；但布局件 wrap_layout_onclick 只接左键、desktop.at 根 col 只有 onclick、shell.at 全文件无 oncontextmenu | renderer.rs:3361-3375,2671-2679; aura_view_builder.rs:5534-5537; desktop.at:47-71 |
| Q7 | 任务栏 icon 尺寸混乱 | shell.at 硬编码 h-10/h-12/h-9/h-8 × w-6/w-5/w-8/w-10 不一；PUA 字形恒 14px 与外框解耦（"大框小图"） | shell.at:105-261 各行; renderer.rs:3092-3093 |
| Q8 | 壁纸无法选图 | 机制已支持磁盘路径（storage `shell.desktop.wallpaper`，boot 验 is_file；load_image_bytes 本地分支 std::fs::read），但设置面板仅手输 input、无浏览/缩略图、且仅 boot 读无热切换 | renderer.rs:9452-9471,4540-4572; settings.at:388-414 |
| Q9 | launcher 无 hover 高亮/光标穿透/搜索框无焦点 | ①行是 row+`hover:` 类——hover 类只有 Button 臂消费，row 静默丢弃；②renderer 全文件无 mouse_cursor 设置，Stack 穿透取下层窗口光标；③召唤时 input_ids 空（首次挂载 view 未建）→ 回退字面量 "prompt_input" 错 Id，focus 空转（真实 Id 是 derive_input_id 派生） | 028-launcher/app.at:164-166; renderer.rs:7736-7742,16744; aura_view_builder.rs:5515-5523 |
| Q10 | Escape 误触退出桌面、无关机入口 | `HotkeyAction::ExitDesktop ← Escape` 在内置键位表；执行臂直接 restore+`iced::exit()`；shell 层无关机键、无确认面板（应用层先例：027 删除确认 popover） | session.rs:3351; renderer.rs:6979-6981,12823-12843 |

关键基建事实：**任务栏/桌面图标面本身是特权 .at App**（assets/shell.at 经
build_shell_component 进程内编译，Stack 一层），因此 Q5/Q6/Q7/Q10-UI 全部
在 DSL 层可解，无需 Rust 手绘任务栏。popover.rs 支持 at_point 坐标锚
（v1 不用，条件面板够用）。用户壁纸目录 `D:\Down\stella-os\wallpapers` 实存
5 张 jpg（raiden/red-demon-girl/room/showdown/violet）。

## 详细设计

### W1 窗口 chrome 与几何

- **A1 三键改造**（Q1-放大/Q2）：
  - `virtual_window.rs:124-132`：traffic_light 三圆点 → 三个 24×24 文本图标
    按钮（`×`、`–`、`▢`；hover 底 `bg-primary/10`），`:139-144` 容器加
    `.center_y(Length::Fill)` 修钉顶边。
  - `session.rs`：`WmCommand` 增 `Minimize(Wid)`/`ToggleMaximize(Wid)`；
    `VWinState` 增 `minimized/maximized: Cell<bool>` +
    `restore_rect: RefCell<Option<Rectangle>>`；`virtual_window.rs:184-186`
    的 98% 派生判定改读真状态。
  - `renderer.rs` 执行臂：Minimize → 置 minimized + 虚拟窗装配层跳过该窗
    （13175-13229 处 filter）；ToggleMaximize → maximized ⇄ restore_rect
    （maximized 时 rect=桌面可用区即减任务栏高度；顺带补 session.rs:926-956
    Resize 臂的桌面区 max clamp）。
  - 任务栏联动：shell 窗口列表注入（sync_shell_windows renderer.rs:9687）
    保留 minimized 窗；`WinFocus` 消费臂对 minimized 窗先解除再聚焦
    （点击任务栏即还原）。
- **A2 聚焦环 overlay**（Q3）：`virtual_window.rs` virtual_window_element 的
  9 层 Stack 顶部追加第 10 层"焦点环"：纯 border 容器（focused 时
  accent 2px 圆角同 WIN_RADIUS、背景透明、无 mouse_area——Stack 非捕获层
  事件穿透），win_box 自身 border 回退常驻弱描边（Surface token）。不透明
  子内容再也盖不到环。
- **A3 resize 可用性**（Q1-拖拽）：handle() 的 mouse_area 按 ResizeEdge 设
  cursor（N/S→`ResizeNs`、E/W→`ResizeEw`、四角→`ResizeNwse/Nesw`）；命中区
  EDGE 6→8、CORNER 14→16；Resize 状态机 clamp 上限=桌面可用区。
- **A4 boot 开窗 fit**（Q4）：renderer.rs:10177-10188 开窗循环按注册表条目
  匹配（source path 或 registry id；ui_desktop 的 APP_B=011-calculator 在
  examples/ui 注册表内有条目）→ `wm_add_win` 后补 `fit_pending/fit_enabled`
  置位 + `registry_id` 回填（session.rs:1983-1998 的 launch 路径同款）；
  400ms ServiceTick fit measure 即自动收缩。无注册表条目的组件维持现状
  60% 初值。

### W2 任务栏与右键菜单

- **B1 底色统一**（Q5a）：shell.at:100 taskbar `bg-card` → `bg-card/95`
  （与虚拟窗 off 档 0.95 对齐；low/high 档任务栏保持 95 不跟随——档位
  语义只作用于窗口内容区，注释言明）。
- **B2 去 accent**（Q5b）：shell.at 非 start 图标按钮（115/151/179/184/
  206-209/220-231/236-239/248-251）style 增 `bg-transparent text-foreground`
  （hover 保留 `hover:bg-primary/10`）；start 键（103-105）不动。
- **B3 窗口条目右键化**（Q5c）：删 shell.at:133-136、156-159 两处 `×` 按钮
  （`WinClose/NativeClose` 消息保留给菜单）；窗口/native icon 按钮加
  `oncontextmenu.prevent: .WinMenu(w.wid)`/`.NativeMenu(...)`；shell.at 增
  条件菜单面板（glass 样式抄 desktop.at:55-71：`bg-card/80 border
  rounded-xl shadow-xl w-44`），项＝聚焦/最小化（W1 新动词经
  `__desktop_cmd` 词表扩展 `win_min`）/关闭（既有 close/close_native）。
- **B4 icon 尺寸统一**（Q7）：外框统一 start 档 `h-10 w-10 px-0 rounded-xl`
  （pinned/窗口/分区/布局/铃铛/齿轮/关机全表归一；native 文本条目改方格
  图标化或高度对齐——见待澄清②）；icon 本体放大两号：renderer.rs:3092-3093
  PUA 字形 14px 常量改"跟随按钮字号（text-* 类解析值），回退 14"，shell.at
  icon 键统一 `text-lg`（≈18px）。
- **B5 关机键 + 确认**（Q10b）：shell.at 托盘挂点（256-258）加
  `button (icon: "power")` → `.ShutdownRequest` → 条件确认面板（027 popover
  式：`确认退出 Auto 桌面？[取消][退出]`）→ 总线新动词 `shutdown`；
  `DesktopCommand::Shutdown`（session.rs enum+parse 表）+
  renderer.rs execute 臂（复用 ExitDesktop 体：restore_all_native_slots +
  shutdown_broker + iced::exit()，renderer.rs:12836-12842）。
- **C1 桌面空白右键**（Q6）：renderer.rs MouseArea 件臂（3790-3809）+
  aura_view_builder.rs:7535-7551 mouse-area 事件表增 `oncontextmenu`
  （右键消息臂，iced mouse_area 原生 `on_right_press`）；desktop.at 根 col
  外包 `mouse-area`（onclick `.BlankPress` 迁入 + `oncontextmenu.prevent:
  .BlankMenu`）→ 条件菜单：更换壁纸…（open_settings 直达壁纸卡）/显示设置。
  **不动 wrap_layout_onclick 公共路径**。

### W3 launcher 与热键

- **D1 hover 高亮 + 光标**（Q9ab）：028-launcher/app.at:164-166 结果行
  `row` → `button`（style `w-full bg-transparent text-foreground
  hover:bg-primary/10 rounded-lg`；button 天然消费 hover 类 + 报 Pointer
  光标，两题同解）；launcher scrim 层包 mouse-area 置 Idle interaction
  阻断光标穿透到后方窗口（renderer MouseArea 臂补 `.interaction()` 支路）。
- **D2 自动焦点**（Q9c）：renderer.rs:7736-7742 首次召唤分支改用
  `derive_input_id` 派生稳定 Id（16744-16749 同款算法，launcher 搜索输入
  bind 键 SetQ）；并在召唤执行末尾补发一条 DM 消息驱动 update_inner 尾部
  `__focus_input` 消费（12294-12314）。实机验收：Ctrl+Space 召唤后直接打字
  即过滤。

### W4 壁纸选择

- **F1 目录扫描 + 缩略图**（Q8a）：storage 新键 `shell.desktop.wallpapers_dir`
  （默认空）；`toggle_settings` 注入点（renderer.rs:7989-8018）扫描目录
  （read_dir，jpg/png，取 stem 作 name、path 作 src，load_desktop_id_list
  9476 同型）注入 `__wallpapers` Obj 数组；settings.at:388-414 壁纸卡增
  目录 input（storage 直写）+ 缩略图 grid（image 件 src=绝对路径，
  load_image_bytes 本地分支已支持；onclick `.DraftWallpaper(path)` 复用
  既有 draft→SaveWallpaper 流）。
- **F2 热切换动词**（Q8b）：`DesktopCommand::SetWallpaper(String)`（enum+
  parse 表 `set_wallpaper`）+ execute_set_wallpaper（仿 execute_set_theme
  renderer.rs:8293-8300：storage_host_publish + `state.desktop.desktop_wallpaper
  = load_desktop_wallpaper()` + snapshot::invalidate_all——壁纸层每帧重建
  读字段即热切换）；desktop.at `MenuWallpaper` 菜单项直达壁纸卡。
  Q8 的"重启生效"痛点随此闭环。

## 测试设计

- **单测（随改随补）**：
  - session 状态机：minimize/maximize/restore 状态迁移、resize max clamp、
    键位表无 Escape→ExitDesktop（改 session.rs:5142-5143 既有断言）。
  - shell_at/desktop_command：`shutdown`/`win_min`/`set_wallpaper` 动词
    parse 往返（desktop_command 测试组扩 3 例）。
  - hotkey：desktop_hotkey_message 纯函数 Escape 不再出 ExitDesktop
    （改 renderer.rs:7062-7064 既有断言）。
  - registry：fit armed 补挂后 boot 路径标志位单测。
- **门禁（Category B：局部 Rust 模块改动）**：
  - 快速：`cargo check -p auto-lang`
  - 模块：`cargo t session` + `cargo t ui` + `cargo t shell_at` +
    `cargo t desktop_command` + `cargo test -p auto-lang --features ui-iced desktop_protocol`
- **实机验收（每波折叠前）**：
  `cargo run -p auto-lang --features ui-iced --example ui_desktop`，
  按验收标准 10 题逐条人工复验 + 截图存档（autoui-verifier
  `scripts/test_vm_mcp.py` 可驱 snapshot/screenshot 辅助）；
  review 终门跑一次 `cargo tf`（518 先例：ui 大改全量门禁）。

## 验收标准

1. [ ] 虚拟窗口拖任一边/角可 resize：光标随边缘变 resize 形状、有命中反馈，
       缩放不越出桌面区、不小于 160×120。
2. [ ] 标题栏三键为 `× / – / ⬛` 图标且在标题栏行内垂直居中：×=关闭；
       –=最小化到任务栏（任务栏 icon 点击恢复）；⬛=最大化/还原切换。
       （T16 二轮细化：三键右置 Windows 序 `– ▢ ×` 紧排，hover 光标
       Pointer 且图标容器背景提亮。）
3. [ ] 聚焦窗口 accent 描边四边完整可见（含主面板左/右/下三边），失焦描边
       回弱色。
4. [ ] 计算器 boot 窗口按内容收缩（fit 生效），无右侧/下方大片空白；launcher
       启动路径 fit 行为不回归。
5. [ ] 任务栏底色与窗口标题栏一致（同为 Surface/0.95）；仅 start 键 accent，
       其余 icon 键常态透明底+前景色；窗口 icon 无内嵌 ×，右键弹
       聚焦/最小化/关闭菜单且各项有效。
6. [ ] 桌面空白处与任务栏空白右键均弹上下文菜单（桌面：更换壁纸…/显示设置），
       点击空白处关闭。
7. [ ] 任务栏全部 icon 外框统一为 start 同尺寸方格，icon 本体约 18px
       （放大两号），无"大框小图/长条歪钮"。
8. [ ] 设置面板壁纸卡：可手输路径或从目录缩略图点选（验证
       `D:\Down\stella-os\wallpapers\room.jpg`），保存立即生效（无需重启），
       重启后持久；非法路径回退默认壁纸。
9. [ ] launcher：条目 hover 出现高亮、光标变 Pointer；召唤后搜索框自动聚焦，
       直接打字即可过滤（无需先点输入框）。
10. [ ] Escape 不退出桌面（键位表已删、实测无效）；任务栏右下角 power 键 →
        确认面板 →「退出」关闭桌面、「取消」留在桌面。
11. [ ] `cargo t session`/`cargo t ui`/`cargo t shell_at`/`cargo t
        desktop_command`/desktop_protocol(ui-iced) 全绿；零新增编译警告；
        review 前 `cargo tf` 全量门绿。

## 执行步骤

每步 = 独立可验证原子任务；波内顺序执行，波间可独立折叠
（511/514/517/525 多波先例）。

### 波1 窗口 chrome 与几何（Q1-Q4）

- [✅ 已完成] T1 (A1a) `session.rs`：`WmCommand` 增 `Minimize(Wid)`/`ToggleMaximize(Wid)`
      变体；`VWinState` 增 `minimized/maximized: Cell<bool>` +
      `restore_rect: RefCell<Option<iced::Rectangle>>` 字段并在 `wm_add_win`
      初始化。验证：`cargo check -p auto-lang`。
      [✅ 已完成] commit 9174ba6ea；check 绿（160 警告=存量基线，session 零新增）
- [ ] T2 (A1b) `renderer.rs`：两新 `WmCommand` 执行臂——Minimize 置位 +
      装配层过滤（13175-13229）、ToggleMaximize ⇄ restore_rect（可用区=
      host_size 减任务栏）；`virtual_window.rs:184-186` 派生判定改读
      maximized；`sync_shell_windows`（9687）保留 minimized 窗、`WinFocus`
      臂解除 minimized。验证：`cargo t session`。
      [✅ 已完成] commit 9174ba6ea；执行臂/装配过滤/hit_test 过滤/StartDrag·StartResize
      unmaximize 挂接；焦点即还原（focus 清 minimized）；投影跨分区全集天然保留
      minimized；cargo t session 89/89（新状态机测试在内）
- [✅ 已完成] T3 (A1c/Q2) `virtual_window.rs:124-144`：三圆点 → `×/–/▢` 图标按钮
      （红=Close 换 ×、黄→`–`=Minimize、绿→`▢`=ToggleMaximize），容器
      `.center_y(Length::Fill)`。验证：实机目测三键居中可用；
      `cargo t virtual_window`（若有）或 `cargo t ui`。
      [✅ 已完成] commit df3cbc2af；×/–/□ 字形键 24×24 命中 center_y 居中；实机目测并入波1折叠（T15）；
      schema a2vue virtual_window 2/2
- [✅ 已完成] T4 (A2/Q3) `virtual_window.rs`：Stack 顶追加焦点环 overlay 层
      （focused → accent 2px 环），win_box border 回退常驻弱描边。
      验证：实机聚焦/失焦截图四边完整。
      [✅ 已完成] commit df3cbc2af；焦点环 Stack 顶层非捕获空层事件穿透；实机截图并入波1折叠（T15）
- [✅ 已完成] T5 (A3/Q1) `virtual_window.rs` handle() 加 ResizeEdge→cursor 映射 +
      EDGE 6→8/CORNER 14→16；`session.rs:926-956` Resize 臂加可用区 max
      clamp。验证：`cargo t session`（clamp 单测）+ 实机拖拽八向。
      [✅ 已完成] commit df3cbc2af；ResizingHorizontally/Vertically/DiagonallyDown/Up 映射；
      家族各自钳制（视口优先 MIN）；clamp 单测两条入 wm_drag_and_resize_interaction；
      session 89/89；实机八向拖拽并入波1折叠（T15）
- [✅ 已完成] T6 (A4/Q4) `renderer.rs:10177-10188`：boot 开窗按注册表匹配补
      fit armed + registry_id 回填；补单测。验证：实机计算器窗口收缩贴合；
      `cargo t ui` 相关组。
      [✅ 已完成] commit 11ff0f349；ui_desktop 传源路径；arm_boot_fit_windows
      后缀唯一命中；boot_entry_matches 单测 1/1；实机计算器收缩并入波1折叠（T15）

### 波2 任务栏与右键菜单（Q5/Q6/Q7/Q10-UI）

- [✅] T7 (B1/B2/Q5ab) `assets/shell.at`：taskbar `bg-card`→`bg-card/95`；
      非 start 图标按钮（115/151/179/184/206-209/220-231/236-239/248-251）
      增 `bg-transparent text-foreground`。验证：`cargo t shell_at` + 实机。
      [✅ 已完成] commit 9706f5f3d；实机：任务栏底色与标题栏同色、仅 start 键 accent
- [✅] T8 (B3/Q5c) `shell.at`：删两处 `×` 按钮（133-136/156-159）；窗口/
      native icon 加 `oncontextmenu.prevent` → `.WinMenu/.NativeMenu`；
      增条件右键菜单面板（聚焦/最小化/关闭，glass 样式抄 desktop.at:55-71）；
      `__desktop_cmd` 词表增 `win_min`（session.rs parse 表 + execute 臂转发
      [✅ 已完成] commit 9706f5f3d；实机：窗口条目无内嵌 ×，右键「聚焦/最小化/关闭」弹出且动作有效
      WmCommand::Minimize）。验证：`cargo t desktop_command` + 实机右键。
- [✅] T9 (B4/Q7) `shell.at` icon 键外框统一 `h-10 w-10 px-0 rounded-xl`、
      [✅ 已完成] commit 9706f5f3d；实机：外框统一 start 档、icon 本体放大（PUA/lucide 跟随字号）
      字号 `text-lg`；`renderer.rs:3092-3093` PUA 字形改跟随字号回退 14。
      验证：实机任务栏截图（框齐/图大）；`cargo t ui`。
      [✅ 已完成] commit 9706f5f3d；实机：空白右键「更换壁纸…/显示设置」弹出、BlankPress 收起；desktop 142/142（金样重生成）
- [✅] T10 (C1/Q6) renderer MouseArea 臂 + builder 事件表增 oncontextmenu
      右键臂；`desktop.at` 根包 mouse-area（`.BlankMenu`）+ 空白右键菜单
      [✅ 已完成] commit 9706f5f3d（power 字形 82e48658b 后补）；实机：电源键→确认面板→取消留桌面
      （更换壁纸…/显示设置）。验证：`cargo t ui`（builder 解析）+ 实机。
- [✅] T11 (B5/Q10b) `shell.at` 托盘加 power 键 + 确认面板；`DesktopCommand::
      Shutdown`（enum+parse+execute 臂复用 ExitDesktop 体）。
      验证：`cargo t desktop_command` + 实机（取消留在桌面/退出真退）。

      [✅ 已完成] commit 6dc6480f1；实机：结果行 hover tint 可见（Calculator 行）
### 波3 launcher 与热键（Q9/Q10a）

- [✅] T12 (D1/Q9ab) `028-launcher/app.at:164-166` 行 button 化（hover 高亮 +
      Pointer）；scrim 包 mouse-area 置 Idle 阻断光标穿透。
      [✅ 已完成] commit 6dc6480f1；实机：召唤后直打 'calc' 落入搜索框并过滤 1/1；Escape 后桌面存活
      验证：实机 hover/光标；`cargo t ui` 回归 028 断言。
- [✅] T13 (D2+E1/Q9c+Q10a) `renderer.rs:7736-7742` 焦点 Id 用
      derive_input_id 稳定派生 + 召唤末尾补发消息驱动 `__focus_input`；
      `session.rs:3351` 删 Escape→ExitDesktop，同步 5142-5143/
      renderer.rs:7062-7064 断言与 ui_desktop.rs:26 注释。
      验证：实机召唤后直打字过滤；Escape 无效；`cargo t session`。

      [✅ 已完成] commit 82e48658b；实机：set_wallpaper room.jpg 即时铺满无需重启，已恢复默认
### 波4 壁纸选择（Q8）

- [✅] T14 (F1/F2) storage 键 `shell.desktop.wallpapers_dir` +
      `toggle_settings` 注入目录扫描 `__wallpapers`；`settings.at` 壁纸卡
      目录 input + 缩略图 grid；`DesktopCommand::SetWallpaper` 全链
      （enum/parse/execute_set_wallpaper 热切换）。
      验证：`cargo t desktop_command` + 实机选 room.jpg 即时生效、重启持久。

### 波5 追加反馈（用户二轮实机截图，2026-09-03）

- [✅] T16 标题栏三键右置 + hover 反馈（Q2 二轮细化）：
      a) 三键从标题栏左侧移到**右侧**（Windows 惯例序 `– ▢ ×`，关闭
      最右），紧排（间距 2px、右侧 8px 收边），左配重列保持标题居中；
      b) 修复 T3 引入的间隔失控根因——`container.center(Fill)` 把按钮
      宽高都置 Fill（iced 语义：width+height 同时 Fill），改回
      Fixed 宽高 + align 双中；
      c) hover 反馈：按钮化用 iced `button` 原生件——天然 Pointer
      光标 + `Status::Hovered/Pressed` 背景提亮（图标包围容器提亮
      ~10%/18%，圆角 6px，非全条填充——对照用户截图3 VSCode 形态）。
      文件：`crates/auto-lang/src/ui/iced/virtual_window.rs`。
      验证：`cargo t virtual_window`/`cargo t session` + 实机 hover
      截图（光标 Pointer、容器提亮、紧排右置）。
      [✅ 已完成] commit 59bc4ebab；右置紧排 + button 原生 hover/Pointer；
      实机 hover 截图并入 T15 收口验证

### 波6 追加反馈二（用户三轮实机截图，2026-09-03）

- [✅] T17 任务栏 icon 全面统一（Q7 二轮细化）：
      a) 全部 icon 按钮（start/pinned/窗口/布局/铃铛/齿轮/电源/分区触发）
      统一 start 规格：容器 `h-10 w-10 px-0 rounded-xl`、字形/图标本体
      `text-lg`（18px，PUA/lucide 已跟随字号）；
      b) 文本字形钮 lucide 化（⊞→layout-grid、▦→layout-grid、▤→table、
      ▢→square，消除文字/矢量字形视觉尺寸差）；
      c) 底色全部透明（含 start——三轮反馈不再保留 accent 底），hover
      用 `hover:bg-foreground/10`（语义前景 10% 透明：深色主题变亮/
      浅色主题变暗——Win11 形态，用户截图1）。
      文件：`crates/auto-lang/assets/shell.at`。验证：实机截图 + `cargo t shell_at`。
      [✅ 已完成] commit 409d414b8；实机：全栏 icon 同容器同字形（lucide 化）、
      透明底、hover:bg-foreground/10 主题自适应；shell_at 4/4
- [ ] T18 分区切换器重构（新需求）：任务栏不再直列分区条（1 ×/2 ×/+/⇥
      整组退役），换单枚 `square-stack` 触发 icon；点击开合悬浮预览面板
      （任务栏上方，bg-card/95 圆角浮层，宽度=分区数自适应不括满屏）：
      每分区一张卡（ws.label + pager 前 4 窗真缩略 window_thumbnail +
      current 高亮边），点击卡切换分区；面板尾部「新建桌面 +」卡
      （WorkspaceAdd，对照 Win11 截图2）。热键 Ctrl+Alt+[ / ] 切换时
      宿主写 shell `switcher_open="1"` 并由 400ms ServiceTick 计时
      （`DesktopState.switcher_until`，~1.6s）自动收起。
      文件：`crates/auto-lang/assets/shell.at`、`session.rs`、
      `renderer.rs`。验证：实机（触发开合/点击切换/热键 transient）+
      `cargo t shell_at`/`desktop_command`。
      [✅ 已完成] commit 409d414b8 + 8662e43d1；实机：触发开合面板（分区卡
      label+current 高亮+新建桌面卡、宽度自适应居中）、点击卡切换、热键
      Ctrl+Alt+] 往返切换窗口保留、transient 1.6s 自动收起；卡片内容包 col
      修横排；形状断言 4→2 更新；desktop 142/142；缩略叶空显=快照懒捕获
      既有行为（非回归，随 KNOWN-DEBT 候选记录）

### 波7 追加反馈三（用户四轮实机截图，2026-09-03）

- [ ] T19 壁纸满屏 + 默认显示磁盘壁纸：
      a) 顶缝根因——iced Image 默认 `ContentFit::Contain`（按比例 contain，
      宽高比不匹配时上下 letterbox 留空；实测顶部空一条，褐色块=内置
      ricepaper+深色 scrim）。修：`desktop_wallpaper_element` 加
      `.content_fit(ContentFit::Cover)`（铺满裁边，壁纸语义正解）；
      b) 默认壁纸链改造——storage `shell.desktop.wallpaper` 缺席/坏值时，
      不再直接回退 builtin，而是先取「壁纸目录首图」：目录解析链 =
      storage `shell.desktop.wallpapers_dir` → env
      `AUTO_DESKTOP_WALLPAPERS_DIR` → 探测 `D:\Down\stella-os\wallpapers`
      （存在即用，机器便利默认，注释言明）；目录无图再回退 builtin。
      抽 `wallpapers_dir_or_default()` 与 `scan_wallpapers_dir` 共用。
      文件：`crates/auto-lang/src/ui/iced/renderer.rs`。
      验证：实机首启（清空 wallpaper 键）显示 stella 壁纸满屏无顶缝 +
      `cargo t desktop`。
      [✅ 已完成] commit 491a047ff；实机：清键重启后 raiden.jpg Cover 满屏
      无顶缝（默认链 storage→env→stella 探测取目录首图）；壁纸分辨率测试
      按新链放行；desktop 142/142

### 波8 追加反馈四（用户实机反馈，2026-09-03）

- [✅ 已完成] T20 跨窗口首击吞按钮（git 事故恢复后继续，2026-09-03 实施）：
      症状——焦点在窗口1 时首击窗口2 的按钮，焦点切过去但按钮不触发，
      第二次点击才生效；点非交互空白区首击聚焦正常。

      根因（调研定位，证据链闭合）——**聚焦置顶重排发生在 press 与
      release 之间，iced Tree 按位 diff 弄丢 button 的 is_pressed 态**：
      1. 首击按钮：widget 树内 iced button 捕获 press 置 `is_pressed=true`
         （iced_widget-0.14.2 button.rs:307-312），客户区包裹 mouse_area
         因子捕获让行（mouse_area.rs:241）→ `Focus(wid)` 不从 widget 路径
         发出；焦点切换实际来自**并行订阅通道**：`ButtonPressed(Left)` →
         `DM::Wm(GlobalPress)`（renderer.rs:14013-14017）→ 命中臂
         `hit_test → wm_focus`（renderer.rs:13158-13171）。
      2. `WmState::focus` 聚焦即置顶：`z_order.retain + push`（session.rs:
         930-931），窗口2 层序从中途跳到末尾（顶层）。
      3. 桌面层按 `z_order` 迭代装配 Stack（renderer.rs:13507）→ 重排后
         `Stack::diff → Tree::diff_children`（iced_widget stack.rs:153-154
         → iced_core-0.14.0 tree.rs:96-100）**纯按位置 zip 匹配、不看
         widget Id**——错位槽位各拿别窗旧树 diff，button 持有的
         `is_pressed=true` 树被丢弃换新。
      4. release 到达：button 检查 `is_pressed`（button.rs:321）已为
         false → on_press 不发布，首击作废。二次点击时窗口已在顶
         （retain+push 幂等不重排），press→release 状态幸存 → 正常触发。
      5. 旁证：标题栏拖拽/客户区聚焦都是 press 驱动（mouse_area
         on_press），首击即生效——只有 release 驱动的 iced button 吞击。

      修复方向（实施时定稿）——推荐把「置顶重排」与「焦点记录」拆开：
      GlobalPress 命中新窗时只写 `focused`（焦点环即时移动，不改层序），
      `z_order` 置顶推迟到既有 `__mouse_released` 全局臂（renderer.rs:
      14004-14010，拖拽状态机同源）应用——release 后重排无害（无在途
      点击）。备选：Stack 层稳定 Id（iced 0.14 diff_children 无 Id 匹配
      路径，需上游改动，不推荐）。注意 `hit_test` 命中即未被遮挡，推迟
      置顶在 press→release 窗口内无可见层序反转伪影。
      文件：`crates/auto-lang/src/ui/session.rs`（focus 拆分）、
      `renderer.rs`（GlobalPress/`__mouse_released` 两臂）。
      验证：实机——焦点窗1 首击窗2 DualApp `-`/Reset/+ 与计算器键应一次
      完成切焦+按下；已聚焦窗连点无回归；`cargo t session && cargo t
      desktop`。
      [✅ 已完成] commit cf454b489（worktree plan-526-dev，merge ac6a1c7d1）。
      实施=GlobalPress 臂改 `wm_focus_soft`（只记焦点/还原最小化/MRU，
      置顶挂 `pending_raise`）+ `__mouse_released` 臂 `wm_apply_pending_raise`
      偿还；标题栏拖拽/缩放的即时置顶由 StartDrag/StartResize 臂自理
      （命中 chrome 时无按钮在途，不受影响）。
      验证（三层，全部绿）——① session 单测 2 个新用例：软聚焦延迟置顶
      语义 + pending 过期防护（同窗已顶/焦点被改写/窗已关 no-op），
      session 91/91；② layout_tests 机制双测试（真实 iced runtime
      UserInterface build→update(press)→into_cache→rebuild→update(release)
      管线，覆盖 Tree 按位 diff 与 is_pressed 跨事件存活）：修复前行为
      复现（press 即置顶→release 丢点击）+ 修复后（首击命中+release 偿还
      置顶），layout_tests 18/18；③ 组测试 shell_at 4/4、desktop_command
      3/3、desktop 142/142。
      遗留：SendInput 实机单击验证因机器存在持续并发鼠标输入（用户在用，
      40s 无空闲窗口）且 UIPI/按键态干扰不可靠，改由上述无头机制测试承载
      回归门；交互手感请用户日常使用中复核（桌面进程已带修复重启）。

- [✅ 已完成] T21 拖拽/缩放几何钳制 panic（本轮实机新发现，独立于 T20）：
      症状——桌面进程 panic 退出（exit 101）：`f32::clamp` at
      `num/f32.rs:1566` 报 `min > max, min = 8.0, max = -2.5`。时序与
      并发鼠标输入（用户拖拽窗口）强相关；T20 改动零几何数学，非本次
      回归。候选根因——某处 raw clamp 的 min/max 来自可负几何量（如
      `(host.width - 60.0)` 类差值未 `.max(0.0)` 护栏，或视口/可用区在
      极端拖拽下翻转），session.rs:1059 drag 钳制已有护栏但可能存在
      同族未护栏点；不排除 iced 内部布局 clamp 接收了我们下发的负尺寸。
      修复方向——启动加 `RUST_BACKTRACE=1` 复采一次栈；全面审计桌面
      几何路径的 clamp 调用点补 `max(0.0)`/min-max 归正护栏；补负尺寸
      驱动的单测。
      文件：`crates/auto-lang/src/ui/session.rs`、`iced/renderer.rs`
      （以回溯栈定位为准）。
      验证：复现路径回归测试绿 + 实机极端拖拽（边缘拖过对侧、缩到
      最小再拖出屏）不再 panic。
      [✅ 已完成] commit 6fec074d3（worktree 已折叠 master）。根因定位
      （静态审计全 crate clamp 调用点 + panic 参数匹配）：**不在 WM/桌面
      几何，而是 code_editor 滚动条渲染**——render.rs `track_h = viewport_h
      - THICKNESS(8.0) - 2`，编辑区被压到 ~7.5px 高时 track=-2.5，
      `natural.clamp(8.0, -2.5)` 即 panic 原样参数（min=8.0/max=-2.5）。
      该编辑器随 code-editor feature 进桌面进程（F12 console 等嵌入面），
      并发拖拽把嵌入面压扁即触发。修：抽 `scrollbar_thumb(track, natural)`
      纯函数——track 下限 1.0、min 侧取 min(THICKNESS, track)，竖/横两条
      滚动条共用；退化回归单测 `scrollbar_thumb_survives_degenerate_track`
      （含实录参数与全 track 域扫描）；同族 clamp 审计——其余调用点均为
      常量边界或已有 .max 护栏，无同族隐患。桌面实例已常驻
      RUST_BACKTRACE=1 启动，若另有残余 panic 可直接取全栈。
      验证：code_editor 39/39（含新回归测试）+ session/desktop 等组全绿。

### 波9 追加反馈五（用户六轮实机反馈，2026-09-03 五题）

- [✅ 已完成] T23 窗口客户区 hover 事件透穿：app 窗口底色半透明（518 G6 档位，
      纯视觉）但鼠标 hover 事件穿透到下层——拖动窗口经过桌面 icon 或
      背后 app 的 button 时被判定 hover，光标变 Pointer。根因候选——
      Stack 空层事件穿透语义下，客户区包裹 mouse_area 只捕获 press，
      CursorMoved 与 mouse_interaction 查询落到下层（下层 button 返回
      Pointer 并套用 hover 样式）。修：客户区包裹对窗口 bounds 内的
      hover/interaction 收口（mouse_area `.interaction(Arrow)` 挡光标
      查询 + hover 事件捕获路径），遮挡语义与视觉半透明解耦。
      文件：`crates/auto-lang/src/ui/iced/virtual_window.rs`（必要时
      pointer_area/renderer MouseArea 臂）。
      验证：实机拖动窗口扫过桌面 icon 与下层按钮，光标恒 Arrow、下层
      无 hover 提亮；客户区内交互不受影响。
      [✅ 已完成] 波9 提交 be2c17d62。修=`virtual_window.rs` 标题条/客户区
      包裹 mouse_area 加 `.interaction(Idle)`——iced 0.14 Stack::update 自带
      levitate 遮挡语义（上层回报非 None interaction 即对下层悬空光标），
      此前客户区回报 None 导致下层 button hover/Pointer 透穿；半透明是纯
      视觉与遮挡解耦。实机：遮挡点（Todo 窗压住计算器按键）光标句柄恒为
      iced Arrow（65539），无 Pointer 泄漏。

- [✅ 已完成] T24 最大化高度多算 1–2px：底部边界被任务栏盖住。根因候选——
      `toggle_maximize_win` 的 `usable_rect`（ReservedEdges::taskbar）
      与任务栏实际占高/顶缘阴影差 1–2px。修：maximize 高度扣掉任务栏
      实际上缘重叠量（或 taskbar 预留值对齐实测）。
      文件：`crates/auto-lang/src/ui/session.rs`、`ui/layout.rs`。
      验证：实机最大化后窗口底缘完整可见（与任务栏间无遮挡）。
      [✅ 已完成] 波9 提交 be2c17d62。根因坐实：`TASKBAR_HEIGHT=48` 而
      shell.at 任务栏行 `h-14`=56px（Tailwind 4px/单位，实测渲染 56），
      预留少 8px。修：常量 48→56（布局预留/最大化可用区/网格 dock edges
      同源联动）。session 4 处硬编码期望值 752/376→744/372 同步更新；
      session 91/91、desktop 142/142。

- [✅ 已完成] T25 焦点环圆角 vs 窗体内容方角外凸：焦点环 16px 圆角，但 app
      自绘背景方角，底部两角内容凸出环外。修（优先级序）——①客户区
      容器按窗体圆角裁剪（若 iced 0.14 clip 支持圆角路径）；②不可行
      则焦点环与窗体描边改方角（用户认可降级）。
      文件：`crates/auto-lang/src/ui/iced/virtual_window.rs`。
      验证：实机聚焦 app 窗口，底部两角无内容外凸（或环为方角）。
      [✅ 已完成] 波9 提交 be2c17d62。取证：iced 0.14 container clip 为矩形
      viewport 求交（container.rs:351），无圆角裁剪路径 → 方案①不可行，
      按用户认可降级②：`window_radius()` 分角圆角——顶部 16 圆（标题条带
      属 chrome 随 win_box 圆角绘制）、底部 0 方（与 app 方角内容一致），
      win_box/焦点环/阴影同源。实机：环贴合内容，底部两角无外凸。

- [✅ 已完成] T26 桌面 icon 双击打开不生效：桌面 icon 无双击语义。修——
      renderer MouseArea 臂/builder 事件表补 on_double_click 映射 +
      desktop.at icon 声明双击打开对应 app（单击行为保持）。
      文件：`crates/auto-lang/src/ui/iced/renderer.rs`（MouseArea 臂/
      builder 事件表）、`crates/auto-lang/assets/desktop.at`。
      验证：实机双击桌面 icon 打开对应 app；单击/右键行为不回归。
      [✅ 已完成] 波9 提交 be2c17d62。取证推翻立项假设：ondblclick 管线
      （484）与 desktop.at 声明均在，真因是 **DesktopBus 排水只读 shell
      的 `__desktop_cmd`**（session.rs drain_desktop_commands），desktop
      表面自己写的 `activate	<id>` 无人消费。修：drain 补排 desktop_app
      一路（`desktop_app` 字段既有）。实机：handler 直调 desktop
      ActivateApp("013-todo") → Todo 窗口弹出置顶聚焦（activate 执行臂
      renderer.rs:8441 既有）。双击手势本身（iced on_double_click，484
      既有）建议用户日常复核。

- [✅ 已完成] T27 任务栏 icon 右键 ContextMenu 宽度爆表：菜单横向沾满整个
      桌面宽度（应为目标内容宽度 hug）。根因候选——popover/菜单根
      容器 width Fill（shell.at 菜单根或 popover 表面默认撑满）。
      修：菜单根宽度 hug（w-auto 语义），最大宽上限可选。
      文件：`crates/auto-lang/assets/shell.at`（或 popover.rs 表面）。
      验证：实机任务栏 icon 右键，菜单宽度=内容宽，锚定在 icon 上方。
      [✅ 已完成] 波9 提交 be2c17d62（master 合并含 528 W9 缺省面板叠加
      +mut follow-up）。取证链——popover Panel content 布局 limits 上限=
      viewport，面板列无 width 类时 visual-wrap 容器按块级语义 Fill
      （renderer.rs apply_column_style）→ 撑满 1280（探针实证 content
      1280×112、子列 300×104 正确）。修：builder Popover 臂给无 width 类
      的面板注入 `Width(Auto)` + `IcedSize::Shrink` 新变体（convert_size
      Auto 误映射 Full→改 Shrink；assets 无 w-auto 依赖，零回归）。
      实机：菜单宽度=内容宽、锚定 icon 上方。与 528 W9 的 w-72 缺省
      （class 缺省场景）互补，合并冲突已叠加解决。

- [✅ 已完成] T22 标题栏三键命中盒正方形化（用户五轮实机截图反馈，
      2026-09-03）：症状——`– ▢ ×` 命中盒 30×24 长方形，横向视觉松散。
      修：`virtual_window.rs::title_button` 命中盒 30×24 → 24×24 正方形
      （glyph 13/10/13 均容纳；hover 圆角块 6.0 不变；三键组宽 94→76
      逻辑 px）。文件：`crates/auto-lang/src/ui/iced/virtual_window.rs`。
      验证：cargo check + layout_tests 18/18；实机截图（8915 验收进程
      重启后）：三键中心距 26 逻辑 px 紧拢，形态符合预期。
      [✅ 已完成] commit cddbd9abd（worktree plan-526-dev 已折叠 master）。

### 波间回归

- [✅ 已完成] T15 每波折叠前：实机 10 题逐条复验当波项 + `cargo t session && cargo t ui`；
      全波收口后 review 终门 `cargo tf`。
      [✅ 已完成] 实机收口验证（AUTOUI_ACCEPTANCE=1 验收通道 + computer-use 截图，
      2026-09-03）：10 题逐条 PASS——①计算器/新启窗 fit 贴合；②三键右置 – ▢ ×
      紧排居中；③聚焦环四边完整；④最小化→任务栏 icon→focus 还原全链实机；
      ⑤任务栏底色/去紫/无内嵌×；⑥桌面空白+任务栏条目右键菜单（动作有效）；
      ⑦外框统一+icon 放大；⑧壁纸热切换 room.jpg 即时生效；⑨launcher hover
      tint+召唤后直打字过滤（'calc'→1/1）；⑩Escape 后进程存活+电源键确认面板。
      组测试全绿：session 89/89（后 65/65 过滤组）、desktop_command 3/3、
      shell_at 4/4、settings 1/1、desktop 142/142、hotkey 7/7、launcher 4/4、
      desktop_protocol(ui-iced) 含 stage3 压测全绿。
      遗留目视：Popover 弹层锚点首次打开时横向偏左（菜单/确认面板功能与
      消失正常，497 hover 缩略同族先例）——记 KNOWN-DEBT 候选，不阻塞。
      退出总门 `cargo tf` 留 /auto-plan:review 终门执行（Category B 局部
      模块改动，AGENTS.md 门禁表）。

## 复审记录

- **复审人**：ZCode（/auto-plan:review，2026-09-03）
- **范围**：11 波 39 任务（T1–T39）——原十题 + 十轮实机追加反馈；验证基线 = master
  （worktree plan-526-dev 分支已完全折叠，2449c2f56 后无分叉；master 额外含 528 W9
  合并解决方案，以代码为准）。
- **全量门禁**：`cargo tf` 3399 run / **3397 passed / 2 failed**。唯二红
  `docs_gen kitchen_sink_page_in_sync` + `schema_drift schema_drift_fence` = **在案存量**
  （530 复审同款归因复现；531 记录源头 528 面/523 存量；KNOWN-DEBT 在档）——526 分支
  diff（ui/* + assets + examples/028-launcher）不含其输入。526 新增测试
  scrollbar_thumb_survives_degenerate_track 全绿（计数 +1 来源）。
- **逐项判定**：
  | 项 | 判定 | 证据 |
  |---|---|---|
  | 原 10 题（T1–T15） | PASS | 实机逐条 + 截图存档（波间回归记录）；组测试全绿 |
  | T16–T19（波2–7） | PASS | 各波实机 + 组测试（shell_at/desktop/session 等） |
  | T20 跨窗首击 | PASS | 机制双测试（真实 runtime 管线）+ session 单测；全手势用户复核项 |
  | T21 clamp panic | PASS | 根因修复（code_editor 滚动条）+ 退化回归单测 + 同族审计 |
  | T22/T24/T25/T27 | PASS | 实机截图/放大验证 |
  | T23/T26/T30/T33/T34/T36（波9–11） | PASS | 实机（截图/handler 直调链路实证）+ 组测试 |
  | T28/T29/T31/T35 | PASS | 代码+组测试；手势/视觉用户日常复核项 |
  | T32 Ctrl+Tab | PASS（实现）/复核项 | 两臂代码审查 + 组测试；全手势需真实按键，用户复核 |
  | T37 标题栏右键 | PASS（实现）/复核项 | 编译+装配验证；视觉/交互用户复核 |
  | T38 launcher | PASS（实现）/复核项 | app.at 改动+金样/组测试；滚动视觉用户复核 |
  | T39 截图护栏 | PASS | 最小化实机复验：干净错误回包、进程存活 |
- **遗漏捕获（复审闭环）**：T38 launcher app.at 改动未入 f2d6fcf9b 提交（git add 范围漏
  examples/）——复审折叠检查捕获，补提交 a532e673a 并折叠（2449c2f56）✓。
- **延后（用户核准/在档，非 silently dropped）**：①wrap_layout_onclick 布局件公共基建
  （待澄清③，独立立项候选）；②thumbnail 懒捕获空显/popover 首开锚点偏左（KNOWN-DEBT
  已登记 🟢）；③T20/T31/T32/T37/T38 全手势与视觉日常复核（验收通道无法合成 OS 输入，
  机制层均有测试承载）。
- **workaround 猎查**：无新增。T25 分角圆角为用户核准降级；T39 护栏为正解。
- **spec-impact**：supersedes=specs/ui/overview.md 桌面交互语义六项+两护栏；new=无；
  touched=GOAL-009。
- **结论**：全部判定 PASS、无阻塞债 → **status: reviewed**，可进 /auto-plan:merge。
  后续桌面问题（含 PLAN-533 实施）另立新计划跟踪（用户裁定，2026-09-03）。

## 待澄清事项

1. **②native 槽位条目形态**：任务栏 native 窗口条目现为文本长钮
   （`h-9 max-w-32 truncate`），T9 统一方格时是图标化（取 hicon）还是仅
   高度/圆角对齐保持文本？（默认按后者执行，图标化另立。）
2. **③壁纸目录默认值**：`shell.desktop.wallpapers_dir` 默认空（不扫描、仅
   手输路径），不把 `D:\Down\stella-os\wallpapers` 写死入库——用户机器路径
   由用户在设置卡里填一次（storage 持久）。是否需要 env/安装期默认注入？
3. **⑤布局件 hover/右键公共基建**：本计划全案避开 wrap_layout_onclick
   （launcher 用 button、桌面右键用 mouse-area）；布局件级 `hover:` 类消费
   与 oncontextmenu 臂留作后续独立计划（影响全示例回归面大）。
4. **⑥`⬛` 图标字形**：maximize 键用 PUA 图标库现有字形（square/
   maximize 类）还是字符字面量 `▢`？默认查 icon 库取名，无则字符。
5. **⑦T6 fit 匹配键**：boot 开窗按 source path 还是 registry id 匹配注册表
   条目？（默认 source path 后缀匹配，命中唯一才 armed，歧义回退现状。）

### 波10 追加反馈六（用户七轮实机反馈，2026-09-03 六题）

- [✅ 已完成] T28 标题栏三键仍宽体：T22 只收了内层内容盒 24×24，但 iced button
      自身主题 padding（水平更宽）仍在，hover 提亮盒呈宽体长方。修：
      `title_button` 的 button 显式 `.padding(0)`，命中盒=内容盒=24×24
      正方形。文件：`virtual_window.rs`。验证：实机 hover 盒正方形、
      三键紧拢。
- [✅ 已完成] T29 任务栏 icon 右键菜单鼠标一动即消失：popover ondismiss 与条目
      mouse-area onmouseleave 都走 `.HoverEnd`，后者把 win_menu 一并清空
      ——右键开菜单后鼠标滑出条目即被杀，无法移动鼠标去点选项。修：
      HoverEnd 只清 dock_hover；win_menu 的关闭走 popover ondismiss
      （点外部）/选项点击/Esc 专设清理。
      文件：`crates/auto-lang/assets/shell.at`。验证：实机开菜单后鼠标
      随意移动菜单不消失，点选项生效，点外部/Esc 关闭。
      [✅ 已完成] 波10 提交 16fbcf4b4。根因实锤：shell.at:419 HoverEnd
      连带清 win_menu，而条目 mouse-area onmouseleave 即发 HoverEnd——
      右键开菜单后鼠标滑出条目即被杀。修：新增 WinMenuClose 消息，
      HoverEnd 只清 dock_hover；两处 popover ondismiss 改指 WinMenuClose
      （点外部/Esc 关）；动作项自带清零不变。实机：handler 开菜单 →
      SetCursorPos 移出条目 → 截图菜单仍存活 ✓。
- [✅ 已完成] T30 桌面 icon 右键菜单与任务栏不统一：现为准内联渲染（把图标挤
      下）且样式不符。修：改用与任务栏同款 popover（placement/样式对齐）。
      文件：`crates/auto-lang/assets/desktop.at`。验证：实机 icon 右键
      弹悬浮菜单、样式与任务栏一致、不挤压布局。
      [✅ 已完成] 波10 提交 16fbcf4b4。icon 菜单改 popover（锚=图标本体
      button，placement bottom-start，class 与任务栏同款 p-1 border
      rounded bg-card）；被取代的准内联 menu_id 面板块移除。实机：handler
      直调 IconMenu → 悬浮菜单锚定图标、不挤压网格布局。
- [ ] T31 桌面 icon 双击打不开 app（T26 排水修复后仍不工作）：双击手势
      → ondblclick → ActivateApp → __desktop_cmd → drain → LaunchApp 链
      逐环取证（log/HANDLERNotFound/iced 双击检测状态是否被重建打断）。
      文件：`desktop.at`/`renderer.rs` MouseArea 臂。验证：实机双击
      icon 打开对应 app。
      [✅ 已完成] 波9 提交 be2c17d62 的排水修复即为根因。取证：运行日志
      实录用户双击 → `[UI_EVENT] ActivateApp` → `VM_HANDLER_OK`（手势、
      iced 双击检测、handler 三环全通）→ 下游 drain/T26 修复后
      ActivateApp("013-todo") 实机弹出 Todo 窗口全链验证。用户此前测试
      落在旧二进制（T26 合并前的 master 实例）。双击手势请日常复核。
- [✅ 已完成] T32 Ctrl+Tab 切换语义标准化：现首按仅打开切换器且选中=当前窗，
      需多按 Enter。修为 Windows/Linux Alt-Tab 语义——首按 Ctrl+Tab 即
      预选下一个窗口（MRU 序），松开 Ctrl 即提交聚焦；按住期间再按
      Ctrl+Tab 继续下一个（尾回卷首）；Esc 取消。
      文件：`session.rs`/`renderer.rs`（switcher 状态机与热键臂）。
      验证：实机 3 窗场景连按/松手/回卷行为。
      [✅ 已完成] 波10 提交 16fbcf4b4。两臂实现——①SummonSwitcher 臂：
      召唤后紧接一次 .Advance（RebuildMru 复位 sel=0=当前窗后顺延一位，
      首按即预选下一个）；按住期间再按走既有 Advance（尾回卷首 overlay
      自管）。②桌面 __modifiers_changed 臂（DM::Window 处理块新增）：
      prev 含 Ctrl/新值不含 且 switcher 可见 → 注入 .Pick 提交选中
      （Focus → __desktop_cmd → 排水聚焦）。Esc 取消、Tab/←→ 手动选、
      Enter 提交均保持。实机全手势（真实 Ctrl+Tab 按住/松开）需 OS 键盘
      输入，验收通道不含——请用户日常复核。
- [✅ 已完成] T33 布局预设历史切换：右下角"重新排列窗口"icon 应用预设布局后，
      再按同一 icon 应回到应用前的手动布局（快照-恢复切换）。
      文件：`shell.at`/`session.rs`/`renderer.rs`（快照存储与切换臂）。
      验证：实机手动摆窗→应用预设→再按同键→恢复原位。
      [✅ 已完成] 波10 提交 16fbcf4b4。实现——①shell.at 重排 icon 改发
      `layout_toggle	<mode>` 新动词（与裸 SetLayout 的内部重排语义区分，
      native slot min-size 扩张等路径不误触）；②WmState 增
      layout_snapshot：Free→预设迁移时采集当前分区窗口矩形；③
      TogglePresetLayout 执行臂：同预设再按 = 恢复快照回 Free，跨预设
      保留最初快照。实机：layout_toggle grid 前后截图——双窗对半网格
      ✓ → 再按 → 精确恢复手动排布 ✓。

### 波11 追加反馈七（用户八轮实机反馈 + RUST_BACKTRACE 抓获第二崩溃源，2026-09-03）

- [✅ 已完成] T34 桌面 icon 右键菜单三合一代修复：①双菜单——T30 popover 化时
      旧内联 menu_id 面板未删（python 编辑 cwd 漂移误落 master 误导排查），
      删残留内联块；②菜单项颜色不对（accent 底+看不清的字）——项样式
      对齐任务栏（bg-transparent text-foreground text-left hover:bg-primary/10）；
      ③挤压 icon——内联块删除后 popover 悬浮即 sole 渲染。
      文件：`desktop.at`。验证：实机右键单菜单、悬浮、样式同任务栏。
      [✅ 已完成] 波11 提交 f2d6fcf9b。三问题同源——T30 popover 化时旧内联
      menu_id 面板未删（编辑 cwd 漂移落错树误导排查）→ 双菜单+挤压；菜单
      项无 bg-transparent/text-foreground → 默认 accent 底紫块+深字不可读。
      修：精确锚点 python 删两个内联块（blank+icon），popover 三项样式
      对齐任务栏（px-2 text-sm text-left bg-transparent text-foreground）。
      desktop_surface_at_loads 解析+交互断言全绿（含 a2vue 金样再生 57 行）。
- [✅ 已完成] T35 删任务栏无用方框按钮（用户七轮反馈截图2）：任务栏托盘区
      square 框 icon 无功能。文件：`shell.at`。验证：实机 icon 消失、
      其余托盘不动。
- [✅ 已完成] T36 桌面空白右键无反应：blank_menu 内联面板呈现位置可疑且随
      T34 统一。修：blank 菜单 popover 化（锚=桌面 mouse-area 全域，
      snap 钳制回视口），含"更换壁纸…"与"显示设置"项。
      文件：`desktop.at`。验证：实机空白右键弹悬浮菜单、动作可用。
      [✅ 已完成] 波11 提交 f2d6fcf9b。blank 菜单 popover 化（锚=桌面全域
      mouse-area，placement bottom-start + Panel snap 钳制回视口左下）；
      BlankClose handler 补全（此前 ondismiss 缺 handler 即菜单关不上）。
      含"更换壁纸…"（open_settings 动词）与"显示设置"。实机 handler 直调
      BlankMenu 面板出现（截图存档）。
- [✅ 已完成] T37 app 标题栏右键菜单：至少含 最大化/最小化/关闭，最好含发送到
      其他分区（SendTo 既有动词）。实现：virtual_window 标题条 mouse_area
      加 on_right_press → WmCommand::TitleMenu{wid} → 该窗 Stack 顶层
      chrome 自绘菜单浮层（Rust 装配，非 .at），BlankPress/GlobalPress
      关闭语义对齐 T29（菜单不随鼠标移动消失）。
      文件：`virtual_window.rs`/`session.rs`。验证：实机右键标题条弹
      菜单、各项动作有效、鼠标移动不消失。
      [✅ 已完成] 波11 提交 f2d6fcf9b。实现——①WmCommand 增
      TitleMenuOpen/Close；②标题条 mouse_area on_right_press 开菜单；
      ③virtual_window Stack 顶层 chrome 自绘浮层（180px 面板：最大化
      /还原、最小化、关闭 + 分隔线 + 发送到下/上一分区——SendFocusedTo
      既有臂复用）；④GlobalPress 任意左键按压即关（点外部关语义，菜单
      项消息先于订阅消息处理动作不受影响）。全手势（真实右键/鼠标移动）
      请用户日常复核。
- [✅ 已完成] T38 launcher tag 样式与结果滚动：①分类 tag 栏样式错误（选中/非选
      中的可读性坏了）——统一为与 `all` tag 相同样式；②结果多时无滚动
      条——结果列表容器加 max-h + 滚动（VM 侧 build_scrollable 语义）。
      文件：`028-launcher/app.at`。验证：实机 tag 可读、样式统一；结果
      超长出滚动条可滚。
      [✅ 已完成] 波11 提交 f2d6fcf9b。tag 统一 all 同款（bg-primary/15
      text-primary，muted 未选中态在深底不可读退役）；结果列
      max-h-96 overflow-y-auto（build_scrollable cap 语义出滚动条）。
      实机 overlay 目检请用户复核（验收截图通道不覆盖 overlay 层）。
- [✅ 已完成] T39 autoui_screenshot 零尺寸窗口 wgpu panic（RUST_BACKTRACE 抓获
      第二崩溃源，独立于 T21）：最小化（物理尺寸 0）的桌面窗口执行
      `window::screenshot` → wgpu create_texture "Dimension X is zero"
      → exit 101（wgpu-27.0.1 backend/wgpu_core.rs:1588，栈顶
      iced_wgpu::Renderer::screenshot）。修：screenshot 前查窗口物理
      尺寸为零/最小化则跳过（返回上帧或空）。文件：`mcp_server.rs`
      （autoui_screenshot 工具）/renderer 截图任务装配处。验证：最小化
      状态下 autoui_screenshot 不再杀死进程。