# 194: 基于单态化的泛型方法调用分发 (Monomorphic Dispatch for Generic Methods)

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 让 HashMap/HashSet 等泛型集合支持统一的 `insert(key, value)` / `get(key)` API，由 codegen 根据泛型参数在编译期自动选择对应的 native shim，消除 `insert_str`/`insert_int` 这种手动类型分派。

**Architecture:** 借鉴 Rust 的单态化方案，在 codegen 阶段根据变量声明时的泛型参数（如 `HashMap<str, int>`）推导出 value 类型，将 `m.insert("k", 42)` 编译为 `CALL_NAT HashMap.insert_int` 而非 `CALL_NAT HashMap.insert_str`。核心改动在 codegen 的方法名解析阶段：从 `var_types` 中读取泛型参数，构造带类型后缀的 native 函数名（如 `HashMap.insert_int`），查找注册表获取正确的 native ID。

**Tech Stack:** Rust (VM codegen, native registry), Auto (.at stdlib)

---

## 背景与动机

### 现状

HashMap 和 HashSet 因为 Auto 的 VM 值在栈上没有运行时类型标签，native shim 必须提前知道参数类型才能正确 pop。当前的妥协方案是为每种 value 类型提供独立的方法名：

```auto
// 当前 API — 用户需要记住类型后缀
m.insert_str("name", "Alice")
m.insert_int("age", 30)
let name = m.get_str("name")
let age = m.get_int("age")
```

### 目标

借鉴 Rust 的单态化，让用户写统一的泛型 API：

```auto
// 目标 API — codegen 根据泛型参数自动选择 shim
let m Map<str, int> = Map.new()
m.insert("name", "Alice")
m.insert("age", 30)
let name = m.get("name")
let age = m.get("age")
```

### 为什么选编译期单态化而非运行时类型标签

| | 编译期单态化 (本方案) | 运行时类型标签 |
|---|---|---|
| 性能 | 零开销，直接 CALL_NAT | 每次 push/pop 需检查标签 |
| 代码膨胀 | 极小 — 只改函数名映射，不改字节码 | 需要新增 opcodes |
| 兼容性 | 复用现有 native shim 架构 | 需要重构值表示系统 |
| Auto 现有架构匹配 | 高 — `GenericRegistry`、`GenericTable`、`var_types` 已就绪 | 低 — 需要改 VM 核心值模型 |

## 核心设计

### 1. 泛型方法名 → 单态化 native 函数名

当 codegen 遇到 `m.insert("k", 42)` 时，解析流程：

```
1. m 是 Expr::Ident("m")
2. 从 var_types 查到 m 的类型 → Map<str, int> (或 Type::User with generic params)
3. 提取 value 的泛型参数 → int
4. 构造单态化函数名: "HashMap.insert_int"
5. 查找 BIGVM_NATIVES → 找到 NATIVE_HASHMAP_INSERT_INT (121)
6. emit CALL_NAT 121
```

### 2. Native Registry 注册别名

现有的 `insert_str`/`insert_int` 注册保持不变（向后兼容），额外注册统一入口别名：

```
"HashMap.insert"  → 不注册（不是具体实现）
"HashMap.get"     → 不注册（不是具体实现）

// 保留旧 API 兼容
"HashMap.insert_str" → 120
"HashMap.insert_int" → 121
"HashMap.get_str"    → 122
"HashMap.get_int"    → 123
```

### 3. 类型后缀映射表

定义一个从 `Type` 到 native 函数名后缀的映射：

```rust
fn type_suffix(ty: &Type) -> &'static str {
    match ty {
        Type::Int | Type::I64      => "_int",
        Type::Uint | Type::U64 | Type::USize | Type::Byte => "_uint",
        Type::Float | Type::Double  => "_float",
        Type::Bool                    => "_bool",
        Type::Str(_) | Type::String | Type::StrSlice => "_str",
        _ => ""
    }
}
```

### 4. 向后兼容

旧的 `insert_str`/`insert_int` API 继续工作（native registry 中仍注册了这些名字）。新代码使用统一 API，旧代码无需修改。

## 实现计划

### Phase 1: 核心分发机制

#### Task 1: 在 codegen 中添加泛型方法分派函数

**Files:**
- Modify: `crates/auto-lang/src/vm/codegen.rs` (在 `infer_type_from_var` 附近)

**Step 1: 添加 `type_suffix` 辅助函数**

在 codegen.rs 中添加类型到后缀的映射函数，用于构造单态化 native 函数名：

```rust
/// Plan 194: Map Auto Type to native function name suffix for monomorphic dispatch
fn type_to_native_suffix(ty: &Type) -> &'static str {
    match ty {
        Type::Int | Type::I64 => "_int",
        Type::Uint | Type::U64 | Type::USize | Type::Byte => "_uint",
        Type::Float | Type::Double => "_float",
        Type::Bool => "_bool",
        Type::Str(_) | Type::String | Type::StrSlice => "_str",
        _ => "",
    }
}
```

**Step 2: 添加 `resolve_generic_method_call` 函数**

这个函数在 codegen 解析方法调用时，检查 receiver 的泛型参数，尝试构造带类型后缀的函数名：

```rust
/// Plan 194: Try to resolve a method call to a monomorphic native function name.
///
/// For example, `m.insert("k", 42)` where m: Map<str, int>
/// → tries "HashMap.insert_int" (value type is int)
///
/// Returns None if the type is not a known generic or no type suffix applies.
fn try_mono_dispatch(&self, base_type: &str, method: &str, type_args: &[Type]) -> Option<String> {
    let suffix = self.type_to_native_suffix_from_args(base_type, method, type_args);
    if suffix.is_empty() {
        return None;
    }
    let mono_name = format!("{}{}{}", base_type, suffix, method);
    if BIGVM_NATIVES.lock().unwrap().get_id(&mono_name).is_some() {
        Some(mono_name)
    } else {
        None
    }
}
```

`type_to_native_suffix_from_args` 根据类型名和方法名决定用哪个泛型参数的后缀：

- HashMap: `insert`/`get` 用 value 参数 (第 2 个), `contains`/`remove` 用 key 参数 (第 1 个)
- HashSet: 所有方法用 element 参数 (第 1 个)
- List: `push`/`pop`/`get`/`set` 用 element 参数 (第 1 个)

**Step 3: 在方法调用解析点集成分发**

在 codegen.rs 约第 4138 行（`Type::GenericInstance` 分支），在现有 `generic_registry.mono_name_from_args` 逻辑之后，添加单态化 native 分派尝试：

```rust
// 在生成 mono_name 之后，先尝试单态化 native 分派
let mono_name = self.generic_registry
    .get_template(&inst.base_name.to_string())
    .map(|t| t.mono_name_from_args(&inst.args))
    .unwrap_or_else(|| format!("{}_unknown", inst.base_name));

// Plan 194: 尝试单态化 native 分派
if let Some(native_name) = self.try_mono_dispatch(
    &inst.base_name, method, &inst.args
) {
    Some(native_name)
} else {
    // 回退到原有逻辑
    Some(format!("{}.{}", mono_name, method))
}
```

同样，在 `Type::User` 分支（约第 4155 行），对 HashMap/HashSet 类型添加相同的分派逻辑。

**Step 4: 构建并运行测试**

Run: `cargo build -p auto-lang`
Expected: 编译通过

Run: `cargo test -p auto-lang --lib -- vm`
Expected: 所有 654+ VM 测试通过（无回归）

**Step 5: Commit**

```
feat(vm): add monomorphic dispatch for generic method calls (Phase 1)
```

#### Task 2: 注册 HashMap/HashSet 单态化 native 别名

**Files:**
- Modify: `crates/auto-lang/src/vm/native_registry.rs` (约第 211 行)

**Step 1: 为 HashMap 添加统一方法别名**

在现有的 `HashMap.insert_str` 等注册之后，添加统一入口。由于 HashMap 的 insert 和 get 需要根据 value 类型选择 shim，我们注册所有变体：

```rust
// Plan 194: Monomorphic dispatch aliases for HashMap
// HashMap.insert_int → 121 (same as insert_int)
registry.register_with_id("HashMap.insert_int", 121);
registry.register_with_id("HashMap.get_int", 123);
// HashMap.insert_str → 120 (same as insert_str)
registry.register_with_id("HashMap.insert_str", 120);
registry.register_with_id("HashMap.get_str", 122);
// HashMap.insert_float → 121 (reuse insert_int for numeric)
registry.register_with_id("HashMap.insert_float", 121);
registry.register_with_id("HashMap.get_float", 123);
// HashMap.insert_bool → 121
registry.register_with_id("HashMap.insert_bool", 121);
registry.register_with_id("HashMap.get_bool", 123);
// HashMap.contains / remove — key is always str, no type suffix needed
// (existing HashMap.contains and HashMap.remove already work with string keys)
```

注意：HashMap 的 native shim 本身只区分 str 和 int 两种 value 类型。`_float` 和 `_bool` 后缀映射到 `_int` shim 是因为 native 层暂不支持 float/bool value。如果需要精确类型支持，后续 phase 可扩展。

**Step 2: 为 HashSet 添加统一方法别名**

```rust
// Plan 194: Monomorphic dispatch aliases for HashSet
// HashSet.insert_str → 130 (same as insert)
registry.register_with_id("HashSet.insert_str", 130);
registry.register_with_id("HashSet.insert_int", 130);
registry.register_with_id("HashSet.insert_float", 130);
registry.register_with_id("HashSet.insert_bool", 130);
// contains / remove — same pattern
registry.register_with_id("HashSet.contains_str", 131);
registry.register_with_id("HashSet.contains_int", 131);
registry.register_with_id("HashSet.remove_str", 132);
registry.register_with_id("HashSet.remove_int", 132);
```

**Step 3: 构建并运行测试**

Run: `cargo build -p auto-lang`
Expected: 编译通过

**Step 4: Commit**

```
feat(vm): register monomorphic native aliases for HashMap/HashSet
```

#### Task 3: 更新 var_types 跟踪泛型参数

**Files:**
- Modify: `crates/auto-lang/src/vm/codegen.rs` (约第 1005 行)

**Step 1: 让 HashMap.new() 的 var_types 携带泛型参数**

当前 `HashMap.new()` 创建的 `var_types` 是 `Type::User` 且 `generic_params` 为空。需要改为 `Type::GenericInstance`，携带类型参数信息。

在 codegen.rs 约第 1005 行 `type_name == "HashMap" && method == "new"` 分支中，改为：

```rust
else if type_name == "HashMap" && method == "new" {
    // Plan 194: Track HashMap with generic params
    // Default to Map<str, int> if no type args provided
    let inst = crate::ast::GenericInstanceData {
        base_name: crate::ast::Name::from("HashMap"),
        args: vec![Type::Str(0), Type::Int], // default: Map<str, int>
    };
    self.var_types.insert(
        store.name.to_string(),
        Type::GenericInstance(inst),
    );
}
```

同理对 HashSet：

```rust
else if type_name == "HashSet" && method == "new" {
    let inst = crate::ast::GenericInstanceData {
        base_name: crate::ast::Name::from("HashSet"),
        args: vec![Type::Str(0)], // default: HashSet<str>
    };
    self.var_types.insert(
        store.name.to_string(),
        Type::GenericInstance(inst),
    );
}
```

**Step 2: 构建并运行测试**

Run: `cargo build -p auto-lang`
Expected: 编译通过

**Step 3: Commit**

```
feat(vm): track generic params in var_types for HashMap/HashSet
```

#### Task 4: 更新 HashMap/HashSet stdlib 声明

**Files:**
- Modify: `stdlib/auto/hashmap.at`
- Modify: `stdlib/auto/hashset.at`

**Step 1: 在 hashmap.at 中添加统一的泛型方法声明**

保留旧的 `_str`/`_int` 方法（向后兼容），新增统一 API：

```auto
type HashMap<K, V> {
    // ... 现有方法保留 ...

    // ========================================================================
    // Unified Generic Methods (Plan 194)
    // ========================================================================

    /// Insert a key-value pair (value type determined by generic param V)
    /// Example: m.insert("name", "Alice")   // when V = str
    ///          m.insert("age", 30)          // when V = int
    #[vm, pub]
    fn insert(key K, value V)

    /// Get value by key (return type determined by generic param V)
    /// Example: let name = m.get("name")   // returns str
    ///          let age = m.get("age")      // returns int
    #[vm, pub]
    fn get(key K) V

    /// Check if key exists
    /// Example: let has = m.contains("name")
    #[vm, pub]
    fn contains(key K) bool

    /// Remove a key-value pair
    /// Example: m.remove("age")
    #[vm, pub]
    fn remove(key K)
}
```

**Step 2: 在 hashset.at 中添加统一的泛型方法声明**

```auto
type HashSet<T> {
    // ... 现有方法保留 ...

    // ========================================================================
    // Unified Generic Methods (Plan 194)
    // ========================================================================

    /// Insert an element (element type determined by generic param T)
    /// Example: set.insert("apple")
    ///          set.insert(42)  // when T = int
    #[vm, pub]
    fn insert(value T)

    /// Check if element exists
    /// Example: set.contains("apple")
    #[vm, pub]
    fn contains(value T) bool

    /// Remove an element
    /// Example: set.remove("apple")
    #[vm, pub]
    fn remove(value T)
}
```

**Step 3: Commit**

```
feat(stdlib): add unified generic insert/get methods to HashMap/HashSet
```

#### Task 5: 编写 VM file-based 测试

**Files:**
- Create: `crates/auto-lang/test/vm/21_conv/003_hashmap_mono_insert.at`
- Create: `crates/auto-lang/test/vm/21_conv/003_hashmap_mono_insert.expected.out`
- Create: `crates/auto-lang/test/vm/21_conv/004_hashset_mono_insert.at`
- Create: `crates/auto-lang/test/vm/21_conv/004_hashset_mono_insert.expected.out`
- Modify: `crates/auto-lang/src/tests/vm_file_tests.rs`

**Step 1: 编写 HashMap 统一 API 测试**

`crates/auto-lang/test/vm/21_conv/003_hashmap_mono_insert/hashmap_mono_insert.at`:

```auto
// Test Plan 194: HashMap unified generic insert/get API
fn main() {
    var m = HashMap.new()
    m.insert("name", "Alice")
    m.insert("age", 30)
    print(m.get("name"))
    print(m.get("age"))
    print(m.contains("name"))
    m.remove("age")
    print(m.size())
    m.drop()
}
```

`hashmap_mono_insert.expected.out`:

```
Alice
30
1
1
```

**Step 2: 编写 HashSet 统一 API 测试**

`crates/auto-lang/test/vm/21_conv/004_hashset_mono_insert/hashset_mono_insert.at`:

```auto
// Test Plan 194: HashSet unified generic insert/contains API
fn main() {
    var s = HashSet.new()
    s.insert("apple")
    s.insert("banana")
    print(s.contains("apple"))
    print(s.contains("cherry"))
    s.remove("banana")
    print(s.size())
    s.drop()
}
```

`hashset_mono_insert.expected.out`:

```
1
0
1
```

**Step 3: 在 vm_file_tests.rs 中注册测试**

```rust
// === 21_conv (Plan 193/194: type conversion & monomorphic dispatch) ===
#[test] fn test_21_conv_003_hashmap_mono_insert() { test_vm("21_conv/003_hashmap_mono_insert").unwrap(); }
#[test] fn test_21_conv_004_hashset_mono_insert() { test_vm("21_conv/004_hashset_mono_insert").unwrap(); }
```

**Step 4: 运行测试确认通过**

Run: `cargo test -p auto-lang --lib -- 21_conv`
Expected: 3 passed, 1 ignored (002_neg_i32_to_str)

**Step 5: 运行全部 VM 测试确认无回归**

Run: `cargo test -p auto-lang --lib -- vm`
Expected: 所有测试通过

**Step 6: Commit**

```
test(vm): add HashMap/HashSet monomorphic dispatch tests
```

### Phase 2: 扩展到 List（可选，低优先级）

#### Task 6: 为 List 添加单态化方法分派

**Files:**
- Modify: `crates/auto-lang/src/vm/codegen.rs`
- Modify: `crates/auto-lang/src/vm/native_registry.rs`

List 已有部分单态化（`CREATE_LIST_INT` 等 opcodes），但 `List.push`/`List.get` 等方法调用走的是通用 `List.push` native ID（101）。此 task 将 `List.push(value)` 根据元素类型分派到不同的 shim。

**Step 1:** 在 `type_to_native_suffix_from_args` 中添加 List 的分派规则（element 参数是第 1 个泛型参数）。

**Step 2:** 注册 `List.push_int`、`List.push_str`、`List.get_int`、`List.get_str` 等别名。

**Step 3:** 测试 `let l = List.new(); l.push(42); l.push("hello")` 能正确工作。

**Step 4: Commit**

```
feat(vm): extend monomorphic dispatch to List methods
```

## 文件清单

| 文件 | 变更类型 | Phase |
|------|---------|:-----:|
| `crates/auto-lang/src/vm/codegen.rs` | 修改 (添加分派函数) | 1 |
| `crates/auto-lang/src/vm/native_registry.rs` | 修改 (注册别名) | 1 |
| `stdlib/auto/hashmap.at` | 修改 (添加统一方法) | 1 |
| `stdlib/auto/hashset.at` | 修改 (添加统一方法) | 1 |
| `crates/auto-lang/test/vm/21_conv/003_hashmap_mono_insert/` | 新建 | 1 |
| `crates/auto-lang/test/vm/21_conv/004_hashset_mono_insert/` | 新建 | 1 |
| `crates/auto-lang/src/tests/vm_file_tests.rs` | 修改 (注册测试) | 1 |

## 验证方案

### 单元测试

```auto
// test_mono_dispatch.at — 验证统一 API
fn main() {
    // HashMap: str key, int value
    var m = HashMap.new()
    m.insert("score", 100)
    print(m.get("score"))

    // HashMap: str key, str value
    var s = HashMap.new()
    s.insert("msg", "hello")
    print(s.get("msg"))

    // HashSet: str element
    var set = HashSet.new()
    set.insert("foo")
    print(set.contains("foo"))
    print(set.contains("bar"))

    // 向后兼容: 旧 API 仍然工作
    m.insert_int("level", 5)
    print(m.get_int("level"))

    print("mono: all tests passed")
}
```

### 向后兼容测试

现有的 13 个 HashMap/HashSet VM 测试（001-013）全部保持不变，无需修改。

## 与现有系统的兼容性

### 现有 API

- `m.insert_str("k", "v")` → 继续工作（native registry 中仍注册）
- `m.insert_int("k", 42)` → 继续工作
- `m.get_str("k")` → 继续工作
- `m.get_int("k")` → 继续工作

### 新 API

- `m.insert("k", "v")` → codegen 分派到 `HashMap.insert_str` (与上面等价)
- `m.insert("k", 42)` → codegen 分派到 `HashMap.insert_int` (与上面等价)
- `m.get("k")` → 需要类型信息才能分派（var_types 中有类型参数）

### 分派优先级

1. **单态化分派**（新）: `HashMap.insert_int` → 有泛型参数时优先使用
2. **直接查找**（现有）: `HashMap.insert_str` → 兼容旧代码
3. **泛型模板方法**（现有）: `Pair_int_str.get_key` → 用户自定义泛型类型

## 工期估计

| Phase | 工作量 | 依赖 |
|-------|--------|------|
| Phase 1 (Task 1-5) | 1 天 | 无 |
| Phase 2 (Task 6, 可选) | 0.5 天 | Phase 1 完成 |
