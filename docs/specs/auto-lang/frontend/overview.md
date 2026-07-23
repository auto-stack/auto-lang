# frontend（词法/语法/AST）

> **Status**: implemented（核心管线完整；方言体系 PR-2 已落地；`.?`/`.*`/`.@` 等符号属性、位操作、Auto Flow 未实现）

## 职责

源码 → AST：词法分析（含 f-string 插值 `$var`/`${expr}`）、递归下降语法分析、
方言（dialect）场景化关键字派发、`use` 语句快速扫描与模块路径解析。
产出 `Code`/`Stmt`/`Expr` 树，供求值器、AutoVM、C/Rust 转译器四类后端消费
（docs/design/01 §Compilation Pipeline）。

## 现状

- 管线为 `Lexer → Parser → AST`，`Lexer` 是 parser 的内部组件（lib.rs 中 `mod lexer` 不公开）。
- `Parser` 持有 `Lexer`、`TypeStore`、`InferenceContext`、`ModuleTracker` 与方言表，
  不再依赖 `Universe`（plan-090）。
- 方言体系（轴 A）已实现：`Dialect` trait + `UiDialect`（`Scenario::UI` 下接管
  `widget`/`msg`/`model`/`view`/`on`），产出仍是基础 `Stmt` 变体。
- `use` 体系两层：预处理用 `scan_use_statements`（字符串级，免解析）；
  解析后用 `ModulePath`（`pac`/`super` 前缀）+ `ModuleResolver`/`FilesystemResolver` 落盘。
- AST 序列化三件套 `ToNode`/`ToAtom`/`AtomWriter` 覆盖全部 AST 类型（S 表达式文本）。
- `.as` / `.to` 点属性已在 parser 实现（`Expr::Cast`/`Expr::To`，parser.rs:1979/2257），
  早于 docs/design/10 的"未实现"描述（见分歧记录）。

## 关键入口

- `crates/auto-lang/src/lexer.rs:Lexer`（内部模块，经 `Lexer::next` 供 token）
- `crates/auto-lang/src/token.rs:TokenKind`、`token.rs:Token`、`token.rs:Pos`
- `crates/auto-lang/src/parser.rs:Parser`、`Parser::parse`、`Parser::parse_stmt`、`Parser::parse_expr`
- `crates/auto-lang/src/parser.rs:Parser::build_dialects`、`Parser::try_dialect_stmt`
- `crates/auto-lang/src/parser_helpers.rs:ModuleTracker`、`parser_helpers.rs:LambdaIdGenerator`
- `crates/auto-lang/src/ast.rs:Code`、`ast.rs:Stmt`、`ast.rs:Expr`
- `crates/auto-lang/src/ast.rs:ToNode`、`ast.rs:ToAtom`、`ast.rs:AtomWriter`
- `crates/auto-lang/src/ast/module_path.rs:ModulePath`、`module_path.rs:PathPrefix`
- `crates/auto-lang/src/dialect.rs:Dialect`、`dialect/ui.rs:UiDialect`
- `crates/auto-lang/src/use_scanner.rs:scan_use_statements`、`use_scanner.rs:UseStatement`
- `crates/auto-lang/src/resolver.rs:ModuleResolver`、`resolver.rs:FilesystemResolver`
- 一键入口：`crates/auto-lang/src/lib.rs:parse`（`parse(code) -> AutoResult<Code>`，lib.rs:2114）

## 使用示例

```rust
// 整段源码解析（lib.rs 公开入口）
let code = auto_lang::parse("fn add(a int, b int) int { a + b }")?;

// 预处理扫描 use 依赖（不解析全文）
let uses = auto_lang::use_scanner::scan_use_statements(source);

// 模块路径解析
let resolver = FilesystemResolver::new(PathBuf::from("stdlib/auto"));
let path = resolver.resolve_with_prefix(&module_path, current_file)?;
```

## 已知坑

- `parser.rs` 超 13k 行，语句/表达式/类型解析高度集中，改动需跑全量测试。
- 方言派发用 `mem::take` 移出方言表规避自引用借用（parser.rs:434 注释），新增方言照此模式。
- `view` 在 Core 语言是参数模式关键字、在 UI 场景是语句关键字——同一 TokenKind 两义，
  由 `try_parse_token_stmt` 按 session 场景分派。
- `scan_use_statements` 是字符串匹配：字符串字面量内的 "use xxx" 行、条件编译分支中的
  use 都会被算入，仅适合依赖预估，不能当解析结果用。
- `use_scanner` 支持 `use.rust`/`use.py`/`use c <h>` 等 FFI 导入形式，但去重按 module 名，
  同名不同 items 的 use 只保留第一条。

## 蒸馏来源（Phase 1）

- `docs/design/01-architecture.md`、`docs/design/10-language-syntax.md`
- `docs/plan-reports/01-ast-core.md`、`docs/plan-indices/01-ast-core.md`
- `docs/design/dialect-extension-diagnosis.md`（方言体系来源，dialect.rs 头注指向）
- 代码核对：`crates/auto-lang/src/{lexer,token,parser,parser_helpers,ast,dialect,use_scanner,resolver}.rs`、`ast/`、`dialect/`
