# Plan 453: AutoUI 多 App 会话运行时（AppSession / DesktopSession 拆分）

> **状态**: 🔄 进行中（T1 ✅ T2 ✅；T3 分片推进——**T3a 桌面域 ✅、T3b App 域 ✅**
> （2026-08-27：合计 50 成员迁入会话聚合，ui 域测试 486 绿两轮）。
> T3c-producer ✅（Opened→PENDING_OPENS 通道+2 测试；drain 写回随 T4）。待做：T3c-consumer/T4 扇出、T5 订阅打标……）
> **来源**: Design 23 §6 里程碑 M1——虚拟桌面程序的会话层底座，452（IME spike，
> ✅ 归档）的直接后续。452 spike 报告提供三项直接设计输入：① 主窗口 id 由
> shell 内部生成并丢弃，必须经 `Event::Window(Opened)` 自捕获（窗口注册表的
> 必要性实证）；② 订阅消息返回的 iced Task 在纯 0.14 应用中正常执行
> （renderer.rs:6048 的 wart 是动态渲染器包装层局部问题）；③ 桌面级键盘监听
> 天然可用（无焦点也可见），全局热键无需额外机制。
> **架构依据**: `docs/design/23-autoui-virtual-desktop.md`（R3 退化桌面 /
> R4 接缝 / R6 兼容纪律；不变式 I2/I3）；编号解析见
> `docs/plans/autos-desktop-program.md` 计划一览。
> **基线**: master fd454efad
> **性质**: 渲染层内部重构（renderer.rs 会话化），对外零行为变化。

## 1. 目标

把"一进程 = 一窗口 = 一 App"的 `run_dynamic_iced` 单体改造为**多 App 会话
运行时**：

- **DesktopSession**（进程唯一）：OS 窗口注册表、全局订阅路由、MCP 宿主、
  焦点与主题策略、DevTools 基础设施。
- **AppSession**（每 App 一个）：`DynamicComponent` + VmBridge 原样复用，
  附各自的输入缓存、订阅声明与 panic 边界。
- 同一进程内多 OS 窗口、各承载一个 AppSession。渲染形态不变（每窗口一棵
  Element 树）；本计划只建会话层，VirtualWindow 单窗口嵌入留给 454。

验收核心 = Design 23 不变式 **I2**（`auto run file.at` 行为逐项一致）+
**I3**（不得出现 standalone/desktop 双路径分支，R3 独立窗口 = 退化桌面构造）。

**非目标**：VirtualWindow 容器与 WM（454）；AppWindow 渲染叶子接缝的后端化
（454+）；跨 App 业务路由 DesktopBus（454，本计划仅预留 `(AppId, Message)`
封装形状）；进程外隔离（386）；MCP 自动化的 `(AppId, widget)` 寻址实现
（T8 仅预留类型）。

## 2. 设计要点

### 2.1 会话模型与消息封装

```rust
pub struct AppId(pub u64);            // 本计划内与 window::Id 一对一

pub struct AppSession {
    pub id: AppId,
    pub component: DynamicComponent,           // ui/dynamic.rs 现成对象
    // 从 DynamicState 迁入的 App 域状态（input_values 等，按 T1 清单）
}

pub struct DesktopSession {
    apps: BTreeMap<AppId, AppSession>,
    windows: BTreeMap<iced::window::Id, AppId>, // spike 输入①：Opened 时登记
    desktop: DesktopState,                      // DevTools/MCP/订阅等公共域
}

/// 统一消息扇出形状——454 VirtualWindow 直接复用
pub enum DesktopMessage {
    App(AppId, IcedMessage),
    Desktop(DesktopEvent),                      // hot_reload / window / tick…
}
```

### 2.2 DynamicState 字段迁移映射（T1 先行）

初判三组（基于 renderer.rs:5555 起 ~60 字段抽查，其余 T1 补全成施工图）：

- **App 域**：`component`、`input_values`、todos 类跟随 VM 的状态、
  `pending_window_resize`。
- **桌面域·DevTools**：debug_mode、hovered/selected_widget/vnode 系列、
  inspect_mode、inspector_subtab/sections、devtools_open/tab、console_output
  （十余个 `RefCell` 字段 + `current_modifiers`）；**`LAST_MODIFIERS`
  thread-local 改挂 DesktopState**。
- **桌面域·基础设施**：MCP server 句柄（现每次 run 起一个，renderer.rs:5852
  → 桌面唯一）、hot_reload watcher、toast 队列、theme。

迁移纪律：**保形状搬迁**——453 只把字段按域装进 DesktopState/AppSession，
不顺手重构 DevTools 结构（避免 scope 爆炸；那属于后续独立小计划）。

### 2.3 订阅路由

- `subscription()` 拆为 `DesktopSession::subscription()`：桌面级订阅（hot
  reload、shell SSE、窗口事件）+ 按 App 生成的订阅（widget_tick、键盘绑定）。
- `listen_with` 回调用第三参 `window_id` 反查 windows 注册表 → 打上 AppId
  标签汇入 `DesktopMessage::App`。
- 任务出口统一从 Desktop 层返回（spike 输入②支持直接沿用 iced Task；
  6048 的已知坑以 desktop_mcp.py 作守门回归）。

### 2.4 panic 边界

- `AppSession` 的 update/view 调用外裹 `catch_unwind`；panic → 桌面 toast +
  该窗口切崩溃占位页，其余窗口不受影响。
- 前置审计：DynamicComponent/VmBridge 路径上不存在跨 unwind 逃逸的
  锁持有或 FFI 边界（T6 子项，检查通过才允许 catch_unwind 生效）。

## 3. 任务表

| # | 任务 | 内容 |
|---|---|---|
| T1 | 字段清点施工图 | ✅ 完成（2026-08-26）：`reports/453-t1-dynamic-state-inventory.md`——57 字段四组归位 + 3 项结构外全局态，含合并裁定 M1 / 备注 N1 / 清理候选 C1 |
| T2 | 会话类型落地 | ✅ 完成（2026-08-27）：`ui/session.rs`——AppId/AppState(13)/DevToolsState(36)/DesktopState(M1 合并+KEYBOARD_BINDINGS 迁入)/WindowEntry/DesktopMessage；7 个 renderer 私有类型提 pub(crate)；单测 5 绿 + ui::iced 回归 40 绿 |
| T3 | 入口瘦身 | `run_dynamic_iced`（renderer.rs:5736）变薄壳：构建单 App DesktopSession → 进入同一 loop；lib.rs:2933–3430 链路只改组装不改语义 |
| T4 | 消息扇出 | update 入口按 DesktopMessage 分派至 AppSession；LAST_MODIFIERS 迁入 DesktopState 并清理 thread-local 读点 |
| T5 | 订阅路由 | §2.3；keyboard_subscription（renderer.rs:5336）等按 AppId 标签化 |
| T6 | panic 边界 | §2.4（审计 + catch_unwind + 崩溃页 UI） |
| T7 | 双 App 验证 | 示例：一个进程两个 AppSession 各占一 OS 窗口（DevTools/输入值互不串扰）；作为 M1 验收 demo |
| T8 | MCP 兼容冻结 | mcp_server/mcp_action_subscription 维持 single-app 语义；`AppTarget` 类型占位不给实现 |

## 4. 验收

1. **I2 逐项一致**：现有测试全绿作硬门槛——desktop_mcp.py 50/0、UI 相关
   测试、示例冒烟、a2vue 金样不受影响。
2. **I3 门禁**：grep 无 `standalone`/`desktop` 条件双路径；单一 loop。
3. 双 AppSession demo 正常运行且隔离（日志、输入、DevTools 选择互不污染）。
4. 注入 panic 只落当前窗口崩溃页，另一窗口持续可交互。
5. `cargo test -p auto-lang --features ui-iced` 与 clippy 全绿；session.rs
   有独立单测（路由表、panic 边界、退化桌面对等性）。

## 5. 风险

| 风险 | 缓解 |
|---|---|
| LAST_MODIFIERS thread-local 读点多且散 | T1 施工图先行列出全部读点；提供 DesktopState::modifiers 访问器逐一替换 |
| DevTools 与 DynamicState 耦合深 | 保形状整体搬迁，功能不在本计划重构 |
| 任务出口行为差异引发隐性回归 | desktop_mcp.py 50/0 作守门；可疑点对照 spike 输入②定位层级 |
| 13k 行大文件合并冲突 | T2 类型先行并入主线；T3–T6 分批小步提交 |

## 6. 关联

- **Design 23**：R3/R4/R6、I2/I3、§6 里程碑图。
- **Plan 452**（archive）：spike 报告 reports/452-ime-spike.md（设计输入来源）。
- **Plan 365/386**：会话层为 386 Stage 3 宿主提供其复用的会话/订阅底座
  （I1 零删除约束的第一批受益者）。
