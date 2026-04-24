# 10 - Build Tooling and Infrastructure

## Overview
AutoLang's build infrastructure evolved from a file-based full-compilation architecture to the AIE (Auto Incremental Engine) query-based incremental compilation system, achieving sub-second hot reload for development. Alongside the compiler core, the tooling stack grew to include a unified CLI, Rust FFI sandbox, Tauri IPC integration, and package management with bun support.

## Plan Summary

| Plan | Title | Status | Summary |
|------|-------|--------|---------|
| 063 | AIE Architecture Migration | 🔧 | Migrate from file-based to query-based incremental compilation; phases 1-3.6 complete, MCU and later phases deferred |
| 064 | Split Universe into Compile-time and Runtime | ✅ | Separate Universe into AIE Database (compile-time) and ExecutionEngine (runtime); hybrid architecture accepted |
| 065 | AIE Integration with lib.rs Entry Points | ✅ | Wire AIE session management, QueryEngine, and Database into public lib.rs API |
| 066 | Incremental Transpilation Database Migration | ✅ | Migrate C/Rust transpilers to use AIE caching; 2.67x C speedup, 1.86x Rust speedup |
| 092 | Rust FFI via Sandbox Compilation | ✅ | Enable AutoVM to dynamically load Rust crates at runtime via sandboxed ABI-stable compilation |
| 093 | AutoMan Redesign for Rust Support | ⏳ | Extend auto-man to support Auto/Rust mixed projects alongside existing Auto/C projects |
| 111 | Auto CLI Refactor and Unification | ⏳ | Merge auto-man CLI into universal auto CLI with standardized command structure |
| 112 | AutoMan B.P.B.E Architecture Refactor | ⏳ | Redesign AutoMan around Backend, Port, Builder, Export architecture for multi-target builds |
| 146 | AutoShell SmartCmd Integration | ⏳ | Integrate nushell/uutils libraries for structured shell command output and cross-platform support |
| 151 | Tauri IPC Mode for api-example | ⏳ | Generate complete Tauri IPC backend by transpiling api.at + db.at to Rust |
| 186 | Switch from npm to bun for Vue/Web Projects | ⏳ | Replace npm with bun for faster installs via global cache hard-linking |
| 202 | Auto Playground | 🔧 | Web-based code editor + VM execution + transpilation viewer (Vue 3 + axum backend) |
| 212a | LSP + VSCode Extension Modernization | ✅ | TextMate grammar rewrite, LSP completion sync, Document Symbols, code snippets |
| 212b | Rust FFI E2E Dynamic Loading | ✅ | dep serde_json -> cargo build cdylib -> AutoVM load .dll -> call |
| 214 | Python FFI (use.py) | ✅ | Embed Python interpreter in AutoVM |
| 216 | C FFI Build Pipeline Integration | ✅ | CLI integration for C FFI bindgen into build pipeline (Phase 4 of Plan 216) |
| 219 | Playground Source Map | ✅ | Source mapping for transpiler output to enable clickable error locations |

## Status Summary
- Completed: 5 | Partial: 1 | Planned: 8 | Deprecated: 0

## Key Achievements
- AIE incremental compilation architecture delivered with file hashing, dirty tracking, and transpiler caching achieving 2-3x speedups
- Complete Universe split into compile-time Database and runtime ExecutionEngine, enabling cleaner architecture and LSP readiness
- Rust FFI sandbox enables dynamic crate loading at runtime with ABI stability through controlled compilation environments

## Remaining Work
- Auto CLI unification merging auto-man capabilities into the universal auto binary
- AutoMan B.P.B.E architecture refactor for Backend/Port/Builder/Export multi-target model
- AutoMan Rust support (a2rs) as a first-class project type alongside C targets
- Plan 212b: Rust FFI dynamic loading end-to-end (compile_dep -> cdylib -> VM load -> call)
- Plan 214: Python FFI via embedded interpreter (blocked on Plan 212b)
