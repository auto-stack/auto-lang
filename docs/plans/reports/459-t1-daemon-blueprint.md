# Plan 459 T1 施工图：iced::daemon 迁移评估

> **日期**：2026-08-28　**性质**：只读评估（T2–T5 前置，同 453 T1/T4c 方法）
> **对象**：`renderer.rs` `run_dynamic_iced`（5909–8163）、`session.rs` 全量、
> iced 0.14 `daemon.rs`/`application.rs`/`window.rs` 源码（registry 实测）。
> **基线**：master bdb4df01b。

## 1. daemon 入口实测签名（iced 0.14.0 源码为准）

| 挂点 | application（现状） | daemon（目标） | 迁移动作 |
|---|---|---|---|
| boot | `Fn() -> impl IntoBoot`（State 或 (State, Task)） | **同左** | 返回 `(session, Task)` 携带开窗 Task |
| update | `Fn(&mut State, Msg) -> Task<Msg>` | 同左 | 零改动（外壳三分支重构） |
| view | `Fn(&State) -> Element`（**无 window::Id**） | `Fn(&State, window::Id) -> Element` | per-window 路由 + 打标 |
| title | `Fn(&State) -> String` | `Fn(&State, window::Id) -> String` | per-window 标题 |
| theme | `Fn(&State) -> Theme` | `Fn(&State, window::Id) -> Option<Theme>` | 忽略 win，原样 |
| subscription | `Fn(&State) -> Subscription` | 同左 | per-app 扇出（§4） |

**两个决定性差异**：
1. **daemon 默认不开窗**：boot 必须返回 `window::open(Settings)` 的 Task；
   `window::open` **同步返回 `(Id, Task<Id>)`** —— boot 期即可拿到
   window::Id 并同步 `register_window`（`pending_initial_size` 等待-转正
   通路对 boot 开窗不再必要；Opened 事件臂保留作幂等兜底）。
2. **全窗口关闭不退出**：daemon 需显式 `iced::exit()`。`WindowClosed` →
   `windows` 空 → 返回 `iced::exit::<DesktopMessage>()`（= 期望语义
   "全窗口关闭才退"，比 application 的单窗即退更准）。
   开窗 Task 用 `Task::discard()` 丢弃完成通知（iced_runtime task.rs:222）。

`window::Settings::default()`：resizable=true、decorations=true、visible=true，
与 application 默认窗口一致；`position` 用 `Position::Specific` 级联偏移
（第 n 窗 +48n px，避免 demo 双窗完全重叠）。

## 2. 消息形状定案（§2.3 落地）

- **`DesktopMessage` 扩第三变体 `Window(iced::window::Id, IcedMessage)`**：
  承载 listen_with 的 Resized/mouse/modifiers 等**带窗口上下文**的事件。
  update 侧 `app_of_window(win)` 现场解析（活注册表，无构造期焦点陈旧性）。
- **widget 消息保持 `App(AppId, IcedMessage)` 形状冻结**（453 T8 冻结 +
  session 单测锚定）：daemon view 按窗口构造，出口闭包捕获 app_id 打标
  `DM::App(app_id, m)` —— `map_to_app` 硬编码退役。
- update 分派：
  - `DM::App(app, m)` → `window_of_app(app)` 反查（459 一窗一 App）→
    `split_mut_at(app, win)`；无窗口（注册竞态）空转返回。
  - `DM::Window(win, m)` → `app_of_window(win)` → 同上。
  - `DM::Desktop(ev)` → 注册表维护；Closed 后空 → `iced::exit()`。

## 3. 窗口级 4 字段归位 WindowEntry（453 T4c 预留位兑现）

`window_size`/`pending_window_resize`/`initial_resize_done`/
`initial_focus_done` 从 DesktopSession 平铺迁入 `WindowEntry`（字段已
预置、零消费方）。`SessionViewMut/Ref` 四字段改指 `windows[win]` 条目
（`split_mut_at(app, win)` = apps+desktop+windows 三路字段级拆借，NLL
无冲突），**update_inner ~1900 行函数体零改动**。增 `app_id: AppId`
字段供 view 的 MCP 门控 / console 过滤。平铺字段删除（含 `__test_session`）。

新会话构造：`DesktopSession::allocate_app(component) -> AppId`（递增
计数器）+ `register_window`；`primary_app()` = 注册表最小 AppId（=
注册表首个登记窗口，453 §2.3 的"主窗口语义"，454 由 WM 接管）。
`desktop_app_id()` 硬编码退役（唯一残留=测试锚定处删除）。

## 4. 订阅按 App 扇出（453 T5 原案）

| 订阅 | 归属 | 打标 |
|---|---|---|
| hot_reload_tick / widget_tick / __timer_tick | per-app（source_path/tick_interval/pending_timers 都是 component 私有） | `DM::App(app)` |
| toast 到期 tick | 桌面共享 toasts —— **只订一份**，归 primary_app | `DM::App(primary)` |
| mcp_action / mcp_heartbeat / shell_event | 进程级服务，MCP 单 App 语义冻结（T8） | `DM::App(primary)` |
| keyboard_subscription | **per-app**：bindings Arc 捕获进闭包（退役 KEYBOARD_BINDINGS 全局静态），listen_with 的 window_id 过滤本 App 窗口 —— 按键只路由到发生窗口的 App | `DM::App(app)` |
| listen_with（Resized/mouse/modifiers） | 单份，消息带窗口上下文 | `DM::Window(win, m)` |
| desktop_window_events | 原样（已产 DesktopEvent） | — |

update_inner 返回 Task 的回标：外壳 `task.map(move |m| DM::App(app, m))`
（app 为 Copy 捕获）；update_inner 内部零 map_to_app（grep 实测）。

## 5. DevTools/console 隔离（验收"互不串扰"的支撑改造）

- **DevToolsState 从 DesktopState 迁入 AppState**（per-App）：renderer
  155 处 `.desktop.devtools` → `.app.devtools` 机械替换（grep 实测全部
  在 renderer.rs，结构体字段与新()` 不变）。F12 经 keyboard 订阅按窗口
  路由 → 只开发生窗的 DevTools；选中/hover/检查器互不串扰。
- **console 打标**：`enable_ui_console` 缓冲载荷改 `Vec<(u64, String)>`
  （u64=AppId 原值，0=进程级）；`print` builtin 推入处读 **CURRENT_APP
  AtomicU64**（update 外壳入口 set，UI 线程串行无竞态；boot/shell 线程
  读到 0=全局）。view 排空按 `tag==0 || tag==app` 过滤 → 各窗口 Console
  只见自己的 print + 进程级行。`vm::ui_console` 环形缓冲（CLI 用）不动。

## 6. view 侧定案

`view_desktop_fn(state, win)`：`app_of_window` 反查 → 未登记窗口回退
占位元素（计划 §2.2）→ `split_ref_at` → `dynamic_view(view)`（签名从
&DesktopSession 改为按值 SessionViewRef）→ catch_unwind → `.map` 打标。
`dark_mode`/`accent`/`set_window_width` TL 在各自窗口 view 构建内
set→convert 同步自消（无需门控）；**MCP 快照同步门控 `app==primary`**
（T8 单 App 语义：MCP 永远看到主 App，demo 双窗不互踩）。
MCP 截图维持 `iced::window::oldest()`（=主窗口，N1 per-window 记 454）。

## 7. panic 注入开关（T5"示例内置开关"定案）

`AUTOUI_PANIC_PROBE=1` + demo `.at` 内 `button "Crash" { onclick: .panic_probe }`：
- update 外壳 DM::App 臂入口：event==`panic_probe` 且 env 开 → `panic!`
  （落在 catch_unwind 内）+ 记录 `PROBE_CRASHED_APP=app_id`（static）。
- view 出口：`PROBE_CRASHED_APP==本窗 app_id` 且 env 开 → `panic!`
  → view 侧 catch_unwind 落崩溃页元素（该窗持续显示，另一窗不受影响）。
双 static + 两处 env 门控臂，验收后保留作隔离性回归仪器（计划文注明）。

## 8. 提交序列（每步独立可编译）

| # | 内容 | 验证 |
|---|---|---|
| C0 | 本施工图（docs-only） | — |
| C1 | session.rs：DM::Window 变体、allocate_app/primary_app/window_of_app、窗口级字段归位 WindowEntry、split_at 拆借视图、DevToolsState 迁 AppState、console 打标 | session/ui:: 单测 |
| C2 | renderer daemon 迁移：boot 开窗、update 三分支、view per-window、订阅扇出、exit-on-empty、155 处 devtools 路径替换 | `cargo check` + ui:: 子集 |
| C3 | 双窗口 demo：lib.rs 抽 `build_dynamic_component`、`run_dynamic_iced_multi`、example `ui_dual_app` + demo `.at`、panic 探针 | 实机双窗 demo |
| C4 | 验证收尾：I2 五套 desktop_mcp 复跑、I3 复查、T5 隔离实录、文档核销 | 见计划 §4 |

## 9. 影响面与风险对账（计划 §5）

- lib.rs 组装段（run_file_dynamic_ui_inner 500 行）：**不动 iced 链路**，
  仅 C3 抽构件构造函数（复用现有解析/注册逻辑，行为不变）。
- 单窗口行为：boot 仍开一窗（尺寸/标题/主题/订阅形状同 application），
  I2 五套 desktop_mcp 为硬门槛。
- MCP/DevTools 多窗口缺口（N1 per-window 截图、跨窗 toast 到期刷新）：
  按计划记 454，不阻塞 demo。
