# 08 - Async & Concurrency

## Overview
AutoLang implements an Actor-based concurrency model using Task/Msg primitives, an async Future/Await system with `~T` syntax, and a scheduler-driven message dispatch loop. The system provides micro-concurrency via `.go` suffix operators that dispatch async work to background worker pools, backed by Tokio's work-stealing runtime.

## Plan Summary

| Plan | Title | Status | Summary |
|------|-------|--------|---------|
| 121 | AutoLang Task/Msg Foundation | ✅ | Lexer/AST/Parser for task definitions, spawn/send FFI, TaskRegistry |
| 122 | Value Access and Assignment Refactoring | ✅ | `.move` accessor replacing deprecated `.take` with deprecation warnings |
| 123 | TypeStore Unification | ✅ | Consolidate all type registries into TypeStore as single source of truth |
| 124 | Async Future/Await System | ✅ | `~T` syntax, `.await`, CREATE_FUTURE/AWAIT_FUTURE opcodes, backpressure |
| 125 | Phase 3 - Polymorphic Routing | ✅ | Implicit union types, MessageContext runtime, pattern matcher integration |
| 126 | Phase 4 - Micro-Concurrency Engine | ✅ | `.go` suffix operator, implicit worker pool, ownership-safe capture semantics |
| 127 | AutoVM TaskSystem Execution | ✅ | Bytecode compilation for Task/Msg systems, on-block compilation, ctx.reply() |
| 128 | Scheduler Message Dispatch Loop | ✅ | Zero-shared-mutable-state scheduler with Tokio async, Arc + mpsc channels |
| 195 | HTTP Client + auto.http Unification | ⏳ | Upgrade to reqwest, unify http_stream, add async HTTP (Phase 3.2 blocked by Plan 196) |

## Status Summary
- Completed: 8 | Partial: 0 | Planned: 1 | Deprecated: 0

## Key Achievements
- Complete Actor-based concurrency stack from lexer through bytecode to runtime execution
- Async/await with `~T` type syntax, `.await` polling, and `.go` micro-concurrency dispatching to worker pools
- Scheduler with zero-shared-mutable-state architecture using Tokio work-stealing, Arc for read-only bytecode, and mpsc channels for mailboxes

## Remaining Work
- Plan 127 Phase 4 (Ask/Reply synchronization) deferred pending async/sync bridge
- Plan 195: HTTP client upgrade to reqwest with async support (Phase 3.2 blocked by Plan 196)
- Future work on pub visibility for task message types and wildcard re-exports
- Performance optimization of the scheduler under high task counts
