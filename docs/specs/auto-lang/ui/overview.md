# ui（AURA / UI 引擎 / 桌面运行时）

> **Status**: active（主战场：vue 轨 codegen 成熟化 + VM 轨视觉 parity + 虚拟桌面线推进中）
> 最近刷新：2026-08-29（Plan 386 归档回写：路线 B 桌面协议 v1.1——五通道/命名管道+共享内存传输/broker/L2 两进程落地）

## 职责

Auto 的 UI 子系统，围绕 **AURA**（UI-IR）组织，2026-08 起扩展为**桌面运行时**：

- **前端解析**：UI 方言（`widget`/`msg`/`model`/`view`/`on`/`setup`/`actions`，dialect 机制）。
- **AURA 提取与校验**：WidgetDecl → 视图树/状态/事件三元 IR，按 `schema/aura.at`（唯一契约源，plan-435）校验。
- **代码生成**（`ui_gen/`）：a2vue（主力）、a2rust、a2jet/a2ark、ts adapter、widget 契约表、block 层。
- **VM 运行时渲染**（`ui/`）：vnode/事件路由/VM 桥接、iced/gpui/headless 后端、code_editor 与
  autodown_editor 内建 widget、热重载、MCP 调试服务。
- **桌面运行时**（2026-08 新增）：`session.rs`（DesktopSession/AppSession 双层会话 + WmState/
  WmCommand 桌面消息）、`ui/iced/virtual_window.rs`（VirtualWindow 渲染）、排布纯函数与
  DesktopBus v0——单 OS 窗口内多 App 虚拟桌面。
- **a2ui 协议** 与 **`#[api]` 前后端契约**（`src/api/`）。

## 现状（2026-08-28）

**组件线（plan-437 落地，ADR-18/19）**：chart 族（line/bar/area/donut）schema 契约落库
+ vue 发射 spec 驱动化（特判臂退役）；四类图官方 Auto 组件（AutoLineChart 等四件，
widget-parens props + Init 几何 + 段记录打包，载体 widgets-gallery components/）；
VM 轨子组件 Init 渲染期补发（props 播种→Init→build，vue onMounted 对齐）——
派生计算型组件双轨可用的地基。契约细节见 [design/chart-components.md](design/chart-components.md)。

**导航组件线（plan-482 落地）**：nav-item/nav-group/nav(search:) 组件族——
class 契约单一来源（`ui_gen/nav_contract.rs` ↔ 脚手架 NavItem/NavGroup 镜像，
单测锁死双端不可漂移）；hover/active/disabled 三态、icon（lucide svg/emoji 双
通道）/label+desc/badge 槽、路由 to:（vue RouterLink / VM `__navigate`+历史栈
`router.back()`）与 onclick 状态双模式、data-active 与 nav-name/nav-desc 语义
锚；顺修 lucide_svg 裸片段缺陷（此前全部 VM lucide 图标空渲染）。三 app 落地：
015-notes/018-book-reader/widgets-gallery + 外部 auto-musk(052)/auto-os-config(012)。

**slot 替换（plan-476 落地）**：VM 轨 widget 插座/填充与 vue 轨语义对齐——调用位
`slot(name:X){..}`/裸子节点渲染到子 widget outlet，父作用域求值+父事件路由+逐帧重求值；
机制为构建期 `SlotFills` 父 builder 捕获 + 五容器×双胎兄弟拼接，
详见 [design/slot-substitution.md](design/slot-substitution.md)。

**桌面线（452→459→462→463→465 落地，464 未开工，386 暂缓）**：
452 翻转"Windows 非 compositor"裁定并验证 IME/焦点分区可行 → 459 iced daemon 多 OS 窗口 +
会话化（453 的 DesktopSession/AppSession 拆分）→ **462 路线 A 地基**：VirtualWindow widget
（Stack/clip/mouse_area 组合）+ WmState/Wid + DM::Wm 消息 + 桌面级键盘路由 → 463 桌面 shell：
全屏 borderless 宿主 + 任务栏 overlay + free/grid/master-stack 排布纯函数 + pac.at 应用注册表 +
DesktopBus v0。**465 vue 端虚拟桌面已落地**（M4）：`auto run --desktop` 宿主 scaffold
（auto-man `generate_desktop_host`：scan_apps 过滤 + `src/apps-registry.ts` 静态 import 注册表 +
子组件/stores 合并）+ WM 运行时（auto-man `assets/wm/`：store.ts WmStore、layout.ts 布局 TS
直译〔I6 与 layout.rs 共享期望值表 layout_cases.json〕、keyboard.ts R12 热键捕获段、
VirtualWindow/Taskbar.vue DOM 叶——schema/aura.at `vue:` 登记源两端同源）；E1 `(AppId,event)`
注入形状与 E2 AppWindow 叶枚举成文（`docs/plans/reports/465-t4-wm-dom-leaf.md`）；
tauri 全屏壳复用同一宿主页。**464 launcher 已落地**（SummonLauncher 懒挂载 + 真注册表平行串列注入 + windowless 特权 App 拆借垫片）。**472 AutoShell 地基已落地（shell-track M1）**：DesktopBus v1 对账定案（候选 B 传输 + `desktop.*` 动词词表 8 动词，Design 25 §3 注记回写）+ 投影协议 v1 合同 （`schema/projection-protocol-v1.md`：`__wm_*` 六字段全集/`__wm_workspaces`/指纹门控，双端对拍基线）+ workspace 分区驱动（WmState 加法增域、默认 2 分区、过滤六点）+ shell.at 升格 `widget Desktop` dock（图标化/pinned activate/切换条/`shell.dock.*` 数据级配置，pinned 宿主解析 {id,icon} 注入）。M2（switcher/pager）消费面就绪。**478 shell-track M2 已落地**：投影协议升版 v1.1（`__wm_mru` 当前分区 MRU 序投影 + `__wm_workspaces.label` 1 基标签 + 指纹扩段 + 动词词表增 `workspace_add`/`workspace_close`/`send_to`，vue 端对拍基线）+ switcher overlay（`assets/switcher.at` 进程内嵌第二枚 overlay 槽：Ctrl+Tab 召唤/推进、MRU 快照平行串列注入 + RebuildMru、Tab/←→/Enter/Esc 键盘流、点击聚焦、键盘独占 + Esc 仲裁 + 仅 visible 推层）+ dock 切换条升格 pager（1 基 label、当前分区高亮、每分区 × 删除——宿主 toast 门（非空不删/末分区保底）、尾部 + 增分区即入）+ send_to 跨区发送（Ctrl+Alt+Shift+←/→，WmCommand::SendFocusedTo）+ 驱动配套（remove_workspace 重排相邻前驱/下标压实/clamp/焦点让渡、move_win_to_workspace、mru_in_workspace）。热键表：Ctrl+Tab 改道 switcher（Alt+Tab 保留 CycleWindow，463 键位 v1 不动）。**479 shell-track M3 已落地**：463 瞬时 toast 升格「浮现+历史聚合」双面（`push_notification` 单入口——入史+未读+落盘+浮现+面板活更新，既有 8 处 toast 调用点改道）+ 通知中心（dock 铃铛+未读 badge——`__wm_notes_unread` 条件消费空串/零双守卫 + 第三枚 overlay 槽 `assets/notification_center.at` 右下锚定面板：懒挂载/快照注入 note_ids·kinds·msgs·ats+RebuildNotes、开面板未读清零、逐条 ×/全部清除/Esc 关、键盘独占+Esc 仲裁+仅 visible 推层）+ `notify`/`notes_toggle`/`notes_clear`/`notes_dismiss` 动词（词表 v1.2）+ storage 定长槽 `shell.notes.0..9` 持久化（persist_notes 全量重写/restore_notifications boot 恢复，NOTES_CAP=50 内存 FIFO MRU）+ 投影协议升版 v1.2（`__wm_notes` 全量 {id,kind,msg,at} + `__wm_notes_unread` 未读串 + 指纹尾接 `|notes:{len}:{front_id}:{unread};` 段，vue 端对拍基线）。M4 settings/OS 桥 457·386 待续。**473 native dock 假洞 Phase 1 已落地**：`ui/native_dock/`（NativeSlot 模型+状态机+策略纯逻辑 + Win32 适配层 #[cfg(windows)]/非 Windows no-op——EnumWindows+PID 发现/DWM ext-frame 几何写读回/GWL_STYLE 剥离还原/DWMWCP_DONOTROUND 直角/sink_desktop_below z 序〔insertAfter 语义实测勘误：对 desktop 调 SetWindowPos 沉到 slot 之下〕/WinEventHook 五事件 OUTOFCONTEXT 钩子线程〔RwLock 数据槽×占用 AtomicBool 分立防回调死锁〕）；DesktopBus 动词增 `dock_native`（pid=/hwnd=）/`undock_native`；WmState.native_slots 注册表 + 槽位伪 Wid 进 apply_layout 同轮排布（min-size 扩张 C3/free 恒等）+ sync_native_geometry DPI 排水（CoordMapper 桌面原点×GetDpiForWindow+标题条客户区内缩）+ 槽位框 chrome（标题条 min/close）+ C2 独占全屏拒绝 + C4 拖走/B7 回收事件臂 + B8 退出批量恢复；feature 阶梯 native-dock（windows optional+target 双门控，默认档零开销）/test-native-dock + tools/native-fixture 夹具（JSON-lines start/bounds/close）+ 真第三方进程 E2E 六测试；Phase 2 剪贴板/3 OLE 拖放/4 真洞待续（真人清单顺延 KD-P473-2）。**386 路线 B 桌面协议 Stage 1+2 已落地（v1.1）**：`ui/desktop_protocol/`
（五通道消息 + 二进制编解码 + 双端状态机 + 命名管道/共享内存传输 + broker
入口裁决 + L2 detach-attach，re-exec 两进程集成验证，状态保持经 revision
连续性证明）；spawn-client 双态启动 + broker 孵化中转就绪，live-iced 渲染
消费面换接与 Stage 3 多 App 内存实测归 shell-track/后续。设计源：
[Design 23/24/25](../../../design/autoui/README.md)、[桌面协议 v1.1](../../../design/autoui/desktop-protocol-v1.md)。

**vue 轨（codegen 正确性 + 工程化）**：444 修五类 vue-tsc 缺陷（回调通道/emits 名册派生/变体断言）；
443 defineModel 降级收窄（bound_model_channels 预扫）；451 actions/menubar/toolbar/快捷键消费补进
codegen；457 ~60 个 shadcn 组件编译期内嵌 auto-man（冷启动离线化）；437 chart 族声明驱动发射
（SVG 直通三端同源，借 442 A4）。

**VM 轨（视觉 parity + 运行时对齐）**：450/451-image/452-login 逐项拉平 shadcn 语义（圆角 SDF 掩膜、
Text 盒模型、按钮 14px/500、input 透明背景）；455 双端 parity 跟踪器把标准下沉为引擎规范（focus
ring 2px、margin 盒模型），矩阵约 9 绿 / 8+ 待审计；458 theme/accent 一等配置（CLI/pac.at/env
三通道，双端默认 dark+indigo）；446 清偿实战上报的渲染薄弱点（A1 多 store 消歧编译期报错、
J1/J2 渲染器子树，批五转正中）。

**DSL 与内建 widget**：425 component fn 糖化退役双轨（widget 单轨）；426/436 setup{} 三相位语义
（vue 落地、解释器 L1、a2r 显式报错）；448 msg 去名 + 内联 lambda 简写；413–428 code_editor 全链
（自研 cosmic-text ViEditor、折叠逐 run 管线、多 tab、vue CodeMirror 契约对齐）；449 确立 VM 组件
写法边界（三缺口登记：回调 props 退化/快照子树不可见/片段条件不求值）。

**示例轨道**：examples/ui 024-charts（437/445）、025-dashboard（438）已交付；026-database（439）、
027-file-manager（440）草案可领取；028-launcher 归 464。

## 关键入口

- `dialect/ui.rs:UiDialect` · `aura/extract.rs` · `aura/schema_loader.rs`（契约源自 `schema/aura.at`）
- `ui_gen/vue.rs:VueGenerator` · `ui_gen/api.rs` · `ui_gen/widget/registry.rs`（widget/chart 契约表）
- `ui/widget_registry.rs` · `ui/render_support.rs` · `ui/event_router.rs` · `ui/aura_view_builder.rs`
- 桌面线：`ui/session.rs`（DesktopSession/AppSession + WmState/WmCommand/DM::Wm）·
  `ui/iced/virtual_window.rs`（VirtualWindow）· `ui/iced/renderer.rs`（view_desktop_fn/run_dynamic_iced_multi）·
  `ui/desktop_protocol/`（路线 B 桌面协议 v1.1：五通道/传输/shm/broker/状态机）
- 样式与主题：`ui/style/`（class/theme/iced_adapter）· `ui/action_config.rs`（actions 配置层，
  热重载/OS keymap/表达式条件）
- 内建编辑器：`ui/code_editor/` · `ui/autodown_editor/` · `ui/handler_codegen.rs` · `ui/hot_reload.rs`
- `ui/mcp_server.rs`（AutoUI MCP 调试服务）· `a2ui/schema.rs:A2UIMessage`

## 使用示例

```auto
widget Counter {
    setup { greet = fn() { print("hi") } }   // 每实例前导（426）
    msg { Inc, Dec }                          // 448 起可去名
    model { count int = 0 }
    view { col { button + { onclick: .Inc } h2 > Count: ${.count} } }
    on { .Inc => { .count += 1 } }
}
```

## 已知坑

- VM 组件三缺口（449 实测）：回调 props 退化（组件一律无 props 读 store 规避）、快照组件子树不可见、
  片段参数化条件不求值——修好前 041 组件化受限。
- 446 批五（U2–U6）在 worktree 待 merge；463 合入后需实测内存给 386 提供数据。
- 455 矩阵多数示例（008–025）仍 Pending Audit；示例矩阵编号与目录有漂移（todo 在计划内）。
- 桌面线由并行会话活跃推进：WM 无独立 wm.rs 文件（WmState/WmCommand 实体在 session.rs，Plan 471 实测
  校正）；virtual_window 的 schema/aura.at 契约登记（462-I4）master 暂未见，随桌面线收尾合入。
- 465 未落地前，vue codegen 的 modal `position:fixed`/teleport-to-body 假设与虚拟桌面容器冲突（改造点已定位 vue.rs:6310/3599/4057）。
- a2vue 工程庞大（vue.rs 万行级），缺陷修复走 444 式"五类分簇"模式；013/015/011 构建失败系 master 预存（R006/R007）。
- router 双语法并存（Plan 105/106，见 docs/router.md）。

## 蒸馏来源

- 本模块 spec 于 2026-08-28 由 Plan 471 刷新：蒸馏 437–465 活跃计划 + 4xx 归档计划 + 365–428 早期 UI 计划。
- 设计层：[Design 20（分离架构）](../../../design/20-autoui-separation-architecture.md)、
  [Design 16（App 生成战略）](../../../design/16-app-generation-and-ai-authoring.md)、
  [design/autoui/](../../../design/autoui/README.md)（虚拟桌面三部曲）。
- 过程记录：`docs/plans/plans.md 索引表` + `docs/plans/KNOWN-DEBT-AND-RISKS.md`（445/449/414/422/444 条目）。
