# 14 - Language Features and Syntax

## Overview

AutoLang's language features span a wide range of syntax enhancements, type system extensions, and developer ergonomics improvements. The twenty-four plans in this category cover everything from the foundational `ext` statement for OOP-style method extension, through type system unification (enum migration, string tiers, type conversion), to build infrastructure (caching, unified type context) and tooling (debug overlays, VSCode integration). Eleven plans are fully implemented, two are partially complete, one is deprecated, and ten remain in planning or early implementation stages. Collectively, these plans aim to make AutoLang more expressive, safer, and more consistent with its own design philosophy of being a self-hosting language with a clean, minimal syntax.

## Plan Index

| Plan | Title | Status | Summary |
|------|-------|--------|---------|
| 035 | ext Statement | Done | OOP-style method extension for existing types |
| 036 | Unified Auto Section | Done | Multi-file stdlib loading with .at/.vm.at/.c.at split |
| 040 | Tag Types Summary | Partial | Tag variant construction and matching (4/5 tasks done) |
| 044 | ext Enhanced Multiplatform Architecture | Planned | Interface contract + physical completion for platform-specific stdlib |
| 045 | AutoLang-AutoUI Integration | Planned | EvalMode::Config for parsing UI scripts |
| 050 | Auto Prelude System | Planned | Auto-importing common symbols into every module |
| 072 | Logical Operators and/or | Deprecated | Reverted to && and || symbols |
| 082 | AutoCache -- Global Build Cache | Planned | Content-addressable store for compilation artifacts |
| 084 | Unified Type Context | Done | Consolidated TypeStore for type/function/spec/generic info |
| 086 | Widget Registry from Stdlib | Partial | Load widget specs from .at files instead of hardcoded Rust |
| 139 | Atom Serialization System | Planned | Serde-like Auto-Atom serialization with compile-time codegen |
| 155 | String Type Refactoring | Planned | Three-tier string system: StrLit, StrSlice, String |
| 156 | Unified Enum Migration | Done | Merged enum and tag into single keyword with three forms |
| 168 | shared Variable + pub Keyword Migration | Done | shared as static storage, pub as keyword prefix |
| 169 | Multi-Line Strings | Done | Triple-quote strings for embedded newlines |
| 182 | Debug Mode for Rust Desktop UI | Planned | Chrome DevTools-inspired debug overlay |
| 185 | VSCode Extension Reuses Vue Build | Planned | Eliminate duplicate webview build |
| 190 | Extend use.rust for Rust Stdlib Access | Planned | Import any Rust stdlib type via use.rust |
| 193 | Conv Type Conversion System | Draft | Unified .to() method with Conv spec for type-safe conversions |
| 194 | Monomorphic Dispatch for Generic Methods | Complete | Compile-time type-based dispatch for HashMap/HashSet generic APIs |
| 197 | VM Enum/Data, Generic Lists, Pattern Debug | Complete | All 5 phases: string eq, method chaining, struct debug, enum data, List<UserType>, Option<T> |
| 206 | Closure HOF + call_closure API | Complete | call_closure public API, List.map/filter/reduce/find/for_each shims |
| 207 | Enum Multi-Field Destructuring | Complete | Multi-binding destructuring and named arg construction for enum variants |
| 208 | Result Heap Object | Complete | CREATE_OK/CREATE_ERR heap objects, IS_OK, UNWRAP_OK/ERR, ERROR_PROPAGATE |

## Status

**Implemented**: Plans 035, 036, 084, 156, 168, 169, 194, 197, 206, 207, 208 (eleven plans fully complete).

**Partial**: Plans 040 (tag method parsing deferred, ext workaround works), 086 (parser/AST done, WidgetLoader not yet built).

**Planned**: Plans 044, 045, 050, 082, 139, 155, 182, 185, 190, 193.

**Deprecated**: Plan 072 (and/or keywords reverted to && and || to avoid conflicts with bitwise method names).

## Design

### Type Extension and the ext Statement

The `ext` statement, completed in Plan 035, is one of the most impactful language features in AutoLang. It allows developers to add methods to any type after its initial definition, including built-in types like `str`, `int`, and `bool`. The syntax mirrors Rust's `impl` blocks but uses the keyword `ext` to distinguish itself, and it integrates with AutoLang's implicit `self` convention: methods inside `ext` blocks can use `.prop` shorthand for `self.prop`.

The implementation added `Stmt::Ext` to the AST, extended the lexer with `TokenKind::Ext` and `TokenKind::Static`, integrated the parser to recognize `ext Type { methods }` blocks, and wired the evaluator to register methods on both user-defined types (via TypeDecl) and built-in types (via TypeInfoStore). Static methods are distinguished by the `static fn` prefix. The entire feature was delivered in two days with 548 tests passing.

Plan 044 builds on the `ext` mechanism to enable multiplatform architecture. The core idea is an "interface contract + physical completion" pattern: a base `.at` file defines the public interface of a type, while platform-specific `.c.at` and `.vm.at` files use `ext` blocks to add private fields and provide implementations. For example, `io.at` defines `type File { path str }`, while `io.c.at` uses `ext File { _fp *FILE }` to add a C-specific file handle, and `io.vm.at` uses `ext File { _handle uint64 }` for a VM handle. The C transpiler then generates a merged struct with both public and private fields. Phase 1 (AST and parser support for ext fields) is complete, but transpiler integration and stdlib migration remain.

### Standard Library File Organization

Plan 036 redesigned how the standard library is structured by splitting single files into multi-file modules with platform-specific suffixes: `.at` for pure Auto code (loaded in all contexts), `.vm.at` for VM-specific code (loaded only by the interpreter), and `.c.at` for C-specific code (loaded only by the transpiler). The loading order follows a layered architecture principle -- platform-specific files load first, then the shared `.at` file on top.

The implementation introduced a `get_file_extensions()` method that returns the appropriate file suffixes based on `CompileDest`, and a file-merging strategy that concatenates contents (stripping section markers) before parsing. This ensures the merged result behaves identically to the original single file. The `io.at` module was split into three files, `sys.at` was split into two, and `math.at` and `str.at` were confirmed as pure Auto code requiring no split. Several Auto-implemented methods were added, including `str.split()`, `str.lines()`, `str.words()`, and `File.read_all()`. The plan was completed with 556 tests passing.

### Unified Enum System

Plan 156 is a significant language unification that merged the separate `enum` (scalar) and `tag` (algebraic data type) keywords into a single `enum` keyword supporting three physical forms. `EnumKind::Scalar` handles traditional integer-backed enumerations like `enum Color { Red, Green, Blue }` and typed enumerations like `enum HttpCode u16 { OK = 200 }`. `EnumKind::Homogeneous` handles enumerations where all variants share the same payload type, such as `enum Vertex Point { LeftTop, RightTop }`. `EnumKind::Heterogeneous` replaces the old `tag` keyword, supporting variants with different payload types like `enum Msg { Quit, Move Point, Write string }`.

The implementation was carried out in five phases. Phase 1 extended the `EnumDecl` struct with `EnumKind` and refactored `EnumItem` to carry optional `scalar_value` and `payload_type` fields. Phase 2 rewrote `enum_stmt()` to auto-detect the form based on the token following the enum name: `{` triggers body parsing with deferred classification, a type name triggers Homogeneous, and an integer type name triggers Scalar with repr type. The `tag` keyword was deprecated by redirecting it to `enum_stmt()`. Phases 3 through 5 adapted all transpilers (C, Rust, TypeScript), updated pattern matching with `Cover::Enum`, migrated all existing tag tests to enum syntax, and cleaned up dead `tag_stmt()` code.

### String Type System

Plan 155 introduces a three-tier string type system that cleanly separates string literals (`StrLit`), borrowed string slices (`str` / `StrSlice`), and owned dynamic strings (`String`). The design deliberately avoids explicit reference syntax -- AutoLang has no `&` operator. When users write `str`, they get a borrowed slice (equivalent to Rust's `&str`). The `String` keyword produces an owned, growable dynamic string. String literals `"hello"` have compile-time known lengths tracked via `StrLit(usize)`.

Phase 2 of the implementation is complete: `Type::String` was added to the AST type system and the auto-val value system, the parser maps `"String"` to `Type::String`, type inference handles cross-type unification (String coerces to StrSlice implicitly), the VM codegen was updated for all string types, and all transpilers (C, Rust, TypeScript, Python, ArkTS, Jet/Kotlin, and UI generators) were adapted. The cosmetic Phase 1 renames (changing `Type::Str(usize)` to `Type::StrLit(usize)` across 300+ sites) were deferred as non-functional.

### Type Conversion with Conv

Plan 193 designs a unified type conversion system based on a `Conv<From, To>` spec and `.to(TargetType)` syntax sugar. The goal is to replace ad-hoc conversion functions with a single, discoverable interface: `42.to(String)` produces `"42"`, `"123".to(int)` produces `123`, and `"abc".try_to(int)` returns `None`. The system also includes `TryConv<From, To>` for fallible conversions.

The design specifies a search order when the compiler encounters `expr.to(TargetType)`: first check the `auto.conv` stdlib module, then user imports, then the current scope. A Phase 2 feature adds escape analysis for `.to(str)` conversions, where a temporary owned String must be created but the resulting slice must not outlive its scope -- returning `n.to(str)` from a function would be a compile error, while `print(42.to(str))` is fine. The plan is still in draft, awaiting spec constraint resolution and parser support for the `.to(Type)` syntax.

### Visibility and Storage Keywords

Plan 168 delivered two related changes. The `shared` keyword was introduced as a static storage modifier, transpiling to Rust's `static` with `Lazy<Mutex<T>>` wrapping. For example, `shared var COUNTER int = 0` becomes `static COUNTER: Lazy<Mutex<i32>> = Lazy::new(|| Mutex::new(0))`. The `pub` keyword was migrated from an annotation (`#[pub]`) to a prefix keyword matching Rust's convention. The parser's `parse_stmt()` now intercepts `pub` and dispatches to the appropriate declaration parser (`pub fn`, `pub type`, `pub enum`, `pub spec`, `pub use`). The old `#[pub]` annotation was silently deprecated rather than removed for backward compatibility. All 25+ stdlib files were updated from `#[pub]` to `pub` prefix syntax.

### Multi-Line Strings

Plan 169 added triple-quote string syntax (`"""..."""`) for embedded newlines, useful for prompts, templates, and code generation. The implementation is entirely in the lexer: a `multi_str()` method reads `"""..."""` preserving literal newlines. When encountering runs of quote characters, the lexer counts them and only the last three close the string, so `""""` produces one `"` of content plus closing `"""`. A shared `escape_str()` helper was added to the transpiler base module for consistent string emission across all transpiler backends.

### Prelude and Auto-Import System

Plan 050 designs a Rust-inspired prelude system that automatically imports common symbols (May, List, print, say, core types) into every module, eliminating repetitive `use` statements. The prelude is defined in `stdlib/auto/prelude.at` as a set of re-exports. The compiler injects it at the start of every module parse, with an optional `#[no_prelude]` attribute for opt-out. A key sub-feature is auto-wrapping for `?T` return types: functions declared with `?int` returns would automatically wrap bare integer expressions in `May.val()`. The prelude file itself was created by Plan 055, but the parser integration, scope merging, and auto-wrapping mechanism are not yet implemented.

### Unified Type Context

Plan 084 successfully consolidated scattered type information into a single `TypeStore` shared across the compiler pipeline. Previously, type declarations were duplicated between Parser and Codegen via separate `InferenceContext` instances, with synchronization through complex `Rc<RefCell<TypeRegistry>>` wrappers. The new `TypeStore` centralizes type declarations, function declarations, spec declarations, and generic templates in a single structure accessed via `Arc<TypeStore>`. All four phases are complete: TypeStore creation, Parser integration, Codegen integration, and InferenceContext integration. Unified lookup APIs were added so that type queries follow a consistent priority order: TypeStore first, then InferenceContext, then Universe as fallback.

### Debug Tooling

Plan 182 designs a Chrome DevTools-inspired debug overlay for Rust desktop UI frameworks (GPUI, iced). The `DebugLayer` sits between the VTree and the backend renderer, providing hover highlights, bounding-box visualization, live property editing, and source-file round-tripping. Three phases are complete: foundation (DebugLayer struct, LayoutReporter trait, hit-test module), selection and panel (inspector, overlay generation, DebugPanel), and box model + source map (EdgeInsets, BoxModel, SourceMap with .at file byte-range mapping). The editing phase (inline property editing with pending edits, two-phase preview/commit model, and `.at` file round-tripping) and widget tree manipulation phase are not yet implemented. The architecture is designed to be backend-agnostic and mode-agnostic, with a `DebugEditSink` trait that supports both transpiled mode (write to .at files) and future VM scripting mode (patch VM state directly).

### Rust Stdlib Interop

Plan 190 extends the `use.rust` mechanism to allow importing any Rust stdlib or third-party type with compile-time type awareness. The design adds a `Type::Rust(RustSource)` variant to distinguish known Rust imports from truly unknown types, fixes two blocking bugs (built-in crates like `std` requiring unnecessary `dep` declarations, and `std = "*"` being written to Cargo.toml), registers imported types in TypeStore, and propagates Rust provenance through generic instances so that `HashMap<str, int>` correctly emits `use std::collections::HashMap` in generated Rust code. The a2c transpiler produces clear errors when encountering Rust types, while a2r passes them through. Ten test sets covering collections, filesystem, sync primitives, time, paths, threading, serde_json, and regex are planned.

### Build Infrastructure

Plan 082 proposes a content-addressable global build cache (AutoCache) for cross-project compilation artifact reuse, though detailed design is not yet available. Plan 185 targets the VSCode extension generator, eliminating the duplicate Vue webview project by reusing `gen/vue/dist/` output instead of maintaining a separate `webview-ui/` directory, which would halve frontend build times for VSCode targets.

### Widget and UI Infrastructure

Plan 086 proposes loading widget specifications from `stdlib/aura/widgets/*.at` files instead of hardcoded Rust defaults, with `#[primary]` annotation support for shorthand property syntax. Plan 045 designs an EvalMode::Config for parsing UI configuration scripts with both runtime and transpilation paths. Plan 139 designs an Atom serialization system (Serde-like) with compile-time code generation for `to_atom()` and `from_atom()` methods, dual Compact/Pretty output formats, and an AtomReader with recursive descent parsing. The Atom system's Phase 1 (writer traits, CompactWriter, PrettyWriter) and Phase 2 (AtomReader, AtomSerializable/AtomDeserializable traits) are fully specified but not yet implemented.

## Open Questions

- The Conv type conversion system (Plan 193) is still in draft, pending decisions on spec constraint resolution and whether `Conv` implementations can be verified at compile time.
- Plan 155's cosmetic Phase 1 renames (Str to StrLit across 300+ sites) are deferred indefinitely; the functional `Type::String` addition is complete.
- Plan 050's auto-wrapping for `?T` return types requires type system integration that depends on the May generic type system being fully stable.
- Plan 182's editing phase requires AURA extraction to preserve AST byte offsets, which is a cross-cutting change not yet scoped.
- Plan 190's `use.rust` wildcard imports cannot be resolved without rustdoc JSON integration, which is explicitly out of scope.

## Source Plans

- Plan 035: ext Statement
- Plan 036: Unified Auto Section
- Plan 040: Tag Types Summary
- Plan 044: ext Enhanced Multiplatform Architecture
- Plan 045: AutoLang-AutoUI Integration
- Plan 050: Auto Prelude System
- Plan 072: Logical Operators and/or (Deprecated)
- Plan 082: AutoCache -- Global Build Cache
- Plan 084: Unified Type Context
- Plan 086: Widget Registry from Stdlib
- Plan 139: Atom Serialization System
- Plan 155: String Type Refactoring
- Plan 156: Unified Enum Migration
- Plan 168: shared Variable + pub Keyword Migration
- Plan 169: Multi-Line Strings
- Plan 182: Debug Mode for Rust Desktop UI
- Plan 185: VSCode Extension Reuses Vue Build
- Plan 190: Extend use.rust for Rust Stdlib Access
- Plan 193: Conv Type Conversion System
- [194-monomorphic-dispatch.md](../plans/194-monomorphic-dispatch.md)
- [197-vm-adt-generic-lists-pattern-debug.md](../plans/197-vm-adt-generic-lists-pattern-debug.md)
- [206-closure-hof-call-closure-api.md](../plans/206-closure-hof-call-closure-api.md)
- [207-enum-multi-field-destruct-construction.md](../plans/207-enum-multi-field-destruct-construction.md)
- [208-result-heap-object.md](../plans/208-result-heap-object.md)
