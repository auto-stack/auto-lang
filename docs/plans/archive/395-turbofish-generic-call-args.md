---
plan: 395
title: turbofish-generic-call-args
affects: [auto-lang/parser, auto-lang/ast, auto-lang/a2r]
status: complete # draft | in-progress | complete  # 2026-08-07 核查回填：Phase 1-4 全部落地并合入主分支（merge 0ca86a7c），仅 status 字段此前忘改
---

# Plan 395: Auto 调用泛型实参（turbofish）—— `expr.method<Type>(args)`

> **For Claude:**
> - 构建/测试命令：`cargo test -p auto-lang --lib --features test-trans -- tests::a2r_tests`
>   （a2r golden 基线，见 Plan 393 同款命令）；全量回归另加 `tests::a2c_tests` + VM 套件。
> - 动机来源：auto-ai **Plan 021 缺口 3**（serde 迁移）被迫在 retranspile.sh 用 sed 注入
>   turbofish（`node.deserialize::<RoleDecl>()`，见 auto-ai 提交 `ccf7a6a`）——Auto 无
>   turbofish 语法，a2r 无法从 `Ok(s)` 模式绑定推断 `deserialize<T>` 的 `T`（E0282）。
> - 语法（用户拍板，2026-08-06）：`"42".parse<u32>()`——`<Type>` **直连**方法名，无 `::`。
>   `::` 已被 Plan 391 用作路径分隔符，Rust 式 `parse::<u32>()` 在 Auto 中不可解析。
> - worktree 约定：实施时开 `plan-395/turbofish-call-args`（`D:/autostack/auto-lang-395`）。
> - 范围：**仅 auto-lang 语言/a2r 能力**。auto-ai 侧的消费迁移（.at 改语法 + 删 sed）是
>   本计划完成 + 重建 auto.exe 之后的后续（见 §7），不属本计划。

---

## §1 Goal / 目标

给 Auto 增加**调用泛型实参**语法：`expr.method<Type1, Type2>(args)` 与自由调用
`fn<Type1, Type2>(args)`，a2r 转译成 Rust turbofish `expr.method::<T1, T2>(args)` /
`fn::<T1, T2>(args)`。

- **直接动机**：`node.deserialize<ClientScalars>()` 消除 auto-ai 的 sed workaround
  （loader.at 3 条 + role_config.at 1 条 turbofish sed）。
- **间接收益**：`json.decode[T]`（Index hack，rust.rs:3569）等历史变通可逐步迁移为
  正规语法；未来任何需要显式泛型实参的桥接 Rust 调用（`parse::<T>`、`collect::<Vec<_>>`…）
  都可用 Auto 原生表达。
- **不改变现有语义**：`a < b` 比较、`List<str>` 类型实例作值、`Foo<Bar>(x)` 泛型类型
  构造（GenName 路径）全部保持现状（回溯消解保证零回归）。

---

## §2 现状与根因

### 2.1 `<` 在表达式位置的现状

- `<` 是二元比较 `Op::Lt`（`infix_power` 返回 `PREC_CMP`=8，`parser.rs:129`）。
- `"42".parse<u32>()` 今天**静默错解析**为 `(("42".parse) < u32) > ()`（比较链），
  无任何错误提示——这是最危险的部分：不拦截就会悄悄产生错误语义。
- 表达式位置唯一处理 `<` 的是 `atom()` 的 GenName 类型实例分支
  （`parser.rs:3139-3162`），仅当 ident 是**已知类型**（`is_type`）才触发；方法名
  （如 `parse`）不是类型，`<` 落回比较。

### 2.2 既有"调用时指定类型"的三种机制（都不算调用泛型实参）

| 机制 | 语法 | 发射点 | 缺陷 |
|---|---|---|---|
| `json.decode[T](text)` | `[T]` Index hack | `rust.rs:3569-3599` → `serde_json::from_str::<T>(&text)` | 借 Index 语法绕过，仅限 json.decode 特判 |
| `.to(int)` / `.to(str)` | 类型作**实参** | `rust.rs:3210-3265` → `.parse::<i32>().unwrap()` | 类型是参数不是语法，仅限 to 特判 |
| `str.to_uint(x)` | 硬编码 | `rust.rs:4762-4767` → `.parse::<u64>().unwrap_or(0)` | 类型写死在转译器源码 |

三者均为**特判改写**，无一读自 AST 的显式泛型实参字段。

### 2.3 `Call.type_args` 已有字段不可复用

`ast/call.rs:12-14`：
```rust
pub type_args: Vec<(Name, Type)>,   // Plan 061: 泛型参数名 → 具体类型（推理绑定）
```
由 `infer/expr.rs:587-629` 在解析后填充（如 `duplicate(42)` → `[("T", Int)]`）。
语义是**推理产物**，且带参数名；用户显式实参（无名、多实参）塞进去会与推理冲突。
→ 需独立字段 `generic_args: Vec<Type>`。

### 2.4 E0282 实证（动机，auto-ai 侧）

`loader.at`：`is client_node.deserialize() { Ok(s) -> ... }` → a2r 输出
`match client_node.deserialize() { ... }` → **E0282**（`T` 无法从无类型 `Ok(s)` 推断）。
`retranspile.sh` 用 3 条变量名锚定 sed 注入 `::<ClientScalars/DaemonScalars/ProviderScalars>`。
每新增一个 `deserialize()` 调用点要加一条 sed——脆弱、不可扩展。本计划给出正规解法。

---

## §3 设计

### 3.1 语法定义

```
expr . method < Type (, Type)* > ( args )     # 方法调用带泛型实参
ident  < Type (, Type)* > ( args )            # 自由调用带泛型实参
```

- `<...>` 内复用 `parse_type()` 全部类型语法：`uint`、`str`、`ClientScalars`、
  `List<str>`（嵌套，`>>` 两个 Gt token）、`?T`（Option 后缀）。
- 零实参 `< >` 不允许（无意义，不提供）。
- **歧义消解规则**：仅当 lhs 是 callable 形状 **且** `<` 后是类型起始 **且** 完整解析出
  `<types>` 后紧跟 `(` 时，才按泛型调用；否则整体回退为普通 `<` 比较（见 3.3 回溯）。

### 3.2 AST — `Call` 加字段 `generic_args: Vec<Type>`

```rust
pub struct Call {
    pub name: Box<Expr>,
    pub args: Args,
    pub ret: Type,
    pub type_args: Vec<(Name, Type)>,   // 既有：Plan 061 推理绑定（不动）
    pub generic_args: Vec<Type>,        // 新增：用户显式泛型实参（turbofish）
    pub pos: Option<Pos>,
}
```

- **构造点全量更新**（编译错误逐一补齐，~26 处）：
  `parser.rs`(2243 collect`!` 脱糖 / 2394 enum 变体构造 / 2460 方法调用合并 /
  10934 主 `call()`)、`infer/expr.rs`(1071, 1075)、`ui/handler_codegen.rs`(123, 1256)、
  `ui_gen/ark_adapter.rs`(763, 867, 885, 903, 990)、`ui_gen/kotlin_adapter.rs`(732, 862)、
  `ui_gen/ts_adapter.rs`(1117, 1127, 1195, 1288, 1311)、`vm/codegen.rs`(12043, 12460)、
  `ast/call.rs` 测试(393, 406, 423)、`vm/tests_closures_borrow_check.rs`(191)。
- **Display/ToAtom**（`ast/call.rs:36-348`）补 `generic_args` 输出（错误信息/往返打印）。
- **后端无破坏**：a2r/c.rs/VM codegen 均以 `if let Expr::Call(call) = ...` 字段访问
  消费，无 `Call` 结构化解构——加字段只影响**构造点**，不影响匹配点。

### 3.3 Parser — `expr_pratt_with_left`（parser.rs:1993）加 `<` 拦截

**新增 parser 字段** `pending_generic_args: Vec<Type>`（构造器初始化 `Vec::new()`）。

**拦截点**：循环顶部（`::` 路径分隔处理之后、`let mut op = ...` 之前）：

```
if is_kind(Lt) 且 lhs 是 callable（Ident | Dot(_,Ident) | Bina(_,Dot,Ident)）:
    守卫: < 后一 token 是类型起始
          （lexer.next()+push_token 单 token 前瞻，同 parser.rs:2030 `::` 处理模式）
    尝试（克隆式回溯）:
        保存 cur + lexer.buffer + prev（克隆）
        consume '<'; types = [parse_type()] + (',' parse_type())*
        若 !is_kind(Gt) → 恢复，回落
        consume '>'
        若 !is_kind(LParen) → 恢复，回落（普通比较链）
        成功 → pending_generic_args = types; continue
    （下一 token 必为 '('，LParen 臂触发构造 Call）
```

**LParen 臂**（parser.rs:2210）：`let args = self.args()?;` 后取走
`pending_generic_args` 挂到 `self.call(lhs, args)` 构造的 `Call.generic_args` 上。

**回溯的保存/恢复**：parser 无现成 checkpoint。方案：克隆 `self.cur`、`self.prev` 与
`self.lexer.buffer`（VecDeque<Token>，`push_token` 已证明可回推）；恢复即回写三者。
若 `lexer.buffer` 字段不可见，则给 Lexer 加 `save_state()`/`restore_state()` 两个小方法。
`chars` 迭代器位置不必恢复——已 lex 的 token 在 buffer 中，重消费不碰 chars。

**`node_or_call_expr` 的 GenName 分支扩展**（parser.rs:10641，参数位置/语句位置一致性）：
```
ident 后是 '<' 且 next_token_is_type():
    当前：无条件 → GenName("ident<T>")
    改：解析 <types> 后，若下一 token 是 '(' → 设 pending_generic_args + 返回 Expr::Ident，
        让 Pratt 循环的 LParen 臂构造泛型调用；否则维持 GenName（List<str> 作值不变）
```

**无 lexer 改动**：`Lt`/`Gt` token 已存在（`token.rs:46-49`）；`>>` lex 成两个 Gt，
嵌套泛型由递归 `parse_type` 逐层消费。

### 3.4 a2r — generic_args 非空时输出 turbofish（trans/rust.rs）

两个发射点：

1. **方法调用 Dot 路径**（`rust.rs:6362`）：
   ```rust
   // 现: write!(out, ".{}(", rust_name)?;
   // 改: write!(out, ".{}", rust_name)?;
   //     if !call.generic_args.is_empty() { write!(out, "::<{}>", join(rust_type_name)) }
   //     write!(out, "(")?;
   ```
2. **自由调用/fall-through**（`rust.rs:7284-7285`）：
   ```rust
   // 现: self.expr(&call.name, out)?; write!(out, "(")?;
   // 改: 中间插 write!(out, "::<{}>", join(rust_type_name))
   ```

类型映射复用 `rust_type_name`（`rust.rs:1140`）：`uint`→`u32`、`int`→`i32`、
`str`→`String`、`bool`→`bool`、`ClientScalars`→`ClientScalars`、
`List<str>`→`Vec<String>`、`Map<str, T>`→`std::collections::HashMap<String, T>`。
裸 `u32` 解析为 `Type::User("u32")`，`rust_type_name` 原样输出——用户示例语法
`"42".parse<u32>()` 直接可用，无需 Auto 内置 u32 类型。

**特判改写不动**：`str.to_uint`、`json.decode[T]`、`.to(Type)` 等先于 fall-through
返回的特判分支忽略 `generic_args`（对这些方法写 `<T>` 无意义，不报错也不输出）。

**信任源码原则**：a2r 无桥接 Rust 方法签名注册表，无法判断目标方法是否真泛型；
`<T>` 是 .at 作者显式声明 → 无条件输出 turbofish（非泛型方法会编译失败，由作者负责）。

### 3.5 a2c + VM — 零改动

- a2c（`trans/c.rs`）：`call()` 以字段访问消费，新字段静默忽略（C 无 turbofish；
  泛型是 mangle 名）。注释注明。
- VM（`vm/codegen.rs`）：同上，字段忽略（运行时类型擦除，`<T>` 仅编译期语义）。

---

## §4 实现方案（分步）

### Phase 1 — AST 字段（机械）
1. `ast/call.rs` Call 加 `generic_args: Vec<Type>`；Display/ToAtom 补输出。
2. 全仓库 ~26 个 `Call { ... }` 构造点加 `generic_args: Vec::new()`（编译错误驱动）。
3. `cargo check`（auto-lang crate）通过。

### Phase 2 — Parser
4. Parser 加 `pending_generic_args: Vec<Type>` 字段 + 构造器初始化。
5. `expr_pratt_with_left` 循环顶部 `<` 拦截 + 克隆式回溯（或 Lexer save/restore 方法）。
6. LParen 臂挂载 generic_args。
7. `node_or_call_expr` GenName 分支扩展（`(` 跟随才路由泛型调用）。
8. 手验：`a < b` 仍是比较；`"42".parse<uint>(x)` 解析出带 generic_args 的 Call。

### Phase 3 — a2r
9. `rust.rs:6362`（Dot 方法）+ `rust.rs:7284-7285`（fall-through）两发射点插 turbofish。
10. 手验转译：`node.deserialize<ClientScalars>()` → `node.deserialize::<ClientScalars>()`。

### Phase 4 — 测试 + 回归
11. 新 golden 用例（见 §5）。
12. `src/tests/a2r_tests.rs` 注册 `#[test]`。
13. 全套回归：a2r + a2c + VM，零新增失败。

---

## §5 验证方案

### 5.1 新 golden 用例

`test/a2r/05_expressions/NNN_turbofish/turbofish.at` + `turbofish.expected.rs`
（expected 为 a2r 实际输出，byte 级匹配，含 `// Auto-generated by a2r transpiler` 头）：

| 覆盖点 | .at | 期望 a2r 输出 |
|---|---|---|
| 方法 turbofish | `"42".parse<uint>()` | `"42".parse::<u32>()` |
| 自由调用 | `identity<str>("x")` | `identity::<String>("x")` |
| 用户类型 | `node.deserialize<ClientScalars>()` | `node.deserialize::<ClientScalars>()` |
| 多实参 | `pair<int, str>(1, "a")` | `pair::<i32, String>(1, "a")` |
| 歧义消解 | `let c = a < b` | `let c = a < b;`（仍是比较） |
| GenName 不变 | `let t = List<str>` | `let t = List<String>;`（或既有 GenName 渲染） |

测试注册：`src/tests/a2r_tests.rs` 加
`#[test] fn test_05_expressions_NNN_turbofish() { test_a2r("05_expressions/NNN_turbofish").unwrap(); }`
（discovery runner 亦会自动拾取）。

### 5.2 回归命令

```bash
cargo test -p auto-lang --lib --features test-trans -- tests::a2r_tests   # a2r golden 基线
cargo test -p auto-lang --lib --features test-trans -- tests::a2c_tests   # a2c 无回归
cargo test -p auto-lang --lib                                          # VM/infer 无回归
```

**期望**：零新增失败（回溯消解保证 `a < b` 等既有比较解析不变）。

### 5.3 手验形态（端到端动机场景）

```rust
// .at
is node.deserialize<ClientScalars>() {
    Ok(s) -> { /* ... */ }
    Err(e) -> return Err(...)
}
// a2r 输出必须为
match node.deserialize::<ClientScalars>() {
    Ok(s) => { /* ... */ }
    Err(e) => return Err(...),
}
```

---

## §6 风险与实证陷阱

1. **回溯 restore 完整性**：恢复必须覆盖 `cur` + `lexer.buffer` + `prev` 三者；
   `lexer.buffer` 字段可见性未确认——不可见则加 Lexer `save_state`/`restore_state`。
2. **`pending_generic_args` 顺序安全**：拦截强制 `(` 紧跟 `<types>`，下一循环迭代必
   命中 LParen 臂，无嵌套交错风险（内层调用先于外层消费自己的 pending）。
3. **GenName 分支改造的边界**：仅当 `>` 后紧跟 `(` 才路由泛型调用；`List<str>` 作值
   （无 `(`）必须维持 GenName——这是既有代码的依赖。
4. **旧 auto.exe PATH 陷阱**（沿用 Plan 393 §6 教训）：worktree 测试必须用
   `worktree/target/debug/auto.exe` 全路径，PATH 指向主仓库 master 旧构建会导致
   "改对了但从不生效"的假象。
5. **`>>` 嵌套关闭**：`foo<List<str>>()` 需要递归 parse_type 逐层消费两个 Gt token；
   与类型位置 `List<List<int>>` 行为一致，无新代码。

---

## §7 后续（auto-ai 侧）→ ✅ 消费迁移已完成（2026-08-06）

- `loader.at` / `role_config.at`：`is node.deserialize<ClientScalars>()` 等替换
  `is node.deserialize()`，**删除** ai-config/auto-ai-agent 两个 retranspile.sh 的
  turbofish sed（`client_node/daemon_node/provider_node` 锚定的 3 条 + `RoleDecl` 1 条）。
  **已闭环**：用本 worktree 的 auto.exe 重跑 retranspile，三转译 crate 0 错 +
  workspace 全绿 + 测试全绿；生成的 loader.rs/role_config.rs 为原生 `::<T>`（无 sed 痕迹）。
- 可选：`json.decode[T](text)` 迁移为 `json.decode<Type>(text)`（rust.rs:3569 特判
  改为读 `generic_args`）。**未做**（auto-ai 仅 auto-ai-client/lib.at:107 一处使用，
  Index hack 仍工作；迁移需改 a2r 特判，留作 follow-up）。
- 验证：auto-ai 三个转译 crate 0 错 + workspace 全绿 + rust-ref 测试全绿。**✅**

---

## §8 实施记录（2026-08-06，worktree `plan-395/turbofish-call-args` @ `D:/autostack/auto-lang-395`）

### Phase 1 — ✅ 闭环
`Call` 加 `generic_args: Vec<Type>` 字段（ast/call.rs），25 处构造点加
`generic_args: Vec::new()`（parser 4 + infer 1 + handler_codegen 2 + ark 5 + kotlin 2 +
ts 5 + vm/codegen 2 + tests_closures 1 + ast/call.rs 3），Display/ToNode 补输出。
`cargo check` 0 错。

### Phase 2 — ✅ 闭环
- Parser 加 `pending_generic_args: Vec<Type>` 字段（3 个构造器初始化）。
- `expr_pratt_with_left` 循环顶部 `<` 拦截（callable lhs + 单 token 前瞻守卫 +
  克隆式回溯）：`is_callable_lhs` + `try_parse_call_generic_args`。
- Lexer 加 `LexerState` + `save_state()`/`restore_state()`（`chars` 迭代器位置 +
  计数器 + buffer 全量快照，`buffer` 私有字段不可直接访问）。
- LParen 臂挂载 `generic_args`；`node_or_call_expr` GenName 分支扩展
  （`>` 后跟 `(` 时路由泛型调用，`List<str>` 作值仍 GenName）。
- 手验：`"42".parse<uint>()` / `node.deserialize<ClientScalars>()` /
  `pair<int, str>(1, "a")` / `a < b` 消解 / `List<str>` 值 全部正确。

### Phase 3 — ✅ 闭环
`emit_turbofish_args` 辅助 + **三个**发射点（计划写的是两个，实施发现方法调用有
remap/非-remap 两条路径）：
1. `rust.rs` Dot remap 路径（`.{rust_name}` 后）——remap 表命中的方法。
2. `rust.rs` Dot 通用 fallback（`.{method_name}` 后，6867 区）——**非-remap 方法
   （`deserialize`/`parse` 等）实际走这里**，计划 §3.4 的"两发射点"漏了这条。
3. `rust.rs` 自由调用 fall-through（7284-7285）。
类型经 `rust_type_name` 映射（uint→u32、str→String、User→名称）。

### Phase 4 — ✅ 闭环
- 新 golden：`test/a2r/05_expressions/012_turbofish`（方法/自由调用/用户类型/
  多实参/`a < b` 消解/`List<str>` 值，6 覆盖点全验）。
- `src/tests/a2r_tests.rs` 注册 `test_05_expressions_012_turbofish`。
- 回归：a2r golden **330/0**（含新用例）；gdscript **63/0**；tscn **12/0**；
  全量 lib **2829/22——与基线逐项一致（22 个失败为既有，零新增）**。

### 附：实施中发现的两个计划外问题
1. **`Expr` 枚举 +24B 栈帧税**（`generic_args` 使 Call/Expr 变大）：深递归转译路径
   （dodge_player 的 gdscript + tscn 两测试）从 2MB 默认栈溢出。按代码库既有模式
   （`test_a2r_deep`）加 `test_a2gd_deep`/`test_a2tscn_deep`（16MB 线程）修复，
   非功能回归（大栈下 0.01s 通过）。
2. **crossbeam_spawn 栈溢出为既有问题**（干净基线 f3ab1632 同样溢出，与 Plan 395
   无关），回归时以 `--skip` 排除。

### 验收
- a2r golden 330/0（含 012_turbofish）；gdscript 63/0；tscn 12/0；
  全量 lib 与基线逐项一致（零新增失败）。
- 动机场景手验：`is node.deserialize<ClientScalars>() { Ok(s) -> ... }` →
  `match node.deserialize::<ClientScalars>()` ✓。
- **待办（本计划完成后的 auto-ai 消费迁移，见 §7）**：重建 auto.exe → 改 .at 删 sed。
