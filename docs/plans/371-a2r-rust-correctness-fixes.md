# Plan 371: a2r → Rust 正确性修复（解锁 Auto 版 auto-ai-agent 跑通）

> **状态**：部分实施（2026-07-25，分支 `plan-013/react-runnable`）
> **来源**：plan-013（auto-ai 移植）阶段 6 的真实阻断——组装好的 rust/ crate
> 有 ~344 个 cargo 错误，经最小复现 + 源码追踪，定位到 **3 个系统性 a2r 缺陷**。
> **目标**：修好这 3 个 a2r 根因，让 `crates/auto-ai-agent/rust/` 通过
> `cargo check` 并跑通一个真实 ReAct 问答。修复对所有未来 a2r 输出都生效。

## 实施进度（2026-07-25）

| 缺陷 | 状态 | 验证 |
|---|---|---|
| **C**（i+1 auto-borrow） | ✅ 已修已验证 | driver.a2r.rs 的 `result.as_str()`/`agent_result.as_str()`（enum/struct 参数错加的）消除；真 String→&str 的 `.as_str()` 保留 |
| **A**（spec 跨模块解析） | ✅ 已修已验证（两条路径） | Phase 1.5 预注册 spec（project-merged 路径）+ 单文件路径的 sibling 扫描 + `rust_type_name` 的 Type::User 守卫。`role Role`/`client Client` 现都正确译为 `Box<dyn Role>`/`Box<dyn Client>`。E0782 全消 |
| **B**（Option 方法） | ✅ 已修已验证 | `args.get(key).as_string()` 译为 `a2r_std::json::as_string_opt(...)`（运行时助手已存在，现正确 emit）。最小复现 + driver.at/skill.at 验证通过 |

**三个诊断出的系统性 a2r 缺陷全部修完。** 剩余 ~363 个 cargo 错误是 **B1 类
codegen 细节问题**（int/uint 混用如 `u32 - i32`、缺 `#[derive(Clone)]`、trait
bound 缺失等），是**另一类、逐个错误**的工作，不属本计划（371）的 3 个系统性
缺陷。这类问题的修法见下「剩余工作」。

### ⚠️ 关键发现：缺陷 A 的修复受限于"调用模型"

`auto trans --path X.at rust`（单文件入口，`rust.rs:12217 transpile_rust()`）用空
`TypeStore` 起 parser，**完全不经过 Phase 1.5 预注册**（Phase 1.5 只在
`transpile_rust_project_merged` / `auto build -r rust` 路径里）。所以单文件 transpile
时，任何跨模块 spec（`use role_def: Role`）都解析为 `Type::User`（裸名），缺陷 A 的
Phase 1.5 修复帮不上。

我的 rust/ 组装用的是"逐文件 transpile + 手工拼接"，因此踩到这个限制。两条出路：
1. **改用项目级入口**：给 auto-ai-agent 加 `pac.at`，用 `auto build -r rust`
   （auto-coder 的做法）——这样 Phase 1.5 生效，跨模块 spec 正确解析。但 `build`
   生成自己的文件结构，我手写的 lib.rs/Cargo.toml/client_impl.rs/main.rs 就不适用了。
2. **补单文件路径的 spec 预注册**：在 `transpile_rust()`（`rust.rs:12217`）和
   `trans_rust_with_session()`（`lib.rs:3472`）里，transpile 前用
   `compile.rs:1434 parse_module_to_type_store`（已在 1468-1470 注册 spec）把
   同目录/依赖文件的 spec 填进 parser 的 `type_store`。这样单文件 transpile 也能
   解析跨模块 spec，我的逐文件组装就能用。

**下次会话建议先做出路 2**（补单文件 spec 预注册），改动局部、不动调用模型，最直接
解锁我现有的 rust/ 组装。然后再做缺陷 B。

## 背景与触发

plan-013 把 auto-ai-agent 的 Rust 用 Auto 复刻（`.at`），再用 a2r 译回 Rust
（`.a2r.rs`），组装成 `crates/auto-ai-agent/rust/`（扁平模块 + extern-crate
垫片，照搬 auto-coder 已验证的模式）。组装本身正确（依赖解析、结构、垫片都
对），但 `cargo check` 报 344 错，**全是 a2r 生成的 Rust 不正确**，与 .at 源码
无关（源码经 `auto trans` 验证合法）。错误分布：

| 错误码 | 数 | 性质 | 归属缺陷 |
|---|---|---|---|
| E0308 | 146 | 类型不匹配 | 多为下面三类的级联 |
| E0599 | 37 | 方法找不到 | **缺陷 B**（Option 方法）+ **缺陷 C**（enum .as_str） |
| E0277 | 36 | trait 未实现 | **缺陷 A**（spec 解析顺序）的级联 |
| E0782 | 9 | "expected a type, found a trait" | **缺陷 A**（裸 spec 字段/参数） |
| 其余 | ~116 | clone/borrow/move/名字 | 多为级联 |

修好下面 3 个根因，预计消掉大半错误（其余多为级联，根因消后会自动消失或
大幅减少）。

---

## 缺陷 A（最高优先级）：spec 跨模块/乱序解析→Type::User（应 Type::Spec）

### 现象
- `Arc<Tool>`（同文件、spec 先于使用）正确译为 `Arc<Box<dyn Tool>>`。
- 但裸 spec 字段/参数 `role Role`、`client Client`（跨模块 `use role_def: Role`
  后使用）译为裸 `Role`/`Client`，Rust 需要 `Box<dyn Role>` → E0782。
- 每个 builtin_role 文件还各自重复声明一个本地 `trait RoleTrait`（而非引用
  中心 `trait Role`），是同一根因的另一种表现。

### 根因（已定位到行）
**不是 codegen bug，是 parser 的 spec 解析顺序问题。** a2r 的类型 lowering
其实一致且正确：`Type::Spec(X)` 恒为 `Box<dyn X>`（`rust.rs:875`），
`Type::User(X)` 恒为裸 `X`（`rust.rs:873`）。问题在 parser 决定把名字解析成
哪个变体：

- `parser.rs:922 lookup_type()`：只有当 spec 已注册进 `type_store.spec_decls`
  时才返回 `Type::Spec`（`parser.rs:957-959`）；否则 fallthrough 到
  `Type::User` 占位符（`parser.rs:980-981`）。
- spec 在 `parser.rs:7734 define()` 时注册。**同文件、spec 先于使用**总是成立
  （如 `tool.at` 的 `pub spec Tool` 在第 26 行，`Arc<Tool>` 在第 71 行）。
- 但**跨模块/乱序**失败：
  - 单文件入口 `rust.rs:12217 transpile_rust()` / `lib.rs:3472
    trans_rust_with_session()` 用空 `TypeStore` 起 `Parser`，不调
    `resolve_uses()`，imported spec 从不注册。
  - project-merged 入口 `rust.rs:12883 transpile_rust_project_merged()` 的
    Phase 1.5 预注册（`rust.rs:12932-12993`）**只处理 `type`/`enum`，不处理
    `spec`**；Phase 2 按 `discover_modules` 顺序解析（用方模块常先于定义模块）。
  - 结构体之所以没事，是因为 Phase 1.5 预注册了它们；spec 没这待遇。

### 修复（根因，让 spec 解析与 struct 一样顺序无关）
**主修点**：`trans/rust.rs:12951-12961`（project-merged 的 Phase 1.5 预扫描）。
在 `type`/`enum` 分支旁加 `"spec "` / `"pub spec "` 分支，调
`store.register_spec_decl(...)`（或至少注册一个只含名字的 `SpecDecl::new(name,
vec![])`）。这样跨模块/乱序的 spec 也能在 `lookup_type` 走 `Type::Spec` 分支
（与 struct 同机制）。完整 SpecDecl（含方法）在 Phase 2 解析时填入；名字足够
让 `lookup_type` 选对分支。

**辅助**（单文件路径，若要支持跨文件裸 spec）：
- `lib.rs:3500-3527`（sibling 扫描，现仅收集 struct 字段）和
  `rust.rs:12217-12264`：解析前用 `compile.rs:1434 parse_module_to_type_store`
  （已在 `compile.rs:1468-1470` 注册 spec）把依赖文件的 spec 填进 parser 的
  `type_store`。

**为何这是最佳修法**：修 AST 本身，让所有下游消费者（type inference、call-site
boxing flag `rust.rs:7483`、`is_spec_param` `rust.rs:6257`）一致看到
`Type::Spec`。单在 `rust_type_name` 兜底（把已知 spec 名的 `Type::User` 改写为
`Box<dyn>`）会留下 `spec_param_flags` 仍算 false → call-site 不发 `Box::new()`。

### 验证
- 加测试：`test/a2r/12_specs/` 现只覆盖 `as Flyer`/`[]Flyer`/`Arc<Tool>`，**缺
  裸 spec 字段 + 裸 spec 参数的测试**（这正是回归漏检的原因）。补一个
  `bare_spec_field_param.at`：跨两个模块（`spec S` 在 mod_a，`type T { f S }`
  在 mod_b），断言 `cargo check` 生成 `Box<dyn S>`。
- 对 auto-ai-agent：re-transpile agent.at 后，`Agent.role`/`client` 应为
  `Box<dyn Role>`/`Box<dyn Client>`，builtin_roles 应 `impl Role for X`（而非
  各自 `trait RoleTrait`）。E0782 全消。

---

## 缺陷 B：Option(?T) 上的方法调用未做 optional dispatch

### 现象
`args.get("path").as_string()` 译为 `args.get(&"path").as_string();`——
`JsonValue.get` 返回 `JsonValue?`（`stdlib/auto/json.at:70`），Rust 是
`Option<&Value>`，没有 `.as_string()` → E0599。

### 根因（已定位到行）
- Auto 的 VM 对 `?T` 做可选方法分发（None→默认/短路，Some(v)→`v.method()`）。
- a2r 的方法调用 lowering（`Expr::Dot(object, method)` 在 `fn call()` 的
  `rust.rs:4214-4218` 起）**完全不检查 receiver 是否 `Type::Option`**，直接
  1:1 译成 `object.method()`。
- 运行时助手 `a2r_std.rs:322 as_string_opt(val: Option<&Value>) -> String`
  **已存在且正是此用例的 lowering 目标**，但 transpiler 从不 emit 它
  （只对**模块形式** `json.as_string(val)` 发 `a2r_std::json::as_string`，
  `rust.rs:3574-3582` 等；方法形式不在 `Expr::Dot` 分发表 `rust.rs:4218+`
  和重命名表 `rust.rs:5265` 里）。

### 修复
在 `rust.rs:4214`（`Expr::Dot(object, method_name)` 进入处）、`4218` 的
`match method_name` **之前**，加一个 Option-aware 守卫：
1. 用 `infer_type_from_expr`（`rust.rs:6635+`，已在 `6431` unwrap Option）推断
   receiver 类型。
2. 若 receiver 是 `Type::Option(_)`：
   - 方法名映射到已知运行时 op（如 `as_string`→`a2r_std::json::as_string_opt`）
     时，emit 助手调用；
   - 否则按 Auto 的 None 语义 emit `opt.map(|x| x.method(args))`（或
     `opt.unwrap().method()`，取决于语义；auto-ai 用例是"None→空串"，map+unwrap_or
     合适）。

**最小可行版**：先只特判 `as_string`→`as_string_opt`（覆盖 auto-ai 的全部此
类用例：driver.at:420、skill.at:377），再泛化。

### 验证
- 测试：`?JsonValue` 上调 `.as_string()` → 应 emit `a2r_std::json::as_string_opt(...)`。
- 对 auto-ai-agent：driver/skill 的 `args.get(...).as_string()` 消错。

---

## 缺陷 C：self-方法调用的 `i+1` auto-borrow 把 .as_str() 错加到 enum/struct 参数

### 现象
`self.dispatch(result, task_msg, last_handoff)`（`result` 是 enum
`AdvanceResult`）译为 `self.dispatch(result.as_str(), ...)`——enum 没有
`.as_str()` → E0599。`self.build_handoff(role_id, agent_result, content)`
（`agent_result` 是 struct）同样被错加 `.as_str()`。

### 根因（已定位到行）
`rust.rs:5786-5802` 的 auto-borrow 启发式有个 `i+1` 前瞻：
```rust
let is_str_param = if obj_is_type_chain { ... }
  else { flags[i] || flags[i+1] };   // 5791-5794
if is_str_param && !is_str_slice_var(arg) && !is_int_var(arg) {
    write!(out, ".as_str()")?;
}
```
`fn_str_param_indices`（`rust.rs:7444-7457`）按**裸函数名**键控、**不含 self
槽**。对 `build_handoff(role_id str, result AgentResult, content str)`，flags=
`[true, false, true]`。`i+1` 移位（本意为 flags 含 self 时补偿）读到**下一个**
参数的 flag：
- arg1(`agent_result`/struct): `flags[1]=false || flags[2]=true` → **true** → 错加。
- 对 `dispatch(result, task_msg, last_handoff)` flags=`[false,true,false]`：
  arg0(`result`/enum): `flags[0]=false || flags[1]=true` → **true** → 错加。

守卫 `is_str_slice_var`/`is_int_var`（5797-5799）救不了（enum/struct 两者皆非）。
**注意**：真 String→&str 的情形（`content`、`step_id`）`.as_str()` 是**对的**，
不能一刀切关掉 auto-borrow。

### 修复
**首选（类型感知）**：`i+1` 前瞻改成查 `local_var_types`/`fn_param_types`
（**已填充**：局部在 `rust.rs:7254`，参数全类型在 `rust.rs:7467`）——只在参数
实际类型是 str/String 时才加 `.as_str()`，enum/struct 不加。这彻底解决且不
误伤真 String→&str。

**备选（最小改动）**：直接去掉 `i+1` 前瞻（`rust.rs:5791-5794` 改成只用
`flags[i]`），但需确认无 `flags` 含 self 的合法 str-参数场景被误伤（实测
auto-ai-agent 内的真 str 参数都靠 `flags[i]` 正确触发，`i+1` 是多余且有bug的）。

### 验证
- 测试：`fn f(result SomeEnum, s str)` + `self.f(result, s)` → 应只给 `s` 加
  `.as_str()`，不给 `result` 加。
- 对 auto-ai-agent：driver 的 `result.as_str()`/`agent_result.as_str()` 消错。

---

## 实施顺序与验收

1. **缺陷 C 先做**（最小、最孤立、风险低）：改 `rust.rs:5791-5794` 的 `i+1`
   前瞻为类型感知。re-transpile driver.at，确认 enum/struct 参数不再被加
   `.as_str()`。
2. **缺陷 B**：加 Option-aware 方法 lowering（先特判 `as_string`→`as_string_opt`）。
3. **缺陷 A**（影响面最大、最高杠杆）：Phase 1.5 预注册 spec。补裸 spec
   字段/参数的回归测试。
4. 每修一个，re-transpile 全部 auto-ai-agent .at，跑 `cargo check`，记录错误数
   下降曲线（预期 A 修完后大半 E0277/E0782/E0308 级联消解）。
5. **最终验收**：`cd crates/auto-ai-agent/rust && cargo check` 0 错误 →
   `cargo run` 打印出 LLM 真实回答（daemon 在跑，glm-5.2 可用）。

## 范围外（明确记账）
- 全栈自举（plan-013 选项 B：扩 a2r-std http + Auto 重写 complete）→ 独立后续。
- 流式（plan-013 §D 阶段 2/3）→ 独立后续。
- workflow.at 补全 → 独立后续。
- a2r 的其它已知 codegen 小毛病（缺 derive、borrow/move 细节）→ 若修完 A/B/C
  后仍有零星错误，按需处理；预计 A/B/C 修好后剩余应为个位数、可手修。

## 风险
- 缺陷 A 的 Phase 1.5 改动触及 project-merged 路径的核心，需确保不破坏现有
  `test/a2r/` golden（283 个）。改完跑全量 a2r 测试。
- 缺陷 B 的泛化（非 as_string 的 Option 方法）需确认 Auto 的 None 语义统一，
  避免引入"该短路却 unwrap"的新 bug。先特判、后泛化。
