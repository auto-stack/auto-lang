# 修复 Parity Workaround — Auto 编译器 Bug 修复计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or superpowers:executing-plans. Steps use checkbox (`- [ ]`) syntax.

**Goal:** 逐个修复 Plan 355 parity 验证发现的 19 个 Auto VM/转译器 bug，使 parity 库的 workaround 可被移除或简化。

**Root cause source:** 根因分析来自对 `crates/auto-lang/src/` 的深度调查，详见本文档各 Task。

**Design spec:** `docs/design/rust-library-replication-roadmap.md`, `parity/docs/known-divergences.md`

---

## 阶段总览

| 阶段 | Bug | 风险 | 影响 parity 库 |
|------|-----|------|--------------|
| 1 | A4, A1, A3, A5 — 位运算与字面量 | 低 | sha2, base64 |
| 2 | B4, B5 — bool 相关 | 低 | regex, rusqlite, 全部测试 |
| 3 | C3, C5, C4, E1, C1 — 全局变量与作用域 | 中 | serde_json, regex, base64, sha2 |
| 4 | B1, D1, B6 — 类型系统核心 | 高 | url, serde_json, regex, rusqlite |
| 5 | G1, G2, G3 — 并发 | 中 | tokio |
| 6 | H3 + workaround 移除验证 | 低 | 全部 |

---

# 阶段 1: 位运算与字面量

### Task 1: A4 — 修复 `.shr(n≥32)` 循环移位

**根因:** `vm/native.rs:322-328` `shim_int_shr` 用 `u32::wrapping_shr`，对 n≥32 做 `n & 31` 而非返回 0。

**修复:** `vm/native.rs` — `shim_int_shr` 和 `shim_int_sar` 加 `if n >= 32` 守卫；同步检查 `shim_int_bits_read`、`shim_int_bit_test`。

- [ ] 修 `shim_int_shr`: `if n >= 32 { push_i32(0) } else { push_i32((val as u32 >> n) as i32) }`
- [ ] 修 `shim_int_sar`: 同理用 `if n >= 32 { sign-fill } else { val >> n }`
- [ ] 检查并修 `shim_int_bits_read` / `shim_int_bit_test` 中的 `wrapping_shr`
- [ ] `cargo build && cargo test --lib -p auto-lang -- shr`
- [ ] 验证 sha2 parity: `auto-parity run sha2`
- [ ] Commit

### Task 2: A1 — 修复位运算方法链误算

**根因:** `vm/native_catalog.rs:738-748` 中 `auto.int.*` bitwise native 返回类型注册为 `Void`；codegen `infer_object_type` 对链式调用接收者类型推断失败。

**修复:**
1. `vm/native_catalog.rs:738-748` — 将 `and/or/xor/not/shl/shr/sar/rol/ror/count_ones/...` 返回类型从 `Void` 改为 `Int`（`bit_test` 改 `Bool`）
2. `vm/codegen.rs:8821-8838` `stdlib_method_return_type` — 添加 int bitwise 方法到返回类型表
3. 补充短名（`int.and` 等）到 `fn_return_types` 查找路径

- [ ] 修 native_catalog 返回类型
- [ ] 修 stdlib_method_return_type
- [ ] 验证 `x.and(3).shl(4)` 链式调用在 VM 中正确
- [ ] `cargo build && cargo test --lib -p auto-lang`
- [ ] 验证 base64 + sha2 parity
- [ ] Commit

### Task 3: A3 — 修复 ≥2³¹ 整数字面量解析为 0

**根因:** `infer/expr.rs:65` 中 `Expr::I64(_)` 推断为 `Type::Int`；codegen store 只存 1 slot 但 `CONST_I64` push 了 2 slot，导致高位被丢弃。

**修复:** `infer/expr.rs:65` — `Expr::I64(_) => Type::I64`，`Expr::U64(_) => Type::U64`。验证 codegen 的 `is_two_slot` 判定（`vm/codegen.rs:1838-1856`）会因类型变为 I64/U64 而正确设为 true。

- [ ] 修改类型推断
- [ ] 验证 `let x = 0x80000000` 在 VM 中正确存储
- [ ] `cargo build && cargo test --lib -p auto-lang`
- [ ] 验证 sha2 parity（mk32 workaround 可简化）
- [ ] Commit

### Task 4: A5 — 验证 int 跨调用帧损坏已随 A1 修复

**根因:** 与 A1 同源（返回类型误判导致堆栈布局错）。

- [ ] 写测试验证 sha2 的 `append_hex_byte` 子函数调用返回正确值
- [ ] 若仍有问题，单独修复
- [ ] Commit（如有改动）

---

# 阶段 2: bool 相关

### Task 5: B4 — 修复 bool 跨模块边界损坏

**根因:** bool 返回 shim 用 `push_i32(0/1)`（TAG_I32）而非 `push_nv(encode_bool())`（TAG_BOOL）。EQ 操作码做 raw u64 比较，不同 tag 永远不等。

**修复:**
1. `vm/native.rs` — 搜索所有 `push_i32(0)` / `push_i32(1)` 且声明返回 `Bool` 的 shim，改为 `push_nv(encode_bool(result))`
2. `vm/engine.rs:6026` — EQ/NE 结果也用 `push_nv(encode_bool(...))` 而非 raw sentinel
3. 审计 `vm/native.rs:3694,3706,3718`（str.contains/starts_with/ends_with）等

- [ ] 修复所有 bool 返回 shim
- [ ] 修复 EQ/NE 结果 push
- [ ] `cargo build && cargo test --lib -p auto-lang`
- [ ] 验证 regex + rusqlite parity
- [ ] Commit

### Task 6: B5 — 修复 bool `.to(str)` 永远输出 "true"

**根因:** `encode_bool` 用 `i32::MIN`/`i32::MIN+1` 作 sentinel（均非 0）；`TYPE_BOOL_TO_STR` 用 `pop_i32` 读取后 `if val != 0` 永远为 true。

**修复:**
1. `vm/engine.rs:2487-2495` `TYPE_BOOL_TO_STR` — 用 `decode_bool(task.ram.pop_nv())` 替代 `pop_i32()`
2. `vm/native.rs:5686-5692` `shim_bool_to_str` — 同理

- [ ] 修复两处 bool-to-str 转换
- [ ] 验证 `true.to(str) == "true"` 和 `false.to(str) == "false"` 在 VM 中正确
- [ ] `cargo build && cargo test --lib -p auto-lang`
- [ ] 验证全部 parity 库（bool 字符串化 workaround 可移除）
- [ ] Commit

---

# 阶段 3: 全局变量与作用域

### Task 7: C3 — 修复 for+break+全局变量非法字节码

**根因:** `vm/codegen.rs:2689-2738` `Iter::Cond`（while 风格 for 循环）缺少 `push_scope()`/`pop_scope()`，导致 break 时栈上残留未作用域化的局部变量。

**修复:** `vm/codegen.rs:2689-2738` — 在 body 编译前 `push_scope()`，编译后 `pop_scope()`，与其他 for 变体保持一致。

- [ ] 添加 push_scope/pop_scope
- [ ] 验证 `for global_cond { ... break }` 在 VM 中正确
- [ ] `cargo build && cargo test --lib -p auto-lang`
- [ ] 验证 serde_json parity
- [ ] Commit

### Task 8: C5 — 修复 if 内 var 赋值不可靠返回

**根因:** `vm/codegen.rs:786-818` `Stmt::If` 用 `Stmt::Block` 包装分支，`Stmt::Block` 的 `push_scope/pop_scope` 使 `var` 被限制在块作用域，分支结束后 binding 丢失。

**修复:** `vm/codegen.rs:786-818` — if 分支编译不通过 `Stmt::Block` 包装，或对 `var`（非 `let`）声明提升到外层函数作用域。

- [ ] 修改 if 分支的 var 作用域处理
- [ ] 验证 if 内 var 赋值在分支外可读
- [ ] `cargo build && cargo test --lib -p auto-lang`
- [ ] 验证 regex parity
- [ ] Commit

### Task 9: C4 — 修复调用后 return 栈下溢

**根因:** `vm/codegen.rs:748-749` `needs_pop` 条件 `(!Void || !last_was_native_void)` 应为 `(!Void && !last_was_native_void)`。当 native void shim 没压值但条件允许 POP 时，POP 吃掉调用者栈上的值。

**修复:** `vm/codegen.rs:748-749` — 将 `||` 改为 `&&`。

- [ ] 修改 needs_pop 条件
- [ ] `cargo build && cargo test --lib -p auto-lang`
- [ ] 验证 regex parity
- [ ] Commit

### Task 10: E1 — 修复模块级 const 跨边界不可见

**根因:** `vm/codegen.rs:1248-1250` 只将 `StoreKind::Var` 加入 `global_vars`；`Const`/`Let` 编译为局部变量，随 init task 结束而消失。加载路径（4404-4412）也不查 `import_scope`。

**修复:**
1. `vm/codegen.rs:1248-1250` — 将 `StoreKind::Const`（考虑 `Shared`）也加入 `global_vars`
2. `vm/codegen.rs:4404-4412` — 添加分支：若 `import_scope` 含该名且为值全局，emit `emit_global_load_qualified`

- [ ] 修改 global_vars 准入条件
- [ ] 修改标识符加载路径查 import_scope
- [ ] 验证模块级 const 跨 `use` 可见
- [ ] `cargo build && cargo test --lib -p auto-lang`
- [ ] 验证 base64 + sha2 parity（const workaround 可移除）
- [ ] Commit

### Task 11: C1 — 修复模块级 var str 不可读

**根因:** `vm/codegen.rs:9275-9290` `emit_global_load`/`emit_global_store` 未经 `add_string` 去重就 append 到 string pool；dep merge 后 NanoValue 中的索引错位，decode 出变量名而非值。

**修复:** `vm/codegen.rs:9275-9290` — 全局限名通过 `add_string`（去重）获取索引，确保 pool merge 后一致性。

- [ ] 修改全局名 interning 走 add_string
- [ ] 验证模块级 var str 的 `.char_at` / `.len` 正确
- [ ] `cargo build && cargo test --lib -p auto-lang`
- [ ] 验证 regex parity（参数传递 workaround 可简化）
- [ ] Commit

---

# 阶段 4: 类型系统核心

### Task 12: B1 — 修复 struct 经 Result 跨模块边界损坏

**根因:** `lib.rs:875-892` — `object_keys`/`object_types` pool merge 后，dep 模块的 `CREATE_OBJ` 操作数的 `key_index` 未重映射，指向了主模块的 pool 条目。

**修复:** `lib.rs:875-892` — 扩展 remap walker（或新增 `remap_obj_indices`），对 `CREATE_OBJ` 的 u16 `key_index` 做重映射，与 string pool remap 同理。

- [ ] 实现 CREATE_OBJ key_index remap
- [ ] 验证 struct 经 Result Ok 跨 `use` 边界正确
- [ ] `cargo build && cargo test --lib -p auto-lang`
- [ ] 验证 url + rusqlite parity（struct workaround 可移除）
- [ ] Commit

### Task 13: D1 — 修复递归 enum 解析失败 + payload 绑定失败

**根因:**
1. `parser.rs:4333-4411` — enum 名在 body 解析后才注册，递归变体（`Node(Tree, Tree)`）无法解析自身类型
2. `vm/codegen.rs:3072` — `is` 绑定只在 `generic_registry.has_template(variant)` 时提取，否则丢弃 bindings

**修复:**
1. `parser.rs:4356` — 在 `parse_enum_body` 前前向声明 enum 名为 `Type::Enum`
2. `vm/codegen.rs:3072-3134` — 当 `has_data_payload` 为 false 但 `tag_cover.bindings` 非空时，仍尝试字段提取
3. `parser.rs:6325-6350` — 对 `Type::User` 出现的 enum，从 type_store 解析变体 payload 类型

- [ ] 添加 enum 前向声明
- [ ] 修复 is 绑定回退逻辑
- [ ] 验证递归 enum（`tag Tree { leaf(int); node(Tree, Tree) }`）可解析和匹配
- [ ] `cargo build && cargo test --lib -p auto-lang`
- [ ] 验证 serde_json + regex parity（扁平化 workaround 可移除）
- [ ] Commit

### Task 14: B6 — 验证 type 方法 + fn new 字段读取错误已随 B1 修复

- [ ] 验证 type 方法 + fn new 在跨模块边界正确
- [ ] 若仍有问题，单独修复
- [ ] 验证 url parity（自由函数 workaround 可移除）
- [ ] Commit（如有改动）

---

# 阶段 5: 并发

### Task 15: G1 — 修复 `.go` spawn 栈下溢

**根因:** `vm/codegen.rs:7735-7743` `Expr::Go` 只 emit 内部表达式 + `SPAWN_GO`，不 push 函数地址和参数数。engine 的 `SPAWN_GO` 期望栈上有 `[target, arg_count, args...]`。

**修复:** `vm/codegen.rs:7735-7743` — `Expr::Go` 需正确设置 SPAWN_GO 的栈布局，或改为像 `SPAWN` 一样用立即操作数传递函数地址。

- [ ] 修复 Expr::Go codegen
- [ ] 验证 `~{ ... }.go` spawn 在 VM 中不崩溃
- [ ] `cargo build && cargo test --lib -p auto-lang`
- [ ] 验证 tokio parity（spawn 测试可添加）
- [ ] Commit

### Task 16: G2 — 修复 `Handle[T]` 泛型语法解析失败

**根因:** parser 拒绝类型名后的 `[`。

**修复:** `parser.rs` 类型解析路径 — 允许 `Handle[T]`、`Chan[T]` 等泛型类型语法。

- [ ] 修改类型解析支持 `[` 后跟泛型参数
- [ ] 验证 `Handle[int]` 语法可解析
- [ ] `cargo build && cargo test --lib -p auto-lang`
- [ ] Commit

### Task 17: G3 — 添加 channel 的 Auto 语法绑定

**根因:** CHAN_NEW/SEND/RECV 操作码存在但无 Auto 语法调用。

**修复:** parser + codegen — 添加 `chan_new[T](buf)` / `.send()` / `.recv()` 的语法到操作码绑定。

- [ ] 实现 channel 语法绑定
- [ ] 验证 channel send/recv 在 VM 中工作
- [ ] `cargo build && cargo test --lib -p auto-lang`
- [ ] 验证 tokio parity（channel 测试可添加）
- [ ] Commit

---

# 阶段 6: 转译器 + Workaround 移除验证

### Task 18: H3 — 修复 a2r 重绑定非 Copy 值不 clone

**根因:** `trans/rust.rs:1262-1264` `Expr::Ident` emit 不查 escape tier，非 Copy 值重绑定时 emit move 而非 clone。

**修复:** `trans/rust.rs:1262-1264` — 对 `OwnershipTier::Clone` 绑定的非 primitive 类型，emit 时追加 `.clone()`。

- [ ] 修改 Expr::Ident emit 查 escape tier
- [ ] `cargo build && cargo test --lib -p auto-lang`
- [ ] 验证 url parity
- [ ] Commit

### Task 19: 全量 Parity Workaround 移除 + 验证

逐库移除已修复 bug 对应的 workaround 代码，验证三方仍 100% 一致。

- [ ] base64: 移除位运算单独 let 绑定（A1）、const 重复定义（E1）
- [ ] url: 移除自由函数 workaround（B1/B6）、避免别名（H3）
- [ ] serde_json: 尝试用递归 tag 替代扁平字符串（D1）、移除 loop+return（C3/C4）
- [ ] regex: 移除 int 返回替代 bool（B4）、参数传递替代全局（C1）
- [ ] sha2: 尝试用 hex 字面量替代 mk32（A3）、移除内联（A5）、移除 shr 守卫（A4）
- [ ] rusqlite: 移除 int 替代 bool（B4）
- [ ] tokio: 添加 spawn/join/channel 测试（G1/G2/G3，如已修复）
- [ ] 全量 `auto-parity all` 验证
- [ ] 更新 `known-divergences.md`
- [ ] Commit

---

## 验证矩阵

每个 Task 完成后必须：
1. `cargo build --bin auto` — 编译通过
2. `cargo test --lib -p auto-lang` — 无新回归
3. `auto-parity run <affected_lib>` — 三方一致率不降
4. Commit

## Plan Self-Review

- 19 个 bug 全部覆盖 ✓
- 按 6 阶段从低风险到高风险排列 ✓
- 每个 task 有根因、修复位置、验证步骤 ✓
- 阶段 4（B1/D1）是最高杠杆修复，标注为高风险 ✓
