# Plan 160: 添加 `Map<K, V>` 类型到 AutoLang

**日期**: 2026-04-08
**状态**: ✅ 已完成（Phase 1-5 全部实施）
**目标**: 在 AutoLang 语言层面添加 `Map<K, V>` 内置类型，作为 Object 的类型化版本
**关联**: Plan 159 Phase 6B-1 (a2r 转译器增强)

---

## 1. 动机

AutoLang 有 `List<T>` 作为 Array 的类型化版本（`[1,2,3]` + `List<int>` 注解 → `Vec<i32>`），
但没有 Object 的类型化版本。Object 字面量 `{k: v}` 无法表达"所有 key 是 str、所有 value 是 int"。

这导致：
- **a2r**: 无法将 Object 转译为 `HashMap<K, V>`，只能输出无类型的结构体初始化语法
- **a2c**: C 代码中没有类型化的 map 数据结构
- **AutoCode**: 工具注册表、消息历史等数据结构无法用 Auto 表达

### 类比：List 是怎么做的

| 无注解 | 有注解 | VM 运行时 | a2r 转译 |
|---|---|---|---|
| `[1, 2, 3]` | `List<int>` | `Array { values: Vec<Value> }` | `Vec<i32>` |
| `{a: 1, b: 2}` | `Map<str, int>` | `Obj { values: IndexMap<ValueKey, Value> }` | `HashMap<String, i32>` |

### 不是标准库类型

`Map<K, V>` 和 `List<T>`、`[N]T`、`Option<T>` 同级，是**语言内置类型**。
不需要在 stdlib 中添加单独的 HashMap 类型。

---

## 2. 设计

### 2.1 AST 定义

在 `crates/auto-lang/src/ast/types.rs` 的 `Type` 枚举中新增：

```rust
// 现有
List(Box<Type>),              // List<T> - 动态列表

// 新增
Map(Box<Type>, Box<Type>),    // Map<K, V> - 类型化字典
```

### 2.2 语法

```auto
// 类型注解
let headers Map<str, str> = { "Authorization": "Bearer xxx" }
let counts Map<str, int> = { "hello": 3, "world": 5 }

// 函数参数
fn process(data Map<str, str>) { ... }

// 结构体字段
type Config {
    env Map<str, str>
    ports List<int>
}
```

### 2.3 转译目标

| 转译器 | `Map<K, V>` 输出 | 说明 |
|---|---|---|
| **a2r** | `std::collections::HashMap<K, V>` | 标准 HashMap |
| **a2c** | `map_K_V*` (自定义结构) | 需要 C 运行时支持 |
| **a2ts** | `Record<K, V>` | TypeScript Record |
| **a2py** | `dict` | Python dict |
| **a2ark** | `HashMap<K, V>` | ArkTS HashMap |
| **VM** | `Obj` (忽略类型参数) | 运行时复用现有 Obj |

### 2.4 Object 字面量转译规则（a2r）

根据上下文类型注解决定转译方式：

| 上下文 | Object 字面量 `{k: v, ...}` 转译为 |
|---|---|
| 类型注解为 `Map<K, V>` | `HashMap::from([(k, v), ...])` |
| 类型注解为具名 struct | `StructName { field: val, ... }` (现有行为) |
| 无类型注解 | `serde_json::json!({...})` (兜底) |

### 2.5 默认值

- `List<T>` 的默认值是空列表 `[]`
- `Map<K, V>` 的默认值是空 map `{}` 或 `Map.new()`

---

## 3. 影响范围

### 3.1 生产代码（17 个文件，~36 处 match 分支）

| # | 文件 | `Type::List` 引用数 | 改动说明 |
|---|---|---|---|
| 1 | `ast/types.rs` | 8 | Type 枚举定义 + unique_name/default_value/substitute/Display/From/ToAtomStr |
| 2 | `parser.rs` | 2 | 解析 `Map<K, V>` 类型语法 |
| 3 | `vm/codegen.rs` | 4 | 变量类型记录 + 类型推断 |
| 4 | `vm/monomorphize.rs` | 3 | 单态化支持 |
| 5 | `vm/generic.rs` | 2 | 泛型名称提取 + GenericInstance |
| 6 | `vm/generic_registry.rs` | 3 | 泛型注册测试 |
| 7 | `vm/pattern_matcher.rs` | 1 | 模式匹配 |
| 8 | `hash.rs` | 1 | 类型哈希 |
| 9 | `api/mod.rs` | 1 | type_to_string |
| 10 | `implicit_union.rs` | 1 | 隐式联合类型 |
| 11 | `trans/rust.rs` | 1 | rust_type_name |
| 12 | `trans/c.rs` | 2 | c_type_name + substitute |
| 13 | `trans/ts_types.rs` | 1 | TypeScript 映射 |
| 14 | `trans/python.rs` | 1 | Python 映射 |
| 15 | `ui_gen/ark/state.rs` | 3 | ArkTS 类型映射 |
| 16 | `ui_gen/ark/generator.rs` | 3 | ArkTS 生成 |
| 17 | `auto-man/src/ark.rs` | 1 | auto-man crate |

### 3.2 测试文件（4-5 个文件）

| 文件 | 说明 |
|---|---|
| `generic_tests.rs` | 泛型测试 |
| `monomorphize_tests.rs` | 单态化测试 |
| `bigvm_generic_integration_tests.rs` | VM 泛型集成测试 |
| `plan_088_tests.rs` | Plan 088 测试 |

### 3.3 a2r / a2c 测试用例

新增测试目录验证 Map 类型的转译：

- `test/a2r/XXX_map/` — Rust 转译测试
- `test/a2c/XXX_map/` — C 转译测试（可后续添加）

---

## 4. 实施阶段

### Phase 1: AST + Parser ✅ 已完成

**改动文件**: `ast/types.rs`, `parser.rs`

- ✅ `Type::Map(Box<Type>, Box<Type>)` 变体添加到 Type 枚举
- ✅ unique_name / default_value / substitute / Display / From / ToAtomStr 全部实现
- ✅ Parser 识别 `Map<K, V>` 语法（2 个类型参数）
- ✅ auto_val 映射为 `auto_val::Type::User("Map")`

### Phase 2: VM 路径 ✅ 已完成

**改动文件**: `vm/codegen.rs`, `vm/generic.rs`, `vm/monomorphize.rs`, `vm/pattern_matcher.rs`

- ✅ codegen: infer_type_from_var 返回 "Map"
- ✅ generic: type_to_simple_name + extract_generic_instance
- ✅ monomorphize: is_monomorphizable + collect
- ✅ pattern_matcher: `Type::Map` 匹配 `Value::Obj`

### Phase 3: 转译器四件套 ✅ 已完成

**改动文件**: `trans/rust.rs`, `trans/c.rs`, `trans/ts_types.rs`, `trans/python.rs`

- ✅ a2r: `HashMap<K, V>`
- ✅ a2c: `map_K_V*`
- ✅ a2ts: `Record<K, V>`
- ✅ a2py: `dict`

### Phase 4: 其他子系统 ✅ 已完成

**改动文件**: `hash.rs`, `api/mod.rs`, `implicit_union.rs`, `infer/unification.rs`,
`ui_gen/ark/state.rs`, `ui_gen/ark/generator.rs`, `auto-man/src/ark.rs`

### Phase 5: 测试 ✅ 已完成

- ✅ a2r 测试 `128_map_type`: struct 字段 + 变量声明
- ✅ a2r 测试 `129_map_func`: 函数参数 + 返回值
- ✅ 2544 测试全部通过

**改动文件总计**: 14 个生产代码文件 + 2 个测试文件

### 后续工作（未实施）

- Object 字面量 `{k: v}` 在有 Map 类型注解时转译为 `HashMap::from([...])`
  （需要给 a2r 的 `expr()` 方法添加类型上下文传递）
- a2c 的 C 运行时 map 数据结构
- VM 中 Map 专属操作方法（insert/get/contains_key 等）

---

## 5. 风险

| 风险 | 状态 |
|---|---|
| 22 个文件改动容易遗漏 | ✅ 无遗漏，cargo check/test 验证 |
| auto_val 中是否也加 Map 变体 | ✅ 用 `User("Map")` 替代，不需要新变体 |
| a2c 的 C 运行时 map 结构 | ⏸️ 类型名称已生成，运行时待补 |
| Object 字面量需要类型上下文 | ⏸️ 记录为后续工作 |

---

## 6. 与 Plan 159 的关系

Plan 159 Phase 6B-1 需要 HashMap 类型支持。本 Plan 完成后：
- ✅ a2r 能将 `Map<str, int>` 转译为 `std::collections::HashMap<&str, i32>`
- ✅ VM 路径下 `Map` 复用 `Obj` 运行时
- ✅ AutoCode 的工具注册表、消息历史可用 `Map<K,V>` 类型表达
