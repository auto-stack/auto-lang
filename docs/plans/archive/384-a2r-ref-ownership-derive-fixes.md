# Plan 384：a2r 引用注入 / 类型推断 / derive 精确化（auto-musk Plan 015 治本）

> **Status**: 计划已制定，待实施。
> **来源**: auto-musk Plan 015 —— 合并编译 153→0 错误。0 错误是用「产物 .rs 手改」绕开的，
> 重新转译会重现。本计划在 a2r 层治本，目标是「全量重新转译 + nativeize + cargo check
> 无需产物手改即可 0 错误」。
> **影响仓库**: `auto-lang`（`crates/auto-lang/src/trans/rust.rs` + `parser.rs`）+ `auto-musk`（`nativeize.pl`）
> **风险**: 中-高 —— A3 触动 call site 代码生成（核心路径），需 golden 测试覆盖。
> **承接**: Plan 380（a2r Rust 互操作补全，已合并）。

---

## 0. 问题全景（10 类，实测来自 auto-musk 合并编译）

auto-musk Plan 015 §3 记录了 10 类 a2r 问题（A1-A10）。经本计划调研，其中：

| # | 问题 | 复现性（当前 master） | 本计划处理 |
|---|---|---|---|
| **A3** | 引用注入缺失（`value_get_str(args)` 漏 `&args`） | ✅ 复现，影响 ~50 错误 | **核心** |
| **A2** | 无类型参数推断成 `i32`（`fn complete(req)` → `req: i32`） | ✅ 复现，A2/A6/A8 共同根因 | **核心** |
| **A5** | struct 过度 derive（`Arc<dyn T>` 自动 derive Debug/Eq/Ord） | ✅ 复现 | 实施 |
| **A6** | backtick raw string 强制走 `format!`（花括号吞 `}`） | ✅ 复现 | 实施 |
| **A7** | 全路径类型 `Arc<dyn mod::Type>` 解析报错 | ✅ 复现（解析期） | 实施 |
| **A9** | `.delete()` 无条件转 `.remove()`（误伤 axum Router） | ✅ 复现 | 实施 |
| A8 | spec 写显式 `self` 时 `spec_decl` 不 skip | ⚠️ 潜在（当前 auto-musk .at 已规避） | 顺手补 |
| A10 | `nativeize.pl` 硬编码 `crate::extern_impl`（应 `super::`） | ✅ 复现（auto-musk 侧） | 实施（auto-musk） |
| A1 | clap Subcommand 误生成 | ❌ **不复现**（见 §A1 勘误） | 仅记录 |
| A2-trait | 本地 trait 与 re-export 上游 trait 冲突 | ✅ 复现但属用户语义错误 | 报错/诊断 |

---

## A1 勘误（重要）

auto-musk Plan 015 §3-A1 说「a2r 对 `#[derive(Parser)]` 误加 Subcommand」。**经全 crate 搜索
`rust.rs` 里 `Subcommand` 出现 0 次**——a2r 不主动加 Subcommand。实际是当时的 `main.at` 源码
写了 `#[derive(Parser, Subcommand)]`，a2r 原样透传。当前 `main.at` 已改为只 `#[derive(Parser)]`，
**A1 不再复现**。

a2r 真正的小问题是：`use_stmt` 的 `companion_imports`（rust.rs:9177）硬编码
`("clap", "use clap::Parser;")`——只补 Parser，不按用户 derive 内容补 Subcommand/ValueEnum。
这是 **A1b（低优先级增强）**：检测 `#[derive(...)]` 内容动态补 import。

---

## A3：引用注入缺失（最高优先级，影响 ~50 错误）

### 现象
- `value_get_str(args, k)`：被调 `value_get_str` 形参 `&Value`，a2r 生成 `value_get_str(args, k)`（漏 `&args`）
- `resolve_within_project(path)`：形参 `&str`，漏 `&path`
- `Ok("x")` 返回 `Result<String,_>` 时漏 `.to_string()`（部分场景）
- 连锁：args 被 move → E0382

### 根因（已确认）
1. **a2r 已有被调函数签名表** `fn_param_types: HashMap<AutoStr, Vec<Type>>`（rust.rs:215），
   在 `fn_decl`（8203-8209）和 `trans()`（13986-14031）填充。**数据齐全**。
2. **call site 没用它做引用注入**：`call()` 函数 rust.rs:6585-6841 的参数生成循环，靠
   `is_str_param`/`is_struct_param` 等实参侧标志位驱动，**没有「形参是 `&T` 引用 → 实参加 `&`」的分支**。
3. **隐藏前提**：`Type::Reference(Box<Type>)` 在 `ast/types.rs:45` 有定义，但 **parser.rs 从不构造它**
   （`@T` 无解析分支，`parse_type_base` 9502-9604 无 `TokenKind::At`）。所以即便 call site 查
   `fn_param_types`，`value_get_str(v @Value)` 的形参类型也不是 `Type::Reference`。

### 修法（分两步）

**步骤 1（parser，前置）**：让 `@T` 形参解析成 `Type::Reference`。
- `parser.rs:parse_type_base`（9502）或 `parse_ident_or_generic_type`（8990）加 `TokenKind::At` 分支：
  消费 `@`，递归解析 inner type，包成 `Type::Reference(Box<Type>`。
- 确认 `rust_param_type_name`/`rust_type_name`（1008-1011）已正确渲染 `&T`（agent 确认已对）。

**步骤 2（rust.rs call site，核心）**：在 `call()` 参数循环（6585-6697）加 `needs_ref_borrow` 分支。
- 取 `param_types[i]`，若为 `Type::Reference(_)` 且实参是 owned 值（Ident、非字面量、非已是 ref），
  在实参前写 `&`。
- 参考现有 `&mut` 注入（6704-6710）和 `contains` 的 receiver 类型判定（5611-5631）写法。

**步骤 3（Ok 注入，小）**：`expr()` 的 `Expr::Ok`（1486-1508），放宽 payload 判定。
- 1491 行除字面量外，也覆盖 `Expr::Ident(name)` 且 name 在 `current_fn_str_params`（`&str` 形参）的情况。

### 难度
- 步骤 1：中（parser 加分支，需回归 `@T` 解析）
- 步骤 2：中-大（call site 核心路径，需 golden 测试覆盖各种实参形态）
- 步骤 3：小

### 验证
- 新增 golden：`test/a2r/16_interop/` 加「`&T` 形参 → `&args` 注入」案例
- 回归：auto-musk 全量 .at 重新转译，tools.rs/spec_tools.rs/orch_tools.rs 的 `value_get_*(&args)` 应自动生成

---

## A2：无类型参数推断成 `i32`（A2/A6/A8 共同根因，性价比最高）

### 现象
- `spec Client { fn complete(req) ~str }` → `fn complete(req: i32) -> String`（req 应保持未知或报错）
- `spec DriveSink { fn on_event(ev) }` → `fn on_event(&self, ev: i32)`（ev 是 StreamEvent，被填 i32）
- 凡是无 `: 类型` 注解的参数，一律 `Type::Int` → `i32`

### 根因（已确认）
`parser.rs:7752`（`fn fn_params`）：
```rust
let mut ty = Type::Int;  // 参数类型缺省硬编码
```
注释暗示「后续推断」，但实际从未覆盖到 spec 方法签名（`spec_decl` L10874 用 `rust_param_type_name`，
不经 `effective_param_type_name` 的 Ok/Err 兜底）。

### 修法
- **治标（小）**：`spec_decl`（rust.rs:10874）改用 `effective_param_type_name`（1237），与 `fn_decl` 对齐
- **治本（中）**：`parser.rs:7752` 把默认值改成 `Type::Unknown`（新增变体或复用），并在
  `rust_type_name`/`rust_param_type_name` 对 `Type::Unknown` emit 诊断（warning 或回退占位类型）
- **影响面扫描**：所有依赖 `Type::Int` 默认值的地方需回归

### 难度：小（治标）/ 中（治本，要扫依赖）

---

## A5：struct 过度 derive（`Arc<dyn T>` 自动 derive Debug/Eq/Ord）

### 现象
`OwnedRole { inner: Arc<dyn Role> }` → `#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]`，
`dyn Role` 不支持 Debug/Eq/Ord → E0277/E0369。同理 `MuskAgentFactory { state: Arc<AppState> }`。

### 根因（已确认）
- `type_decl()`（rust.rs:9411-9447）：`type_decl.attrs.is_empty()` 时自动 derive，字段类型检测
  只有 `type_has_float`/`type_contains_enum`（不检测 `dyn Trait`/`Arc<dyn>`）→ 出全套 derive
- 后处理兜底 `fix_dyn_trait_derives`（12334）regex **只匹配 `Box<dyn`，不匹配 `Arc<dyn`**
- `fix_non_ord_derives`（13508）marker 白名单含 `Box<dyn` 不含 `Arc<dyn`
- **合并模式 `apply_merged_regex_fixes`（15507）完全没有这两个 fix**——auto-musk 走合并模式

### 修法
- **生成层（治本）**：`type_decl`（9414-9447）加 `type_contains_dyn(ty)` 字段扫描（仿 `type_has_float`），
  递归检查 `Type::User("dyn ...")` 和 `Type::GenericInstance` 的 args 含 dyn → 降到 `#[derive(Clone, Debug)]`
  （`Arc<dyn T>` 是 Clone 的，但不 PartialEq/Eq/Ord）
- **regex 兜底（补齐）**：`fix_dyn_trait_derives`（12336）`Box<dyn` → `(Box|Arc|Rc)<dyn`；
  `fix_non_ord_derives`（13513）加 `Arc<dyn`/`Rc<dyn`；并把这俩 fix 加进 `apply_merged_regex_fixes`

### 难度：小-中（生成层加扫描函数 + regex 补 + 合并模式接入）

---

## A6：backtick raw string 强制走 `format!`

### 现象
`let schema = \`{"type":"object",...}\`` → `fn main() { let schema: String = format!("{{..."); }`
花括号不平衡（`}` 被吞），且进 `fn main()` 与 extern_impl 的 const 遮蔽。

### 根因（已确认）
1. **顶层 `let` 进 main**：`rust.rs:14074-14115`（L14088 `StoreKind::Let` → main）
2. **backtick 语义错配**：`lexer.rs:568` 注释明示「Backtick strings are raw」，但 a2r 对无 `$` 插值的
   backtick 仍走 `Expr::FStr` → `format!`（rust.rs:2397），导致花括号需转义而 raw string 不该转义

### 修法
- **backtick → raw string**（中）：`Expr::FStr`（2397）开头检测 `fstr.parts` 是否全是 `Expr::Str/CStr`
  （无表达式插值）→ emit `r#"..."#`（`#` 数量直到不含 `"#`）；含 `$` 才 `format!`
- **顶层 let → 模块级 const**（小-中）：`rust.rs:14078-14088` 顶层 `StoreKind::Let` 归入 `decls`，
  emit `const NAME: &str = ...;`（库模式）

### 难度：中

---

## A7：全路径类型 `Arc<dyn mod::Type>` 解析报错

### 现象
`inner Arc<dyn auto_ai_agent::Role>`（字段类型含 `::`）→ parser 报 `Expected term, got RBrace`
（误导到文件尾）。短名 `Arc<dyn Role>` 正常。

### 根因（已确认，parser.rs）
- Auto 用 `.` 做路径分隔，`parse_ident_or_generic_type`（8990-9110）只认 `.`（9021）
- `::` 在词法层是两个 `Colon`（token.rs 无 `ColonColon`），parser 消费完首个 ident 后剩余 `::Role`
  无法处理 → 后续撞 `:` 报错
- 错误抛出：parser.rs:3189-3195

### 修法（两选一）
- **方案 A（parser 加 `::`）**：`parse_ident_or_generic_type`（9021）识别连续两个 `Colon`，拼成
  `auto_ai_agent::Role` 保留 `::` 存入 `Type::User`；同步 `qualify_type_name`（1095-1167）认 `::`
  不误加 `crate::` 前缀
- **方案 B（用 `.`）**：Auto 源码用 `auto_ai_agent.Role`（`.` 是既定惯例，9021 已支持）。零代码改动。
  **建议先验证方案 B 是否 work**（改 .at 用 `.`），不行再做方案 A

### 难度：中（方案 A）/ 零（方案 B）

---

## A9：`.delete()` 无条件转 `.remove()`

### 现象
axum 路由 `.delete(handler)` → `.remove(handler)`（HashMap 方法映射误伤 Router）。

### 根因（已确认）
`call()` 两处方法名映射表无条件 `delete → remove`：
- rust.rs:4449-4450（Bina/Dot 方法调用）
- rust.rs:5656（Expr::Dot 方法调用）
两处都无 receiver 类型守卫。

### 修法（小）
仿 `contains` 的 receiver 类型判定（5611-5631）：只有 receiver 是 Map（HashMap/BTreeMap）时才转 `remove`，
否则透传 `delete`。判定：`local_var_types.get(receiver)` 是否 Map 类型。

### 难度：小（~10 行）

---

## A8 残留：spec 写显式 `self` 时 `spec_decl` 不 skip

### 现象（潜在）
当前 auto-musk .at 已规避（spec/ext 都不写 self）。但若 `spec X { fn f(self, ev) }`，
`spec_decl`（rust.rs:10866）无条件 emit `&self` + 不 skip method.params[0] → `fn f(&self, self, ev)` 非法。

### 修法（小）
`spec_decl`（10868 前）和 type-as-spec impl（10118 前）补 `skip_first_self`（仿 `fn_decl` 8108-8125）。

### 难度：小

---

## A10：`nativeize.pl` 硬编码 `crate::extern_impl`（auto-musk 侧）

### 现象
lib.at/server_serve.at nativeize 后 `use crate::extern_impl::*`（应 `super::extern_impl::*`）。

### 根因（已确认）
`auto-musk/backend/crates/musk/auto-src/nativeize.pl:135-137` 硬编码 `use crate::extern_impl::*;`。
所有产物位于 `auto_generated` 子模块，正确是 `super::`。

### 修法（小，auto-musk 侧）
`nativeize.pl:136` 改 `use super::extern_impl::*;`。或更稳健：不在 nativeize 注入，
改由 `mod.rs` 统一 `pub use extern_impl::*;`。

### 难度：小

---

## A2-trait：本地 trait 与 re-export 上游 trait 冲突

### 现象
`spec Client {...}` 生成 `trait Client`，与 extern_impl `pub use auto_ai_agent::Client` 冲突。

### 根因
`spec_decl`（10812）无冲突检测，每个 spec 都生成 trait。

### 修法（中）
`spec_decl` 开头检查 name 是否已在 `use.rust` 引入符号集 → 报错（推荐，因 spec 想描述上游 trait
时是用户语义错误）或跳过生成。

### 难度：中

---

## 实施顺序（按 ROI / 风险）

1. **A2（i32 推断）** —— 改 parser 一处，多症状受益，风险低。先做。
2. **A10（nativeize.pl）** —— auto-musk 一行改，立即见效。
3. **A9（delete→remove）** —— ~10 行类型守卫，低风险。
4. **A8 残留（skip_first_self）** —— 几行，低风险。
5. **A5（过度 derive）** —— 生成层加扫描 + regex 补，中。
6. **A7（全路径）** —— 先验证方案 B（用 `.`），不行再 parser 加 `::`。
7. **A6（backtick raw string）** —— 中，需区分库/可执行模式。
8. **A3（引用注入）** —— 核心，风险中-大，放最后。需 parser `@T` + call site 双改 + golden 测试。

每步实施后：auto-musk 全量重新转译 + nativeize + cargo check，记录错误数下降。

## 验收标准

auto-musk 执行：
```
cd backend/crates/musk/auto-src
for f in *.at; do A2R_CRATE_ROOT=0 auto.exe trans --path "$f" rust; done
for f in *.a2r.rs; do perl nativeize.pl "$f"; done
for f in *.a2r.rs; do base=$(basename "$f" .a2r.rs); cp "$f" "../src/auto_generated/${base}.rs"; done
cd ../../.. && cargo check
```
**目标：0 错误，且无需任何产物 .rs 手改**（auto-musk Plan 015 的所有产物手改都应被 a2r 自动生成取代）。

## 关键代码位置索引（均在 `crates/auto-lang/src/`）

| 问题 | 文件:函数:行 |
|---|---|
| A3 call site | `trans/rust.rs: call() 6585-6841` |
| A3 签名表 | `trans/rust.rs: fn_param_types 215; 填充 8203/13986` |
| A3 Ok 注入 | `trans/rust.rs: expr() Expr::Ok 1486-1508` |
| A3 @T 解析 | `parser.rs: parse_type_base 9502-9604` |
| A2 i32 默认 | `parser.rs: fn_params 7752` |
| A2 spec 类型 | `trans/rust.rs: spec_decl 10874; effective_param_type_name 1237` |
| A5 derive 生成 | `trans/rust.rs: type_decl 9411-9447` |
| A5 regex 兜底 | `trans/rust.rs: fix_dyn_trait_derives 12334; fix_non_ord_derives 13508; apply_merged_regex_fixes 15507` |
| A6 顶层 let | `trans/rust.rs: 14074-14115` |
| A6 backtick | `trans/rust.rs: Expr::FStr 2397; lexer.rs: fstr 542` |
| A7 路径解析 | `parser.rs: parse_ident_or_generic_type 8990-9110; Expected term 3189` |
| A9 方法映射 | `trans/rust.rs: 4449-4450; 5656` |
| A8 skip_self | `trans/rust.rs: spec_decl 10866; fn_decl(skip) 8108` |
| A10 nativeize | `auto-musk/.../nativeize.pl: 135-137` |
