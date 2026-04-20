# 06 - Code Generation

## Status

Code generation is substantially implemented across multiple backends:

- **C transpiler (a2c)** (`trans/c.rs`): Mature. Supports functions, structs, enums, imports via `use c <header>`, type mappings. Full test suite in `test/a2c/`.
- **Rust transpiler (a2r)** (`trans/rust.rs`): Active. Used for self-hosting compilation paths.
- **ArkTS generator (a2ark)** (`ui_gen/ark/`): Complete. 5 modules (generator, modifier, project, state, mod) with 12+ widget test cases. Maps AURA widgets to HarmonyOS ArkTS components.
- **Jetpack Compose generator (a2jet)** (`ui_gen/jet/`): Complete. 11 modules (generator, components, form, layout, list, modifier, navigation, state, project, theme, mod) with full Material3 support and Android project generation.
- **Vue generator** (`ui_gen/vue.rs`): Implemented for web target.
- **Shared UI gen** (`ui_gen/shared/`): Registry, state, style, and Tailwind utilities shared across backends.

Not yet implemented: ASTL unified syntax tree, AutoGen template engine, a2c+LVGL embedded UI backend, Token Compiler.

## Design

### C Transpiler (a2c)

The C transpiler converts Auto source to C99 code for embedded systems and native compilation.

**Type mapping**: Auto types map directly to C types:

| Auto | C |
|------|---|
| int, i32 | int32_t |
| byte, u8 | uint8_t |
| i64 | int64_t |
| f32 | float |
| f64 | double |
| bool | bool (stdbool.h) |
| str | struct { char* data; int len; } |
| cstr | char* |

**C interop** uses `use c <header>` syntax to include C headers. Functions callable from Auto use natural call syntax -- the compiler handles ABI compatibility. The `cstr` type bridges Auto's length-prefixed `str` with C's null-terminated `char*` via `.cstr()` conversion.

**Auto Binary Format (ABF)**: Compiled output includes a FragHeader (magic "AUTO", version, code_size, const_size, reloc_count), pure bytecode code section, and a relocation table for linking.

### ArkTS Generator (a2ark)

Generates ArkTS code from AURA widgets targeting HarmonyOS applications.

**Architecture**: `AuraWidget -> ArkGenerator -> ArkTS Code`, with a WidgetRegistry for component mappings and ArkModifierDsl for Tailwind-to-ArkTS style conversion.

**Component mappings**: Direct 1:1 mapping from AURA tags to ArkTS built-ins (col->Column, row->Row, box->Stack, text->Text, button->Button, input->TextInput, image->Image, checkbox->Checkbox, switch->Toggle, slider->Slider, tabs->Tabs, dialog->AlertDialog).

**Code generation rules**: Generated code uses TypeScript syntax (not Kotlin). Array literals require explicit type annotations. `Object` replaces `any` (ArkTS forbids `any`/`unknown`). Object literals must match declared interfaces.

**Testing**: Located in `test/a2ark/` with 12+ widget test cases. Each has `input.at` + `input.expected.ets`. Run via `cargo test -p auto-lang --lib -- generator::tests::test_0`.

### Jetpack Compose Generator (a2jet)

Generates Jetpack Compose Kotlin code with Material3 design for Android applications.

**Architecture**: `AuraWidget -> JetGenerator -> Kotlin/Compose Code`. Consists of 11 modules totaling 3500+ lines: generator (main orchestration), components (Material3 registry), form (Input/Checkbox/Switch/Slider), layout (Column/Row/Box/Card/Scroll), list (LazyColumn/LazyRow/Grid), modifier (Tailwind-to-Compose DSL), navigation (NavHost/routes), state (mutableStateOf conversion), project (full Android project generation), theme (Color/Theme/Spacing).

**Project generation**: Creates complete Android projects with Gradle build files, Material3 theme, manifest, and organized source structure. Supports default, custom package, and custom theme variants.

**State conversion**: AURA model declarations map to `mutableStateOf` with automatic getter/setter generation. Handler logic translates to Kotlin lambdas.

**Implementation phases**: All 7 phases complete (basic structure, form components, modifier DSL, layout/navigation, lists/data, project generation, testing/docs).

### General-Purpose Code Generator (AutoGen)

AutoGen is a template-based code generator that takes Atom-format data and Auto-script templates to produce code files.

**Architecture pipeline**: DataLoader (Atom/Auto files) -> TemplateEngine (Auto interpreter executes templates) -> GuardProcessor (preserves hand-written code) -> OutputGenerator (file writing with dry-run support).

**Key design decisions**:
- Data format is Atom only (no separate JSON/YAML parsers needed since Atom is a JSON superset).
- Templates are Auto scripts themselves, leveraging full language power via `use` for includes.
- Guard blocks use C-style delimiters (`/// begin of guard: <id>`) to protect hand-written sections during regeneration.
- Dual interface: CLI tool (`autogen`) and library API with builder pattern.

### Auto Syntax Tree Language (ASTL)

ASTL is a unified intermediate representation that enables MxN to M+N transpilation between languages.

**Core idea**: Instead of implementing separate transpilers between every pair of languages (MxN), ASTL provides a common syntax tree. Each new language only needs a Parser (code -> ASTL) and Codegen (ASTL -> code), reducing the problem to M+N.

**Format**: ASTL uses Atom format. A simplified Atom representation of code closely resembles the source language. For example, C's `int main() { printf("Hello"); return 0; }` becomes `fn main int { call printf ("Hello"); ret 0 }` in compact ASTL.

**Properties**: Defined by Auto-language Schema constraints. Supports the union of all target language ASTs. Can be viewed as an independent programming language (similar to Lisp S-expressions) enabling potential self-hosting.

### Atom Tree Builder API

A builder-pattern API for constructing Atom tree structures (Node/Array/Obj) ergonomically in Rust.

**Three tiers**:

1. **Chain methods** (implemented): `Node::new("config").with_prop("version", "1.0").with_child(Node::new("db"))` -- zero-cost abstractions on existing types.
2. **Builder pattern** (planned): `NodeBuilder::new("config").prop("version", "1.0").child_if(condition, child).build_atom()` -- supports conditional construction.
3. **Macro DSL** (future): `atom!(node("config") { database("db") { host: "localhost" } })` -- declarative construction similar to `serde_json::json!`.

## Open Questions

- ASTL Schema definition format: whether to use `@annotations` on `type` fields or implicit position-based rules.
- a2c+LVGL reactive state management: choose between minimal dirty-flag runtime, compile-time dependency tracking, or polling mode.
- AutoGen watch mode implementation (file watcher + debouncing).
- Whether the Token Compiler should be a separate crate or integrated into the main compiler.

## Source Documents

- [raw/a2c-lvgl-analysis.md](raw/a2c-lvgl-analysis.md) -- a2c + LVGL embedded UI analysis
- [raw/a2ark.md](raw/a2ark.md) -- ArkTS (HarmonyOS) generator reference
- [raw/a2jet.md](raw/a2jet.md) -- Jetpack Compose generator reference
- [raw/autogen.md](raw/autogen.md) -- AutoGen general code generator design
- [raw/astl.md](raw/astl.md) -- Auto Syntax Tree Language concept
- [raw/c.md](raw/c.md) -- C language interoperability
- [raw/atom-builder-api-design.md](raw/atom-builder-api-design.md) -- Atom tree builder API
