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
| 437 | 024-charts-official-library | ✅（archived） | archive/ | chart 族 schema 契约落库+spec 驱动发射（ADR-18）；四类图官方 Auto 组件化；VM 子组件 Init 渲染期补发（ADR-19）；v1 SVG 直通 |
| 439 | 026-database | 📋 草案 | plans/ | SQLite 客户端；Tree widget 交付方；FFI 未就绪降级内存引擎 |
| 440 | 027-file-manager | 📋 草案 | plans/ | 双栏 Finder；422 contextmenu 首战；fs 走后端 use.rust |
| 441 | 028-launcher | 📋 被 464 吸收 | plans/ | 升级真注册表+真启动；焦点原语移交 462 |
| 442 | cross-platform-closure | 🟡（C3 观察期） | plans/ | musk 后端切 AutoVM；A4 SVG 直通为 437 渲染基座 |
| 446 | vm-backend-os-config-field-report | ✅（archived） | archive/ | 实战上报 VM 渲染薄弱点清偿；A1 多 store 消歧编译期报错；批五收口+下游结算完成（2026-08-29，reports/446-downstream-settlement.md） |
| 448 | autoui-syntax-improvements | 🟡（A/B 已实施） | plans/ | msg 去名 + 内联 lambda 简写；铸名提前到 parser（ADR-10） |
| 455 | auto-ui-parity | 🟡（矩阵 ~9 绿/8+ 待审计） | plans/ | 全示例双端 parity 审计；标准下沉引擎规范（ADR-17） |
| 458 | auto-ui-theme-system | 🟡（施工修订回填） | plans/ | theme/accent 一等配置三通道；双端默认 dark+indigo |
| 462 | virtual-window-wm | ✅ | plans/ | 路线 A 地基：VirtualWindow+WmState+DM::Wm+桌面键盘路由（ADR-14） |
| 463 | desktop-shell-auto-arrange | ✅（execution_done） | plans/ | 全屏桌面+任务栏+排布纯函数+DesktopBus v0+pac.at 注册表（ADR-15） |
| 464 | launcher-app | ✅ | archive/ | launcher 为普通 App 经 overlay 召唤；desktop.launch 真启动（真注册表模糊排序 + windowless 拆借垫片） |
| 465 | vue-virtual-desktop | ✅ | archive/ | vue 页=虚拟桌面，每窗 createApp 隔离（ADR-16 设计） |
| 472 | shell-track-m1-projection-dock | ✅（reviewed→archived） | archive/ | AutoShell 地基：投影协议 v1（schema/projection-protocol-v1.md）+ DesktopBus v1 对账定案（候选 B + 8 动词词表）+ workspace 分区驱动 + dock 升级（图标/pinned/配置链） |
| 386 | autoui-renderqueue | ⏸ 暂缓（复活条件 2/3 就绪） | plans/ | 路线 B 分离渲染（100MB→1-5MB/app），蓝图 Design 25 §7 |
| 476 | vm-slot-substitution | ✅（reviewed→archived） | archive/ | VM 轨 slot 替换：SlotFills 父作用域捕获 + outlet 臂 + 五容器×双胎兄弟拼接；清偿 musk KD-048 UPSTREAM① |
| 478 | shell-track-m2-switcher-pager | ✅（reviewed→archived） | archive/ | shell-track M2：switcher overlay（Ctrl+Tab 召唤 MRU 面板，第二枚 overlay 槽）+ dock 升格 pager（1 基标签/高亮/增删分区）+ send_to 跨区发送 + 投影协议 v1.1（__wm_mru/label/三动词，vue 对拍基线） |
| 479 | shell-track-m3-notification-center | ✅（reviewed→archived） | archive/ | shell-track M3：通知中心（S6）——463 瞬时 toast 升格「浮现+历史聚合」双面（dock 铃铛+未读 badge + 第三枚 overlay 槽 notification_center.at 右下锚定面板：逐条 ×/全部清除/Esc）+ notify/notes_toggle/notes_clear/notes_dismiss 动词（词表 v1.2）+ storage 定长槽 shell.notes.0..9 持久化（boot 恢复）+ 投影协议 v1.2（__wm_notes/__wm_notes_unread/指纹 notes 段，vue 对拍基线） |
| 473 | vm-native-window-dock | ✅（reviewed→archived） | archive/ | native dock 假洞 Phase 1：NativeSlot 原生窗口收编进 vm 桌面 WM（伪 Wid 同轮排布+min-size 扩张 C3+free 恒等）+ Win32 适配层（发现/几何写读回 DWM ext-frame/样式剥离还原/corner/sink_desktop_below z 序/WinEventHook 五事件）+ dock_native/undock_native 动词 + sync_native_geometry DPI 排水 + 槽位框 chrome + C2 独占全屏拒绝 + B8 退出批量恢复 + tools/native-fixture 夹具（JSON-lines）+ 真第三方进程 E2E 六测试（B1/B2/B3/C3/C4/C5/B7）；真人清单顺延 Phase 1.5（KD） |
| 483 | vm-text-input-double-focus | ✅（reviewed→archived） | archive/ | VM 双 input 双焦点/键盘双投递修复（上游 musk 011）：根因=全窗共享 prompt_input Id×未捕获 Tab 触发链（text_input 无 Tab 臂→__focus_prompt→focus 同 Id 全置焦）——iced_test 机制级复现;修复=derive_input_id 每框唯一稳定 Id（主键 widget+event）+devtools.input_ids 渲染期 DFS 登记+五聚焦点改址（Tab/refocus/初始/__focus_input/launcher）;顺修 autoui_type 归因对位双修（find_view_by_path 同构委托 extract_children_ref+mcp_sync_vtree 同源快照）;042 复现 example+p483 六测+D4 四测;tf 3260 全绿;债务 P483-1..5（真人键盘复验顺延 P483-3） |
| 480 | desktop-protocol-stage3 | ✅（reviewed→archived） | archive/ | 路线 B 桌面协议 Stage 3 收官（386 §0 承接）：client_runtime.rs（AppProjector 投影器 v1 text/button+线性堆叠+命中区 + ClientPump 协议泵 step/run 双形态 + ReconnectPolicy 弹性重连 + L3 快照编解码）+ auto 双模入口（--autodesk-client/--autodesk-incubate/--app386，无标记零改动）+ enable_broker 孵化通道（serve 线程+ServiceTick 落地）+ BrokerClient 驻留多 App 宿主 + N=3/5 压测（全 Active/逐 App 点击帧递增/30s 存活）+ 内存边际增量（Private 4.81MiB/App 临界达标·WS 23.17MiB/App 未达标——度量+判定形态，报告 480-memory-baseline.md）+ L1 换窗往返（detach_surface_to_os_window/attach_surface_back）+ L3 v2a 快照迁移（StateSnapshot tag 11 注入恢复 count/revision 连续）+ 修复 recv_wait 丢消息/shm 段名跨进程撞名两真缺陷；协议文档 v1.2；tf 3255 全绿；债 P480-R1..2 |
| 482 | autoui-nav-item | ✅（reviewed→archived） | archive/ | nav 组件族：nav-item/nav-group/nav(search:) 双端契约组件（nav_contract.rs 单源↔脚手架镜像,测试锁）+VM 路由历史栈/router.back+**lucide_svg 文档包装（修复全部 VM lucide 图标空渲染既有缺陷）**+三 app 替换（015-notes/018/gallery+外部 musk-052/os-config-012）;债务 P482-1..4 |
| 485 | vm-native-clipboard | ✅（reviewed→archived） | archive/ | 原生剪贴板 Phase 2（473 路线）：文件/图片两族互通——ui/clipboard_native.rs（纯 codec DROPFILES/DIBV5↔RGBA 全平台可测 + Win32 双门控 CF_HDROP/CF_DIBV5+PNG 双挂 64MP 防爆）+ 四 natives auto.clipboard.files_get/files_set/image_get/image_set（2934-2937，降级臂空表/false/null 零平台分支）+ GlobalClipboardTestLock 跨进程测试互斥（nextest 多进程互清根治）+ 043-clipboard-bridge 示例（三卡实机 T4 五项 PASS：Explorer Ctrl+C/V 往返/PrtSc→缩略/画图粘贴出图）+ tf 3275 全绿；债务 P485-1 剪贴板外部竞写偶发/P481-6 实机末步需人工（winit raw-input 限制） |
| 481 | autoui-text-selection-copy | ✅（reviewed→archived） | archive/ | 展示型文字选择/复制：text/label `selectable`(opt-in,缺省零变化)全链登记(schema/aura.at→View/VNode→renderer 分流) + 自研 SelectableText widget(绘制同参复用 iced text,buffer() 命中;拖选/双击词选(UAX#29 CJK 连字)/Ctrl+C/Esc 手势集,选区纯逻辑 selection.rs 零依赖单测) + vue 显式化user-select:text + a2vue 金样 011 + 001/004 示例点亮 + evidence 10 图;冒烟抓出 CursorMoved 旧坐标真 bug;债务 P481 D1-D4(arboard 兜底未启用/OS 剪贴板往返测试/font-mono 不可选/实机键路末步环境阻断) |
| 486 | native-dock-trigger-surface | ✅（reviewed→archived） | archive/ | native dock Phase 1.5 触发面（473 承接）：DragWatch 拖入手势（纯逻辑状态机+MOVESIZESTART 钩子+NativeDragOver 高亮 overlay）+投影 v1.3（native 条目 N<slot>/指纹/focus_native·close_native 动词）+shell.at native 任务栏分支+T4 E2E（drag_sim SendInput 真拖四要素+SC_MOVE 注入退路）+T5 实机冒烟 B1/B5/B8 留痕+P473 债务清偿回写；tf 3282 全绿；债务 P486-1 事件泵吞吐 |
| 487 | shell-track-m4-settings | ✅（reviewed→archived） | archive/ | shell-track M4 系统设置面板（S7，Design 25 §6 第四站）：settings.at 第四 overlay 槽（Dock/通知/关于三分区：位置/开关热生效单选 + pinned 行内增删 storage.set 直写 + 通知持久化开关 + 版本常量）+ 三动词 open_settings/set_dock_position/set_dock_enabled 与执行臂 apply_dock_edges_now（键写回→dock_edges 键重推导〔I9〕→relayout+槽位排水+投影热同步）+ dock 配置链升格读写闭环 + shell.notes.enabled 单点门控（push_notification false 短路/缺席兼容）+ 协议 v1.4（486 先合占 v1.3，487 按协调叠 v1.4）+ 齿轮入口双分支 + settings_* 七测无头全绿 + T4 实机（齿轮渲染/重启预写键三断言铁证；交互项注入受阻转 headless——P487-1 债）；tf 3283 全绿；债务 P487-1..3 |
| 489 | fix-ui-iced-suite-reds | ✅（reviewed→archived） | archive/ | ui-iced 特性档 4 既有红清零（P487-2 债收敛）：i18n 双测=corpus zh.json 被 .gitignore *.json 吞落测未入库（入库+!test/**/i18n/*.json 否定）/broker adjudicate 间歇红=生产固定管道环境干扰（adjudicate_on 参数化缝+pid 管道 hermetic，A/B 实证旧红新绿）/code_editor=print bool 现语义 true/false 断言对齐（逐行精确比对）；4074×2 连绿+tf 3283；merge 补：desktop_dock_edges/desktop_shell_at_builds 两测 storage 隔离（实机用设置面板后必红）→4081 全绿 |
| 491 | vm-tab-focus-traversal | ✅（reviewed→archived） | archive/ | VM 轨 Tab/Shift+Tab input 焦点环遍历（483 登记表扩展，上游 musk 011 §三点五）：未捕获 Tab 按 shift 分派 __focus_next/prev_input（Named 臂无修饰前缀须分流）+ FindFocusedInput operate 探针读实际持焦（含点击直聚,恒出 Some(None) 防断链）+ focus_traverse 登记表 DFS 回环求址（不在表/无聚焦→首项）+ update 臂 operate().then(focus) 两段链;登记表空回落 057 prompt 链（ash-gui/028 零回归）;p491 七测先红后绿+tf 3283+ui-iced 零新增红;真键盘顺延 P483-3 真人清单（含 musk Tab 流条目） |
| 490 | desktop-hotkey-cleanup | ✅（reviewed→archived） | archive/ | 桌面热键清理+VM 布局件点击 parity（用户 2026-08-30 双缺陷吸收）：HotkeyTable 数据级可配置（session.rs 纯逻辑+desktop_hotkey_message 订阅内核化+shell.keys.* boot 读入坏值静默回退;G1 Alt+Tab 退役〔storage 逃生舱〕/G2 分区迁 Ctrl+Alt+[ ]〔Intel 核显冲突族〕/launcher Ctrl+Space+Ctrl+Alt+Space 双收显性化〔IME 机,入口文案标注〕;488 Ctrl+V 臂协调收编 Paste 入表）+G4 row/col/div onclick 三层修复（View 三节点+字段穿透+aura 六分发点提取+wrap_layout_onclick mouse_area;Vue 零改动双端闭合;028 候选行鼠标点不中根因）;p490 五测四红转绿+hotkey 七测+tf 3283×2;真键盘/真鼠标顺延 P483-3 |
| 493 | mention-native-textarea | ✅（reviewed→archived） | archive/ | textarea mentions 能力声明（抽象度上提,叠加技法下沉实现层）：`mentions:`/`mention_class:` props+**受限 state-rooted 名单解析**（bindings→read_state_as_vec 深拷贝,computed/调用链→None 降级——html 降级 UAF 同构红线,护栏测锁）+builder 期 mention_segments 纯函数（@\w+ 小写匹配,无 display 替换,覆盖==value）+VM SpanKind::Mention（blue-600,无 per-span 背景登记）+Vue codegen backdrop/透明 **兄弟对**发射（类串自 textarea 推导:transparent 位换/删输入面 token/保底追色/增背板结构 token）+__autoMentionHtml helper 随 script 一次发射+mentions/mention_class/height 纳 attr 过滤（height=VM-only 几何契约）;musk mention_input 迁移（删 backdrop/text-transparent/计算属性,名单 var handler 刷新）+composer height:72（单行观感根因=text_editor 缺省 Fixed(30)）;plan493_ 11 测+tf 3286 全绿+ui_gen 708+017 冒烟 A-E+playwright 9/9;活体:器具根/子双形态 blue-600 精确像素+musk 段日志;债务：.send 链垃圾引用崩溃（双仓 master 先在 A/B 实证）/musk 现场像素终验 pending（⑥）/iced textarea 内容键碰撞新候选;P493-1..6 台账 |
| 492 | engine-view-text-fixes | ✅（reviewed→archived） | archive/ | 引擎正确性专项——三族静默失效定案+chart 绕开摘除闭环：族A2 f-string `${}`+字面量`[]` 判 **误归因**（五层验证不可复现+金丝雀负对照证伪,用户裁定;484 绕开形态 `{slot}` 实为纯字面量,布局靠 flex-1 意外生效）/M2 parser primary-shorthand 识别 `[` 后缀（`text t["label"]` 整链挂 prop=Expr::Index,镜像 Dot/LParen peek）/M3 vue 文本内容 Index 字符串键保引号（复用属性路径 bound_value）+不支持形式 R046 告警替换静默 dump（VM 轨缺口入账 492-R1）/M4 族C 定案：无 codegen 分叉,真机制=裸 prop 名 RHS 位解析错→parse_package_widgets **静默整文件丢弃**（点前缀直接形态全链可用,带参 msg 双轨平反）/M5 包编译失败三层显式诊断契约（装载 parse_warnings 逐条 log::warn/合成 record_synth_failures+take_synth_failures 双路径/链接 fatal 具名）/M6 四组件三副本绕开全退役（Init 内 `.type`/`.curve` prop 比较直选域+`msg { Init, Hover(int) }`+dollar 插值;grep 锚清零,golden 再生成）;门禁 tf 3293/ui-iced 4116/chart 专项+golden/双端目检（Vue 六卡+tooltip 截图/VM 树证据）全绿;DEBT 行48·49·52 闭环+492-R1 新入账;P492-1..6 台账 |
| 488 | vm-native-dragdrop | ✅（reviewed→archived） | archive/ | 原生互操作 Phase 3——OLE 拖放双向+虚拟文件落地（473 路线,前置 485/486）：ui/native_dnd.rs（DndDataObject IDataObject 三格式族 CF_UNICODETEXT/CF_HDROP/FileGroupDescriptorW+FileContents〔HGLOBAL,Explorer 实证接受〕+IEnumFORMATETC / DesktopDropTarget **STA 线程持对象**〔marshal 代理注册+自持泵〕+**WM_NULL 唤醒 ticker**〔DragEnter/Over 上膛——Drop 的跨线程 SendMessage 型投递仅主线程进取消息态才送达,松手无输入即滞留,三轮实机定案〕/ DndDropSource seen-button 语义〔程序化起拖首拍 keys=0 不误判释放〕）+ start_drag **调用线程内联阻塞**（OLE 循环须在输入线程,独立 STA 实测 QCD 零调用永卡——"受理即返"架构修正）+ auto.dnd.start（2938,裸名 intrinsic 两处,降级臂 false 零平台分支）+ 宿主挂载 ensure_host_drop_target（Revoke winit 0.30 自带目标→Register 我方〔spike 实证单 HWND 单目标〕,HWND 身份键控+ServiceTick 400ms 自愈）+ 拖入路由（drop_hit_app_at_local z 序分区命中）+ 事件面（aura.at virtual_window events 三枚 on_native_drop/paste/dnd_finished;**VmBridge call_handler_with_record** VM 堆记录编码〔列表=heap ListData/嵌套拍平/空串哨兵契约——record Str 判 nil 与 .str()-in-if 两 VM 缺陷绕开〕）+ 桌面级 Ctrl+V 热键臂（418 文本→485 文件/图片,490 收编入键位表）+ 夹具 --offer 拖源/drop 日志 + E2E T3/T4（SendInput 真拖,drag_sim OLE 原语,严格 idle-wait 断言）+ 044-dnd-bridge 示例（T6 载具,三轮实机+代理端到端截图实证即时显示）+ AUTO_DND_TRACE 五跳诊断;一阶折叠门禁 tf 3283/tv 三红皆存量基线;P485-2 分诊定案（双后端 .line 去重真分叉）;债务 P488-D1..D5 |
| 494 | vm-native-true-hole | ✅（reviewed→archived） | archive/ | 原生真洞 Phase 4（473 路线收官,双 spike 证伪后机制替换）：透明 swapchain 证伪（wgpu HWND surface 仅 Opaque/DComp 不上屏〔疑 ToDesk 环境〕/色键分层破坏呈现）+HTTRANSPARENT 跨进程证伪（MSDN 同线程文义）→ **SetWindowRgn 洞排除**（raise_desktop_above 单步 z 翻转+apply_hole_regions RGN_DIFF 逐洞扣除——洞区窗口不存在=视觉+输入穿透 OS 区域语义）+shell.native.hole 模式位（默认 off）+失败自动回退假洞（refresh_hole_regions_at 可测核心+stale hwnd 实测）+T1 纯逻辑（window_local_holes/z 模型）+真实测试（z 不变量首可见邻居〔IME 楔位教训〕/Region 穿透/复位）+T3 E2E 铁证（洞心 SendInput 跨进程精确穿透 ±6/洞外零泄漏;夹具 click 日志+win32::test_support）;tf 3303/ui 776/E2E 9+2 全绿;G4 覆盖层洞边裁剪+T5 物理机复验=已批准债务 P494-1/2/3;P494-1..6 台账 |
| 496 | shell-track-m5-desktop-surface | ✅（reviewed→archived） | archive/ | 桌面本体 S9（shell-track 第五站,463 z 槽消费）：第五面 desktop.at 常驻 boot 装载挂桌面层 z 槽（Stack 先于虚拟窗=壁纸之上/窗口之下）+壁纸双径（#hex 根 bg 插值实铺/图片宿主图层,boot 解析回退 #090e1a）+图标网格（pinned∪custom 去重 hidden 排除,storage 三键 boot 生效）+ondblclick VM 全链（MouseArea.on_double_click+convert_view_messages 补显式臂修 VM 动态路径 Empty 兜底缺口）+右键三项（activate 两臂/hidden 直写/open_settings）+空白点击 463 语义+settings 外观分区壁纸写手+投影协议 v1.4 内字段扩展（__desktop_* 族,零新动词）+a2vue 真资产金样（vue 插值 class 缺口修复五点落码）;tf 3304/ui 777/T1-T3+金样绿;T4 实机=预写键单帧三断言+交互 OS 注入受阻转 headless 对表（P496-1/2）;P496-1..6 台账 |
| 497 | shell-track-s3-status-thumbnails | ✅（reviewed→archived） | archive/ | Status 栏收官（S3——桌面特性线最后整体未动面）：每窗口真缩略三件套——快照核心 snapshot.rs（T1 定案**裁剪式整窗快照**：screenshot×scale_factor 裁剪+box 降采样≤256+TTL 2s 进程缓存+抓取队列 500ms 冷却+失效三点接线）+window_thumbnail 七表登记（渲染臂 Handle::from_rgba 直绘/miss→lucide fallback+request_capture；@/wm/WindowThumbnail vue 占位）+消费三面（switcher 行/dock hover popover/pager 分区网格+宿主 SnapshotShot 抓取编排闭环 miss→一帧升级）+dock 时钟（__wm_clock ServiceTick 分钟注入，唯一非门控字段）+托盘组右置两态；缺陷修复 untracked popover 臂缺失/invalidate 冷却/Shell dirty；T5 实机六项 PASS；tf 3316；债务 P497-1/2；P497-1..6 台账 |
| 501 | vm-desktop-osconfig-integration | ✅（reviewed→archived） | archive/ | vm 桌面 os-config 统一 settings center 接通（Design 25 S7 全量,487 跨仓深桥兑现）：daemon 生命周期管理器 osconfig_daemon.rs（DaemonStatus 三态+std TCP 裸 HTTP 检活〔reqwest blocking tokio panic 弃〕+detached spawn 不杀共享语义+就绪轮询 ≤5s+badge_projection；17701/spawn 期 AUTOOS_BACK_PORT 覆盖缺省 17901；发现序 storage shell.osconfig.daemon>相邻仓 target〔二进制实名现场核验修正〕>PATH 留扩展位 P501-1）+注册表多扫描根（aggregate_scan 主根优先 id 去重+shell.apps.extra_dirs 语法+相邻仓探测缺省 os-config id,scan_siblings 可关）+launch 臂依赖面（pac daemon: autoos〔跨仓 0e81196〕→ensure_ready→AUTOOS_DAEMON 注入;Offline 不阻断——daemon_view 自带 UX;back: { project } cdylib 桩桥 Plan 061 链桌面补齐,back_keepalive）+settings 系统分区（入口卡+三态徽标+offline 点击即重试）;T3 集成档六段面包屑（配置根 USERPROFILE/HOME 重定向零污染）;合并后 master 3323 全量+189 scoped+T3 绿;债务 P501-1/2;P501-1..6 台账 |
| 505 | desktop-debt-batch-1 | ✅（reviewed→archived） | archive/ | 桌面 DEBT 批处理一期（四族,GOAL-009）：A 事件泵 drain-while-empty 成批上行+START/END 稳定分区前置（drain_slot_events 纯函数+快甩同批即判单测;NativeSlotEvents 批形态,Disconnected 自愈保持）+B 五小件（shell.at 双分支→flex-col-reverse 单份+__dock_border 投影边线〔实拍揭 taskbar if 条件样式 live 装配丢失→投影拼接规避+回归锚〕;投影协议 v1.5 pager 旗标/more 标签派生面指纹零扩展;a2vue 注册件 props 透传双金样;daemon PATH 三级;shutdown_broker 五退出点）+C 实机验收通道（autoui_desktop MCP 注入工具+AUTOUI_ACCEPTANCE 门+DesktopInject 五面队列走真实按钮同臂+ADR 规程+acceptance_channel.py 四场景;P487-1/P496-1/P501-2 三债七帧实机照补拍——含 os-config 外部仓 App 实拉起）+D P488-D4 发起方锚定（拖出会话代号+dispatch 环比对）+壁纸热切换定案天然支持;tf 3329+ui-iced 4227 全绿;债务 P505-1/2;P505-1..6 台账 |
| 504 | calculator-fit-window-osconfig-stdlib | ✅（reviewed→archived） | archive/ | 示例桌面化三件套（011-calculator 样板,GOAL-009/010）：pac `window:"fit"` 首帧内容 shrink 测量自适应窗口（独立 VM 窗 1293x836→397x428 实机 + 桌面虚拟窗 LaunchSpec.fit 双路径;register_window 同键覆盖抹 fit_pending 根因修复+回归锁）+ title/settings 上移 os-config（per-app `~/.config/autoos/apps/calculator/config.at` 目录形态循 musk 先例;launch 直读文件注入 theme/accent 不经 daemon;modules.d/auto-calculator.at 热注册实测）+ stdlib 静态分发（Math.pow=f64::powf / Str.is_digit 单字符 ASCII 谓词 VM Rust shim + Vue ts_adapter `/^[0-9]$/.test` 映射;011 删本地 is_digit/pow 手写;056/057 VM 文件测试）;tf 3326 全绿+双端 e2e 绿;债务 P504-1..4（18_ffi 存量格式化/fit 一次性测量 Scientific 裁剪/desktop 实况 e2e 输入通道/boot 直开无 pac 语境）;P504-1..6 台账 |
| 503 | desktop-visual-refresh | ✅（reviewed→archived） | archive/ | 桌面视觉体系刷新——stella-os 风格移植（GOAL-007 双端同源）：coral accent 校准玫瑰粉 #c4706a（三处同源副本）+ dock 图标格 48px rounded-xl/激活竖条+bg-primary/15/4px 运行圆点（shell.at 双分支+Taskbar.vue,栏高 56px）+ 弹层 glass 三件套（bg-card/80+细边+柔影——右键菜单/通知中心/切换器全去 gray 硬编码）+ 壁纸 scrim（renderer.rs 图片壁纸上叠 background 语义色 10%/35%,无 blur 降格 parity）+ launcher 重写（语义 token/分类胶囊 PickCat 三分支过滤/品牌色图标底块 bg-[#hex21] 8 位 hex+宿主 apps_colors 注入 launcher_brand_color 哈希兜底）+ 窗口 chrome（TITLEBAR 36/radius 16/柔影 light12% dark40% focused 加深/1px accent/60/macOS 三色圆点——yellow/green 视觉位 P503-1;最大化几何判定去圆角）;引擎补齐（TDD）：style 串循环成员插值 ${member.field} VM+vue 双端 + bg-[#hex]/N 解析组合 + shell_packs_compile 五 pack 编译冒烟 + plan503_tests 6 项;golden 两基线更新（gallery=palette+kitchen-sink 存量补采样;a2vue desktop 金样同步）;实机双端目检（vm 全屏+vue desktop host,vite 3001）逐项 PASS;tf 3330/3330;tv/tt 红均 master 存量同集;债务 P503-1/2;P503-1..6 台账 |
| 498 | chart-interaction-states | ✅（reviewed→archived） | archive/ | chart 交互状态机（484 静态渲染续篇,GOAL-007 双端同源）：M0 mouse-area on_click 引擎臂（View::MouseArea.on_click 五处贯通 iced on_press/vue @click+schema 双源登记〔补登 496 漏写 ondblclick〕）+四图族 emphasis 二态（line/area 图例行 HoverSeries 高亮 3px·downplay 0.25·转折点圆圈浮现；bar 竖带 hoverGroup 该组描边 1.5/其余 0.3；donut 图例/扇区 Hover 中角外移 12px+白描边 2px）+legend onclick Toggle 显隐（隐藏几何跳过+图例 opacity-40,隐藏优先悬停）；段记录槽位化（path/pts/a·l/d·e/segs0..3 弃 .segs）;哨兵 9 规避负数字面量 view 比较缺陷（P498-1）+图族专属字段名解耦 VM 单态同名字段串扰（P498-2）;vue CSS transition 淡入/VM 直切；plan498 六测+vue Playwright 实机（46→44→46 往返/悬停 sw3）+VM 截图;tf 3330;P498-1..6 台账 |
