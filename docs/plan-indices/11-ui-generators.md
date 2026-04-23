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
| 205 | DynamicComponent VM UI | ⏳ | VM-driven dynamic UI rendering with VmBridge, AuraViewBuilder, and iced integration |
| 212a | LSP + VSCode Extension Modernization | ⏳ | TextMate grammar rewrite, LSP completion sync, Document Symbols, code snippets |

## Status Summary
- Completed: 8 | Partial: 4 | Planned: 11 | Deprecated: 0

## Key Achievements
- Multi-platform AURA pipeline generates native code for Vue, Jetpack Compose, and ArkTS (HarmonyOS) from a single widget DSL
- Incremental UI compilation reuses AIE infrastructure, only regenerating changed widgets during development
- Jet Gallery reference app provides 51 widget demos as the quality target for generated code

## Remaining Work
- AURA Widget Library migration from hardcoded definitions to declarative .at widget specs with WidgetRegistry
- Stdlib widget library consolidation (~45 components from component-gallery into stdlib/aura/widgets)
- Conditional UI backend inclusion and auto-ui migration into the main workspace
- Plan 205: DynamicComponent VM-driven UI rendering for hot-reloadable AURA widgets
- Plan 212a: LSP + VSCode extension modernization (grammar, completions, snippets)
