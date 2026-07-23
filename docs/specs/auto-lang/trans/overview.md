# trans（多目标转译器）

> **Status**: partial（骨架待蒸馏，Phase 1）

## 职责

AST → 目标语言源码：C（a2c）、Rust（a2r）、JavaScript/TypeScript、Python、GDScript，
以及 r2a（Rust→Auto 逆翻译）。

## 关键入口

- `crates/auto-lang/src/trans.rs`、`trans/`
- 后端：`trans/c.rs`、`rust.rs`、`javascript.rs`、`python.rs`、`gdscript.rs`、`r2a.rs`、`ts_*.rs`

## 蒸馏来源（Phase 1）

- `docs/design/06-code-generation.md`
- `docs/a2r-transpiler-guide.md`、`javascript-transpiler.md`、`python-transpiler.md`
- `tests/a2c_tests.at`、`a2r_tests.at`、`a2ts_tests.at`
