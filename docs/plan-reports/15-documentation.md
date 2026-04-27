# 15 - Documentation and Learning Resources

## Overview

AutoLang's documentation and learning ecosystem spans 21 plans organized around three pillars: interactive documentation sites, progressive example collections, and developer tooling. The work covers component gallery documentation with live preview, multi-platform tutorial projects (QuickStart series for HarmonyOS, unified cross-platform examples for six targets), a document authoring format (AutoDown), routing infrastructure for SPA navigation, and an AI-friendly compiler mode. Three plans are completed (comptime examples, Jet Gallery, navigation example), one is partially done (QuickStart Sprint A), and the remainder are planned but not yet started.

## Plan Index

| Plan | Title | Status | Summary |
|------|-------|--------|---------|
| 032 | Source Mapping for Self-Hosting | Planned | Source-to-C mapping for IDE-grade error messages in transpiled code |
| 097 | TodoMVC Example | Completed | Multi-backend TodoMVC driving AURA language feature development |
| 103 | AutoUI Component Gallery | Completed | shadcn-vue style docs site with live preview and code copy |
| 104 | Add shadcn-vue Components | Planned | Full shadcn-vue component support in AURA transpilation |
| 105 | Auto Router | Planned | URL-driven routing with routes block, outlet, and link elements |
| 106 | Router `use` Syntax Improvement | Planned | Cleaner routing definition using use keyword and module path conventions |
| 107 | Hyphenated Identifiers | Planned | Allow hyphens in identifiers with space-based disambiguation |
| 108 | Component Gallery Page Files | Planned | 38 missing widget page files and 7 existing page updates |
| 109 | AutoDown Document Format | Planned | Text-first document DSL transpiling to Typst, DOCX, React/Vue |
| 132 | api-example Read-Only Demo | Planned | First working multi-platform front+back transpilation demo |
| 137 | Comptime Example Codebase | Completed | Example code demonstrating compile-time execution features |
| 141 | QuickStart Sprint A | Partial | Reimplement 12 QuickStart tutorial projects as Auto projects |
| 144 | 04-Tabs Project | Planned | Bottom tab navigation QuickStart example for ArkTS |
| 145 | Jet Gallery (Android Compose) | Completed | Standalone Jetpack Compose reference app with widget registry |
| 148 | 05-Nav Navigation | Completed | Page-level navigation within tabs for QuickStart examples |
| 149 | KnowledgeMap Data Loading | Planned | Replace static placeholder content with real JSON-loaded data |
| 150 | AI Mode (--ai flag) | Completed | JSON output mode for AI-friendly compiler interaction |
| 157 | Login Quickstart Example | Planned | 06-Login example design for quickstart tutorial series |
| 183 | Unified UI Examples | Planned | Progressive cross-platform UI examples for all 6 targets |
| 188 | Tier 3 Blocker Resolution | Planned | Fix prerequisites blocking Tier 3 mini-app examples |
| 189 | Tier 4 Prerequisites | Planned | Resolve feature gaps for Tier 4 real-app examples |
| 196 | Plan Reports by Topic | Complete | Create 16 summary report files in docs/plan-reports/ organized by topic area |
| 202 | Auto Playground | Complete | Web-based code editor + VM execution + transpilation viewer (Vue 3 + axum backend) |
| 209 | ac-examples Modernization | Complete | 33/33 examples pass; Phase 0 complete, Phases 1-6 (modernization with new features) deferred |
| 210 | Book Listing Test Harness | Complete | Auto-discovery test harness for 1136 code listings in the book |
| 218 | Plan Status Reconciliation | Complete | Reconcile plan status across all tracking documents and reports |

## Status

**Implemented**: Plans 097 (TodoMVC), 103 (Component Gallery), 137 (Comptime Examples), 145 (Jet Gallery), 148 (Navigation), 150 (AI Mode), 202 (Playground), 209 (ac-examples)

**Partial**: Plan 141 (QuickStart Sprint A -- Sprint A tutorials partially done, Sprints B-D remain)

**Planned**: Plans 032, 104, 105, 106, 107, 108, 109, 132, 144, 149, 157, 183, 188, 189

## Design

### Component Gallery and Widget Documentation

The AutoUI Component Gallery (Plan 103) is the central documentation site for AURA widgets, modeled after the shadcn-vue documentation style. The gallery was built in six phases: project infrastructure, core component definitions (Button, Input, Text, Card, Badge, Label, Accordion, Tabs), documentation pages, code display components with copy functionality, navigation with sidebar and client-side routing, and a build/deploy pipeline. All six phases are complete, and the gallery runs as a generated Vue application from `examples/component-gallery/`. A key technical achievement was fixing the AURA parser to support string literal conditions in `if/else` chains (e.g., `if .currentPage == "button"`) and handling quote escaping in the Vue generator's `convert_condition()` function.

Plan 104 extends this work by mapping the full shadcn-vue component library (66 unique components) into AURA. The mapping is organized into five phases by priority: core components (Card, Dialog, Badge, Tabs, Checkbox, Switch, Label, Textarea, Separator), form components (Select, Combobox, Slider, RadioGroup, DatePicker, Form), layout and navigation (Accordion, Breadcrumb, DropdownMenu, Sidebar, Pagination, ScrollArea), feedback and overlay (Alert, Toast, Tooltip, Popover, HoverCard, Drawer, Sheet), and data display (Avatar, Skeleton, Progress, Calendar, Carousel, Command, TagsInput). Each component requires changes to three layers: the AURA schema in `schema.rs`, the Vue generator in `vue.rs`, and an example page. Currently only Button and Table are fully mapped.

Plan 108 fills in the documentation content. The gallery currently has 10 page files (3 complete: button, input, index; 7 crude: accordion, badge, card, checkbox, label, tabs, text). This plan defines 45 tasks to update the 7 crude pages and create 38 new pages, organized in batches: simple components (Alert, Avatar, Breadcrumb, Separator, Skeleton, Progress), form inputs (Textarea, Select, Switch, Slider, RadioGroup), navigation components (Pagination, Menubar, NavigationMenu, DropdownMenu, ContextMenu), overlay components (Dialog, AlertDialog, Drawer, Sheet, Popover, HoverCard, Tooltip), complex components (Table, DataTable, Calendar, DatePicker, Carousel, Collapsible, Combobox, Command, Form), and feedback components (Toast, Sonner, Toggle, ToggleGroup, ScrollArea, Sidebar). Each page follows a consistent template with h1 title, description, installation codeblock, preview-card examples, and a properties table.

### Routing Infrastructure

Auto Router (Plan 105) introduces URL-driven routing into AURA widgets. The syntax adds a `routes` block alongside `view` and `on` blocks, an `outlet` element that renders the matched route component, a `link` element for declarative navigation, and a `nav()` function for programmatic navigation. The implementation spans the full compiler pipeline: new token types (Routes, Outlet, Link, Route, Nav), AST nodes (RouteDef, RoutesBlock), parser support for route definitions and the outlet/link elements, AURA type extensions (AuraRoute, AuraRoutes, Outlet, Link variants), Vue generator support for `<router-view>`, `<router-link>`, and router configuration file generation, and CLI integration in `cmd_vue.rs` to detect routes and generate `router/index.ts` with vue-router dependency. The plan also outlines future phases for an `app` keyword to separate application-level configuration from widget definitions, and advanced routing features like nested routes, guards, and lazy loading.

Plan 106 refines the routing syntax by replacing the component instantiation style (`"/" => IndexPage {}`) with a `use`-based convention (`"/" => use index`). This change provides three benefits: the import path becomes deterministic (`@/pages/index.vue` instead of guessed `@/pages/IndexPage.vue`), lazy loading is the default (using `() => import(...)` syntax), and file naming follows lowercase convention. The two syntaxes are designed to coexist for backward compatibility.

Plan 107 proposes hyphenated identifiers, enabling names like `preview-card` and `button-primary` in AURA source. The disambiguation rule is simple: a hyphen between valid identifier characters is part of the identifier; a hyphen with spaces is the subtraction operator. The change is localized to the lexer's `identifier()` function, which checks whether the character after `-` is a valid identifier character before including it.

### Progressive Tutorial and Example Systems

The QuickStart series (Plans 141, 144, 148, 149, 157) reimplements 12 HarmonyOS QuickStart tutorials as Auto projects, driving ArkTS generator development. Sprint A (Plan 141) covers tutorials 01-03 (HelloWorld, Banner/Swiper, Components) and extends the generator with Swiper support, new modifiers (fontFamily, lineHeight, objectFit, layoutWeight), extended Tailwind parsing, Image src handling, and ForEach key function generation. Sprint B tutorials (04-06) cover Grid, List, and MVVM patterns. Sprint C (07-09) covers WebView, data-driven UI, and navigation.

Plan 144 (04-Tabs) introduces bottom tab navigation using a Tabs widget that maps to ArkTS `Tabs` with `@Builder tabBarBuilder`. The generator detects the `Tabs` + `TabsList` + `TabsTrigger` + `TabsContent` pattern and generates the corresponding ArkTS structure with `TabsController` and `currentIndex` state management. Plan 148 (05-Nav) adds page-level navigation within tabs using HarmonyOS `Navigation` + `NavPathStack`, with `@Provide`/`@Consume` decorators for sharing the path stack between parent pages and child components, and `nav()` function calls for programmatic `pushPathByName`. Plan 149 (KnowledgeMap Data Loading) would replace static placeholder content with real JSON loaded at runtime using `Json.load()`, requiring lifecycle block support (`aboutToAppear()`), indexed for loops (`for i, item in list`), and data file transpilation to JSON rawfiles. Plan 157 (06-Login) designs a login form example with responsive GridRow/GridCol layout and Input widget maxLength support.

The Unified UI Examples system (Plan 183) takes a broader approach, defining 24 progressive examples organized into four tiers. Tier 1 (001-005) covers basics: helloworld, counter, converter, profile-card, login. Tier 2 (006-010) covers composed UI blocks: hero-section, stats-board, pricing-table, article-feed, contact-form. Tier 3 (011-016) covers mini apps: calculator, stopwatch, todo, weather, notes, calendar. Tier 4 (017-024) covers real apps: chat, book-reader, video-app, music-player, blog-viewer, kanban, realworld, widget-gallery. Each example generates to all six targets (Vue, Jetpack Compose, ArkTS, GPUI, Tauri, VSCode WebView) from a single `.at` source file, with a README showing side-by-side generated output. Tier 1 and Tier 2 are the current implementation scope; Tier 3 and Tier 4 are blocked by feature gaps.

Plan 188 identifies four specific blockers preventing Tier 3 examples from reaching their full potential: template string interpolation (`${.var}`) crashing in view blocks, non-functional grid layout widgets, absence of timer/async tick mechanisms, and typed message variant handlers not being generated correctly. Plan 189 extends the analysis to Tier 4, identifying dark mode toggle support, multi-page routing validation, and static mock patterns for missing features (drag-and-drop, auth, markdown) as prerequisites. The strategy is pragmatic: implement what AURA can support now (dark mode, routing), and use static mock patterns for what requires external JavaScript libraries.

### Document Authoring

AutoDown (Plan 109) is a text-first document DSL that transpiles to multiple backends: Typst/PDF, DOCX/Word, and HTML/Vue. The design uses three core symbols (`#` for headers, `$` for code/logic, `%{}` for math) with a Flip mechanism that switches between text mode and code/math mode. The implementation is organized into five phases: core infrastructure (mode-aware lexer, document AST, recursive descent parser), backend transpilers (Typst, DOCX via docx-rs, HTML with MathJax/KaTeX), CLI and API integration, advanced features (AutoMath parser for function-style math expressions, reusable document components, template variables), and testing with snapshot-based verification. The AST is structured as `AdocDocument` containing `AdocSection` nodes with `AdocBlock` and `AdocInline` content, supporting paragraphs, lists, code blocks, math, tables, conditionals, and component calls.

### Developer Tooling

AI Mode (Plan 150, completed) adds an `--ai` global flag to the `auto` CLI that switches all output to structured JSON. In AI mode, the logo and progress messages are suppressed, errors are formatted as JSON with fields for message, code, severity, spans, and help text, and success results are wrapped in `{"status": "success", "result": ...}`. The implementation touched `crates/auto/src/main.rs`, wrapping all command handlers with conditional output formatting. This enables AI assistants and IDE integrations to consume compiler output programmatically without parsing human-readable text.

Source Mapping (Plan 032) addresses a different tooling need: mapping C runtime errors back to AutoLang source locations in transpiled code. The design combines custom JSON source maps for programmatic access with C `#line` directives for gcc/clang error reporting. The implementation tracks source locations through the transpiler using `write_with_location()` calls that record both the generated C line and the original `.at` position, then produces a `.cmap` file alongside each generated `.c` file. A C-language error formatter in the stdlib would then produce miette-quality error messages with source snippets, ANSI colors, and multi-line error display.

The api-example demo (Plan 132) aims to be the first end-to-end proof of the multi-platform transpilation story: an AURA frontend widget transpiles to Vue with fetch API calls, Auto type definitions transpile to Rust with Axum routes, and HTTP JSON over localhost connects them. The implementation extracts type definitions from AST, generates TypeScript API clients with native fetch, generates Rust server scaffolding with Axum route handlers, and wires the Vue generator for async API calls in event handlers.

### Compile-Time Examples

The Comptime Example Codebase (Plan 137, completed) provides 16 working example files organized into three levels. Level 1 (basic) covers const evaluation (`#{ 1 + 2 }`), arithmetic, variable interpolation, nested expressions, and boolean logic. Level 2 (intermediate) covers platform selection, loop unrolling, pattern matching, conditional chains, and combined loop-conditions. Level 3 (advanced) covers factorial, Fibonacci, power tables, bitmask calculations, state machine patterns, and configuration validation. Each example uses CommentTest markers (`#[expect_value(X)]` or `#[expect_output("...")]`) for future automated testing. Notably, `#if`, `#for`, and `#is` are parsed but not yet transformed at compile time; the examples use runtime equivalents pending the Compile-Time Execution Engine (Plan 095).

## Open Questions

- How should the Component Gallery scale to 48+ pages while maintaining consistent documentation quality and shadcn-vue accuracy across all widget pages?
- Should AutoDown target Markdown compatibility as a priority, or diverge freely with its Flip-based mode system?
- What is the right abstraction for timer/async ticks in AURA -- declarative `timer` widgets, `on tick` handlers, or async model properties?
- When should the progressive example system (Plan 183) prioritize breadth (more examples at current quality) versus depth (fixing blockers to unlock full-featured examples)?
- Can the routing syntax (Plan 105/106) be generalized cleanly across all six generator targets, or should it remain Vue-specific initially?

## Source Plans

- 032-source-mapping.md
- 097-todomvc-example.md
- 103-component-gallery.md
- 104-shadcn-vue-components.md
- 105-auto-router.md
- 106-router-use-syntax.md
- 107-hyphen-identifiers.md
- 108-component-gallery-pages.md
- 109-auto-down-implementation.md
- 132-api-example-demo.md
- 137-comptime-examples.md
- 141-quickstart-sprint-a.md
- 144-04-tabs-project.md
- 145-jet-gallery.md
- 148-05-nav-navigation.md
- 149-knowledgemap-data-loading.md
- 150-ai-mode-design.md
- 157-login-quickstart-design.md
- 183-unified-ui-examples.md
- 188-tier3-blocker-resolution.md
- 189-tier4-prerequisites.md
- [196-plan-reports.md](../plans/196-plan-reports.md)
- [202-playground-design.md](../plans/old/202-playground-design.md)
- [209-example-modernization.md](../plans/old/209-example-modernization.md)
- [210-book-listing-test-harness-design.md](../plans/210-book-listing-test-harness-design.md)
- [218-plan-status-reconciliation.md](../plans/218-plan-status-reconciliation.md)
