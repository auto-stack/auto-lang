# frontend（词法/语法/AST）

> **Status**: partial（骨架待蒸馏，Phase 1）

## 职责

源码 → AST：词法分析、语法分析、方言（dialect）处理、use 扫描与模块解析、宏展开。

## 关键入口

- `crates/auto-lang/src/lexer.rs`、`token.rs`
- `crates/auto-lang/src/parser.rs`、`parser_helpers.rs`
- `crates/auto-lang/src/ast.rs`、`ast/`
- `crates/auto-lang/src/dialect.rs`、`dialect/`
- `crates/auto-lang/src/use_scanner.rs`、`resolver.rs`

## 蒸馏来源（Phase 1）

- `docs/design/01-architecture.md`、`docs/design/10-language-syntax.md`
- `docs/plan-reports/01-ast-core.md`
