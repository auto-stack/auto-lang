# 类型推断与检查

## 范围

`infer/` 子系统（推断上下文、Robinson 统一、约束、表达式/语句/函数检查）、
`typeck/` 参数检查、`trait_checker.rs` spec 符合性检查。

## 原则

混合策略：表达式走局部自底向上推断（从字面量与运算推类型）；函数签名走
简化 Hindley-Milner（基于统一 + 作用域管理）。错误收集而非中止——
`InferenceContext` 持有 `errors`/`warnings` 累加器，单趟报告多错（infer/mod.rs Phase 6）。

## 细节

### 模块结构

| 文件 | 职责 |
|---|---|
| `infer/context.rs:InferenceContext` | type_env（变量→类型）、约束集、作用域链（变量遮蔽）、当前返回类型、`Arc<RwLock<TypeStore>>` |
| `infer/unification.rs:unify` | Robinson 统一，occurs check 防无限类型（`α = List<α>`） |
| `infer/constraints.rs:TypeConstraint` | 约束表示：Equal / Callable / Indexable / Subtype |
| `infer/expr.rs:infer_expr` | 表达式推断，覆盖 20+ 表达式类型 |
| `infer/stmt.rs:check_stmt` / `functions.rs:check_fn` | 语句与函数签名检查 |
| `infer/errors.rs` | "Did you mean?" 建议（suggest_type / suggest_variable 等） |
| `infer/task_types.rs:TaskTypeChecker` | task 类型检查（Plan 125 Phase 3.6） |
| `infer/registry.rs:TypeRegistry` | **DEPRECATED**，见 typestore.md |

### 统一规则（简化）

```
(Unknown, T)          -> Ok(T)               // Unknown 通配一切
(T, T)                -> Ok(T)
(Int, Uint)           -> Ok(Uint) + warning  // 强制转换
(Int, Bool)           -> Err(Mismatch)
(Array(a), Array(b))  -> 递归统一元素与长度
```

协议：`unify` 入口经 `InferenceContext::unify`（infer/mod.rs:34）；字段检查
`check_field_type`（infer/mod.rs:49）对 `Type::Unknown` 两侧均直接放行——
这是刻意的宽容，但也意味着错误可能被吞。

### 与 parser 的集成现状

- `parser.rs:6598-6654` 多处调用 `infer::infer_expr`（已接入，design/02 的"未接入"过时）。
- `parser.rs:8246` 起在 `impl X as Spec` 处调用 `TraitChecker::check_conformance`。

### typeck/ 与 trait_checker

- `typeck/param_check.rs:ParamChecker`（Plan 088 Phase 6）：收集 `ParamMode::View`
  参数，遍历函数体，`Stmt::Store` 写到 view 参数时报 `TypeError::CannotModifyViewParam`
  （auto_type_E0204）。**局限**：`Stmt::If` 跳过细查、不深入函数调用内部；
  且在 `typeck/` 之外无调用点，未接入编译管线。
- `trait_checker.rs:TraitChecker`：核对 spec 方法逐一实现、参数个数与返回类型匹配，
  错误聚合返回 `Vec<AutoError>`。

## 显式非目标

- 不做全程序 HM 推断；推断限于表达式局部与函数签名边界。
- `ParamChecker` 不做跨函数调用链的不可变追踪。
- NLL（非词法生命周期）不属于推断层，属 ownership/ 的遗留项。

> 来源: docs/design/02-type-system.md（§Type Inference）、docs/plan-indices/03-error-handling.md（plan-010）、crates/auto-lang/src/infer/、typeck/param_check.rs、trait_checker.rs
