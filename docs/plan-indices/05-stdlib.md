# 05 - Standard Library

## Overview
AutoLang's standard library is built on an AutoLang-first architecture where all components are written in .at source files and transpiled to C or executed by the VM. The multi-platform ext mechanism (.at + .vm.at + .c.at) provides clean separation between public interfaces and platform-specific implementations.

## Plan Summary

| Plan | Title | Status | Summary |
|------|-------|--------|---------|
| 020 | Stdlib IO Expansion | ✅ | File I/O with multi-platform ext architecture: type File with open/read_text/read_line/close |
| 027 | Stdlib C Foundation | ✅ | AutoLang-first stdlib architecture with .at/.vm.at/.c.at file organization and May<T> tag type |
| 041 | List Dynamic Array | ✅ | Dynamic List type with 11 VM methods (push/pop/len/get/set/insert/remove/clear/reserve) |
| 042 | Dynamic String (dstr) | 🔧 | Byte-level dynamic string type built on List field with OOP methods |
| 043 | Generic Slice Type | ⏳ | Slice<T> with []T syntax, range operators, and immutable views into contiguous sequences |
| 051 | Auto Flow Iterators | ✅ | Zero-cost iterator and functional programming system with map/filter/reduce and auto-forwarding |
| 052 | Storage-Based List | ✅ | List<T, S: Storage> with pluggable strategies (Heap, InlineInt64) and monomorphization |
| 053 | Mutable Variables in Loops | ✅ | Fixed mut variable compilation errors in while loops enabling iteration and accumulator patterns |
| 054 | Context Environment | ⏳ | Unified runtime environment injection system with compile-time/prelude/startup phases |
| 102 | HTTP Server Stdlib | ✅ | HTTP server built on Task/Msg async framework (Plan 069) with net/json/url modules |
| 119 | a2rs Backend Stdlib | 🔧 | Backend stdlib for HTTP, Redis, SQLite enabling server-side AutoLang applications |
| 143 | Stdlib Widget Library | ⏳ | Migration of ~45 components from component-gallery into stdlib/aura/widgets (7 categories) |
| 160 | Map Type | ✅ | Built-in Map<K,V> type as typed version of Object, with a2r HashMap generation support |

## Status Summary
- Completed: 8 | Partial: 2 | Planned: 3 | Deprecated: 0

## Key Achievements
- Multi-platform ext architecture enabling .at (interface) / .vm.at (VM impl) / .c.at (C impl) separation
- List<T> type with full VM support and transpiler mappings to C, Rust, Python, and JavaScript
- Zero-cost iterator system (Auto Flow) with map/filter/reduce and auto-forwarding from containers
- Built-in Map<K,V> type enabling typed key-value collections with Rust HashMap transpilation

## Remaining Work
- Complete dstr type with advanced features (from_str, to_str, iteration support)
- Implement generic Slice<T> with range operators and borrow checking
- Build out backend stdlib (Redis, SQLite) for server-side AutoLang
- Migrate widget library from component-gallery into stdlib/aura/widgets
