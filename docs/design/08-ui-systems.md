# 08 - UI Systems

## Status

UI system components are at varying stages of implementation:

- **AURA** (`crates/auto-lang/src/aura/`): Schema and type definitions implemented (7 modules: atom, extract, schema, schema_loader, types, validate, mod). The core extraction pipeline from widget declarations to AURA IR is in progress.
- **a2ark** (ArkTS backend): Complete with 12+ widget tests covering layout, form, display, navigation, and dialog components.
- **a2jet** (Jetpack Compose backend): Complete with 11 modules and full project generation capability.
- **a2vue** (Vue backend): Implemented in `ui_gen/vue.rs`.
- **Shared UI gen** (`ui_gen/shared/`): Widget registry, state converter, style system, and Tailwind utilities.
- **Design Token Compiler**: Design defined (`crates/auto-lang/src/tokens/` planned), not yet implemented.
- **Frontend-backend communication**: Architecture defined, `#[api]` annotation support is planned.
- **AutoDown**: Conceptual design complete, no implementation yet.

## Design

### AURA (Auto UI Representation Abstract)

AURA is the official UI intermediate representation -- the "UI-IR" layer that sits between Auto source code and platform-specific backends.

**Design philosophy**: Structure and logic are absolutely decoupled. AURA extracts three pure elements from widget declarations:
1. **View tree** (UI skeleton) -- pure layout with bindings, no logic
2. **State definitions** (reactive model) -- typed state signatures with defaults
3. **Event handlers** (message routing) -- preserved as AST blocks or compiled bytecode

**Trigger mechanism**: AURA processing activates only when `scenario: "ui"` is set in `pac.at` or via `auto build -s ui`. The lexer treats `widget`, `view`, `model`, `on` as regular identifiers. The parser checks the session scenario and promotes these to contextual keywords only in UI mode, preventing namespace pollution in core scenarios.

**Extraction pipeline**:
1. **Parsing**: Generates AST with native `WidgetDecl` nodes (not desugared).
2. **Extraction**: Compiler extracts `model` and `view` into 1:1 lossless AURA structures. Handler logic is preserved as `LogicPayload` (either `AstBlock` for AOT backends or `Bytecode` for dynamic execution).
3. **Backend dispatch**: AURA feeds into target-specific generators (a2vue, a2jet, a2ark, a2lvgl).

**Surface syntax example**:

```auto
widget Counter {
    msg Msg { Inc, Dec }
    model { count int = 0 }
    view {
        col {
            button + { onclick: .Inc }
            h2 > Current Count: ${.count}
            button - { onclick: .Dec }
        }
    }
    on {
        .Inc => { .count += 1 }
        .Dec => { .count -= 1 }
    }
}
```

**Simplified syntax layer**: AURA provides progressive syntax sugar for common patterns:
- **`center` component**: Sugar for `col` with `w-full h-full justify-center items-center`.
- **Primary prop shorthand**: `text "Hello"` expands to `text (text: "Hello")`.
- **Trailing style**: Style properties can appear after children in element bodies.
- **Content-first principle**: Children and content precede style declarations for readability.

**Widget library**: Defined in `stdlib/aura/widgets/` with `#[spec]` and `#[backend]` annotations. Categories: Layout (Column, Row, Stack, Scroll), Form (Button, Input), Display (Text, Image), Navigation (Swiper), Semantic (header, footer, nav, main).

### Scenario-Based Programming

Auto supports "scenario programming" where the compiler behavior is driven by project context rather than global language features.

**`pac.at` configuration**: Declares project scenario (`ui`, `core`, `shell`), backend target, and build settings. This is the single source of truth for LSP, compiler, and build tools.

**Compiler session**: A `CompilerSession` struct carries the scenario state through the pipeline. The parser uses this to activate contextual keywords only when appropriate. In `scenario: "core"` projects, `let widget = create_window()` is perfectly valid with no conflicts.

**LSP integration**: The language server reads `pac.at` on initialization to configure the correct parser mode. Diagnostics, hover, and completion all respect the scenario context.

### Design Token System

A cross-platform design token system for visual consistency across all UI backends.

**Architecture**: Tokens are defined once in Auto type definitions (or JSON), then compiled to platform-specific output by the Token Compiler.

**Token categories**: Color (semantic + palette), Spacing (4px-based scale), Border Radius, Font Size, Shadow, Animation Duration, Responsive Breakpoints.

**Platform outputs**:

| Platform | Output Format |
|----------|--------------|
| Vue/Web | CSS Variables + tailwind.config.js |
| Rust/gpui | const tokens module |
| Android/Kotlin | Material Theme (Color.kt, Theme.kt, Spacing.kt) |
| HarmonyOS/ArkTS | @Styles + static classes |
| Embedded/C (LVGL) | #define macros + helper functions |

**AI integration**: Tokens are designed to be AI-generatable. An AI agent can produce a complete `tokens.at` file given a design brief, which then compiles to all platforms.

**AURA integration**: Token references in widget styles are resolved at compile time to concrete values, enabling zero-runtime-overhead cross-platform theming.

### Frontend-Backend Communication

Auto defines a unified `#[api]` annotation system that generates platform-appropriate communication layers.

**Architecture varies by target**:

| Route | Frontend | Backend | Communication | API Layer |
|-------|----------|---------|--------------|-----------|
| a2vue | Vue/TypeScript | Rust | IPC or HTTP | Required (dual mode) |
| a2rust | AutoUI (Rust) | Rust | Direct call | Not needed |
| a2jet | Jetpack Compose | Kotlin | Direct call or HTTP | Optional |
| a2lvgl | LVGL (C) | C | Direct call | Not needed |

**a2vue dual mode**: The compiler generates three TypeScript files from `#[api]` declarations: `api-interface.ts` (types), `api-tauri.ts` (Tauri IPC for desktop), and `api-http.ts` (Axios HTTP for web). A runtime `api.ts` detects the environment and selects the correct implementation.

**API annotations** support: custom names (`name = "getUserById"`), REST mapping (`method = "GET", path = "/users/:id"`), caching (`cache = 60`), and auth requirements (`auth = true`).

### AutoDown Document Generation DSL

AutoDown is a text-dominant document generation language within the Auto ecosystem. It targets the gap between Markdown (limited expressiveness) and LaTeX/Typst (syntax alienation from modern programming).

**Three-symbol escape system**:
- `#` -- Header domain (Markdown-compatible)
- `$` -- Logic domain (Auto language takeover: `${expr}`, `$if`, `$for`, `$component { }`)
- `%{ }%` -- Math domain (replaces LaTeX's `\` syntax with function-style expressions)

**Flip preprocessing**: AutoDown is not a standalone renderer but a state-machine preprocessor. It flips between Text Mode (default) and Code Mode (on `$` or `%{`). Code blocks compile to standard AURA nodes, making AutoDown documents compatible with all AURA backends.

**Multi-backend output**: Typst (PDF), DOCX (Word via MathML), React/Vue (web documentation). The math AST translates to each backend's native format.

**AI template workflow**: Word templates with annotation placeholders are converted to AutoDown templates, filled by AI, then compiled back to DOCX with 100% format preservation.

### Vue.js Router Architecture

Auto's multi-page UI abstraction maps to Vue Router patterns. The `Outlet()` component in Auto translates to Vue's `<router-view>` or React's `<Outlet />`. Navigation menus with `route:` bindings generate `<router-link>` elements.

This abstraction enables single-page app architecture (layout + outlet pattern) across all web-facing backends without coupling Auto syntax to any specific framework's routing API.

## Open Questions

- AURA schema validation: runtime vs. compile-time enforcement of widget property types.
- Design Token Compiler implementation priority relative to core compiler features.
- AutoDown math syntax: whether `%{ }%` conflicts with existing language constructs.
- Reactive state strategy for a2c+LVGL (dirty-flag runtime vs. compile-time tracking vs. polling).

## Source Documents

- [raw/aura.md](raw/aura.md) -- AURA core architecture and widget system
- [raw/design-token-system.md](raw/design-token-system.md) -- Design token system design
- [raw/frontend-backend-communication.md](raw/frontend-backend-communication.md) -- Frontend-backend API architecture
- [raw/scenario.md](raw/scenario.md) -- Scenario-based programming and contextual parsing
- [raw/auto-down.md](raw/auto-down.md) -- AutoDown document generation DSL
- [raw/vue-router.md](raw/vue-router.md) -- Vue.js router architecture patterns
