### 01-ast-core.md (3 uncompleted)
- | 011 | Auto-Atom Refactoring | ⏳ | Transform auto-atom from prototype to production-ready library with error handling, query API, JSON support |
- | 013 | Unify Args and Props | ⏳ | Eliminate separate `Args` structure, unify with `props` using IndexMap `num_args` boundary counter |
- | 014 | Unify Body, Nodes, Kids | ⏳ | Unify `body`, `body_ref`, and `nodes` fields into single `kids` field with `Kid` enum |
### 02-type-system.md (2 uncompleted)
- | 018 | Type Composition Improvements | 🔧 | Fix and improve `has` composition with method resolution; gaps remain in evaluator and transpiler |
- | 055 | Storage Injection | ⏳ | Platform-aware storage strategy injection (MCU=Fixed, PC=Dynamic) for `List<T>` |
### 03-error-handling.md (1 uncompleted)
- | 009 | Runtime Error Integration | 🔧 | Replace `panic!` calls in evaluator with `RuntimeError` system, source location tracking, and stack traces |
### 05-stdlib.md (6 uncompleted)
- | 042 | Dynamic String (dstr) | 🔧 | Byte-level dynamic string type built on List field with OOP methods |
- | 043 | Generic Slice Type | ⏳ | Slice<T> with []T syntax, range operators, and immutable views into contiguous sequences |
- | 054 | Context Environment | ⏳ | Unified runtime environment injection system with compile-time/prelude/startup phases |
- | 119 | a2rs Backend Stdlib | 🔧 | Backend stdlib for HTTP, Redis, SQLite enabling server-side AutoLang applications |
- | 143 | Stdlib Widget Library | ⏳ | Migration of ~45 components from component-gallery into stdlib/aura/widgets (7 categories) |
- | 195 | HTTP Client + auto.http Unification | 🔧 | Upgrade to reqwest, unify http_stream, add async HTTP (Phase 3.2 blocked by Plan 196) |
### 06-transpilers.md (14 uncompleted)
- | 007 | Implement a2r Transpiler | 🔧 | Auto-to-Rust transpiler following a2c architecture patterns; basic phase 1 complete |
- | 067 | Strengthen Rust Transpiler | ⏳ | Gap analysis to bring a2r to feature parity with a2c (34 vs 161 tests) |
- | 083 | a2r with .rs.at and #[rs] | 🔧 | Platform-specific Rust implementation files and #[rs] annotation support |
- | 100 | a2js to a2ts Migration | 🔧 | Upgrade JavaScript generator to TypeScript with ArkTS variant support |
- | 162 | .to(Type) Method Keyword | ⏳ | Explicit type conversion method keyword complementing .as(Type) reinterpret cast |
- | 164 | a2r ext for Trait | ⏳ | External trait implementation via ext Type for Trait syntax in a2r |
- | 165 | Struct Destructuring in is | ⏳ | Rust-style {field1, field2} struct destructuring in is match arms |
- | 166 | a2r Generic Constraints | ⏳ | Emit #[with(T as Trait)] annotations as <T: Trait> in Rust output |
- | 174 | Conditional UI Backends | ⏳ | ui-headless feature flag for UI-less builds, skipping GPUI/ICED dependencies |
- | 175 | Migrate auto-ui into auto-lang | ⏳ | Move GPUI and ICED backends from standalone auto-ui into auto-lang workspace |
- | 180 | a2rust-ui Generator | ⏳ | Wire RustGenerator into auto gen for Rust UI backend (GPUI examples) |
- | 181 | a2vscode Generator | ⏳ | VSCode extension generator from AURA widgets using a2vue + webview panel |
- | 187 | a2ts Vue Adapter | ⏳ | Replace Vue generator's inline JS with a2ts delegation for proper TypeScript output |
- | 240 | Rust Cookbook a2r Tests | ⏳ | Systematic a2r test suite from Rust Cookbook examples |
### 07-vm-runtime.md (5 uncompleted)
- | 039 | VM Tests Migration to AutoVM Tests | 🔧 | Migrate vm_tests.rs tests to autovm_tests.rs by complexity level |
- | 074 | Use Statement Multi-Directory Search | 🔧 | Multi-directory module lookup for use statements |
- | 077 | Unified Object Registry + Generic ListData | 🔧 | Single registry for heap objects, generic ListData<T> storage (50%) |
- | 118 | VM Test Failures Analysis | 🔧 | Systematic fix of 76+ failing VM tests (183/197 passing) |
- | 177 | VM File-Based Test Framework | ⏳ | Replace inline tests with file-based .expected.out/result/error assertions |
### 08-async-concurrency.md (1 uncompleted)
- | 195 | HTTP Client + auto.http Unification | 🔧 | Upgrade to reqwest, unify http_stream, add async HTTP (Phase 3.2 blocked by Plan 196) |
### 10-build-tooling.md (8 uncompleted)
- | 063 | AIE Architecture Migration | 🔧 | Migrate from file-based to query-based incremental compilation; phases 1-3.6 complete, MCU and later phases deferred |
- | 093 | AutoMan Redesign for Rust Support | ⏳ | Extend auto-man to support Auto/Rust mixed projects alongside existing Auto/C projects |
- | 111 | Auto CLI Refactor and Unification | ⏳ | Merge auto-man CLI into universal auto CLI with standardized command structure |
- | 112 | AutoMan B.P.B.E Architecture Refactor | ⏳ | Redesign AutoMan around Backend, Port, Builder, Export architecture for multi-target builds |
- | 146 | AutoShell SmartCmd Integration | ⏳ | Integrate nushell/uutils libraries for structured shell command output and cross-platform support |
- | 151 | Tauri IPC Mode for api-example | ⏳ | Generate complete Tauri IPC backend by transpiling api.at + db.at to Rust |
- | 186 | Switch from npm to bun for Vue/Web Projects | ⏳ | Replace npm with bun for faster installs via global cache hard-linking |
- | 243 | LSP & VSCode Modernization | 🔧 | Rewrite auto-lsp over Database+QueryEngine; Phase 1 (compiles), Phase 2-6 pending |
### 11-ui-generators.md (15 uncompleted)
- | 096 | Scenario UI Architecture | ⏳ | AURA architecture migration from DSL preprocessing to dedicated UI AST |
- | 098 | AURA Widget Schema Specification | ⏳ | Schema system for widget validation, LSP autocomplete, and error diagnostics |
- | 099 | shadcn-vue Migration | 🔧 | Migrate Vue generator to shadcn-vue components; generator updated, full 43-element coverage in progress |
- | 114 | Hybrid Routing (Convention + Config) | ⏳ | Hybrid routing with auto-discovered convention routes and config-based overrides |
- | 133 | Jetpack Compose Generator Enhancement | 🔧 | Extend Jet generator to full AURA syntax; core components done, 40+ remaining |
- | 140 | AURA Widget Library | ⏳ | Replace hardcoded component definitions with .at widget files and WidgetRegistry |
- | 142 | AURA ArkTS Transpilation | ⏳ | Transpile all 54 AURA widgets to ArkTS components for HarmonyOS |
- | 143 | Stdlib Widget Library | ⏳ | Migrate ~45 components from component-gallery into stdlib/aura/widgets |
- | 144 | 04-Tabs Project | 🔧 | Bottom tab navigation demo with 3 tabs translating to ArkTS Tabs component |
- | 147 | unified-demo a2jet Alignment | 🔧 | Align unified-demo and a2jet with jet-gallery reference; basic components done |
- | 174 | Conditional UI Backend Inclusion | ⏳ | Add ui-headless feature flag so default builds skip all UI dependencies |
- | 175 | Migrate auto-ui into auto-lang | ⏳ | Move GPUI and ICED backend runners from standalone auto-ui into auto-lang workspace |
- | 180 | a2rust-ui Generator | ⏳ | Wire RustGenerator into auto gen for GPUI-based Rust UI examples |
- | 181 | a2vscode Generator | ⏳ | Generate VSCode extension projects from AURA widgets with webview panel rendering |
- | 234 | A3UI A2Vue Replica | ⏳ Phase 3+ | A2UI Composer Vue replica — Phase 0-2 complete, Widget Editor/Catalog/Theater remaining |
### 12-testing.md (2 uncompleted)
- | 110 | AutoDown Comprehensive Test Suite | 🔧 | Establish test suite for AutoDown covering lexer, parser, transpilers, math, and edge cases |
- | 179 | Migrate vm_tests.rs to File-Based vm_file Tests | 🔧 | Migrate ~130 inline VM tests to file-based .at test files; ~167 file-based tests, vm_tests.rs slimmed |
### 13-self-hosting.md (5 uncompleted)
- | 028 | Generic Types and Monomorphization | ⏳ | Full generics with monomorphization for type-safe containers and algorithms |
- | 029 | Pattern Matching System | ⏳ | Comprehensive pattern matching extending `is` statement with structs, enums, guards |
- | 030 | Trait System Completion | ⏳ | Full trait/polymorphism system with generic traits, associated types, dynamic dispatch |
- | 031 | Bootstrap Strategy | ⏳ | Three-stage bootstrap to resolve compiler-stdlib circular dependency |
- | 033 | Self-Hosting Compiler | ⏳ | AutoLang compiler written in AutoLang targeting C via a2c transpiler |
### 14-language-features.md (13 uncompleted)
- | 044 | ext Enhanced Multiplatform Architecture | ⏳ | Interface contract + physical completion pattern for platform-specific stdlib |
- | 045 | AutoLang-AutoUI Integration | ⏳ | EvalMode::Config for parsing UI scripts, runtime + transpilation paths |
- | 050 | Auto Prelude System | ⏳ | Rust-inspired prelude auto-importing common symbols into every module |
- | 072 | Logical Operators `and`/`or` | ❌ | Deprecated -- reverted to `&&`/`||` symbols for consistency |
- | 082 | AutoCache -- Global Build Cache | ⏳ | Content-addressable store for cross-project compilation artifact reuse |
- | 084 | Unified Type Context | ⏳ | Consolidate scattered type information into a single shared TypeStore |
- | 086 | Widget Registry from Stdlib | 🔧 | Load widget specs from .at files instead of hardcoded Rust defaults |
- | 139 | Atom Serialization System | ⏳ | Serde-like Auto-Atom serialization/deserialization with compile-time codegen |
- | 155 | String Type Refactoring | ⏳ | Three-tier string system: StrLit, StrSlice, and owned String |
- | 156 | Unified Enum Migration | ⏳ | Merge enum (scalar) and tag (ADT) into single `enum` keyword with three forms |
- | 182 | Debug Mode for Rust Desktop UI | ⏳ | Chrome DevTools-inspired debug overlay for GPUI/iced desktop frameworks |
- | 185 | VSCode Extension Reuses Vue Build | ⏳ | Eliminate duplicate webview build by reusing gen/vue/dist output |
- | 190 | Extend use.rust for Rust Stdlib Access | ⏳ | Import any Rust stdlib type/function via use.rust with compile-time type awareness |
### 15-documentation.md (18 uncompleted)
- | 032 | Source Mapping for Self-Hosting | ⏳ | Source-to-C mapping for IDE-grade error messages in transpiled code |
- | 097 | TodoMVC Example | ⏳ | Multi-backend TodoMVC in AutoLang (Vue, Iced, GPUI) |
- | 103 | AutoUI Component Gallery Documentation Site | ⏳ | shadcn-vue style docs site with live preview and code copy |
- | 104 | Add shadcn-vue Components | ⏳ | Full shadcn-vue component support in AURA transpilation |
- | 105 | Auto Router | ⏳ | URL-driven routing with `routes` block, `outlet` and `link` elements |
- | 106 | Router `use` Syntax Improvement | ⏳ | Cleaner routing definition using `use` keyword and module path conventions |
- | 107 | Hyphenated Identifiers | ⏳ | Allow hyphens in identifiers (e.g., `preview-card`) with space-based disambiguation |
- | 108 | Component Gallery Page Files | ⏳ | 38 missing widget page files and 7 existing page updates for gallery |
- | 109 | AutoDown Document Format | ⏳ | Text-first document DSL transpiling to Typst, DOCX, React/Vue |
- | 132 | api-example Read-Only Demo | ⏳ | First working multi-platform front+back transpilation demo |
- | 141 | QuickStart Sprint A | 🔧 | Reimplement 12 QuickStart tutorial projects as Auto projects |
- | 144 | 04-Tabs Project | ⏳ | Bottom tab navigation QuickStart example for ArkTS |
- | 149 | KnowledgeMap Data Loading | ⏳ | Replace static placeholder content with real JSON-loaded data |
- | 150 | AI Mode (--ai flag) | ⏳ | JSON output mode for AI-friendly compiler interaction |
- | 157 | Login Quickstart Example | ⏳ | 06-Login example design for quickstart tutorial series |
- | 183 | Unified UI Examples | ⏳ | Progressive cross-platform UI examples for all 6 targets |
- | 188 | Tier 3 Blocker Resolution | ⏳ | Fix prerequisites blocking Tier 3 mini-app examples (011-016) |
- | 189 | Tier 4 Prerequisites | ⏳ | Resolve feature gaps for Tier 4 real-app examples (017-024) |
### 16-shell-tools.md (2 uncompleted)
- | 153 | AutoShell AI Agent Design | ⏳ | Multi-granularity AI agent with LLM providers, tools, MCP, and multi-agent coordination |
- | 159 | AutoCode Coding Agent | ⏳ | AI-powered coding agent integrated into AutoShell |
