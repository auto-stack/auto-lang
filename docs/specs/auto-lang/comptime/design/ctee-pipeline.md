# CTEE 管线与集成

## 范围

CTEE 在编译管线中的位置、变换算法、七处集成点，以及"下游假设 AST 已净化"这一不变量。
求值细节见 `comptime-eval.md`。

## 管线位置

```
源码 → Lexer → Parser → 原始 AST（含 Hash* 节点）
     → CTEE::transform（Stage 1：Meta-Eval）
     → 纯净 AST → indexer / infer / codegen / transpile（Stage 2）
```

不变量：**任何语义处理发生前，`Hash*` 节点必须已被消除**。indexer（indexer.rs:166）与类型推断
（infer/stmt.rs:128）对 `HashIf/HashFor/HashIs/HashBrace` 一律跳过——它们不是容错，而是依赖该不变量。

## 集成点（全部已核对）

| 调用方 | 位置 | 说明 |
|---|---|---|
| `lib.rs:execute_autovm_with_path` | 668–671 | AutoVM 脚本执行主路径 |
| `lib.rs:test_code` | 1281 附近 | 测试运行器 |
| `lib.rs:debug_file` | 2652 附近 | 调试入口 |
| `lib.rs:create_vm_from_source` | 2871 附近 | 外部嵌入造 VM |
| `lib.rs:trans_rust_with_session` | 3467 附近 | a2r session 路径 |
| `trans/c.rs:transpile_c` | 4524 附近 | C 转译 |
| `trans/rust.rs:transpile_rust` | 11886 附近 | Rust 转译；plan-310 的逃逸分析 pass 明确插在此调用之后 |

模式统一：`let mut ctee = CTEE::new(); ctee.transform(&mut ast)?;`。
`CTEE::with_target(os, arch)` 存在但**无任何调用方**——交叉编译目标常量目前不可注入，
与 09-compiler.md Open Questions（"comptime 是否需要目标平台模拟器"）对应。

## 变换算法（transformer.rs）

- `transform`：逐语句消费 `Code.stmts`，每条语句映射为 0..n 条新语句，`source_lines` 同步平铺
  （展开的语句继承原行号）。
- `transform_stmt`：仅匹配四种 `Stmt::Hash*`，其余原样返回。
- `#if`：求值条件，真则递归变换 then 分支；假则走 `else`（`ElseIf` 递归）；无 else 则整条删除。
- `#for`：求值迭代源 → 逐值把循环变量名写入 builtins → 变换循环体并拼接 → 结束后移除该变量。
- `#is`：求值 target，按分支顺序首个命中者生效（值相等 / 条件为真 / else），无命中则删除。
- `#{ }`（语句位）：求值后 `value_to_expr` 回写为字面量表达式语句。
- 嵌套：`#if`/`#for`/`#is` 分支体内的语句递归走 `transform_stmt`，故 comptime 构造可嵌套。

## 已知的管线缺口

- **表达式位 `Expr::Comptime` 不经 CTEE**：`transform_stmt` 不下钻普通语句内部的表达式。
  VM 后端在 codegen 兜底——直接编译内层表达式、运行时求值（vm/codegen.rs:8115 附近，代码内 TODO 自述）；
  `trans/c.rs`、`trans/rust.rs` 中未检索到 `Expr::Comptime` 专门分支。
- **a2r 管线中的位置被外部依赖**：plan-310 把所有权逃逸分析锚定在 `CTEE::transform` 之后、
  `RustTrans::trans` 之前——调整 CTEE 调用位置时需同步检查该约束。

## 显式非目标

- CTEE 不做类型检查、不访问 TypeStore；纯 AST→AST 变换。
- 不做跨模块 comptime：每个编译入口独立构造 `CTEE::new()`，无会话级共享状态。
- 不做增量/缓存：每次编译全量重跑变换（与 AIE 增量架构无集成）。

> 来源: docs/design/09-compiler.md、docs/plans/old/095-compile-time-execution-engine.md、docs/plans/archive/310-auto-ownership-escape-analysis.md、crates/auto-lang/src/comptime/transformer.rs、crates/auto-lang/src/lib.rs、crates/auto-lang/src/trans/c.rs、crates/auto-lang/src/trans/rust.rs、crates/auto-lang/src/indexer.rs、crates/auto-lang/src/infer/stmt.rs
