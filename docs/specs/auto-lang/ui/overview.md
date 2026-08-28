# ui（AURA / UI 引擎 / 桌面运行时）

> **Status**: active（主战场：vue 轨 codegen 成熟化 + VM 轨视觉 parity + 虚拟桌面线推进中）
> 最近刷新：2026-08-28（Plan 471，蒸馏 437–465 桌面线计划）

## 职责

Auto 的 UI 子系统，围绕 **AURA**（UI-IR）组织，2026-08 起扩展为**桌面运行时**：

- **前端解析**：UI 方言（`widget`/`msg`/`model`/`view`/`on`/`setup`/`actions`，dialect 机制）。
- **AURA 提取与校验**：WidgetDecl → 视图树/状态/事件三元 IR，按 `schema/aura.at`（唯一契约源，plan-435）校验。
- **代码生成**（`ui_gen/`）：a2vue（主力）、a2rust、a2jet/a2ark、ts adapter、widget 契约表、block 层。
- **VM 运行时渲染**（`ui/`）：vnode/事件路由/VM 桥接、iced/gpui/headless 后端、code_editor 与
  autodown_editor 内建 widget、热重载、MCP 调试服务。
- **桌面运行时**（2026-08 新增）：`session.rs`（DesktopSession/AppSession 双层会话）、`wm.rs`
  （WmState/VirtualWindow）、排布纯函数与 DesktopBus v0——单 OS 窗口内多 App 虚拟桌面。
- **a2ui 协议** 与 **`#[api]` 前后端契约**（`src/api/`）。

## 现状（2026-08-28）

**桌面线（452→459→462→463 落地，464/465 已设计未开工，386 暂缓）**：
452 翻转"Windows 非 compositor"裁定并验证 IME/焦点分区可行 → 459 iced daemon 多 OS 窗口 +
会话化（453 的 DesktopSession/AppSession 拆分）→ **462 路线 A 地基**：VirtualWindow widget
（Stack/clip/mouse_area 组合）+ WmState/Wid + DM::Wm 消息 + 桌面级键盘路由 → 463 桌面 shell：
全屏 borderless 宿主 + 任务栏 overlay + free/grid/master-stack 排布纯函数 + pac.at 应用注册表 +
DesktopBus v0。464（launcher 为普通 App，Ctrl+Space 召唤 + `desktop.launch` 真启动）与 465
（vue 页面级虚拟桌面，每窗 createApp 隔离）已立项设计。386 RenderQueue（路线 B 分离渲染）复活
条件 2/3 就绪、明确暂缓。设计源：[Design 23/24/25](../../../design/autoui/README.md)。

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
- 桌面线：`ui/session.rs`（双层会话）· `ui/wm.rs`（WmState/VirtualWindow）·
  `ui/iced/renderer.rs`（view_desktop_fn/run_dynamic_iced_multi）· `ui/iced/virtual_window.rs`
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
- 465 未落地前，vue codegen 的 modal `position:fixed`/teleport-to-body 假设与虚拟桌面容器冲突（改造点已定位 vue.rs:6310/3599/4057）。
- a2vue 工程庞大（vue.rs 万行级），缺陷修复走 444 式"五类分簇"模式；013/015/011 构建失败系 master 预存（R006/R007）。
- router 双语法并存（Plan 105/106，见 docs/router.md）。

## 蒸馏来源

- 本模块 spec 于 2026-08-28 由 Plan 471 刷新：蒸馏 437–465 活跃计划 + 4xx 归档计划 + 365–428 早期 UI 计划。
- 设计层：[Design 20（分离架构）](../../../design/20-autoui-separation-architecture.md)、
  [Design 16（App 生成战略）](../../../design/16-app-generation-and-ai-authoring.md)、
  [design/autoui/](../../../design/autoui/README.md)（虚拟桌面三部曲）。
- 过程记录：`docs/plans/plans.md 索引表` + `docs/plans/KNOWN-DEBT-AND-RISKS.md`（445/449/414/422/444 条目）。
