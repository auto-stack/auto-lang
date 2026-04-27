# 11 - UI Generators and Frontend Systems

## Overview

AutoLang's UI stack centers on AURA (Auto UI Representation Abstract), a declarative widget DSL that transpiles to multiple native backends. The generator pipeline evolved from a single Vue.js output into a multi-platform system targeting Vue.js (web), Jetpack Compose (Android), ArkTS (HarmonyOS), and planned GPUI/ICED (native Rust) and VSCode extension backends. Twenty-one plans chart this evolution: foundational FFI infrastructure for stdlib functions, the AURA parser and schema, four active code generators, an incremental compilation layer, widget library standardization, and several planned integrations that will bring Rust-native UI and VSCode extension generation into the fold.

## Plan Index

| Plan | Title | Status | Summary |
|------|-------|--------|---------|
| 094 | Hybrid FFI Bridge | Done | VMConvertible trait, #[rust_fn] macro, all 43 stdlib FFI shims |
| 096 | Scenario UI Architecture | Planned | AURA migration from DSL preprocessing to dedicated UI AST |
| 097 | TodoMVC Example | Done | Full TodoMVC demo compiled to Vue and Rust/AutoUI backends |
| 098 | AURA Widget Schema | Planned | Schema system for widget validation and LSP autocomplete |
| 099 | shadcn-vue Migration | Partial | Vue generator updated to shadcn-vue; all 7 phases done, migration of examples pending |
| 113 | a2jet (Auto to Jetpack Compose) | Done | Complete Jetpack Compose generator across 7 phases |
| 114 | Hybrid Routing (Convention + Config) | Planned | Auto-discovered convention routes with config overrides |
| 133 | Jetpack Compose Generator Enhancement | Partial | Extending Jet generator to full AURA syntax; core done, ~40 components remaining |
| 134 | Jet Generator View Body | Done | Recursive node-to-Compose mapping in generate_view_body() |
| 135 | UI Incremental Compilation | Done | Incremental generation reusing AIE infrastructure with UICache |
| 136 | Jet Backend Incremental Adoption | Done | Gradual Jet backend expansion in unified-demo |
| 138 | ArkTS (HarmonyOS) Backend | Done | Complete ArkTS backend verified in DevEco Studio |
| 140 | AURA Widget Library | Planned | Replace hardcoded component definitions with .at widget files and WidgetRegistry |
| 142 | AURA ArkTS Transpilation | Planned | Transpile all 54 AURA widgets to ArkTS components |
| 143 | Stdlib Widget Library | Planned | Migrate ~45 components from component-gallery into stdlib/aura/widgets |
| 144 | 04-Tabs Project | Partial | Bottom tab navigation demo; generator support done, project partially complete |
| 145 | Jet Gallery | Done | Standalone Android Compose reference app with 51 widget demos |
| 147 | unified-demo a2jet Alignment | Partial | Aligning demo and generator with jet-gallery; basic components done |
| 174 | Conditional UI Backend Inclusion | Planned | ui-headless feature flag to skip UI dependencies in default builds |
| 175 | Migrate auto-ui into auto-lang | Planned | Move GPUI and ICED backend runners from standalone crate into auto-lang |
| 180 | a2rust-ui Generator | Planned | Wire RustGenerator into auto gen for GPUI-based Rust UI |
| 181 | a2vscode Generator | Planned | Generate VSCode extension projects from AURA widgets |
| 205 | DynamicComponent VM UI | Complete | VM-driven dynamic UI rendering with VmBridge, AuraViewBuilder, and iced integration |
| 212a | LSP + VSCode Extension Modernization | Complete | TextMate grammar rewrite, LSP completion sync, Document Symbols, code snippets |
| 217 | a2ui Composer Implementation | Complete | A2UI composer with three-panel layout, palette/canvas/inspector, builds and runs |

## Status

**Implemented**: 094, 097, 113, 134, 135, 136, 138, 145, 205, 217 (10 plans fully complete)

**Partial**: 099, 133, 144, 147 (4 plans with significant progress)

**Planned**: 096, 098, 114, 140, 142, 143, 174, 175, 180, 181 (10 plans not yet started or early stage)

## Design

### AURA Architecture and Parser Extensions

The AURA system defines a declarative widget DSL embedded in AutoLang. A widget declares its reactive model state, typed messages, a view tree, and event handlers. Plan 096 laid out the original architecture: a scenario-driven compilation pipeline where `pac.at` specifies `scenario: "ui"` and a backend target, enabling context-sensitive parsing where `widget`, `view`, `model`, `on`, and `msg` become keywords only in UI mode. The AURA intermediate representation is extracted from the parsed AST as a lossless 1:1 mapping, then dispatched to the appropriate backend generator.

Plan 097 (TodoMVC) drove concrete parser extensions: for-loop rendering in view blocks (`for item in .list { ... }`), if/else conditionals, computed properties, dynamic class binding with `{ "active": condition }` syntax, event handlers with parameters (`onclick: .Delete(todo.id)`), and sub-component instantiation. The AST gained `ViewNode::ForLoop`, `ViewNode::Conditional`, `ViewNode::Component`, and `AuraExpr` variants for MethodCall, Array, Lambda, and FieldAccess. A key architectural decision in Plan 097 consolidated the Iced and GPUI generators into a single Rust generator targeting an abstract `auto_ui::prelude::*` trait, reducing the backend count from three (Vue, Iced, GPUI) to two (Vue, Rust).

Plan 098 specified a schema system for AURA widgets, defining which blocks are required (`msg`, `model`, `view`) and optional (`computed`, `on`), along with element categories (layout, content, typography, data, navigation, overlay, form, feedback, display, media, utility) and their allowed props. This schema is intended for parse-time validation, error diagnostics, and LSP autocomplete but has not yet been implemented.

### Vue.js Generator and shadcn-vue Migration

The Vue generator was the first AURA backend, producing Vue 3 Single File Components from widget definitions. Plan 097 established the core transformations: AURA for-loops become `v-for` directives, conditionals become `v-if`/`v-else`, input elements use `v-model` for two-way binding, computed properties generate `computed()` wrappers, and state references like `.count` convert to reactive refs.

Plan 099 migrated the Vue generator from plain Tailwind CSS to shadcn-vue components, adding accessibility through Radix Vue primitives. The implementation introduced a `ShadcnRegistry` mapping all 43 AURA elements to shadcn-vue components (Button, Input, Dialog, Table, Tabs, etc.), a `ComponentRegistry` tracking imports, and seven phases covering core components, layout, overlays, data, forms, and project scaffolding. Each phase added prop-specific generation: Button handles variant/size/disabled, Input handles v-model/type/placeholder, Dialog supports v-model:open, and Table supports colspan/rowspan. The generator produces complete project scaffolds including `components.json`, `package.json` with radix-vue dependencies, Vite configuration, Tailwind config with CSS variables, and a `cn()` utility helper. All 7 phases are complete with 33 unit tests passing.

### Jetpack Compose Generator (a2jet)

Plan 113 established the complete a2jet generator architecture for producing Jetpack Compose Kotlin code. The module is organized into ten sub-modules under `crates/auto-lang/src/ui_gen/jet/`: the main `JetGenerator`, a `Material3Registry` for component mappings, specialized generators for form (`FormGenerator`), layout (`LayoutGenerator`), list (`ListGenerator`), and navigation (`NavigationGenerator`), plus `ModifierDsl` for Tailwind-to-Compose conversion, `StateConverter` for model-to-mutableStateOf, `ProjectGenerator` for full Android project scaffolding, and `ThemeConfig` for theming. The generator maps AURA elements to Material3 components: col/row to Column/Row with Arrangement, input to OutlinedTextField, checkbox to Checkbox, card to Card with variant support, and grid to LazyVerticalGrid.

Plan 134 implemented the critical `generate_view_body()` method, providing recursive node-to-Compose traversal. The method handles all `AuraNode` variants: `Element` dispatches to layout, form, list, or generic handlers; `Text` produces `Text()` composables with string interpolation; `ForLoop` generates `items()` or `itemsIndexed()` calls; `Conditional` produces Kotlin if/else blocks; `Component` renders child composable references with prop forwarding; and `Link` handles navigation. An `expr_to_kotlin()` converter translates AURA expressions including binary operators, unary negation/not, field access, and method calls to Kotlin equivalents.

Plan 133 is enhancing the Jet generator to cover the full AURA syntax surface, driven by the jet-gallery reference app (Plan 145, 51 widget demos). Completed enhancements include Card variants (elevated/outlined/filled), Chip variants (assist/filter/input/suggestion), FlowRow with ExperimentalLayoutApi, and Tabs with TabRow/TabContent state management. High-priority native components (Progress, Image with Coil/AsyncImage, Badge, RadioButton, ListItem) are done. The remaining work covers overlay widgets (Dialog, Sheet, Tooltip), complex composites (Select, Accordion), and reaching full parity with jet-gallery's 51 widget demos.

Plan 136 adopted the Jet backend incrementally in the unified-demo project, expanding component by component. Plan 147 continues this alignment, tracking widget coverage against jet-gallery with detailed priority tiers: green (simple native, quick to add), yellow (moderate composites like Table, Avatar), and red (complex stateful widgets like DropdownMenu, Dialog).

### ArkTS (HarmonyOS) Generator

Plan 138 built the complete ArkTS backend for HarmonyOS, producing `.ets` files from AURA widgets. The architecture mirrors the Jet backend with sub-modules for the generator, component registry, state management, project scaffolding, and modifiers. The component mapping translates AURA primitives to ArkTS equivalents: col/row to Column/Row, button to Button, input to TextInput, checkbox to Checkbox, tabs to Tabs with TabsController, and ForEach for list rendering. State management uses `@State` decorators for reactive fields and a dispatch pattern with TypeScript enums and switch statements for message handling.

A significant bug-fix effort addressed Kotlin syntax contamination in the ArkTS generator. The root cause was code adapted from the Jet backend without proper language adaptation. Six bugs were identified and fixed: sealed class replaced with TypeScript enum, missing `case` keyword in switch, missing `break` statements, double-dot (`..`) member access syntax, wrong Button construction order, and missing import statements. The project was verified to compile and run in DevEco Studio. Plan 144 added Tabs component support with the `@Builder` pattern for tab bars, `TabsController` state management, and the `TabContent().tabBar()` transformation.

### Incremental UI Compilation

Plan 135 implemented incremental compilation for UI code generation, reusing the existing AIE (Auto Incremental Engine) infrastructure. The system extends `ArtifactType` with `VueComponent` and `KotlinFile` variants, introduces `UIArtifact` and `UIBackend` types for tracking generated files, and implements a persistent `UICache` stored at `.auto/ui-cache.json`. The cache uses BLAKE3 hashing to detect source file changes; on subsequent `auto run` invocations, only modified `.at` files are recompiled. Both `JetProject` and `VueProject` were updated to use incremental generation, showing "(cached)" or "(changed)" status for each file. A post-completion fix addressed hyphen-to-underscore conversion for valid Kotlin identifiers in widget function names, package paths, and theme names.

### Widget Library and Standardization

Plan 140 defines the AURA Widget Library architecture, replacing hardcoded component definitions in each generator with `.at` widget files stored in `stdlib/aura/widgets/`. Widgets are organized into seven categories (layout, form, display, navigation, semantic, overlay, feedback, data) with `#[spec]` annotations for metadata (category, primary_prop, has_children) and `#[backend(ark/jet/vue, component=..., import=...)]` annotations for platform-specific mappings. A `WidgetRegistry` in Rust provides case-insensitive lookup by tag name, and generators query the registry instead of maintaining their own hardcoded maps. The core infrastructure (WidgetSpec, WidgetRegistry types, default registration) is partially implemented.

Plan 143 specifies migrating approximately 45 components from the `component-gallery` example into `stdlib/aura/widgets/` as standardized widgets, enforcing consistent prop naming (`text` for primary content, `variant` for visual variants, `disabled` for state), event naming (`onclick`, `onchange`, `onsubmit`), and annotation requirements. Compound components (Dialog with sub-components, Tabs with TabsList/Trigger/Content) live in a single file. The migration follows a category-by-category order starting with display and form widgets.

Plan 142 focuses specifically on verifying AURA-to-ArkTS transpilation for all 54 widgets, with a test framework using `crates/auto-lang/test/a2ark/` directories containing `input.at` and `input.expected.ets` pairs. The plan covers testing core components, complex widgets (List, Tabs, Dialog), and full app integration.

### Planned Backend Integrations

Plan 174 introduces a `ui-headless` feature flag so default `cargo build` skips all UI dependencies. The headless backend is a no-op renderer that implements the same `Component`/`View` pipeline without any window or event loop, enabling fast iteration on UI logic without compiling GPUI or ICED. The design includes a `HeadlessRenderer` that builds VTrees in memory for testing, a `HeadlessStyle` adapter for style inspection, and integration tests verifying render counts and tree structure.

Plan 175 migrates the GPUI and ICED backend runners from the standalone `auto-ui` crate into `crates/auto-lang/src/ui/`, feature-gated behind `ui-gpui` and `ui-iced`. The migration covers backend renderers (893-line GPUI renderer, auto_render module, VNode entity), 20 example apps, transpiler API functions (`transpile_aura()`, `transpile_vue_aura()`), and the `CodeSink` utility. No new CLI commands are needed; the existing `auto gen` flow routes to the appropriate backend.

Plan 180 wires the existing `RustGenerator` into `auto gen` for generating standalone Rust examples from AURA widgets. The generator produces a single `.rs` file with Component implementations, a `main()` function with ICED/GPUI backend selection via Cargo features, and typed Tailwind-like style methods (`.p(4)`, `.gap(2)`, `.w_full()`) on the ViewBuilder.

Plan 181 targets VSCode extension generation, producing a complete extension project from AURA widgets. The architecture delegates UI rendering to the existing VueGenerator (producing webview content) and generates the extension scaffold (package.json, extension.ts, AppPanel.ts) around it. IPC messaging uses the same AURA message model as Tauri, with `postMessage` as the transport layer. The extension supports sidebar or editor panel placement, configurable via a `vscode {}` block in pac.at.

### FFI Foundation

Plan 094, while not directly a UI plan, is categorized here because the FFI bridge underpins all stdlib functionality used by the VM when executing UI-related code. It implements a hybrid FFI architecture combining static bindings (via `#[rust_fn]` macro, IDs 0-9999, array O(1) lookup) with dynamic bindings (via `use.rust`, IDs 10000+, HashMap lookup). The `VMConvertible` trait provides automatic type conversion between Rust and VM types for String, i32, bool, Result, and Vec. All 43 built-in stdlib shims are implemented across File I/O (10), Environment (3), Time (3), Process (5), Path (5), String (10), Char (7), and Math (4) categories, enabling the self-hosting compiler path.

## Open Questions

- How will the AURA Widget Library (Plan 140) interact with the existing hardcoded registries in each generator? The plan proposes full replacement but migration order and backward compatibility are unspecified.
- The ArkTS generator (Plan 142) needs verification of all 54 widget annotations against the ArkTS SDK. Which components lack `#[backend(ark, ...)]` annotations?
- Plan 175 (auto-ui migration) depends on Plan 174 (conditional UI inclusion) being completed first. What is the timeline for these dependent plans?
- The VSCode extension generator (Plan 181) shares an IPC model with the planned a2tauri generator. Should this shared messaging layer be abstracted now or deferred?
- The jet-gallery reference app has 51 widget demos but only about 20 have corresponding a2jet generator support. What is the priority order for closing this gap?

## Source Plans

- 094-hybrid-ffi-bridge.md
- 096-scenario-ui.md
- 097-todomvc-example.md
- 098-aura-schema.md
- 099-shadcn-vue-migration.md
- 113-a2jet-design.md
- 114-hybrid-routing.md (Plan 119 in file)
- 133-jetpack-compose-generator-enhancement.md
- 134-jet-generator-view-body.md
- 135-ui-incremental-compilation.md
- 136-jet-backend-incremental.md
- 138-arkts-backend.md
- 140-aura-widget-library.md
- 142-aura-arkts-transpilation.md
- 143-stdlib-widget-library.md
- 144-04-tabs-project.md
- 145-jet-gallery.md
- 147-unified-demo-a2jet-alignment.md
- 174-conditional-ui-backends.md
- 175-migrate-auto-ui.md
- 180-a2rust-ui-generator.md
- 181-a2vscode-generator.md
- [205-dynamic-component-vm-ui.md](../plans/old/205-dynamic-component-vm-ui.md)
- [212-lsp-vscode-modernization.md](../plans/212-lsp-vscode-modernization.md)
- [217-a2ui-composer-implementation.md](../plans/old/217-a2ui-composer-implementation.md)
