# Plan 373: a2r B1 类 codegen 细节修复（让 auto-ai-agent rust/ crate 过 cargo check）

> **状态**：✅ 已完成（2026-07-29）— 343 → 0 cargo errors，`cargo run` 跑通 ReAct 问答
> **仓库**：auto-lang（`D:/autostack/auto-lang`）
> **前置**：plan 372（3 个系统性缺陷已修）、plan 013（移植 + rust/ 组装完成）
> **目标**：修复剩余 343 个 cargo 错误，让 `crates/auto-ai-agent/rust/` 通过
> `cargo check`（0 错误），最终 `cargo run` 跑通一个简单 ReAct 问答。
> **验收**：`cd crates/auto-ai-agent/rust && cargo check` → 0 errors →
> `cargo run` 打印 LLM 回答。
> **实际达成**（2026-07-29）：✅ 全部达成。额外推进了 G2（工具端到端）和 G3（REPL），
> 并修复了 a2r `has Spec` lowering 缺陷。详见下文「验收记录与剩余缺口」。

## 背景

plan 372 修了 3 个系统性 a2r 缺陷（spec 解析、Option 方法、auto-borrow），
plan 013 补了 tester.at + 文档同步。但 rust/ 组装 crate 仍有 **343 个 cargo
错误**，全是 B1 类 codegen 细节——a2r 生成的 Rust 不够正确，但每类都有
明确的 a2r 根因和修法。

## 错误全景（343 个，2026-07-29 实测）

### 按错误码分布

| 错误码 | 数量 | 含义 | 主要根因 |
|---|---|---|---|
| E0308 | 135 | 类型不匹配 | i32/u32 混用、Option 解包、enum vs struct |
| E0599 | 44 | 方法找不到 | `.on_event`(10)、`.as_str`(7)、`.substring`(4)、`.to_float`(2)、`.message`(2)、`.clone`(2) |
| E0277 | 37 | trait 未实现 | `Ord`(14)、`PartialOrd`(8)、`Eq`(7)、bound(19) — enum 缺 derive |
| E0658 | 17 | unstable feature | `str_as_str` — a2r 给 `&str` 参数加了多余的 `.as_str()` |
| E0425 | 15 | 名字找不到 | 级联（前序错误导致后续名字解析失败） |
| E0422 | 14 | 名字找不到 | 同上 |
| E0614 | 13 | 不可解引用 | `auto_val::Node` 不可 deref（桥接类型问题） |
| E0507 | 12 | move 借用冲突 | 缺 `.clone()` |
| E0573 | 10 | 期望 item 名 | aggregator 声明不匹配 |
| E0609 | 8 | 字段找不到 | 桥接类型字段名差异 |
| E0608 | 7 | 对非值类型取字段 | Option 上取字段（`?T` lowering） |
| E0382 | 7 | use after move | 缺 `.clone()` |
| E0369 | 4 | 二元操作符不可用 | `i32 - u32`（int/uint 混用） |
| 其他 | 20 | 杂项 | |

### 按文件分布（top 6）

| 文件 | 错误数 | 说明 |
|---|---|---|
| pipeline.rs | 56 | 状态机，大量 enum 构造 + int/uint |
| driver.rs | 53 | 事件构造 + 回调字段 |
| agent.rs | 53 | ReAct 循环，int/uint + async |
| skill.rs | 45 | frontmatter 解析 + JSON |
| roles.rs | 44 | RoleRegistry + 文件 IO |
| role_config.rs | 36 | 桥接类型（auto_val/auto_atom） |

## 修复计划（按根因分组，优先级排序）

### 缺陷 D：i32/u32 类型混用（~57 个错误：E0308 + E0369 + 部分 E0277）

**现象**：Auto 的 `int`(→i32) 和 `uint`(→u32) 是不同类型，a2r 忠实翻译，
但 Rust 不做自动提升。典型：
```rust
let hard_limit: i32 = soft_limit * 5;  // soft_limit 是 u32 → i32 不匹配
let remaining: u32 = hard_limit - turn; // hard_limit 是 i32, turn 是 u32
```

**根因**：Auto 源码自由混用 int/uint（`var turn uint = 0` 但赋给 i32 变量；
`max_turns()` 返回 uint 但参与 int 运算）。a2r 无类型推断/强制转换 pass。

**修法选项**：
1. **a2r 后处理 pass**（推荐）：在 `trans/rust.rs` 的 codegen 后加一个"整型
   宽化"pass，当赋值/运算的两端是 i32/u32 混用时，自动插入 `as i32` / `as u32`
   cast。参照 `local_var_types`（已有）推断目标类型。
2. **.at 源码侧统一类型**：把 `agent.at` 等文件里的 int/uint 统一成 uint
   （或全用 int）。改动面较大、需逐文件审。

**优先级**：高（57 个错误，最大单一类别）

### 缺陷 E：enum 缺 derive（~29 个错误：E0277 Ord/PartialOrd/Eq）

**现象**：a2r 生成的 enum 有 `#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]`
（plan 372 已修过），但某些 enum **仍缺** `Ord`/`Eq`，导致在需要排序/比较的
地方报 E0277。典型：`sort_by_key` 需要 `Ord`，但被排序的 enum 只有 `PartialEq`。

**根因**：a2r 的 enum derive 推断不完整——有些 enum 变体含 `JsonValue`/
`HashMap` 等不可 `Ord` 的类型，a2r 正确地不加 `Ord`；但使用处在 `sort_by_key`
等需要 `Ord` 的场景，源码逻辑上可排序（按 name 字符串排），a2r 没发到。

**修法**：检查每个报 E0277 Ord 的 enum，确认其变体是否含不可 Ord 的类型。
若含→.at 源码侧改用 `sort_by`（闭包比较）替代 `sort_by_key`。若不含→a2r
derive 推断漏了，补。

**优先级**：高（29 个）

### 缺陷 F：多余 `.as_str()` 调用（~17 个错误：E0658 + 部分 E0599）

**现象**：a2r 给已经是 `&str` 的值加了 `.as_str()`，触发 Rust 的
unstable `str_as_str` feature（E0658, 17 个）。典型：
```rust
self.run_inner(task_msg.as_str(), None)  // task_msg 已是 &str
```

**根因**：a2r 的参数传递逻辑对 str 参数统一加 `.as_str()`（auto-borrow），
但接收方已经是 `&str` 时多余。这是缺陷 C 修复的残留——C 修了 enum/struct
参数的误加，但没处理"接收方已是 &str"的情况。

**修法**：在 a2r 的 auto-borrow 逻辑里，检查接收方函数参数类型——如果参数
声明为 `&str`（StrSlice）且实参也是 `&str`，不加 `.as_str()`。或更简单：
**直接在生成的 .rs 里 sed 去掉** `\.as_str()` 当它跟在已经是 &str 的变量后。

**优先级**：中（17 个，但 E0658 是 unstable feature gate，简单 sed 可修）

### 缺陷 G：方法找不到（~22 个 E0599：on_event/as_str/substring/to_float/message/clone）

**子类**：
- **`.on_event`(10)**：driver.at 把 on_event 存为字段，但 a2r 生成的调用
  `self.on_event(ev)` 被当作方法调用（字段是 `fn(PipelineEvent)` 类型，
  应该是 `self.on_event` 后直接 `(ev)`，或 `(self.on_event)(ev)`）。
  修法：a2r 对 fn-type 字段的调用 emit `(self.field)(args)` 而非 `self.field(args)`。
- **`.as_str`(7)**：部分是缺陷 F 的变体，部分是 enum/struct 上调 `.as_str()`。
- **`.substring`(4)**：Auto 的 `str.substring(lo, hi)` 对应 Rust 的
  `&s[lo..hi]`，a2r 直译 `.substring()` 但 Rust str 没有这个方法。
  修法：a2r 把 `.substring(a, b)` 译为 `&self[a..b]`。
- **`.to_float`(2)**：Auto 的 `int.to_float()` 对应 Rust 的 `as f64`。
  修法：a2r 把 `.to_float()` 译为 `as f64`。
- **`.message`(2)**：`AgentError.message()` — a2r 把 enum 的方法调用直译，
  但 Rust enum 方法需要 `impl AgentError { fn message() }` block。如果方法
  定义在 `ext` block 里，a2r 应生成 `impl`。检查 error.a2r.rs 是否漏了 impl。
- **`.clone`(2)**：某些 struct 缺 `#[derive(Clone)]`。

**优先级**：中高（22 个，每个有明确修法）

### 缺陷 H：桥接类型 API 差异（~13 个 E0614 + 部分 E0609/E0608）

**现象**：`auto_val::Node` 不能 deref（E0614），桥接类型字段名不匹配（E0609）。
主要集中在 `role_config.rs`（用 auto_atom/auto_val 解析 .at 配置）。

**根因**：a2r 生成的代码调用桥接 crate（auto_val/auto_atom）的 API，
但 API 签名/语义不完全匹配。这是 plan 013 交接文档 B16 的延续。

**修法**：逐个检查 role_config.a2r.rs 里的桥接调用，对照 auto_val/auto_atom
的真实 Rust API 调整（`.clone()` / deref / 字段名）。大部分是手修生成代码
或调 a2r 的桥接处理逻辑。

**优先级**：中（13 个，集中在 role_config.rs 一个文件）

### 缺陷 I：move/borrow（~19 个 E0507 + E0382）

**现象**：值被 move 后再次使用（E0382），或不可 move 的值被 move（E0507）。
a2r 生成的代码缺 `.clone()`。

**根因**：a2r 在值被多次使用时没自动加 `.clone()`。auto-coder 的
fix_transpiled.py 有类似修复逻辑。

**修法**：a2r post-process pass 检测变量被 move 后再用，自动补 `.clone()`。
或手修生成代码。

**优先级**：中（19 个）

### 缺陷 J：aggregator/module 声明不匹配（~10 个 E0573 + 2 个 E0761）

**现象**：rust/src/lib.rs 声明了 `pub mod xxx`，但 xxx.rs 里的内容不符合
Rust 模块要求（比如文件里声明了重复的 trait/struct 名）。

**根因**：a2r 对跨文件重复定义（如每个 builtin_role 文件都定义 `trait Role`）
没有去重。flat 组装后多个文件定义同名 trait → 冲突。

**修法**：在 a2r 的 spec 处理里，只有 role_def.at 定义 `trait Role`，
其他文件应 `use crate::role_def::Role` 而非重复声明。或手修生成代码去掉重复。

**优先级**：中（12 个）

### 缺陷 K：残余 Box（12 个）

**现象**：`Err(Box::new(...))` 的 Box 没去掉。

**根因**：plan 372 的 enum 检测覆盖了大部分，但 `String` 类型的 Err 仍被
Box。因为 `is_concrete_enum_err` 只检查 enum/struct/tag，不检查 String。

**修法**：在 `is_concrete_enum_err` 的判断里，也检查 `current_fn_err_type`
是否为 String 类型；或在 `Expr::Str` 分支（已有 `.into()` 路径）确保不走
Box::new 分支。

**优先级**：低（12 个）

## 实施建议

### 推荐顺序（按"修一个消多少 + 风险"排）

1. **D（i32/u32，~57 个）**：写 a2r 整型宽化 pass，或先用 sed 脚本批量加 cast。
2. **F（多余 .as_str()，17 个）**：sed 批量去掉 `&str` 上的 `.as_str()`。
3. **E（enum derive，29 个）**：逐个检查 enum，补 derive 或改 sort_by。
4. **G（方法找不到，22 个）**：逐子类修（fn 字段调用、substring、to_float）。
5. **H（桥接类型，13 个）**：手修 role_config.rs。
6. **I/J/K**：收尾。

### 工作区

所有修改在 **auto-lang 仓库**（`D:/autostack/auto-lang`）：
- a2r codegen 修复：`crates/auto-lang/src/trans/rust.rs` + `lib.rs`
- rust/ 组装层手修：`crates/auto-ai-agent/rust/src/*.rs`
- .at 源码侧修（如类型统一）：`crates/auto-ai-agent/src/*.at`

### 重建 + 验证流程（每修一类后跑）

```bash
cd D:/autostack/auto-lang
cargo build --release --bin auto           # 重编 auto.exe（含 a2r 修复）
# re-transpile 全部 .at
base="crates/auto-ai-agent/src"
for f in $(find $base -name "*.at"); do
  ./target/release/auto.exe trans --path "$f" rust
done
# 复制到 rust/src + 手修 Client trait
cd crates/auto-ai-agent/rust/src
# ... (copy + sed agent.rs Client trait)
cd ..
cargo check 2>&1 | grep -c "^error"        # 看错误数下降
```

### 最终验收

```bash
cd crates/auto-ai-agent/rust
cargo check   # → 0 errors
cargo run     # → 打印 LLM 回答（daemon 在跑，glm-5.2 可用）
```

---

## 验收记录与剩余缺口（2026-07-29 最终更新）

### A. 验收达成

```bash
cd D:/autostack/auto-lang/crates/auto-ai-agent/rust
cargo check          # → 0 errors（仅 35 warnings）
cargo run            # → 打印真实 LLM 回答
# [react] talking to daemon at http://127.0.0.1:17654
# [react] task: 你好，请用一句话介绍你自己。
# [react] turns: 1
# [react] answer: 你好！我是由Z.ai训练的GLM大语言模型…
```

### B. 各缺陷的修法分类

"修法" 列指使 cargo check 通过的实际手段。"持久性" 列指 re-transpile 后修法是否
仍然有效（即是否已进 a2r 生成器）。

| 缺陷 | 原始错误数 | 修法 | 持久性 |
|---|---|---|---|
| D (int/uint) | ~20 | 手修 rust/src/*.rs：`as i32`→`as u32`、标注改 `u32` | ⚠️ 部分：`fix_u32_i32_casts` + `fix_integer_type_mismatches`（已有）覆盖了常见模式；剩余的需 a2r 整型宽化 pass（本计划推荐但未做，因为实际 i32/u32 错仅 ~20 个，大部分 E0308 是 String/Option） |
| E (enum derive) | ~36 | (a) 手修：含 JsonValue/ModelTier/Message 的 enum/struct derive 降级为 PartialEq；(b) post_process `fix_non_ord_derives`（本计划新增）自动降级 | ✅ 持久：`fix_non_ord_derives` 已入 post_process 链 |
| F (多余 as_str) | 17 | 手修 sed 去 `param.as_str()`（param 已是 `&str`） | ⚠️ 部分：`.as_str()` 去重依赖调用上下文（接收方是不是 `&str`），未做通用 post_process。实际影响小（新 transpile 的文件不经过这些特定调用路径） |
| G (方法找不到) | ~22 | 手修：`self.on_event`→`(self.on_event)`、`.substring`→`&s[a..b]`、`.to_float`→`as f64`、`.message()`→`.to_string()`/`format!` | ✅ 持久（4/6 子类）：`fix_substring_method`、`fix_numeric_conversion_methods`、`fix_fn_field_calls`（均本计划新增，已入 post_process）；`.message()` 为手修（需 enum 方法体分析）；`.clone` 为手修（缺 `#[derive(Clone)]` 是 struct 级别判断） |
| H (桥接类型) | ~13 | 手修 role_config.rs：`*(*node).clone()`→`node.clone()`、accessor 改 &self 借用、`Config("…")`→`.to_string()` | ⚠️ 手修：role_config.rs 是 a2r-first 桥接文件，本就不走纯 transpile 重建（它的源码 .at 依赖 auto_val/auto_atom，transpile 后仍需手修桥接 API 差异）。本计划修的是 hand-assembled rust/src/ 版本。 |
| I (move/borrow) | ~17 | 手修：`for m in &self.messages`、`step.id.clone()`、`HashMap.get(&k)`、`*n` deref | ⚠️ 部分：`fix_borrowing_issues`（已有）+ `fix_push_move`（已有）覆盖常见模式；剩余的如 `for-in-self.field` 需作用域分析 |
| J (aggregator) | ~12 | 手修：删 `orchestration/mod.rs` + `config/mod.rs`（重复模块）、`Result<None>`→`Result<()>`、`Ok(None)`→`Ok(())` | ✅ 持久（模块删除为一次性）；`fix_result_none_unit`（本计划新增）覆盖 re-transpile |
| K (残余 Box) | ~7 | 手修 sed：`Err(Box::new(X))`→`Err(X)` | ✅ 持久：`fix_residual_error_box`（本计划新增）已入 post_process |
| — (Cargo.toml path) | — | 改同仓依赖为 `../../a2r-std`（4 级跨仓），保持 main checkout 可编译 | ✅ 持久（一次性 manifest 修改） |
| — (builtin_roles) | ~14 | 手修：`impl Role for X` 替换 `trait RoleTrait`+`impl RoleTrait for X` | ⚠️ 已持久化（plan 373 G2）：a2r 的 `has Spec` lowering 现在直接生成 `impl Role for X`。但 rust/src/ 里已有文件不会自动重建（需 re-transpile + 重组装） |
| — (ClientError Clone) | 1 | 手修：AgentError derive 降级为 Debug + 添加 `From<ClientError>` | ✅ 持久（一次性修改 error.rs） |
| — (ContentBlock 结构体) | 2 | 手修：`ContentBlock::Text(resp.content)`→`ContentBlock::Text { text: … }`、`ToolUse(tc.id,…)`→`ToolUse { id: …, … }` | ✅ 持久（plan 373 CB fix）：`seed_known_struct_enum_variants` 注册外部 struct-variant 枚举，a2r 自动输出 `{ field: val }` 构造语法 |
| — (async .await) | ~5 | 手修：在 agent.rs/driver.rs 的多处 async 调用点加 `.await` | ✅ 持久（plan 373 mut-await）：`fn_ret_types` 缓存 + `call_needs_await` 自动在返回 Future/~Result 的调用后追加 `.await` |
| — (&mut self) | ~8 | 手修：`run`/`drive`/`dispatch`/`drive_step` 等方法从 `&self` 改为 `&mut self`（因为它们修改了 self 的字段） | ⚠️ 部分（plan 373 mut-await）：`method_mutates_self` 自动检测直接字段修改（`self.x = ...`）和 mutating 方法调用（`self.x.push(...)`）。**未覆盖间接 mutation**（`self.method()` 内部修改 self）——re-transpile 后约 5 处方法仍需手修 |

### C. 持久性总结

本计划实际达成了两种修复：

**1. 永久性 a2r 修复（已入 `post_process` 链，re-transpile 可重现）：**

| fix_* 函数 | 覆盖的缺陷 |
|---|---|
| `fix_substring_method` | G：`.substring(a,b)` → `&s[a as usize..b as usize]` |
| `fix_numeric_conversion_methods` | G：`.to_float()`→`as f64`、`.to_uint()`→`as u32` |
| `fix_residual_error_box` | K：`Err(Box::new(X))` → `Err(X)` |
| `fix_result_none_unit` | J：`Result<None,E>`→`Result<(),E>`、`Ok(None)`→`Ok(())` |
| `fix_fn_field_calls` | G：`self.field(args)` → `(self.field)(args)`（fn 类型字段） |
| `fix_non_ord_derives` | E：含非 Ord 字段的 enum/struct derive 降级 |
| `fix_missing_trait_impl_uses` | —：缺失的 `use crate::wire::JsonValue;` 等自动补 |
| `fix_string_literal_enum_args` | —：`ToolError::Args("msg")`→`.to_string()` |
| `has Spec` lowering（`trans()` 主 pass） | builtin_roles + echo tool：`has Tool`→`impl Tool for X`、`~Result`→`async fn`、去 `pub`、`#[async_trait]` on impl |

**2. 一次性手修（已应用到 rust/src/*.rs，但 re-transpile 后需重做）：**

以下修改仅存在于当前 hand-assembled `rust/src/*.rs` 中，不属于 a2r 生成器：

- **间接 `&mut self`**（~5 处：drive_step/dispatch/handle_after_submit 等方法通过调用
  修改 self 的子方法来间接 mutation，`method_mutates_self` 不做跨方法分析，检测不到）
- **手写组装层**（lib.rs/config.rs/orchestration.rs/builtin_roles.rs/client_impl.rs/main.rs）：
  这些是手写的 glue 代码，不在 .at 源码中，不会被 re-transpile 覆盖。它们定义模块
  声明、extern-crate 垫片、JsonValue 别名、Client impl、REPL 入口等
- `#[async_trait]` on Client trait（1 处，agent.rs）
- role_config.rs 桥接 API 适配（~13 处，此文件本就是手修的桥接层）
- `for x in &self.field` 循环改写（~10 处）
- AgentError `From<ClientError>` impl（1 处）
- builtin_roles.rs 的 `Box<dyn Role>` 装箱（1 处）

**2026-07-30 re-transpile 实测**（全部 .at → a2r → 复制到 rust/src/ + 恢复手写 glue）：
**285 个 cargo 错误**（原基线 343 → 现在 285 = 下降 58 个 = 17%）。

错误码分布（285 个）：

| 错误码 | 数量 | 主要根因 |
|---|---|---|
| E0308 | 134 | 类型不匹配：String/&str 混用、Option 未 unwrap、`HashMap.get()` 返回 Option 当值用 |
| E0599 | 38 | 方法找不到：`.as_str()` on PathBuf/char、`.message()` on ClientError、`.clone()` on non-Clone |
| E0658 | 16 | 多余 `.as_str()` on &str（auto-borrow 残留，post_process 未完全覆盖） |
| E0425 | 14 | 名字找不到（级联） |
| E0614 | 13 | 桥接类型 deref（role_config.rs 的 `*(*node).clone()` 等） |
| E0507 | 12 | move/borrow（`for m in self.messages` 应为 `&self.messages`） |
| E0277 | 8 | trait 未实现（derive 降级未完全覆盖） |
| E0609 | 8 | 字段找不到（Option 上取字段） |
| E0382 | 8 | use after move（缺 `.clone()`） |
| E0608 | 7 | 对非值类型取字段 |
| E0252 | 6 | 重复名字（RoleTrait 等合成 trait 冲突） |
| E0195 | 3 | lifetime 不匹配（`#[async_trait]` 未加到某些 impl 块） |
| E0423 | 5 | 保留字冲突 |
| 其他 | 17 | 杂项 |

**结论**：re-transpile 后仍有约 285–300 个错误。组装层自动化实验（rebuild.sh 脚本）
成功恢复了手写 glue 文件（lib.rs/config.rs/orchestration.rs/builtin_roles.rs/client_impl.rs/
main.rs），但 **错误数没有显著下降**（301 vs 无脚本 285）——说明组装层不是主要瓶颈。

真正的瓶颈是 **a2r transpile 产物本身的 B1 细节质量问题**（String/&str、move/borrow、
桥接 deref 等），这些是 transpile 过程中的类型推断/borrow 分析不足导致的，不是组装层
能解决的。

**关键发现**：组装层的缺失主要导致 `lib.rs` 的 E0432（模块找不到），而 B1 细节错误
（E0308/E0599/E0658 等）在有无组装层脚本时**数量基本相同**（~285 vs ~300）。

### 推进的后续路径（优先级重新排序）

| 优先级 | 工作项 | 预计消除 | 性质 |
|---|---|---|---|
| ~~最高~~ | ~~组装层自动化~~ | ~~~100+~~ | **已实验：效果远低于预期（~16 个）**，非主要瓶颈 |
| **最高** | String/`&str` 修复（`.as_str()` 去重 + `.to_string()` 补全 + `HashMap.get` Option unwrap） | ~50 | a2r codegen 质量 |
| **高** | `for x in &self.field` 循环改写 post_process | ~15 | a2r codegen |
| **高** | 桥接类型 deref（`*(*node).clone()`） + `HashMap.get` 返回 Option 处理 | ~20 | a2r codegen |
| **中** | 合成 `RoleTrait` 彻底清除（`has Spec` 旧路径残留） | ~7 | a2r codegen |
| **中** | 间接 `&mut self`（跨方法分析） | ~5 | a2r codegen |

**结论更新**：组装层自动化不是 re-transpile 可重现性的瓶颈。真正的瓶颈是 a2r 生成器
对类型推断和 borrow 分析的深度不足。下一步应聚焦于 **String/`&str` 自动修复** 和
**move/borrow 自动修复** 两个 post_process pass。

### D. 计划外扩展（G2+G3）

本计划完成后，额外推进了"写 .at 工具 + REPL 交互入口"（plan 013 G2+G3 里程碑）：

- **echo.at**（30 行）：极简 `has Tool` 工具，transpile 后零手修可编译
- **REPL**（~50 行 main.rs 改动）：交互式循环，注册 EchoTool，支持多轮对话 + 工具调用
- **验证**：`cargo run` → 模型发起 `echo` 工具调用 → 返回 `ECHO: hello world` → 模型给出最终答案 → 2 turns

这些改动已合入 master（commit `13078c2e`、`9f8e63e2`、`aa221bb9`）。

### E. 建议的后续

按优先级：

1. **`&mut self` 分析 pass**（最大缺口，~15 错误）—— a2r 对改变 `self` 字段的方法加 `&mut self`
2. **async `.await` 插入**（~5 错误）—— a2r 对返回 `Future`/`~Result` 的调用点加 `.await`
3. **ContentBlock struct-variant 构造**（2 错误）—— 需要知道变体是 struct 还是 tuple（当前 a2r 一律当 tuple）
4. **re-transpile 验证** —— 修完上面三项后，走一遍完整的"transpile 全部 .at → 组装 rust/src/ → cargo check"，确认达到同样的 0 错误（即"re-transpile 可重现"）

