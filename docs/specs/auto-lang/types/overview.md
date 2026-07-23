# types（类型系统）

> **Status**: partial（骨架待蒸馏，Phase 1）

## 职责

类型表示与推断（unification/constraints/registry）、类型检查、所有权分析
（borrow/lifetime/cfa）、trait 检查、泛型。

## 关键入口

- `crates/auto-lang/src/types.rs`、`infer/`
- `crates/auto-lang/src/typeck.rs`、`typeck/`
- `crates/auto-lang/src/ownership/`
- `crates/auto-lang/src/trait_checker.rs`

## 蒸馏来源（Phase 1）

- `docs/design/02-type-system.md`、`03-error-handling.md`、`04-memory-ownership.md`
