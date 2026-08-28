# ui 相关 plan 索引

> 状态以 plan 文件自述为准；plan 无 Status 行的标"未标注"，必要时附 plan-indices/11 的口径。
> 重编号注意：327/336/337/338/342/351/355/359 曾发生重编号，原占用者已改为 317/318/320/322/330/346/347/348；本表一律用当前号。design 16/17 引用的"Plan 331/336/337/338"与当前文件内容一致，无需映射。

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|------|------|------|------|-----------|
| 096 | scenario-ui | ⏳（index 口径） | old/ | UI 场景架构：保留 View/Model/Msg 三元骨架 |
| 098 | aura-schema | ⏳（index 口径） | old/ | AURA schema：校验 + LSP 补全 + 诊断，落地为 schema/aura.at |
| 099 | shadcn-vue-migration | 🔧（index 口径） | old/ | Vue 生成器迁 shadcn-vue，组件覆盖分批推进 |
| 105 | auto-router | 被 106 取代 | old/ | 路由初版：`"/" => Component {}` 静态 import |
| 106 | router-use-syntax | 现行推荐 | old/ | `use module` 约定 + 懒加载 + 小写文件名（ADR-04） |
| 135 | ui-incremental-compilation | ✅（index 口径） | old/ | UI 增量编译复用 AIE 基建（UICache） |
| 138 | arkts-backend | ✅（index 口径） | old/ | ArkTS 后端全量落地，DevEco Studio 验证 |
| 140 | aura-widget-library | ⏳（index 口径） | old/ | 硬编码组件定义迁 .at widget 文件 + WidgetRegistry |
| 142 | aura-arkts-transpilation | ⏳（index 口径） | old/ | 54 个 AURA widget 到 ArkTS 的转译 |
| 143 | stdlib-widget-library | Approved（plan 自述） | old/ | ~45 组件从 component-gallery 迁入 stdlib/aura/widgets |
| 180 | a2rust-ui-generator | ⏳（index 口径） | old/ | RustGenerator 接入 auto gen（GPUI 路径） |
| 205 | dynamic-component-vm-ui | ✅ | old/ | VM 驱动动态 UI：VmBridge + AuraViewBuilder + iced |
| 217 | a2ui-composer-implementation | ✅ | old/ | 三栏 composer（palette/canvas/inspector），Vue 3 构建 |
| 227 | dynamic-ui-iced | ✅ | old/ | `run_file()` 自动检测 widget/app 起 iced 窗口 |
| 234 | a3ui-a2vue-replica | ✅ | archive/ | a2ui composer 的 a2vue 复刻，7 页全阶段完成 |
| 235 | a2vue-transpiler-gaps | ✅ | old/ | ts_adapter 修复 + storage/event/json/router 内建 |
| 274 | aura-stable-node-id | ✅ | old/ | VNodeId 稳定 ID 体系（ui/vnode.rs） |
| 287 | auto-to-vue-mapping-rules | ✅ | old/ | Auto→Vue 映射规则固化进 ui_gen/vue.rs（含 shadcn） |
| 288 | notes-fullstack-api | Phase 1 ✅ | plans/ | 015-notes Vue 前端对接 `#[api]` 后端，API 函数自动检测 |
| 299 | autoui-mcp-v2 | ✅（Phase 1-3） | archive/ | AutoUI MCP 调试服务 v2（ui/mcp_server.rs 前身） |
| 307 | autoui-devtools-inspector | 主体已合并 | archive/ | AutoUI devtools 检查器并入 master |
| 314 | autoui-mcp-styled-vtree | ✅ | archive/ | `autoui_vtree` 带样式快照 MCP 工具 |
| 320 | single-vm-widget-tree | 未标注 | plans/ | 消除子组件独立 VM：单一 VM widget 树，state/handler 贯通 |
| 323 | calendar-full-app | 未标注（Phase 2 前置已批准） | plans/ | 016-calendar 完整月历；暴露"VM widget handler 无计算能力"缺口 |
| 324 | autoui-widget-library-strategy | 待评估 | plans/ | npm 组件库战略：先修 a2vue 缺陷 + 生成库能力，再建库 |
| 327 | 015-notes-vm-render | 未标注（阻断点跟踪表） | plans/ | 015-notes 在 VM 渲染模式跑通的阻断点清单 |
| 329 | ipc-sse-channel-support | 设计完成，待实施 | plans/ | Tauri Channel 流式推送，M3/M6 的 SSE 底座 |
| 330 | agent-friendly-debug-tools | 未标注 | plans/ | Agent 可用的 AutoUI CLI 调试工具链（headless/JSON/VM 内部诊查） |
| 331 | autoui-vue-widgets-npm-library-design | 设计已确认，实施待执行 | plans/ | @auto-ui/widgets：a2vue 生成的 npm Vue 组件库设计 |
| 333 | vm-ui-compilesession-migration | Phase 1-2 ✅（文末记录） | plans/ | VM/Rust 模式统一走共享 CompileSession |
| 336 | vue-gallery-autoui-widgets-showcase | 设计待确认，实施未开始 | plans/ | vue-gallery 作为 @auto-ui/widgets 的 dogfood 展示页 |
| 337 | vue-gallery-widgets-sync-foundation | 设计待确认，实施未开始 | plans/ | gallery↔widgets 薄同步层；TODO-A=扩到 ~60 widget（Rung 3 天花板） |
| 338 | extend-015-notes-m1-benchmark | 设计待确认，实施未开始 | plans/ | M1 基准：015-notes 扩成中等 CRUD（后续由 354/357/360 接力） |
| 342 | block-tier-phase-a-package-foundation | 设计待确认，实施未开始（代码已先行） | plans/ | block 包格式 + BlockRegistry + blocks-gallery 骨架 |
| 343 | block-tier-phase-b-generator-and-cli | 设计待确认，实施未开始 | plans/ | `auto block add` 双模式 + 静态 acceptance check |
| 351 | shared-store-rung4 | 设计待确认，实施未开始 | plans/ | Rung 4：跨 widget/跨路由共享状态 store |
| 354 | 015-notes-real-app | 实施中 | plans/ | 015-notes 从 CRUD demo 到真实笔记 app（标签/搜索/三列/AutoDown 编辑器） |
| 356 | vue-generator-oom-recursion-fix | ✅ | old/ | 修复 parser OOM + 软关键字递归，015-notes sidebar 完整再生成 |
| 357 | 015-notes-pin-folder-tag-ux | 实施中 | plans/ | pin/目录/tag/dark mode/主题色的 UX 迭代 |
| 358 | auto-lang-generator-defects-fix | 待评审 | archive/ | 生成器/编译器缺陷系统性清单与修复 |
| 360 | notes-ui-redesign-and-accent-theming | ✅ | archive/ | 015-notes UI 现代化 + 主题色切换（P0-P5 问题清单） |
| 361 | short-term-generator-hardening | ✅ | archive/ | 生成器加固：不变量检查 + 代码路径收敛 + 冒烟测试 |
| 362 | fast-feedback-and-watch | 未标注 | archive/ | `auto watch` + 分层重建 + 生成器缓存（Rung 5 反馈回路） |
| 363 | autoui-generation-skill | ✅（autoui-skill crate） | archive/ | AutoUI 生成 skill：安全生成 + 模式库 + 交互式向导 |
| 365 | autoui-pluggable-host-architecture | ✅（被 Design23/462 部分修订） | archive/ | 一核心三 host（dev/libcosmic/AutoOS compositor）的可插拔宿主架构 |
| 366 | cross-platform-ui-test-dsl | 设计阶段，暂不实现 | archive/ | 跨平台 UI 测试契约；当前用 AutoDown 契约 + Playwright 执行 |
| 367 | codegen-quality-improvements | 未标注（逐项状态内联） | archive/ | 让 Auto 产物达到手写水平：分阶段质量改进 |

## 桌面线与 parity 时代（2026-08，Plan 471 补录）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|------|------|------|------|-----------|
| 374 | a2r-store-viewfn-parity | ✅ | archive/ | a2r 管线补 store composable + view fn fragment，rust 模式达 VM parity |
| 411 | vm-visual-alignment-vue | ✅ | archive/ | VM 视觉对齐 vue；确立"结构数据交叉验证视觉差异"方法论 |
| 413 | cross-platform-code-editor | ✅ | archive/ | 自研 code_editor 内建 widget（cosmic-text ViEditor，拒 GPL 复制）全链 |
| 414 | auto-edit-ux | ✅ | archive/ | tab/双行号/折叠 chevron/terminal 图标四轮 UX；确认 VM 无原生 Menu/Toolbar |
| 418 | action-realization-and-config | ✅ | archive/ | Action=声明绑定层/Event=执行层；natives×11+13 handler 落地 |
| 420 | auto-edit-tabs-workspace | 🟡（P4 拖拽延后） | plans/ | tabs 数组态动态化 + 文件开/存闭环；AUTO_OPEN/SAVE_PATH 旁路自动化 |
| 421 | vue-code-editor-contract | ✅ | archive/ | vue 端 CodeEditor 契约对齐（@codemirror 接线 + oncursor/oncontextmenu） |
| 422 | popover-primitive | ✅ | archive/ | iced overlay 真弹层原语（双锚/placement/on_dismiss），退役像素估位 hack |
| 423 | action-config-phase3 | ✅ | archive/ | 配置层热重载/分层 OS keymap/表达式条件引擎，矩阵 48/48 |
| 425 | component-fn-sugar-retirement | ✅ | archive/ | component fn 降为 AST 级糖，widget 单轨，view{} 包裹可选化 |
| 426 | setup-preamble-slot | ✅ | archive/ | setup{} 每实例前导槽，三相位 setup/.Init/.Destroy 语义定版 |
| 428 | code-folding-phase-b | ✅ | archive/ | 正文逐 run 绘制管线（折叠可跳区间），视图态不进 undo 栈 |
| 435 | autoui-component-schema-unification | ✅ | archive/ | 组件契约收敛 schema/aura.at 单源 + CI 拦截漂移（ADR-09） |
| 436 | setup-interpreter-a2r | ✅ | archive/ | setup 语义补解释器 L1；a2r 显式报错止血 |
| 438 | 025-dashboard | ✅ | archive/ | KPI+曲线+进程表 dashboard；单 .Tick 源分频刷新 |
| 443 | define-model-narrowing | ✅ | archive/ | 未绑定 model 通道降级 ref（bound_model_channels 预扫），修深 mutation 断裂 |
| 444 | vue-codegen-ash-shell-057 | ✅ | archive/ | vue codegen 五类缺陷修复（回调通道/emits 派生/变体断言），vue-tsc 清零 |
| 445 | 024-charts | ✅ | archive/ | 四类图+流式滑窗 demo；vm 19/19；.Tick 跨轨语义分歧登记 |
| 449 | auto-edit-componentization | ✅ | archive/ | 041 拆组件；实测登记 VM 组件三缺口（ADR-12） |
| 450 | autodown-panel-registry | ✅ | archive/ | 10 面板词汇登记 registry + iced 映射；vue 映射走 schema.meta |
| 450 | autoui-visual-parity-refinements | ✅ | archive/ | VM 补圆角/负 margin/Text 盒模型/按钮字号对齐 shadcn |
| 451 | actions-dsl | ✅ | archive/ | actions{} 并入 widget DSL，双端消费 + 解析期校验（ADR-11） |
| 451 | image-border-radius-clipping | ✅ | archive/ | VM 光栅图 SDF alpha 掩膜裁圆角，handle 按 radius+size 键控缓存 |
| 452 | autos-virtual-desktop-foundation | ✅ | archive/ | Design 23 转正；翻转 365 R2 裁定；IME/焦点 spike 无阻断 |
| 452 | 005-login-consistency | ✅ | archive/ | VM 补 a/行内标签/input 透明背景/password 掩码，四态对齐 |
| 453 | multi-app-session-runtime | ✅（T7b 由 459 收口） | archive/ | DesktopSession/AppSession 双层会话拆分（ADR-13 前半） |
| 453 | autoui-input-margin-footer-parity | ✅ | archive/ | Input/Textarea/Checkbox wrap_with_margin 对齐 |
| 454 | fstring-restoration-host-hardening | ✅ | archive/ | AA2R f-string 三缺口修复 + 宿主 2.4 收紧（部分证伪回退） |
| 457 | shadcn-components-bundled | ✅ | archive/ | ~60 shadcn 组件编译期内嵌 auto-man，冷启动离线化 |
| 459 | desktop-session-multi-window | ✅ | archive/ | iced::daemon 多 OS 窗口 + panic 隔离（ADR-13 后半） |
| 437 | 024-charts-official-library | ✅（待 review） | plans/ | chart 族契约正式化+声明驱动发射；"引擎给笔，Auto 持笔"，v1 SVG 直通 |
| 439 | 026-database | 📋 草案 | plans/ | SQLite 客户端；Tree widget 交付方；FFI 未就绪降级内存引擎 |
| 440 | 027-file-manager | 📋 草案 | plans/ | 双栏 Finder；422 contextmenu 首战；fs 走后端 use.rust |
| 441 | 028-launcher | 📋 被 464 吸收 | plans/ | 升级真注册表+真启动；焦点原语移交 462 |
| 442 | cross-platform-closure | 🟡（C3 观察期） | plans/ | musk 后端切 AutoVM；A4 SVG 直通为 437 渲染基座 |
| 446 | vm-backend-os-config-field-report | 🟡（批五转正中） | plans/ | 实战上报 VM 渲染薄弱点清偿；A1 多 store 消歧编译期报错 |
| 448 | autoui-syntax-improvements | 🟡（A/B 已实施） | plans/ | msg 去名 + 内联 lambda 简写；铸名提前到 parser（ADR-10） |
| 455 | auto-ui-parity | 🟡（矩阵 ~9 绿/8+ 待审计） | plans/ | 全示例双端 parity 审计；标准下沉引擎规范（ADR-17） |
| 458 | auto-ui-theme-system | 🟡（施工修订回填） | plans/ | theme/accent 一等配置三通道；双端默认 dark+indigo |
| 462 | virtual-window-wm | ✅ | plans/ | 路线 A 地基：VirtualWindow+WmState+DM::Wm+桌面键盘路由（ADR-14） |
| 463 | desktop-shell-auto-arrange | ✅（execution_done） | plans/ | 全屏桌面+任务栏+排布纯函数+DesktopBus v0+pac.at 注册表（ADR-15） |
| 464 | launcher-app | 📋 已立项 | plans/ | launcher 为普通 App 经 overlay 召唤；desktop.launch 真启动 |
| 465 | vue-virtual-desktop | 📋 已立项 | plans/ | vue 页=虚拟桌面，每窗 createApp 隔离（ADR-16 设计） |
| 386 | autoui-renderqueue | ⏸ 暂缓（复活条件 2/3 就绪） | plans/ | 路线 B 分离渲染（100MB→1-5MB/app），蓝图 Design 25 §7 |
