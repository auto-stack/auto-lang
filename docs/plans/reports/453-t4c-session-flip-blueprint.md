# Plan 453 T4c 报告：会话结构翻转施工图

> **日期**：2026-08-27　**性质**：只读分析（T4c 前置评估，同 T1 施工图方法）
> **对象**：`renderer.rs` 运行循环对 `DynamicState` 的全部读点 + `session.rs`
> 会话类型与 renderer 的接缝。
> **结论**：走**路线甲**（`DesktopSession` 直接作运行循环 State），以
> **借用拆分视图（SessionViewMut/Ref）** 承接函数体局部命名，函数体零改动；
> 窗口级 4 字段收编为 DesktopSession 平铺"主窗口状态"（454 归位 per-window）；
> M1 写侧改道 `__modifiers_changed` 消息载荷；Opened 捕获改道
> `desktop_window_events` 消息通路后 `PENDING_WINDOW_OPENS` 过渡通道退役。

## 1. 读点普查（改面前基线，grep 实测）

| 读点形态 | 处数 | 翻转后去向 |
|---|---|---|
| `state.desktop.*` | 173 | **零改动**——DesktopSession 同名直属字段直通 |
| `state.app.*` | 149 | facade `app: &mut AppState` 直通，零改动 |
| `state.component.*` | 131 | facade `component: &mut DynamicComponent` 直通，零改动 |
| `state.window_size` 等 4 窗口级 | 14 | facade `window_size` 等 4 字段直通，零改动 |
| `state.opened_windows` | 2 | 随 DynamicState 溶解删除（注册表接管） |
| `&mut state.component` 传参 | 10 | 改 `&mut *state.component`（自动解引用不适用于函数实参） |
| helper fn 形参 `&DynamicState` | 16+2 | 改 `SessionViewRef`（按值 Copy）/`&mut SessionViewMut` |
| `state.input`、裸 `state.` | 6 | 均为注释，不涉及 |

结构外引用：`DynamicState` 在 renderer.rs 之外**零代码引用**（仅注释）。
`update_inner`（6087–7936，~1900 行）内部不触窗口注册表；视图区（8159+）
无 `&state.component` 形态。

## 2. 路线对比与定案

- **甲 + facade（定案）**：boot 经 `DesktopSession::single` 构造 State；
  `update_inner` 首行 `state.split_mut(desktop_app_id())` 拆出互不相交的字段
  借用（Rust 字段级借用分割保证 component/app/desktop/窗口级同帧可变），
  函数体沿用旧平铺命名 → **~470 处读点零文本改动**。改动面收敛在：
  boot/shell/订阅签名、facade 构造、10 处传参星号、18 处 helper 签名。
- **乙（DynamicState 作壳）**：改动最小，但 State 类型仍非会话聚合，
  I3 纯度欠账需记债务——放弃。
- **甲 + 纯重命名（279 处 sed）**：拒绝。view 侧 helper 以 `&DesktopSession`
  整体借用与 `app` 局部借用交错处有 NLL 冲突风险，且 13k 行文件大规模 churn。

facade 非过渡 hack：它是 Rust 标准的"拆借用重构视图"，与 T1 施工图"保形状
搬迁"同一纪律；`DynamicState` 类型本身溶解，运行循环 State 即 `DesktopSession`，
不构成 standalone/desktop 双路径。

## 3. 窗口级 4 字段的归宿

`window_size` / `pending_window_resize` / `initial_resize_done` /
`initial_focus_done` 收编为 **DesktopSession 平铺的"主窗口状态"**：
view/update 无 window::Id 形参（iced 0.14 `application` 三件套不携带），
只能服务主窗口；`windows: BTreeMap<Id, WindowEntry>` 注册表保持
生命周期职责（Opened 登记 / Closed 注销 / 反查），条目内的窗口级字段
454 多窗口化时接管。boot 初始化 = 旧 DynamicState 逐字段同值，行为不变。
`register_window` 的 `pending_initial_size` 转正只影响注册表条目
（renderer 不读条目字段），无行为面。

## 4. M1（LAST_MODIFIERS 收敛）写侧改道

订阅闭包只在构造期拿到 `&State`（不可变），listen_with 回调在别的线程
执行无法触达状态——这是 thread-local 的由来。改道：修饰键事件的
`__modifiers_changed` 消息带上 `modifiers.bits()` 载荷（走既有 stringly
线格式 `input_value`），update 的既有臂（6766，现仅强制重建）解析载荷写
`desktop.current_modifiers`；view 删除 TL→字段拷贝行，直接读字段。
时序论证：view 必然晚于任一 update 批次重建，与 TL 方案的窗口等价；
首帧前字段初值 `Modifiers::empty()` 与 TL 初值一致。

## 5. 桌面事件扩臂（随翻必做 3/5）

`desktop_window_events()` 扩臂：`Opened{size,..}` → `WindowOpened(id,size)`、
`Focused`/`Unfocused` → `WindowFocused/Unfocused(id)`；外壳 `DM::Desktop`
臂消费：Opened→`register_window`、Closed→`unregister_window`（已接）、
Focused/Unfocused→`desktop.focused_window` 记录（Design 23 §2 会话层
"焦点与主题策略"的最小底座，454 消费）。Opened 捕获改道后，
`PENDING_WINDOW_OPENS` 通道与 drain、业务订阅内的 Opened 臂一并退役
（消息通路取代进程通道，T3c 过渡设计完成使命）。

## 6. 提交序列（每步独立可编译可测）

| # | 内容 | 验证 |
|---|---|---|
| C0 | 本施工图（docs-only） | — |
| C1 | session.rs：DesktopSession 收编窗口级 4 字段 + focused_window + SessionView 拆借视图 + DesktopEvent 扩臂 | session 单测 |
| C2 | renderer 翻转：boot→`DesktopSession::single`、运行循环 State 切换、DynamicState 溶解、drain→注册表接线 | ui:: 子集回归 |
| C3 | M1：修饰键载荷化，TL 删除，唯一源入 DesktopState | ui:: 子集回归 |
| C4 | Opened 改道 desktop_window_events，PENDING_WINDOW_OPENS 退役 | ui:: 子集回归 |
| C5 | 计划文档状态头 + 独立复审 + clippy/fmt | `cargo check/test -p auto-lang --features ui-iced` |

## 7. 验证门与已知豁免

- 每步：`cargo check -p auto-lang --features ui-iced` + 定向
  `cargo test -p auto-lang --features ui-iced --lib ui::`。
- **豁免**：`ui::style::plan411_tests::test_md_hidden_classes_parse`
  为 master 既有失败（`f9bf0b9a1` 引入，与本程序无关），不计入门禁。
- T7（双 App demo + desktop_mcp.py 50/0）为 T4c 后续独立切片，不在本图。
