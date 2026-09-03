---
plan_id: PLAN-526
status: executing             #  # 用户指示：继续实机检查与追加反馈，review/归档暂缓（T19 后） drafting → executing → execution_done → reviewed → archived
feature_name: desktop-shell-ux-fixes
author: [zhaopuming]
created_at: 2026-09-03
updated_at: 2026-09-03

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []

affects: [ui/session, ui/iced, shell.at, desktop.at, settings.at, 028-launcher]
current_step: 19
total_steps: 20
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

- [ ] T20 跨窗口首击吞按钮（用户指示仅记录根因，暂缓修复）：
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

（/auto-plan:review 回填）

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
