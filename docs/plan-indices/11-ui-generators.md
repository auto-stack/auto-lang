# 11 - UI Generators and AURA

## Overview
AutoLang's UI system centers on AURA (Auto UI Representation Abstract), a declarative widget DSL that transpiles to multiple native backends including Vue.js, Jetpack Compose, ArkTS (HarmonyOS), Rust (GPUI/ICED), and VSCode extensions. The generator stack evolved from a single Vue backend to a multi-platform pipeline with schema validation, design token support, and incremental UI compilation.

## Plan Summary

| Plan | Title | Status | Summary |
|------|-------|--------|---------|
| 094 | Hybrid FFI Bridge | ✅ | VMConvertible trait, #[rust_fn] macro, and all 43 stdlib FFI shims |
| 096 | Scenario UI Architecture | ⏳ | AURA architecture migration from DSL preprocessing to dedicated UI AST |
| 097 | TodoMVC Example Implementation | ✅ | Complete TodoMVC demo compiled to Vue.js and Rust/AutoUI backends |
| 098 | AURA Widget Schema Specification | ⏳ | Schema system for widget validation, LSP autocomplete, and error diagnostics |
| 099 | shadcn-vue Migration | 🔧 | Migrate Vue generator to shadcn-vue components; generator updated, full 43-element coverage in progress |
| 113 | a2jet (Auto to Jetpack Compose) | ✅ | Complete Jetpack Compose code generator across all 7 phases |
| 114 | Hybrid Routing (Convention + Config) | ⏳ | Hybrid routing with auto-discovered convention routes and config-based overrides |
| 133 | Jetpack Compose Generator Enhancement | 🔧 | Extend Jet generator to full AURA syntax; core components done, 40+ remaining |
| 134 | Jet Generator View Body | ✅ | Implement generate_view_body() with recursive node-to-Compose mapping |
| 135 | UI Incremental Compilation | ✅ | Incremental UI code generation reusing AIE infrastructure with UICache |
| 136 | Jet Backend Incremental Adoption | ✅ | Gradually extend Jet backend in unified-demo with component-level expansion |
| 138 | ArkTS (HarmonyOS) Backend | ✅ | Complete ArkTS backend with project scaffolding verified in DevEco Studio |
| 140 | AURA Widget Library | ⏳ | Replace hardcoded component definitions with .at widget files and WidgetRegistry |
| 142 | AURA ArkTS Transpilation | ⏳ | Transpile all 54 AURA widgets to ArkTS components for HarmonyOS |
| 143 | Stdlib Widget Library | ⏳ | Migrate ~45 components from component-gallery into stdlib/aura/widgets |
| 144 | 04-Tabs Project | 🔧 | Bottom tab navigation demo with 3 tabs translating to ArkTS Tabs component |
| 145 | Jet Gallery | ✅ | Standalone Android Compose reference app with 51 widget demos |
| 147 | unified-demo a2jet Alignment | 🔧 | Align unified-demo and a2jet with jet-gallery reference; basic components done |
| 174 | Conditional UI Backend Inclusion | ⏳ | Add ui-headless feature flag so default builds skip all UI dependencies |
| 175 | Migrate auto-ui into auto-lang | ⏳ | Move GPUI and ICED backend runners from standalone auto-ui into auto-lang workspace |
| 180 | a2rust-ui Generator | ⏳ | Wire RustGenerator into auto gen for GPUI-based Rust UI examples |
| 181 | a2vscode Generator | ⏳ | Generate VSCode extension projects from AURA widgets with webview panel rendering |
| 205 | DynamicComponent VM UI | ✅ | VM-driven dynamic UI rendering with VmBridge, AuraViewBuilder, and iced integration |
| 212a | LSP + VSCode Extension Modernization | ✅ | TextMate grammar rewrite, LSP completion sync, Document Symbols, code snippets |
| 217 | a2ui Composer Implementation | ✅ | Three-panel composer with palette/canvas/inspector, builds as Vue 3 app |
| 227 | Dynamic UI Iced Backend | ✅ | `run_file()` auto-detects widget/app, iced window |
| 234 | A3UI A2Vue Replica | ✅ | A2UI Composer Vue replica — all 7 phases done; 7 pages, Widget Editor, Catalogs, Theater, Icons (pixel-perfect split to plan 236) |
| 235-a2vue | a2vue Transpiler Gaps | ✅ | ts_adapter fixes + storage/event/json/math/date/router builtins |
| 238 | Charts Replica | ✅ | area/bar/line/donut chart registry + prop mapping |
| 356 | Vue Generator OOM / Recursion Fix | ✅ | Parser OOM (reserved-kw loop var) + ident.field iterable + soft-kw in conditions; full 015-notes sidebar regenerates |
| 360 | 015-notes UI 现代化 + 主题色切换 | ⏳ | CI-style cardification, 5-color accent theming with localStorage, dark mode transition smoothing |
| 361 | 生成器加固 — Validators + Path Convergence | ✅ | Post-generation SFC validators (R001-R007), single-entry generate_component_from_file, 13 playwright smoke tests |
| 363 | AutoUI Generation Skill | ⏳ | Pre-generation knowledge encoding (contracts+patterns), wizard CLI, test skeleton generation |
| 365 | AutoUI Pluggable Host Architecture | ✅ | `HostBackend` unified interface (headless/iced/gpui); `auto-cosmic` crate family (ports+mock+demo+libcosmic host+Linux adapters); cfg-gated cross-platform; de-iced Component trait; gpui backend fixed; RenderQueue moved to Plan 386 |
| 371 | AutoUI MCP 功能大改进 | ✅ | Agent-driven UI automation via MCP (snapshot/action/state/find/screenshot), path-addressed actions (Task 19), pixel-diff screenshots (Task 20), Rust state snapshot (Task 21), L1 special-case generalization, L3 persistent child-component instances, API PATCH/DELETE + store clone + var-name sanitization, 013-todo multi-component verification (VM+Rust 8/0/0) |
| 337 | vue-gallery ↔ @auto-ui/widgets 薄同步层 | ✅ | LIBRARY_WIDGETS self-consistency test, AURA drift guard (covers_aura_tag), `auto ui backlog` command, widgets.ts split (generated+meta), `auto ui build --target gallery-stubs` page scaffolding, InstallHint component |
| 399 | AutoUI 示例 SSE/CRUD 扩展 | ✅ | 017-chat 首个 SSE 实时聊天 App（playwright 9/9）+ Phase 11 a2r 根治（i64/Slice/borrowed_iter/mutated_let）+ Phase 12 typing + Phase 13 混合状态硬检查；路线A 移交 Plan 400；api_gen 后处理兜底登记债务簿 |
| 404 | 022-kanban 示例升级 | ✅ | CRUD + 列移动 + HTML5 拖拽完整 App，playwright 6/6；修 row/col 属性穿透 bug（push_passthrough_attrs vue.rs:7310） |
| 407 | a2vue icon/text 表达式 | ✅ | lucide 图标子节点 + text 节点 t() 函数调用表达式（parser.rs:12876 + golden 005/006 + auto-musk 侧回流完成） |
| 403 | 011 计算器 MCP+Grid+多模式 | ✅ | 需求 1a/1b/1c/2/3 + Phase 403-F VM 浮点修复全落地；1a 的 desktop_mcp.py + acceptance.atd 由 audit-A8 补齐（实机 14/14） |
| 409 | Widgets Gallery 三模式一致性 | ✅ | §1-§10：link 子组件 VM 渲染/主题色/§10 六残留差距全修复（plan409_tests + golden）；CodeBlock/PreviewCard 纯 Auto 化暂缓（登记债务簿） |

| 408 | view fn → Vue 组件合成 | ✅ | P1–P12 + §10 能力缺口全修复（plan408_tests 17 + golden 007-010）；auto-musk 试点完成（023/028）；P5-2 auto clean 由 audit-A1 修复；P5-4 🟢 延期登记 |
| 402 | 038 扫雷示例 | ✅ | vue 完整 + VM 全流程(§13.6/§13.10/右键/计时器);实机目视确认由 G3 闭环(desktop_mcp 21/0,洪水填充/数字/胜负/Reset 真机验证);rust 后端归 407 |
| 411 | VM 视觉对齐 vue(Home/Button) | ✅ | P0/P1/P2 全数落地:响应式前缀/窗口宽度/active/toast + P2-B MCP 四项(Button.content 序列化/check 对齐/快照过滤/layout 回填)+ P1-C Inter 三字重 + P2-A①prism 色板/④表格;§8.5 gap 分支保留与 Inter 截图核对登记债务簿 |
| 418 | auto-edit 动作真实化与 Action 配置化绑定 | ✅ | Phase1 natives ×11(2919-2929)+13 handler;Phase2 ui_config→ACTION_CONFIG→menubar{}/toolbar{} 声明式渲染(样板清零)+快捷键回退+probe 对齐(P2-7 超预期);checked ✓,enabled-if→423,Phase3→423;041 实机 40/40;3 条债务登记 |
## Status Summary
- Completed: 28 | Partial: 4 | Planned: 11 | Deprecated: 0

## Key Achievements
- Multi-platform AURA pipeline generates native code for Vue, Jetpack Compose, and ArkTS (HarmonyOS) from a single widget DSL
- Incremental UI compilation reuses AIE infrastructure, only regenerating changed widgets during development
- Jet Gallery reference app provides 51 widget demos as the quality target for generated code

## Remaining Work
- AURA Widget Library migration from hardcoded definitions to declarative .at widget specs with WidgetRegistry
- Stdlib widget library consolidation (~45 components from component-gallery into stdlib/aura/widgets)
- Conditional UI backend inclusion and auto-ui migration into the main workspace
- Plan 205: DynamicComponent VM-driven UI rendering for hot-reloadable AURA widgets — DONE
- Plan 217: A2UI Composer three-panel app — DONE
- Plan 212a: LSP + VSCode extension modernization (grammar, completions, snippets)
