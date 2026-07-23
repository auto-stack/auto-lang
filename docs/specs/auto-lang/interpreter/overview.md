# interpreter（TreeWalker 解释器）

> **Status**: partial（骨架待蒸馏，Phase 1）

## 职责

AST 直走解释执行，是开发期快速反馈与部分测试的执行引擎之一（与 AutoVM、a2r 并存，
见 `docs/execution-engine-selection.md`）。

## 关键入口

- `crates/auto-lang/src/interpreter/`
- `crates/auto-lang/src/execution_engine.rs`（引擎选择）

## 蒸馏来源（Phase 1）

- `docs/design/01-architecture.md`、`docs/execution-engine-selection.md`
