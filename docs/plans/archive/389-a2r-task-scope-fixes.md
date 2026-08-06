---
plan: 389
title: a2r-task-scope-fixes
affects: [auto-lang/a2r, auto-lang/parser]
status: complete # draft | in-progress | complete
---

# Plan 389: task 作用域三个修复 — on 绑定入 scope / fn 指针 state 字段 / f-string 自引用

> **For Claude:**
> - 构建/测试命令：`cargo test -p auto-lang`（回归）、`cargo test -p a2r-std`（runtime 单测）、
>   `cargo test -p a2r-actor-tests`（行为一致性，默认 `#[ignore]`，CI 显式开）。
> - 前置 skill：无；需熟悉 `parser.rs` 的 `check_symbol`/`infer_ctx` scope 机制 与
>   `trans/rust.rs` 的 `task_decl`/`infer_type_from_expr`/`fn_param_types`。
> - 回归要求：现有 `cargo test -p auto-lang` 测试不退化；VM actor 测试继续通过；
>   a2r `22_actors` 001-012 文本黄金不改变。
> - worktree：`plan-389/a2r-task-scope-fixes`（`D:/autostack/auto-lang-389`）。
> - 来源：auto-ai `docs/plans/021-auto-completion-roadmap.md` **Phase 6**（EventSink 外发，
>   缺口 1 收尾）。

---

## §1 Goal / 目标

修复 a2r 在 actor task 场景暴露的三个限制（auto-ai Plan 021 Phase 6 记录的 R1/R2/R3），
使 EventSink handler 能把整包 `StreamEvent` 转发给 app（`forward(ev)` / `cb fn(...)` 状态 /
f-string 引用 state 字段）。**不改变任何已有转译输出**（001-012 文本黄金必须原样通过）。

## §2 背景 / 三个限制（实证）

来源：auto-ai `agent.at` 的 `task EventSink` 实施（Plan 021 Phase 1b）中发现。全部实证于
2026-08-05。

| # | 限制 | 症状 | 实证 |
|---|---|---|---|
| R1 | on-block TypeBinding（`ev StreamEvent`）绑定不能作表达式变量 | `forward(ev)` / `(self.cb)(ev)` 报 E0201 "undefined variable ev"；`is ev { ... }` 却能工作 | spike 隔离测试 |
| R2 | fn 指针不能作 task state 字段 | `cb = noop_event` 生成 `cb: /* unknown */`（编译失败） | spike 隔离测试 |
| R3 | f-string 引用 task state 字段解析失败 | `log = f"${log}D:${t};"` 报 E0007 "Expected term, got RBrace"；`+` 拼接可绕过 | spike 隔离测试（普通函数里引用未定义变量同样复现） |

## §3 根因（已定位）

### §3.1 R1 — on-block 绑定未注册进 parser scope

- `parse_task_on_block`（`parser.rs:5173`）解析 `TaskMsgPattern::TypeBinding { name, .. }` /
  `WithBindings`，但**从不把绑定名注册进 `infer_ctx`**。
- `check_symbol` 的 `exists()`（`parser.rs:854`）查 `infer_ctx.lookup_type(name)` —— 查不到 → E0201。
- `is ev` 能用是因为 `parse_is`（`parser.rs:6567`）对 is-target 设 `skip_check=true`
  （"it's a name to match, not a variable to resolve"）——绕过了检查，不是真的在作用域里。
- 对照：闭包参数（`parser.rs:3785`）、fn 参数都在 parse 时 `infer_ctx.bind_var`。

### §3.2 R2 — task state 字段类型推导不认识函数引用

- `emit_task_struct`（`trans/rust.rs:8523`）对每个 state 字段 `infer_type_from_expr(init)`。
- `infer_type_from_expr` 的 `Expr::Ident(name)` 分支（`trans/rust.rs:7768`）只查
  `local_var_types`，函数名不在 → `Type::Unknown` → `rust_type_name` → `/* unknown */`。
- 转译器已有 `fn_param_types`（参数类型表，`trans/rust.rs:244`）+ `fn_ret_types`
  （返回类型表，Plan 373）+ `Type::Fn(Vec<Type>, Box<Type>)`（Plan 060）+ `rust_type_name`
  的 Fn 渲染（`fn(p1, p2) -> r`）——只是没接进 `infer_type_from_expr`。

### §3.3 R3 — task state 字段未注册进 parser scope（与 R1 同根）

- `parse_task_state_field`（`parser.rs:5111`）解析字段名 + init，但**从不 `bind_var`**。
- f-string 插值 `${name}` 的表达式走 `rhs_expr` → `parse_expr` → `check_symbol`；state 字段名
  不在作用域 → E0201。
- 真正展示成 E0007 的机制：`body()` 语句循环的错误恢复（`parser.rs:1434`，`add_error` +
  `synchronize`）吞掉插值里的 E0201 并跳到下一语句，把后续 token 重新解析 → 在函数收尾 `}`
  处报出误导性的 "Expected term, got RBrace"（spike 用 `f"${undefined_var}"` 在**普通函数**
  里复现，证明与 task 无关）。

## §4 修复方案

### §4.1 R1 — on-block 绑定入 scope

`parse_task_on_block`：每个 handler 解析 pattern 后、解析 body 前：
- `TypeBinding { name, type_expr }` → `infer_ctx.bind_var(name, *type_expr)`
- `WithBindings { variant, bindings }` → 每个 binding `bind_var(binding, Unknown)`
- 用 `push_scope` / `pop_scope` 包裹每个 handler body，避免绑定泄漏到其它 handler。

### §4.2 R2 — `infer_type_from_expr` 识别函数引用

`infer_type_from_expr` 的 `Expr::Ident(name)` 分支：先查 `local_var_types`，若未命中且
`fn_param_types.contains_key(name)` → `Type::Fn(fn_param_types[name].clone(), fn_ret_types
.get(name).cloned().unwrap_or(Type::Void))`。

配套：预扫描阶段（`trans/rust.rs:16144` 的 fn 签名 prescan）补插 `fn_ret_types`
（当前只在 fn_decl emit 时插入，`9663`）——否则字段 init 引用的函数声明在 task 之后时
返回类型不可用。

### §4.3 R3 — state 字段注册进 parser scope

`parse_task_with_attrs`：解析 task 体前 `push_scope`；每个 state 字段解析后
`bind_var(field_name, Unknown)`（`exists()` 只查名字存在性，类型可后补）；
task 体结束 `pop_scope`。这样 on-handler / start / stop hook 里的裸字段引用
（含 f-string 插值）都通过 check_symbol。

## §5 验证

- 新增 a2r 用例（`crates/auto-lang/test/a2r/22_actors/013_*`，沿用 387 的 parity 风格）：
  - `013_forward_event`：`on { ev Event -> { forward(ev) } }` + `fn forward(ev Event)` 打印
  - `014_fstring_state`：handler 内 `log = f"${log}D:${t};"` 累计
  - `015_fnp_state_field`：`cb = noop` + `(self.cb)(...)` 转发
- 001-012 文本黄金逐字节不变；VM actor 测试全绿；`cargo test -p auto-lang` 无新增回归。
- 回 auto-ai：重建 auto.exe → 升级 EventSink handler 为转发模式 → retranspile 0 错
  → 端到端事件可观察。

## §6 实施记录（2026-08-05，全部 ✅）

- **R1**（`parser.rs`）：`parse_task_on_block` 每个 handler 解析 pattern 后
  `push_scope` + `bind_task_pattern_names`（TypeBinding 绑定声明类型 / WithBindings 绑
  Unknown）+ `parse_on_handler_body` 解析 guard/body + `pop_scope`（解析出错也 pop，
  保证 scope 栈在错误恢复下平衡）。新增两个 helper：`bind_task_pattern_names` /
  `parse_on_handler_body`。
- **R2**（`trans/rust.rs`）：`infer_type_from_expr` 的 `Expr::Ident` 分支先查
  `local_var_types`，未命中且 `fn_param_types` 命中 → `Type::Fn(params, ret)`；
  预扫描阶段补插 `fn_ret_types`（原来只在 fn_decl emit 时插入）。生成 `cb: fn(Event) -> ()`。
- **R3**（`parser.rs`）：`parse_task_with_attrs` 解析 task 体前 `push_scope`，state 字段
  解析后 `bind_var(field_name, Unknown)`，收尾 `pop_scope`。
- 验证：15 个 22_actors 黄金测试全过（001-012 逐字节不变 + 013-015 新增）；spike 独立
  crate 编译运行——`forward(ev)` 转发、`f"${log}..."` 跨消息累计、`cb` 动态重赋值均正确；
  全量回归与基线完全一致（22 个既有失败 + dodge_player 栈溢出均为既有问题，零新增）。
- **协作说明**：实施期间用户并行提交了 `d896d263`（Plan 043 dot 语句栈溢出修复），
  worktree 已快进合并该提交后在其上落盘，无冲突。

## §7 风险与注意

- **R1/R3 的 scope 注册**是 parser 级改动，`check_symbol` 的 false-positive 风险：
  绑定名泄漏到 task 外（用 scope push/pop 防）；与 `skip_check` 的既有豁免路径不冲突。
- **R2 的 `Type::Fn`** 只影响 state 字段类型推导；不改变已有字段（int/str）输出。
- 只构建 debug（用户既定要求）。
