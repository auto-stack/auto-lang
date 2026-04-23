# 05 - Standard Library

## Overview

AutoLang's standard library is built on an AutoLang-first architecture where all components are written in `.at` source files and either executed by the VM or transpiled to C and Rust. The central architectural innovation is the multi-platform `ext` mechanism, which splits each module into three files: `.at` for the public interface, `.vm.at` for VM implementations backed by Rust, and `.c.at` for C transpilation. This separation lets the compiler load the correct implementation based on the compilation target, while presenting a uniform API to users. Over 13 plans have driven the standard library from basic File I/O to a full iterator system with closures, a pluggable-storage List, built-in Map type, and an HTTP server stdlib -- with further work ongoing for dynamic strings, backend services, and a widget library.

## Plan Index

| Plan | Title | Status | Summary |
|------|-------|--------|---------|
| 020 | Stdlib IO Expansion | Done | File I/O with multi-platform ext architecture: type File with open/read_text/read_line/close |
| 027 | Stdlib C Foundation | Done | AutoLang-first stdlib architecture with .at/.vm.at/.c.at file organization and May<T> tag type |
| 041 | List Dynamic Array | Done | Dynamic List type with 11 VM methods (push/pop/len/get/set/insert/remove/clear/reserve) |
| 042 | Dynamic String (dstr) | Partial | Byte-level dynamic string type built on List field with OOP methods |
| 043 | Generic Slice Type | Planned | Slice<T> with []T syntax, range operators, and immutable views into contiguous sequences |
| 051 | Auto Flow Iterators | Done | Zero-cost iterator and functional programming system with map/filter/reduce and auto-forwarding |
| 052 | Storage-Based List | Done | List<T, S: Storage> with pluggable strategies (Heap, InlineInt64) and monomorphization |
| 053 | Mutable Variables in Loops | Done | Fixed mut variable compilation errors in while loops enabling iteration and accumulator patterns |
| 054 | Context Environment | Partial | Unified runtime environment injection system with compile-time/prelude/startup phases |
| 102 | HTTP Server Stdlib | Done | HTTP server built on Task/Msg async framework with net/json/url modules and a2vue dual-mode API generation |
| 119 | a2rs Backend Stdlib | Partial | Backend stdlib for HTTP, Redis, SQLite enabling server-side AutoLang applications |
| 143 | Stdlib Widget Library | Planned | Migration of ~45 components from component-gallery into stdlib/aura/widgets (7 categories) |
| 160 | Map Type | Done | Built-in Map<K,V> type as typed version of Object, with a2r HashMap generation support |
| 191 | Assert Builtins | Planned | Add assert/assert_eq/assert_ne as native intrinsics |
| 195 | HTTP Client + auto.http Unification | Planned | Upgrade to reqwest, unify http_stream, add async HTTP support |

## Status

**Implemented**: Plan 020, 027, 041, 051, 052, 053, 102, 160 (8 plans fully complete)

**Partial**: Plan 042 (dstr has core byte-level API but lacks from_str/to_str/iteration), Plan 054 (core infrastructure complete -- storage types, target detection, environment injection -- but config block syntax and startup code generation are not started), Plan 119 (HTTP server/client FFI done for VM; Redis and SQLite are design-only, no Rust implementations yet)

**Planned**: Plan 043 (Slice<T> with range operators and borrow semantics), Plan 143 (widget library migration from component-gallery), Plan 191 (assert builtins), Plan 195 (HTTP client + auto.http unification)

## Design

### Multi-Platform ext Architecture

The standard library's foundational architecture, established in Plans 020 and 027, replaces the earlier approach of hand-written C code with an AutoLang-first model. Each module is split into three files: the public interface in `.at`, the VM implementation in `.vm.at`, and the C implementation in `.c.at`. The compiler loads files in order based on the compilation target: `.at` followed by `.vm.at` for VM evaluation, or `.at` followed by `.c.at` for C transpilation. Methods annotated `#[vm]` are implemented in Rust within the `crates/auto-lang/src/vm/` directory and registered in the VM function registry. This pattern is used consistently across I/O, system functions, collections, and networking modules.

The file-loading order is critical. The parser in `crates/auto-lang/src/parser.rs` returns different extension lists depending on `CompileDest`: for `Interp` it loads `.at` then `.vm.at`, for `TransC` it loads `.at` then `.c.at`, and for `TransRust` it loads `.at` then `.rust.at`. Interface declarations must be loaded before implementations so that types and method signatures are available when the ext block is processed.

A key design principle is that AutoLang source code uses clean, unprefixed names -- `File.open()` rather than `File_open()` -- and the transpilers add platform-specific prefixes only in generated output. All stdlib types use OOP-style methods defined inside `type` blocks, not module-prefixed free functions.

### Collections: List, Map, and the Storage Strategy Pattern

AutoLang's collection types form a coherent hierarchy informed by the language's dual focus on PC and embedded (MCU) targets.

The **List** type (Plan 041) began as a simple dynamic array with 11 VM methods -- `push`, `pop`, `len`, `is_empty`, `clear`, `get`, `set`, `insert`, `remove`, `reserve`, and the static constructor `new`. It is stored as `ListData` inside `VmRefData`, mirroring the pattern used for HashMap and StringBuilder. Transpilers map List to idiomatic types in each target: `Vec<T>` in Rust, `list` in Python, `Array` in JavaScript, and a custom `list_T*` in C.

Plan 052 deepened the List design with a **storage strategy pattern** inspired by Rust's `Vec<T, Alloc>` and C++'s `std::vector<T, Allocator>`. Rather than locking List to heap allocation -- which would be unusable on MCUs -- the type became `List<T, S: Storage>` where `S` provides raw memory access through three methods: `data()` returns a raw pointer, `capacity()` returns the physical capacity, and `try_grow()` attempts to expand. Two concrete storage strategies were implemented: `Heap<T>` for PC platforms using `malloc`/`realloc`, and `InlineInt64` for MCU platforms with a fixed 64-element stack buffer. The C transpiler monomorphizes `List<T, S>` into specialized structs like `List_int_Heap` with a pointer field, or `List_int_Inline_4` with an embedded array of size 4. A generic `Storage<T>` spec was defined with full vtable generation. The design follows the principle that List logic is storage-agnostic: it calls `S.data()`, `S.capacity()`, and `S.try_grow()` without knowing where memory lives.

The **Map<K,V>** type (Plan 160) was added as a language built-in, the typed counterpart to Object. Like List's relationship to Array, Map provides type annotations for Object literals: `{key: value}` with a `Map<str, int>` annotation gives the compiler enough information to generate `HashMap<String, i32>` in Rust. The `Type::Map(Box<Type>, Box<Type>)` variant was added to the AST, touching 14 production code files and 2 test files. In the VM, Map reuses the existing `Obj` runtime; in a2r it maps to `std::collections::HashMap`; in a2c to `map_K_V*`; in TypeScript to `Record<K,V>`; and in Python to `dict`. Tests 128 and 129 verify struct-field and function-parameter transpilation.

### Iterator System and Functional Programming

Plan 051 delivered a comprehensive zero-cost iterator system called **Auto Flow**. The design centers on two core specs: `Iter<T>` with its `next()` method and lazy adapters (`map`, `filter`, `take`, `skip`, `enumerate`, `zip`, `chain`), and `Iterable<T>` with auto-forwarding default implementations that let users write `list.map(f)` instead of `list.iter().map(f)`. The compiler inlines the forwarding layer entirely, producing the same code as an explicit `.iter()` call.

The implementation proceeded in eight phases across 41+ hours. Phase 1 defined the core specs in `stdlib/auto/iter/spec.at`. Phase 2 implemented `MapIter` and `FilterIter` adapter types with generic type fields. Phase 3 integrated iteration into `List<T,S>` with a `ListIter` type. Phases 4 through 8 added terminal operators (`reduce`, `count`, `for_each`, `collect`), the postfix bang operator `!` for eager collection, extended adapters (`limit`, `skip`, `enumerate`, `zip`, `chain`), predicate operators (`any`, `all`, `find` with short-circuit evaluation), and the `Collect<T>` spec. The system depends on Plans 057 (generic specs), 060 (closure syntax), and 061 (generic constraints), all of which were completed as prerequisites.

### String Types: dstr and the Planned Slice

Plan 042 addresses dynamic string manipulation with a `dstr` type that wraps a `List` field and provides byte-level operations: `push`, `pop`, `get`, `set`, `insert`, `remove`, `clear`, `reserve`, `len`, and `is_empty`. The key insight from the design process is that VM types like List use reference semantics, so calling multiple methods on the same `dstr` instance does not cause ownership issues -- unlike function calls which move parameters. The type is defined entirely in AutoLang with no `#[vm]` or `#[c]` annotations. The remaining work includes `from_str`/`to_str` conversions, string concatenation, splitting, and integration with `for` loops for iteration.

Plan 043 envisions a generic `Slice<T>` type that provides immutable views into contiguous sequences, with `[]T` syntax sugar and range operators (`[start..end]`, `[start..=end]`, `[start..]`, `[..end]`). The design covers slicing for `str`, `dstr`, and static arrays `[N]T`. Key open questions remain around lifetime tracking in the absence of a borrow checker, and whether a `MutSlice<T>` variant will be needed.

### Error Handling and the May<T> Tag Type

Plan 027 introduced `May<T>` as a unified three-state type using the `tag` syntax: `nil Nil`, `err Err`, and `val T`. This provides `is_some()`, `unwrap()`, and constructor methods like `May.empty()`, `May.value(v)`, and `May.error(e)`. The tag-based implementation supports pattern matching with `is` statements and return type inference for pattern matching branches. The design was validated with 34 passing tests. Future extensions include `?T` syntactic sugar, `.?` and `??` operators, and full generics support.

### Mutable Variables and Loop Scoping

Plan 053 fixed a subtle but important bug where mutable variables declared before a `for`/`while` loop could not be reassigned inside the loop body. The root cause was in the C transpiler: conditional for-loops (Auto's `for condition { ... }` which acts as a while loop) were generating `for (condition) { ... }` in C, which is invalid syntax. The fix changed the `Iter::Cond` branch to emit `while (condition) { ... }` instead. Four tests were added covering counter increment, sum accumulation, multiple mutables, and array indexing patterns.

### Environment Injection and Platform Adaptation

Plan 054 lays out a "lifecycle context management" system with three phases: injection (compiler builds a virtual `std.env` module from CLI flags), prelude (a dynamic `std.prelude.at` that exports platform-adapted types), and startup (generated bootstrap code that handles heap initialization on MCUs). The core infrastructure is implemented: storage type system (Fixed vs Dynamic), target detection (Mcu/Pc), environment variable injection (`TARGET`, `DEFAULT_STORAGE`, `HAS_HEAP`), and `List.capacity()` returning target-dependent values. The `--target` CLI flag allows explicit override. The remaining work -- compile-time `if/else` in prelude, `[config]` block parsing for user-defined heap sizes, and startup code generation templates -- is substantial but builds on the completed foundation.

### HTTP and Network Stdlib

Plan 102 delivered a complete HTTP stdlib stack built on the Task/Msg async framework from Plan 069. The implementation covers seven modules: `async` (wrapping SPAWN/SEND/RECV opcodes), `log` (debug/info/warn/error levels), `env` (environment variables and CLI args), `net` (TCP listeners and streams using `std::net` with thread-local handle registries), `json` (encoding/decoding via serde_json with 18 FFI functions), `url` (encoding/decoding with 16 FFI functions), and `http` (server with route registration and client with blocking GET/POST). Native FFI IDs are assigned in the 1000-2299 range across all modules. Phase 5 added an API annotation system with `#[api]` parsing, a `TargetGenerator` trait, and code generators for TypeScript (type definitions and dual IPC/HTTP client), Tauri (`#[tauri::command]` functions), and Axum (route handlers), with 24 passing tests.

Plan 119 extends this vision with a backend stdlib for a2rs (the Rust transpiler backend), targeting HTTP server/client, Redis, and SQLite. The architecture places user AutoLang code at the top, transpiled Rust code in the middle, and Rust crate implementations at the bottom. The HTTP server API uses axum, the Redis client wraps the `redis` crate, and SQLite wraps `rusqlite`. Comprehensive API designs exist for all three services, but only the VM-side HTTP FFI from Plan 102 is implemented. The Redis and SQLite implementations depend on Plan 120 (error types, now complete) and Plan 121 (async system, planned).

### AURA Widget Library

Plan 143 defines the migration of approximately 45 UI components from `examples/component-gallery/` into `stdlib/aura/widgets/` organized into seven categories: display (Text, Image, Badge, Avatar, Separator, Skeleton), form (Button, Input, Checkbox, Switch, Select, Slider, RadioGroup, Textarea, Form), layout (Card, ScrollArea, AspectRatio, Collapsible, Accordion), overlay (Dialog, AlertDialog, Sheet, Drawer, Popover, Tooltip, HoverCard, ContextMenu), navigation (Tabs, Breadcrumb, NavigationMenu, Pagination, Sidebar, MenuBar, DropdownMenu, NavLink), feedback (Alert, Toast, Progress, Sonner), and data (Table, DataTable, Calendar). Standardization rules enforce consistent prop naming (`text` with `#[primary]`, `variant`, `size`, `disabled`), event naming (`onclick`, `onchange`, `onsubmit`), and annotation requirements (`#[spec]`, `#[backend]`). Compound component patterns keep sub-widgets in the same file. The migration process spans 66 tasks across 10 phases.

## Open Questions

- **Slice lifetimes**: How to ensure the underlying storage outlives a Slice<T> in the absence of a borrow checker? The current proposal favors runtime reference counting via `VmRef` or documentation as unsafe.
- **dstr UTF-8 handling**: The dstr type operates at the byte level. Future `from_str`/`to_str` conversions need a policy for invalid UTF-8 -- either returning `May<str>` or panicking with a diagnostic.
- **Inline<T,N> generic syntax**: The parser does not yet support `const N u32` in generic type parameters, blocking the fully generic `Inline<T, N>` storage type. Concrete types like `InlineInt64` serve as a workaround.
- **HTTP server callbacks**: The VM HTTP server uses a placeholder route system because VM callback support for handler functions is not yet implemented. This blocks dynamic route matching.
- **Environment-aware collection**: The `!` bang operator currently defaults to Heap storage for all targets. MCU-aware storage selection during collection is deferred.
- **a2rs async model**: Plan 119's backend stdlib needs an async strategy for a2rs -- currently recommended to start with blocking APIs and add tokio-based async after Plan 121.

## Source Plans

- Plan 020: Stdlib IO Expansion
- Plan 027: Stdlib C Foundation
- Plan 041: List Dynamic Array
- Plan 042: Dynamic String (dstr)
- Plan 043: Generic Slice Type
- Plan 051: Auto Flow Iterators
- Plan 052: Storage-Based List
- Plan 053: Mutable Variables in Loops
- Plan 054: Context Environment
- Plan 102: HTTP Server Stdlib
- Plan 119: a2rs Backend Stdlib
- Plan 143: Stdlib Widget Library
- Plan 160: Map Type
- [191-assert-and-precise-linker-errors.md](../plans/191-assert-and-precise-linker-errors.md)
- [195-http-client-async-unification.md](../plans/195-http-client-async-unification.md)
