# Spec Update: Batch 10 — Concurrency: Tasks and Async

**Date**: 2026-04-16
**Plans Referenced**: Plan 121 (Task/Msg system), Plan 124 (Async/Await), Plan 126 (Micro-concurrency)
**Source Files**: `ast/task.rs` (TaskDef, TaskOnBlock, TaskMsgPattern, TaskAttr, LiteralValue), `ast.rs` (AsyncBlock, Await, Go expr variants), `token.rs` (Task, Spawn, Await, Reply, Go keywords)
**Sections Updated**: Concurrency: Tasks and Async (Section 17 — NEW)

## Old Content

No concurrency documentation existed.

## New Content

### Task Definition
- `task Name { state, start(), on { handlers }, stop() }`
- State fields with mutability and initial values
- Lifecycle hooks: `start()`, `stop()`
- Message handlers with pattern matching

### Task Attributes
- `#[single]` for singleton tasks

### Message Patterns (4 types)
- Simple: `Reset` (no data)
- With bindings: `Add(val)` (named bindings)
- Literal match: `"start"` (exact value)
- Type binding: `msg str` (capture by type)

### Spawning and Communication
- `spawn TaskName()` → `Handle<T>`
- `handle.send(Message)`
- `reply value` for ask/reply RPC

### Async/Await (Plan 124)
- `~T` return type for async functions
- `~{ stmts }` async blocks
- `.await` postfix operator
- `TaskSystem.run()` sync bridge

### Background Execution (Plan 126)
- `.go` postfix operator spawns to worker pool
- Capture semantics: Copy types auto-copied, non-Copy require explicit `.move` or `.clone()`

## Notes

- `TaskDef` struct has attrs (Vec<TaskAttr>), state (Vec<(Name, bool, Expr)>), start_hook, stop_hook, on_block
- `TaskOnBlock` has context_param (for Phase 3 style), handlers (Vec<(TaskMsgPattern, guard, Body)>), else_handler
- Ask/Reply uses compile-time oneshot channel rewriting
