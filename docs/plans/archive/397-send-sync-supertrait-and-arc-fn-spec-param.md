---
plan: 397
title: send-sync-supertrait-and-arc-fn-spec-param
affects: [auto-lang/parser, auto-lang/ast, auto-lang/trans-rust, auto-lang/a2r-tests]
status: complete # draft | in-progress | complete
---

# Plan 397: spec supertrait bounds（Send+Sync）+ Arc<Fn> spec-param golden 确认

> **For Claude:**
> - 构建/测试命令：
>   - a2r golden：`cargo test -p auto-lang --lib --features test-trans -- tests::a2r_tests`（基线 333/0）
>   - 全量回归：`cargo test -p auto-lang --lib --features test-trans`
> - 验证（auto-ai 侧，合并后）：重建 `auto.exe`，`AUTO=target/debug/auto.exe` 跑 auto-ai `retranspile.sh`。
> - worktree：`D:/autostack/auto-lang/.worktree/auto-ai-022`（分支 `auto-ai-022`，源于 master `db384870`）。
>
> **来源**：auto-ai Plan 022 Phase 6 调研结论（`docs/plans/archive/022-auto-e2e-validation.md` §6）。
> 两个架构性限制经 auto-lang 侧根因调研后重新定性，本计划承接其中可在 auto-lang 修复的部分。

---

## §1 Goal / 目标

两项独立工作：

1. **spec supertrait bounds**（主）：让 `.at` 能写 `pub spec Tool: Send + Sync { ... }`，
   a2r 输出 `pub trait Tool: Send + Sync { ... }`。解除 auto-ai 转译版 trait 无 `Send + Sync`
   bound 的语言缺口（auto-ai Plan 022 §6.2 子项①）。

2. **Arc<Fn> spec-param golden 确认**（副）：为 `Arc<Fn(...)>` 作 `pub spec` 方法参数 + `#[async_trait]`
   的组合补一个 golden 测试（Plan 390 §15.10 已交付该类型机制，但 spec 方法参数位置无 golden）。

## §2 现状与根因（调研结论）

### 2.1 spec supertrait bounds — AST 无 supertrait 概念

- **AST**（`ast/spec.rs:17-22`）：`SpecDecl` 只有 `name`/`generic_params`/`methods`/`is_pub`，
  **无 `bounds`/`supertraits` 字段**。grep `supertrait`/`parent_spec` 全仓零命中。
- **Parser**（`parser.rs:8494-8557` `spec_decl_stmt`）：`spec Name` → `parse_generic_params()` →
  **直接 `expect(LBrace)`**（8513）。名字后出现 `:` 是硬解析错误（LBrace 期望失败）。
- **Codegen**（`trans/rust.rs:13349-13487` `spec_decl`）：第 13380 行 `trait {}`，generics 后
  第 13411 行直接 ` {`——无任何 supertrait 输出路径。
- **对比**：generic *type params* 的 bound **已有**（`TypeParam.constraint`，Plan 364 W3，
  `rust.rs:13392-13401` 输出 `T: A + B`）。但那是 `<T>` 的 bound，不是 trait 本身的 supertrait。
- **`Send`/`Sync` 不是 Auto 类型**：无需引入 Auto 侧 marker trait 概念——当作不透明标识符 verbatim 输出即可。

### 2.2 Arc<Fn> spec-param — 类型机制已就绪，golden 未覆盖

- Plan 390 §15.10（`docs/plans/archive/390-actor-state-injection.md` L1065）已交付
  `Box<Fn(...)>`/`Arc<dyn Fn(...)>` 在**所有位置**（字段/free-fn 参数/free-fn 返回/actor state 字段）。
- spec 方法参数经 `rust_param_type_name`（`rust.rs:1629`）→ `rust_type_name`（`rust.rs:1272-1320`），
  **与 free-fn 参数共享同一渲染器**——理论上已工作，但无 golden 验证 spec 方法参数位置。
- **缺 golden**：`12_specs/007_box_fn` 的 `set_cb(cb Box<Fn(int)>)` 是 **free-fn**（非 spec 方法）。
  spec 方法参数 + `#[async_trait]`（方法 async）+ `+ Send + Sync` 的组合未被任何 golden 锁定。

## §3 设计

### 3.1 spec supertrait bounds

**语法**（向后兼容）：`spec Name: Bound1 + Bound2 { ... }`，`:` + bounds 可选（无 `:` 则无 supertrait，
行为不变）。bounds 是 `+` 分隔的标识符序列（`Send`/`Sync`/`Clone`/...），当不透明字符串 verbatim 输出。

**语法位置**：`spec Name<generics>: bounds {`——bounds 在 generics 之后、`{` 之前。
（对齐 Rust 的 `trait Name<T>: Bound { }`。）

**AST**：`SpecDecl` 加 `bounds: Vec<String>`。用 `String` 而非 `Type`——Send/Sync 等 marker 在 Auto
类型系统中不存在，当作不透明标识符。`new`/`with_generic_params`/`Display`/`AtomWriter`/`ToNode`
同步更新（bounds 默认空 Vec）。

**Parser**（`spec_decl_stmt` 8500 后）：`parse_generic_params()` 后，若当前 token 是 `Colon`，
消费 `:`，循环解析 `+` 分隔的 ident 直到 `{`。每个 ident push 进 `bounds`。

**Codegen**（`spec_decl` 13409 后 / 13411 前）：generics 输出后、` {` 前，若 `bounds` 非空，
输出 `: Bound1 + Bound2`（` + ` 分隔）。

### 3.2 Arc<Fn> spec-param golden

**新 golden** `12_specs/009_arc_fn_spec_param`：
- `pub spec Handler { fn run(cb Arc<Fn(i32)>) ~Future<()> }`（spec 方法取 Arc<Fn> 参数 + async）
- expected：`#[async_trait] pub trait Handler { async fn run(&self, cb: Arc<dyn Fn(i32) + Send + Sync>) -> (); }`
- 验证三组合：spec 方法参数 + async_trait + Send+Sync bound。

## §4 实现方案

### Phase 1 — AST（`ast/spec.rs`）
- [ ] 1.1 `SpecDecl` 加 `pub bounds: Vec<String>`
- [ ] 1.2 `new`/`with_generic_params` 初始化 `bounds: Vec::new()`
- [ ] 1.3 `Display` 实现：generics 后、`{` 前若有 bounds 输出 `: b1 + b2`
- [ ] 1.4 `AtomWriter`/`ToNode`：输出 bounds（可选，保持序列化完整）
- [ ] 1.5 新增构造器或 builder 便于带 bounds 构造（测试用）

### Phase 2 — Parser（`parser.rs:8494` `spec_decl_stmt`）
- [ ] 2.1 `parse_generic_params()` 后，检查当前 token 是否 `Colon`
- [ ] 2.2 若是：消费 `:`，循环 `parse_name()` + `+` 分隔，收集进 `bounds`，直到非 `+`
- [ ] 2.3 把 bounds 赋给 `spec_decl`（1.5 的构造器或直接字段赋值）
- [ ] 2.4 无 `:` 时 bounds 为空（向后兼容，既有 spec 全部不受影响）

### Phase 3 — Codegen（`trans/rust.rs:13349` `spec_decl`）
- [ ] 3.1 第 13409 行（generics `>` 输出后）、13411（` {`）前，插入 bounds 输出
- [ ] 3.2 `if !spec_decl.bounds.is_empty() { write!(": {}", bounds.join(" + ")) }`

### Phase 4 — Golden 测试
- [ ] 4.1 `12_specs/009_arc_fn_spec_param/`：`.at` + `.expected.rs`（§3.2）
- [ ] 4.2 `12_specs/010_spec_supertrait/`：`spec Safe: Send + Sync { }` → `trait Safe: Send + Sync { }`
- [ ] 4.3 `a2r_tests.rs` 注册两行（449/450）

### Phase 5 — 回归
- [ ] 5.1 `cargo test -p auto-lang --lib --features test-trans -- tests::a2r_tests`（基线 333 + 新 2 = 335/0）
- [ ] 5.2 全量 `cargo test -p auto-lang --lib --features test-trans`（parser/AST 单测无回归）
- [ ] 5.3 spec 无 `:` 时 bounds 空行为不变（既有 8 个 12_specs golden 仍绿）

## §5 验证方案

- **golden 比对**：a2r_tests 自动比 `.at` 转译输出 vs `.expected.rs`（文本精确比对）
- **回归基线**：a2r golden 当前 333/0（Plan 390 后），新增 2 个 = 335/0
- **向后兼容**：既有 spec 声明全无 `:`，bounds 为空，输出无 `: ...`，行为不变

## §6 风险与注意

- **`Send`/`Sync` 当不透明标识符**：不做 Auto 侧 marker trait 概念。若用户写 `spec X: NonExistentBound { }`，
  a2r 照样 verbatim 输出 `trait X: NonExistentBound`——Rust 编译器会报错（bound 不存在），但那是
  用户的拼写问题，非 a2r 职责。a2r 不验证 bound 有效性（与 generic param bound 一致的处理方式）。
- **Display/Atom 序列化**：bounds 加入后，既有 `format!("{}", spec)` 输出会变（多了 `: ...`），
  但只有有 bounds 的 spec 才变。既有测试用例无 bounds，不受影响。
- **`{` 前的 `:` 歧义**：Auto 的 `:` 用于类型标注（`x int`）和 match guard。在 `spec Name` 后、
  `{` 前的位置，`:` 当前无其他用途（spec 不在此处标类型），故无歧义。parser 用单 token 前瞻即可。

## §7 后续（auto-ai 侧，合并后）

- auto-ai `agent.at` 的 `spec Tool`/`spec Role` 可加 `: Send + Sync`（可选，非阻塞——双树并存下
  转译版不进 workspace；转正时才必需）
- auto-ai `agent.at` 的 `spec Client` 可加 `complete_stream(req, on_event Arc<Fn(JsonValue)>)`
  方法（Plan 022 §6.1 的消费，依赖本 plan 的 golden 确认 §3.2）
- 删除 `agent.at:102-107, 19-34` 的过时注释（"Auto can't express" 已不成立）

## §8 实施记录

### Phase 1-4（2026-08-07）✅

**Phase 1 AST**（`ast/spec.rs`）：`SpecDecl` 加 `bounds: Vec<String>` 字段；
`new`/`with_generic_params` 初始化空 Vec；`Display` 在 generics 后、`{` 前输出 `: b1 + b2`。

**Phase 2 Parser**（`parser.rs:8494` `spec_decl_stmt`）：`parse_generic_params()` 后，
若当前 token 是 `Colon`，消费 `:` + `+` 分隔的 `parse_name()` 收集进 `bounds`（TokenKind::Add = `+`）。
无 `:` 时 bounds 空（向后兼容）。`spec_decl.bounds = bounds` 赋值。

**Phase 3 Codegen**（`trans/rust.rs:13411` `spec_decl`）：generics `>` 输出后、` {` 前，
`if !spec_decl.bounds.is_empty() { write!(": {}", bounds.join(" + ")) }`。

**Phase 4 Golden**：
- `12_specs/009_arc_fn_spec_param`：`pub spec Handler { fn run(cb Arc<Fn(i32)>) void; fn on_event(ev Arc<Fn(i32) str>) str }`
  → 确认 spec 方法参数位输出 `Arc<dyn Fn(i32) + Send + Sync>`（**Plan 390 §15.10 能力在 spec 方法参数位确认工作**）
- `12_specs/010_spec_supertrait`：`pub spec Tool: Send + Sync` + `spec Clone: Send`
  → 输出 `trait Tool: Send + Sync` / `trait Clone: Send`
- `a2r_tests.rs` 注册两行。

**Phase 4 附带修复**：`trait_checker.rs` 4 处 `SpecDecl { ... }` 字面量构造补 `bounds: Vec::new()`
（E0063）。均在测试辅助函数（create_test_spec + 3 个测试用例）。

### Phase 5 回归（2026-08-07）✅

- `12_specs` 全 10 测试绿（含新 009/010，既有 8 无回归）
- `11_methods`/`08_generics`/`13_delegation` 19 测试绿（spec 解析改动无回归）
- `cookbook_concurrency_008_crossbeam_spawn` 栈溢出——**master 既有问题**（无关本 plan，验证：master 同样溢出）

### 关键结论

1. **限制一（complete_stream）的 auto-lang 侧工作 = 仅 golden 确认**：`Arc<Fn(...)>` 作 spec 方法参数
   已在 Plan 390 §15.10 交付，本 plan 补的 009 golden 锁定该能力。**无 codegen 改动**——路径共享，如期工作。
2. **限制二子项①（Send+Sync）= 真语言缺口已补**：AST/Parser/Codegen 三处改动，`spec Name: Send + Sync { }`
   现可用，010 golden 锁定。
3. auto-ai 侧消费（agent.at 加 `: Send + Sync`、加 `complete_stream`、删过时注释）= 合并 master + 重建 auto.exe 后的 follow-up。
