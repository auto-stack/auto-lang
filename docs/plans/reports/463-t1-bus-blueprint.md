# 463 T1 报告：DesktopBus 接缝施工图（命令通路 + shell 形态定案）

> **结论**：
> 1. **命令通路（shell→WM）：候选 B（状态变量命令总线）定案**——泛化已量产的
>    `__toast` 管线；候选 A（VM builtin `desktop.launch()`）降为后续语法糖路径。
> 2. **反方向（WM→shell）：`window_width` 同型状态注入**（宿主写 shell App 声明
>    状态变量 + 置 view_dirty）。
> 3. **shell 形态：.at 特权 App 定案，无需降级**——动态列表渲染能力已证实
>    （025-dashboard 的 `for x in .listVar` 同款）。
>
> 状态：定案（2026-08-28，代码证据链见 §2/§3，未走实机——本报告只依赖
> 已合入 master 的 462/412/458 管线事实，无未验证假设）。

## 1. 定案对象（计划 463 §3.2）

| 决策点 | 定案 | 替代路径（保留） |
|---|---|---|
| shell→WM 命令 | 候选 B：状态变量命令总线 `__desktop_cmd` | 候选 A：VM builtin（`toast()`→codegen 重写同款，作为后续糖） |
| WM→shell 状态 | 宿主写 `__wm_windows` 等声明变量 + view_dirty | store 注入（MCP SharedState 同型）不需要——无跨进程诉求 |
| shell 形态 | .at 特权 App（`assets/shell.at`，宿主 include_str! 编译装载） | Rust 直构（`dynamic_view` 旁路）——已无必要 |
| 命令枚举 | `DesktopCommand::{LaunchApp, CloseWindow, FocusWindow, SetLayout}` | — |
| 结果回执 | 成功=新 `Wid`（写回 shell 状态）；失败=toast（`DesktopState.toasts` 既有） | — |

## 2. 命令通路定案依据（候选 B）

### 2.1 管线先例（全部已量产）

- **`__toast`（Plan 412，同型 App→宿主命令通路）**：.at handler 写
  `__toast` 状态（`handler_codegen.rs:237` 追加式赋值，记录以 `\x1E` 分隔、
  字段以 `\x1F` 分隔）；renderer update 最前读+清（`renderer.rs:6370`），
  逐记录入队。**同进程、同 update 循环、同步消费**——正是 shell→WM 需要的形状。
- **`window_width`（Plan 402/462，同型宿主→App 数据注入）**：App 声明变量，
  宿主每帧 `read_state` 回读（update 侧）；反向 `write_state` + view_dirty
  的注入半边在 458 主题播种（`run_session` 开头 seed dark_mode/accent）已用。

### 2.2 候选 A 否决理由（v1 范围）

- VM builtin（`native_catalog.rs` 路线，如 `storage.get`=1106）在 **VM 执行环
  境内**解析，拿不到 `DesktopSession`（它被 iced update 闭包独占借用）；
  接通需全局 channel + 订阅轮询（`MCP_ACTION_RX`/`SHELL_EVENT_RX` 同型）——
  比状态变量**更多**机制（异步、乱序、轮询延迟），收益为零。
- 候选 A 的正确归宿是**语法糖**：codegen 把 `desktop.launch("x")` 重写为
  `__desktop_cmd` 追加写（`toast()` → `__toast` 完全同构）。v1 shell.at 直接
  显式写状态变量；糖随 464 launcher 顺手落（届时 API 面才冻结）。

### 2.3 消费点与时延（防丢命令）

- 消费函数幂等（读+清），挂 **desktop update 外壳两处**：
  1. 外层 `update` 入口（desktop 模式门控）——任意消息周期先排空；
  2. `DM::App` 派发返回后二次排空——handler 本周期写入下周期即达
     （`__toast` 同款时序，`renderer.rs` 注释已论证该顺序的必要性）。
- 空闲兜底：462 已订的 `desktop_service_tick`（400ms）保证命令至多
  400ms 被消费；按钮点击路径实测 ≤1 周期。丢命令窗口不存在（写与消费
  同在 update 线程，无竞争）。
- 编码复用 toast 约定：`verb\u{1F}arg` 记录、`\u{1E}` 连接
  （`launch\u{1F}011-calculator`、`focus\u{1F}3`、`close\u{1F}3`、
  `layout\u{1F}grid`）。Wid 以 u64 十进制传输。

## 3. 反方向定案（WM→shell 窗口列表）

- shell.at 声明：`var __wm_windows str = ""`（记录 `wid\u{1F}title\u{1F}icon\u{1F}focused`）
  与 `var __wm_meta str = ""`（`layout\u{1F}focused_wid`，任务栏布局按钮态）。
- 宿主侧：WM 状态变化点（focus/close/launch/布局切换/drag 落位）统一调
  `sync_shell_windows(state)`：序列化 → `write_state` 进 shell App →
  置该 App `view_dirty`。**变更驱动 + 400ms tick 兜底**，不做每帧无条件写
  （避免无谓 view 重建）。
- 任务栏渲染：shell.at 用 `for w in .wm_win_list` 同款循环
  （025-dashboard `for p in .procs` 先例，可变长度列表渲染能力已证实，
  计划 §6 风险项「.at 动态列表渲染不确定」就此解除，无需 spike 降级）。
  列表解析：.at 侧 handler 把 `__wm_windows` 拆成 list state（split 语义
  .at 已有；464 launcher 同用）。

## 4. shell 装配形态

- **来源**：`crates/auto-lang/assets/shell.at`，`include_str!` 内嵌，
  boot 期 `build_dynamic_component(SHELL_AT, None)` 装载（与 `auto run`
  同管线，`fire_init` 已调；`lib.rs:3568` 先例）。
- **登记**：desktop boot 先 allocate shell App，`DesktopState` 增
  `shell: Option<ShellHandle>`（记录 AppId + 任务栏高度常量）。shell App
  **不进虚拟窗 z-stack**：`view_desktop_fn` 的 desktop 分支跳过它，
  改在层列表尾部追加 shell 层（底部锚定、全宽、定高 48px）+ 空占位
  overlay 槽（464 消费；层序：背景 → 虚拟窗 z-stack → shell → overlay）。
- **布局预留**：`ReservedEdges { bottom: 48.0 }` 传给 layout 引擎，
  虚拟窗排布不含任务栏区。
- **独立模式零影响**：`RunMode::Standalone` 不装载 shell（I3 配置位分叉），
  `auto run` 管线不变。

## 5. DesktopCommand 形状（session.rs，`WmCommand` 邻位）

```rust
pub enum DesktopCommand {
    LaunchApp(String),      // registry app id（目录名），T7 注册表查表
    CloseWindow(Wid),
    FocusWindow(Wid),
    SetLayout(LayoutMode),  // ui/layout.rs 新枚举：Free | Grid | MasterStack
}
```

- LaunchApp 执行体（T4）：registry 查找 → `build_dynamic_component` →
  `allocate_app` → `wm_add_win`（初位 = free 模式级联偏移）→ 聚焦 →
  成功回执写 shell（`__wm_meta` 追加 `launched\u{1F}wid`）；构建失败 →
  toast 报错 + 占位页组件（T7 占位页 .at，保证「窗起而 App 空白」不出现）。
- CloseWindow/FocusWindow 直通 `WmCommand::Close/Focus`（462 已有臂）。
- SetLayout：`WmState` 增 `layout: LayoutMode`；切换时 layout 引擎
  重算全部矩形写回（T2）。

## 6. taskbar 登记（I4，T5 落地）

- 生产表新增 `taskbar`（tier: builtin_widget，iced: "full"，web: "partial"——
  a2vue 契约 465 实现）→ `SCHEMA_DRIFT_GENERATE_AT=1` 重生成 `schema/aura.at`。
- iced 臂：tracked/untracked **两处镜像**（D-GAP 规则）——
  `"taskbar" => container(row 语义)`（surface 背景 + 顶缘描边 + 水平排布）。
  锚定（贴底）由 §4 的 desktop 装配层做，widget 只负责条形 chrome。
- shell.at 是 taskbar 的唯一 v1 消费者；`virtual_window` 登记随 465
  （462 报告 §5 已冻结该边界，本计划只落 taskbar）。

## 7. 对 T2–T6 的约束传递

| 任务 | 吸收的本报告决策 |
|---|---|
| T2 layout | `ReservedEdges.bottom=48`；free/grid/master-stack 纯函数无副作用；snap 预览矩形几何 |
| T3 全屏 | boot `Settings` 注入点 = `run_session` desktop 臂（decorations/mode）；背景层既有 |
| T4 命令 | §2.3 双消费点 + §5 枚举与执行体 |
| T5 shell | §3/§4 装配 + `assets/shell.at` + §6 taskbar |
| T6 热键 | 键位定案：`Ctrl+Tab` 窗口循环（Windows 下 OS 吞 Alt+Tab，实测定案降级键位，Alt+Tab 到达即优先）、`Ctrl+Alt+G/L` 布局、`Ctrl+Space` 召唤（IME 抢键时 `Ctrl+Alt+Space` 备选）；全部走桌面级键盘订阅新臂（`DM::Wm`/`DM::Desktop(SummonLauncher)`） |
