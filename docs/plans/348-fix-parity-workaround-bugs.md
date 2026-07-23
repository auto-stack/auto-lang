# 修复 Parity Workaround — Auto 编译器 Bug 修复计划

> 原编号 359；2026-07-23 因编号冲突改为 348（原号保留给 359-auto-as-rust-script-rollout）

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or superpowers:executing-plans. Steps use checkbox (`- [ ]`) syntax.

**Goal:** 逐个修复 Plan 347 parity 验证发现的 19 个 Auto VM/转译器 bug，使 parity 库的 workaround 可被移除或简化。

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

---

# 执行状态（2025-07 更新）

## 已完成

### 阶段 1-6 的 14 个 bug 修复

| Bug | 修复 | 验证 |
|-----|------|------|
| A4 (shr wrapping) | ✅ 加 n≥32 守卫 | sha2 10/10 |
| A1 (位运算链) | ✅ native 返回类型 Void→Int | base64 33/33（简单场景）|
| A3 (字面量解析) | ✅ I64/U64 类型推断 | 单测通过 |
| A5 (int 跨帧) | ✅ 随 A1 修复 | sha2 单测通过 |
| B4 (bool 边界) | ✅ encode_bool | regex/rusqlite bool 移除 |
| B5 (bool to_str) | ✅ decode_bool | 全部测试 |
| C3 (for+break) | ✅ push_scope | serde_json |
| C5 (if var 作用域) | ✅ compile_body_inline | regex |
| C4 (call+return) | ✅ needs_pop \|\|→&& | regex |
| E1 (const 边界) | ✅ Const→global_vars (int only) | 部分 |
| C1 (var str 全局) | ✅ add_string interning | 部分 |
| B1 (struct 边界) | ✅ CREATE_OBJ remap | url 改为返回 struct |
| D1 (递归 enum) | ✅ 非 bug，已加回归测试 | — |
| B6 (type 方法) | ✅ 正常工作 | — |
| G1 (spawn 崩溃) | ✅ Task 22 真正并发（body 编译为 out-of-line 函数 + 捕获环境）| tokio |
| G2 (Handle[T]) | ✅ [] 泛型语法 | tokio |
| H3 (a2r clone) | ✅ Expr::Ident 加 clone | url |

### Workaround 清理

| 库 | 移除 | 保留 | 一致性 |
|----|------|------|--------|
| sha2 | A5 | A1/A3/A4（模块作用域仍坏）| 10/10 |
| base64 | A1 | E1（const str 仍坏）| 33/33 |
| regex | B4/C3 | C1（var str 仍坏）| 45/45 |
| url | B1 → Url struct | DIV-URL-VM-2 | 30/30 |
| rusqlite | B4 | float-in-Result split | 65/65 |
| serde_json | header 更新 | C2（StringBuilder）| 56/56 |

**全部 7 库 257 测试 100% 三方一致，0 回归。**

---

## 剩余工作

以下 bug 在 Plan 348 中尝试修复但**不完整**，需要深入处理。清理过程中确认它们在特定上下文仍损坏。

### 阶段 7: 修复不完整的 bug（3 个）

#### Task 20: E1-补全 — 模块级 `const str` / `var str` 全局变量损坏

**问题:** E1 只修复了 `int` 全局变量。`const str` 和 `var str` 仍损坏：
```
var PAT str = "hello"
PAT.len()        // 返回 3（"PAT" 的长度），而非 5
PAT.char_at(0)   // 返回 'P' 的码点（80），而非 'h' 的码点（104）
```
C1 的 `add_string` interning 修复不完整——全局名字虽然 intern 了，但**存储的 string 值**的 NanoValue 索引在 pool merge 后仍错位。

**根因方向:** `emit_global_store` 存储字符串值时，值的 NanoValue 中的 string pool 索引是 codegen 时的局部索引。当 linker 合并多个模块的 string pool 后，该索引不再指向正确的字符串。`remap_string_indices` 只重写了字节码操作数（LOAD_STR/LOAD_GLOBAL），没有重写**已存储在 globals map 中的 NanoValue**。

**修复方案:**
1. 方案 A：在 linker merge string pool 后，遍历 `vm.globals` 中所有 string NanoValue，用 remap 表更新它们的索引
2. 方案 B：全局变量的值在 init task 运行时存储（运行时 pool 已 frozen），确保存储时用的是最终索引

**验证:**
```auto
var PAT str = "hello"
fn check() int { PAT.len() }
fn main() { print(check().to(str)) }  // 应输出 5
```

**影响:** 移除 base64 E1 workaround（alphabet 重复定义）、regex C1 workaround（参数传递替代全局）

#### Task 21: A1-补全 — 位运算链式调用在模块作用域仍误算

**问题:** A1 修复了简单场景的位运算链（`x.and(3).shl(4)` 在 base64 中工作），但在 sha2 的**模块作用域**复杂链式调用中仍误算：
```
// sha2 mk32 中：返回值不正确
fn mk32(b0 int, b1 int, b2 int, b3 int) int {
    return b0.shl(24).or(b1.shl(16)).or(b2.shl(8)).or(b3)  // 误算
}
```
而拆分为单独 let 绑定则正确。

**根因方向:** A1 修复了 `native_catalog` 的返回类型注册（Void→Int）和 `stdlib_method_return_type`，但 `infer_object_type`（codegen.rs ~8841-8894）对**多层链式调用**的接收者类型推断可能仍有问题。第一层 `.shl(24)` 返回 Int，但第二层 `.or(...)` 的接收者（第一层的结果）类型推断可能失败。

**修复方案:**
1. 在 `infer_object_type` 的 `Expr::Call` 分支中，递归推断内部调用的返回类型，而非只查一层
2. 或在 `stdlib_method_return_type` 中补充对链式调用的支持

**验证:**
```auto
fn mk32(b0 int, b1 int) int {
    return b0.shl(24).or(b1.shl(16))  // 应等于 (b0 << 24) | (b1 << 16)
}
fn main() {
    var r = mk32(1, 2)
    print(r.to(str))  // 应输出 16777216 + 131072 = 16908288... 实际计算 0x01020000
}
```

**影响:** 移除 sha2 的 `mk32` 拆分 workaround、base64 的部分 let 绑定

#### Task 22: G1-补全 — `.go` spawn 的真正并发执行

**问题:** G1 修复了 spawn 的栈下溢崩溃，但 spawn 的 async block body 用 placeholder offset 0 编译，导致 body **同步内联执行**而非真正 spawn 到新 task：
```auto
fn main() {
    var counter int = 0
    ~{ counter = 42 }.go  // body_offset=0, SPAWN_GO 跳过 spawn
    print(counter.to(str))  // 输出 42（因为 body 已同步执行），而非不确定值
}
```

**根因方向:** `Expr::Go` 的 codegen（codegen.rs ~7825）编译 async block `~{ ... }` 时，使用 `CREATE_FUTURE` 但 body 的函数地址是 placeholder 0。需要：
1. 将 async block body 编译为独立的 out-of-line 函数
2. 将该函数的真实地址传递给 SPAWN_GO

**修复方案:**
1. 在 codegen 中，遇到 `~{ ... }.go` 时，将 body 编译为一个闭包/独立函数（类似普通 fn 的编译方式），获取真实地址
2. SPAWN_GO 用该地址 spawn 新 task
3. 处理变量捕获（闭包捕获外部变量）

**验证:**
```auto
fn main() {
    var counter int = 0
    ~{
        counter = 42
    }.go
    // spawn 后立即读取——如果真正并发，counter 可能还是 0
    // 如果同步执行，counter 一定是 42
    // 测试至少确认 spawn 不阻塞主流程
    print(counter.to(str))
}
```

**影响:** tokio 可添加真正的 spawn/join 测试

**状态:** ✅ 已修复（Plan 348 Task 22）。

实施细节：
- `Expr::AsyncBlock` 的 codegen 现在将 body 编译为 out-of-line 函数（参照闭包模式：`LOAD_LOC` 推入捕获值 → `CREATE_FUTURE` 消费捕获 → `JMP` 跳过 body → 编译 body → 回填 JMP）。
- `CREATE_FUTURE` 操作码格式扩展：`body_offset: u32, capture_count: u8, capture_name_idx: u16 * capture_count`。捕获值存入 `FutureValue.captures`。
- body 地址通过 `relocs` + `exports` 注册（与闭包一致），确保外层函数的 `FN_PROLOG`/`RESERVE_STACK` 插入移位后，body 地址仍能正确解析。
- `SPAWN_GO` 从 future 取出 body_offset 和 captures，调用 `spawn_task(body_offset, 1024)`，并合成一个 `Closure` 注册到 `self.closures`，将其 id 写入 spawned task 的 `current_closure_id`，使 body 内的 `LOAD_CAPTURED`/`STORE_CAPTURED` 能解析到捕获值。
- `handle_await_future` 同样安装合成的 Closure（并保存/恢复调用者的 closure 上下文），让 `.await` 路径下 body 内的捕获也能解析。
- 当 body 的最后一条语句不产生值（`last_expr_type == Void`）时，body 末尾自动 `PUSH_NIL`，保证 `.await` 路径 RET 不下溢。

语义：
- `~{ counter = 42 }.go; print(counter)` 输出 `0`（spawned task 在自己的 RAM 中修改自己的 captured `counter`，不回写到调用者）。
- `~{ 42 }.await` 返回 `42`。
- 多个 `.go` 连续 spawn 不崩溃（G1 行为保持）。

回归测试：`plan348_concurrency_tests.rs::test_task22_spawn_does_not_run_inline`、`test_task22_await_returns_body_value`、`test_task22_spawn_then_await_coexist`。

未解决（语言设计层面，非本 Task 范围）：
- 真正的"共享可变状态"并发（需要 `Arc<Mutex<T>>` 或类似的共享通道），目前每个 spawned task 是 RAM 隔离的。
- `Handle[T]` 句柄语法和 `.await` 在 spawn 后获取结果（需要 Task 23/Task 24 的语法扩展）。

### 阶段 8: 语言设计限制（需要新语法，非 bug 修复）

以下不是 bug，而是缺少语言特性。需要独立的语言设计决策。

#### Task 23: G3 — Channel 的 Auto 语法绑定

**现状:** VM 有 CHAN_NEW/SEND/RECV 操作码和 `AutoChannel` 运行时（vm/channel.rs），但无 Auto 语法调用它们。

**需要设计:**
- `chan_new[T](buf int)` 的语法（或 `Chan[T]::new(buf)`）
- `chan.send(val)` / `chan.recv()` 的方法绑定到 SEND/RECV 操作码
- 可能需要 `<-` 操作符（如 Go）或方法调用风格

**影响:** tokio 可添加 channel 测试

#### Task 24: `Result[T, E]` 泛型语法

**现状:** parser 不接受 `Result[T, E]` 泛型语法（`[` 后跟类型参数列表）。只能用 `!T` 表示 Result 或 `Result T` 单参数形式。这导致 rusqlite 的 float-in-Result 无法表达，需要 status/value 拆分。

**需要:** `Result[T, E]` 和 `Option[T]` 的方括号泛型语法（G2 已修复了 `Handle[T]`，但 Result/Option 可能需要额外处理）。

**验证:**
```auto
fn get_float() Result[float, str] { Ok(3.14) }
fn main() {
    var r = get_float()
    is r { Ok(f) => print(f.to(str)); Err(e) => print("err") }
}
```

**影响:** 移除 rusqlite 的 status/value 拆分 workaround

#### Task 25: 递归 enum 的 struct variant `is` 解构

**现状:** tuple variant（`node(Tree, Tree)`）的 `is` 解构正常工作（D1 已验证）。但 struct variant（`Pt.p { x int }`）的 `is` 解构解析失败——pratt parser 消费了 `{...}`。

**需要:** parser 在 `is` 分支条件中正确路由 `Kind.variant { field }` 到 struct cover 解析。

**影响:** 更自然的 enum API（可选，当前 tuple variant 已满足需求）

---

## 剩余工作优先级

| 优先级 | Task | 理由 |
|--------|------|------|
| 高 | Task 20 (E1-补全 const/var str) | 影响 base64 + regex，workaround 明确可移除 |
| 高 | Task 21 (A1-补全 位运算链) | 影响 sha2，mk32 可简化为 hex 字面量 |
| 中 | Task 22 (G1-补全 spawn 并发) | 影响 tokio 真正并发测试 |
| 中 | Task 24 (Result[T,E] 语法) | 影响 rusqlite float-in-Result |
| 低 | Task 23 (channel 语法) | 语言设计任务 |
| 低 | Task 25 (struct variant is) | 当前 tuple variant 够用 |
