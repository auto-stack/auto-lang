# types（类型系统）

> **Status**: implemented（核心可用；借检查、参数检查集成仍 partial）

## 职责

AutoLang 编译器的类型系统：类型表示（`Type` 枚举）、类型推断与统一（Hindley-Milner /
Robinson）、类型检查（字段、参数、spec 符合性）、所有权与借用分析（move/view/mut、
生命周期、last-use）、统一类型存储（TypeStore）。跨模块服务 parser、codegen 与各转译器。

## 现状

- 类型表示完整：`Type` 枚举约 35 个变体（`ast/types.rs:24`），含 `Option`/`Result`（Plan 120）、
  `Tuple`（Plan 200）、`Rust`（Plan 190）、`Handle`（Plan 121）；`May<T>` 已从 AST 移除。
- 推断引擎已接入 parser：`parser.rs` 多处调用 `infer::infer_expr`（如 parser.rs:6598-6654），
  `TraitChecker::check_conformance` 在 parser.rs:8246 起接入 spec 符合性检查。
  （design/02 的"未接入 parser"说法已过时。）
- TypeStore 为单一数据源，`InferenceContext` 通过 `Arc<RwLock<TypeStore>>` 共享
  （infer/context.rs:73）；`infer/registry.rs` 标记 DEPRECATED 但仍被
  `type_registry.rs`、`parser.rs`、`vm/codegen.rs`、`autovm_persistent.rs` 引用。
- 所有权模块三阶段：move 语义 ✅、拥有型字符串 ✅、借检查 🔄（ownership/mod.rs 头注释）。
- `ParamChecker` 已实现但在 `typeck/` 之外无任何调用点，未接入编译管线（plan-indices/04 遗留项）。

## 关键入口

- `crates/auto-lang/src/ast/types.rs:Type` — 类型枚举（约 35 变体）
- `crates/auto-lang/src/ast/fun.rs:ParamMode` — View/Mut/Move 参数模式（Copy/Take 已废弃）
- `crates/auto-lang/src/ast/enums.rs:EnumKind` — 统一 enum 三形态判别
- `crates/auto-lang/src/types.rs:TypeStore` — 统一类型/函数/spec/enum/泛型存储
- `crates/auto-lang/src/infer/context.rs:InferenceContext` — 推断上下文（type_env + 约束 + 作用域链）
- `crates/auto-lang/src/infer/unification.rs:unify` — Robinson 统一（occurs check，Unknown 通配）
- `crates/auto-lang/src/infer/expr.rs:infer_expr` / `stmt.rs:check_stmt` / `functions.rs:check_fn`
- `crates/auto-lang/src/typeck/param_check.rs:ParamChecker` — view 参数不可变检查（未接管线）
- `crates/auto-lang/src/trait_checker.rs:TraitChecker` — spec 符合性检查
- `crates/auto-lang/src/ownership/borrow.rs:BorrowChecker` / `lifetime.rs:LifetimeContext` / `cfa.rs:LastUseAnalyzer`
- `crates/auto-lang/src/trans/escape/` — 逃逸分析与智能指针回退（Plan 310，属 trans 但服务所有权语义）

## 使用示例

```auto
fn update(u mut User) { u.age += 1 }   // 定义点 mut，调用点必须 u.mut
let r = read_sensor()                  // r: !int（Result）
enum Msg { Quit, Move Point }          // 异质 enum（ADT）
```

## 已知坑

- 术语漂移：`ownership/borrow.rs` 仍用 `take` 命名（`BorrowKind::Take`），而 `ParamMode`
  以 `Move` 为正名、`Take` 为 deprecated 别名；读代码时两者同义。
- `str`（StrSlice）不允许作容器元素，容器默认 `Str`（StrOwned）——见 design/02 §String。
- `Type::Unknown` 在 unify 中通配一切，错误可能被静默吞掉；`check_field_type` 对 Unknown 直接放行。
- `TypeError` 错误码实际占 E0101-E0106 与 E0201-E0204（`auto_type_E020x`），与 design/03
  的"E0101-E0105"表格不符（见 architecture.md 分歧记录）。

## 蒸馏来源（Phase 1）

- `docs/design/02-type-system.md`、`03-error-handling.md`、`04-memory-ownership.md`
- `docs/plan-indices/02/03/04`、`docs/plans/old/008-208`、`docs/plans/archive/310`
- 代码核对：`crates/auto-lang/src/{types.rs, infer/, typeck.rs, typeck/, ownership/, trait_checker.rs, ast/types.rs, ast/fun.rs, ast/enums.rs, error.rs}`
