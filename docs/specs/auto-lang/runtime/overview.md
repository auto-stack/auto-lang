# runtime（运行时与内建库）

> **Status**: partial（骨架待蒸馏，Phase 1）

## 职责

运行时支撑：作用域/会话管理、内建标准库绑定（libs/）、FFI 桥（ffi.rs、py_ffi.rs、
a2r_std.rs）、database/sse/route 等服务集成。

## 关键入口

- `crates/auto-lang/src/runtime.rs`、`scope.rs`、`session.rs`
- `crates/auto-lang/src/libs/`（builtin std）
- `crates/auto-lang/src/ffi.rs`、`py_ffi.rs`
- `crates/auto-lang/src/database/`、`sse/`、`route/`

## 蒸馏来源（Phase 1）

- `docs/design/05-vm-runtime.md`、`13-networking.md`
- `docs/plan-reports/`（stdlib 相关主题）
