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
| 358 | auto-lang-generator-defects-fix | 待评审 | plans/ | 生成器/编译器缺陷系统性清单与修复 |
| 360 | notes-ui-redesign-and-accent-theming | 未标注 | plans/ | 015-notes UI 现代化 + 主题色切换（P0-P5 问题清单） |
| 361 | short-term-generator-hardening | 未标注 | plans/ | 生成器加固：不变量检查 + 代码路径收敛 + 冒烟测试 |
| 362 | fast-feedback-and-watch | 未标注 | plans/ | `auto watch` + 分层重建 + 生成器缓存（Rung 5 反馈回路） |
| 363 | autoui-generation-skill | 未标注 | plans/ | AutoUI 生成 skill：安全生成 + 模式库 + 交互式向导 |
| 365 | autoui-pluggable-host-architecture | ⏳（index 口径） | plans/ | 一核心三 host（dev/libcosmic/AutoOS compositor）的可插拔宿主架构 |
| 366 | cross-platform-ui-test-dsl | 设计阶段，暂不实现 | plans/ | 跨平台 UI 测试契约；当前用 AutoDown 契约 + Playwright 执行 |
| 367 | codegen-quality-improvements | 未标注（逐项状态内联） | plans/ | 让 Auto 产物达到手写水平：分阶段质量改进 |
