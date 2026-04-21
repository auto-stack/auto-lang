# Plan 201: AutoVM 四大核心能力补齐

> 对比 ac-examples 01~13 的手写 `.at` 与 r2a 转译 `.r2a.at` 后发现的核心能力差距。
> 目标：让 AutoVM 的 enum、闭包、Result、spec 四大能力达到 Rust 对等水平，
> 使 r2a 转译输出的代码能直接在 AutoVM 上运行。

## 动机

通过对比 13 个已验证可运行的 ac-examples，发现两大代码版本的根本分歧：

| 维度 | `.at`（手写） | `.r2a.at`（r2a 转译） |
|---|---|---|
| enum | 扁平 `type` + 字符串 kind 标记 | Rust 原生 enum 带数据变体 |
| 迭代 | 手写 for 循环 | `.iter().map().filter().collect()` |
| 错误 | `ok: bool` 字段 | `Result<T, E>` + `?` 操作符 |
| 多态 | if-else 字符串分派 | trait + `dyn` 动态分派 |

手写版能用，但丢失了 Rust 的类型安全。r2a 版保留了类型安全，但 AutoVM 缺少运行时支持导致核心逻辑被注释掉。

---

## 问题 1：enum 带数据变体 — 多字段支持

### 已完成的基础（Plan 156）

Plan 156（`old/156-unified-enum-migration.md`）已完成 Heterogeneous Enum 的 **AST/Parser/Transpiler** 层面统一：

- **AST**：`EnumKind` 区分 `Scalar`/`Homogeneous`/`Heterogeneous` 三形态 — ✅ 完成
- **Parser**：`enum Name { Quit, Move Point, Write string }` 单字段异构变体语法 — ✅ 完成
- **Transpiler**：C/Rust/TS 三端根据 `EnumKind` 分派生成 — ✅ 完成
- **`tag` 关键字**：废弃，重定向到 `enum` — ✅ 完成

### 仍需补齐：多字段变体（Plan 201 的范围）

Plan 156 完成的是**单字段 payload** 的异构 enum。当前整个链路硬编码为每个变体仅一个 `_0` 字段：

| 层级 | 当前状态 | 需要扩展 |
|---|---|---|
| AST `EnumItem` | `payload_type: Option<Type>` — 单类型 | 新增 `fields: Vec<FieldDef>` 支持多命名字段 |
| Parser `parse_enum_body` | 调用 `parse_type()` 一次 | 需识别 `Variant { field Type, ... }` 语法 |
| Parser `parse_type()` | 无 `LBrace` 分支（dead path） | 需在 enum 上下文中解析结构体式字段列表 |
| Codegen 变体注册 | `ClassTemplate` 只有 `_0` 一个字段 | 需注册多个 `FieldDef` |
| Codegen 构造 | 单参数 `Atom.Int(42)` | 需支持多参数 `Api(status: 429, msg: "err")` |
| Pattern `TagCover` | `elem: AutoStr` — 单绑定变量 | 需支持多绑定 `Api(status, msg)` |
| Codegen 解构 | 只提取 `_0`（index 0），有 TODO 注释 | 需遍历所有字段提取 |

**关键发现**：运行时基础设施 `GenericInstanceData` 的 `fields: Vec<Value>` 和 `field_names: Vec<String>` **已支持多字段**。ClassTemplate 的 `fields: Vec<FieldDef>` 也支持。瓶颈在于 AST/Parser/Codegen 的单字段硬编码。

### 当前可用语法 vs 目标语法

```auto
// 当前可用 — 单字段变体（Plan 156 已支持）
enum Atom { Int int, Str str, None }
let a = Atom.Int(42)
is a { Atom.Int(n) -> print(n) }

// 需要 wrapper type 绕过多字段
type ApiPayload { status uint, message str }
enum ApiError { Http str, Api ApiPayload, None }

// 目标 — 多字段结构体式变体
enum ApiError {
    Http(str)                                    // 单字段（已有）
    Api { status uint, message str, retryable bool }  // 多字段（需新增）
    RetriesExhausted { attempts uint }                // 多字段（需新增）
}

// 目标 — 构造
let err = ApiError.Api(status: 429, message: "rate limited", retryable: true)

// 目标 — 模式匹配多字段解构
is err {
    Http(msg) -> print(msg)
    Api(status, message) -> print(f"$status: $message")
    RetriesExhausted(attempts) -> print(f"$attempts attempts")
    _ -> print("other")
}

// 目标 — 点访问字段（变体确定后）
is err {
    Api -> print(err.status)
    _ -> {}
}
```

### 实现步骤

#### Phase 1A：AST 扩展 — EnumItem 多字段
- 文件：`src/ast/enums.rs`
- `EnumItem` 新增 `fields: Vec<FieldDef>` 字段
- 保持 `payload_type` 向后兼容：
  - `payload_type: Some(T)` + `fields: []` → 单字段变体（现有代码）
  - `payload_type: None` + `fields: [FieldDef(...)]` → 多字段变体（新增）
- 辅助方法 `has_fields()` / `field_count()` 简化后续代码判断

#### Phase 1B：Parser 扩展 — 结构体式变体语法
- 文件：`src/parser.rs`
- 在 `parse_enum_body`（约 line 3900）中，当检测到 `LBrace` 时：
  - 不再调用 `parse_type()`（它会拒绝 `{`）
  - 改为调用新方法 `parse_enum_variant_fields()` 解析 `{ name Type, name Type, ... }`
  - 结果存入 `EnumItem.fields`
- 现有 `VariantName SingleType` 路径不变（兼容性）

#### Phase 1C：Codegen 多字段注册
- 文件：`src/vm/codegen.rs`（约 line 1618-1634）
- 当 `EnumItem.fields` 非空时，用 `fields` 列表创建 `ClassTemplate`
- 字段名使用实际名称（如 `status`, `message`），而非 `_0`
- 当 `EnumItem.payload_type` 存在但 `fields` 为空时，保持现有单字段 `_0` 逻辑

#### Phase 1D：模式匹配增强 — 多字段解构
- 文件：`src/ast/cover.rs`
- `TagCover.elem` 扩展为 `bindings: Vec<AutoStr>`（或新增 `MultiTagCover`）
- 文件：`src/parser.rs` `tag_cover` 方法
  - 解析 `VariantName(x, y, z)` → 多绑定
  - 解析 `VariantName { field1, field2 }` → 命名解构（可选，可后续 Phase）
- 文件：`src/vm/codegen.rs`（约 line 2530）
  - 替换 TODO 注释为多字段循环：
    ```
    for (i, binding) in bindings.iter().enumerate() {
        DUP + GET_GENERIC_FIELD(i) + STORE_LOCAL(binding)
    }
    ```

#### Phase 1E：构造语法增强
- 文件：`src/vm/codegen.rs`（约 line 4356-4426）
- 当 `ClassTemplate` 有多个字段时，`CONSTRUCT_INSTANCE` 已支持多字段（现有逻辑按 field_count 处理）
- 命名构造 `Variant(field: val)` 的参数排序：按 `ClassTemplate.fields` 定义顺序匹配

### 影响的 ac-examples
- `10_api_error_enum` — 直接使用 `enum ApiError { Api { status, message, retryable } }`
- `12_stream_event_types` — 直接使用 `enum StreamEvent` 的多种变体形式
- `13_tool_trait_def` — `enum ToolError` 的双字段变体

---

## 问题 2：闭包作为高阶函数参数 + 后缀链式调用

### 现状

- **闭包语法**：`x => expr` 和 `(a, b) => expr` 已实现（Plan 060）
- **闭包编译**：`OpCode::CLOSURE` + `CALL_CLOSURE` 已实现
- **闭包捕获**：自由变量捕获 + 借用检查已实现
- **缺失**：没有 `map`、`filter`、`reduce` 等内置高阶函数
- **风格**：Auto 倾向后缀式方法调用（`obj.method(args)`），不需要 Rust 的 `.iter()` / `.collect()` 胶水

### 目标：基于闭包的后缀链式调用

Auto 采用 `list.filter(cond).map(x => expr)` 风格的后缀链式调用。
与 Rust 的区别是：**不需要 `.iter()` 和 `.collect()`** — Auto 的集合方法直接在集合上操作并返回新集合。

```auto
// 过滤 + 映射链
let evens = nums.filter(x => x % 2 == 0)
let doubled = nums.map(x => x * 2)
let result = nums.filter(x => x > 3).map(x => x * 2)

// 累积
let sum = nums.reduce(0, (acc, x) => acc + x)

// 副作用遍历
nums.for_each(x => print(x))

// 查找
let first_big = nums.find(x => x > 100)

// 判断
let has_negative = nums.any(x => x < 0)
let all_positive = nums.all(x => x > 0)

// 排序（可选闭包）
let sorted = nums.sort()
let sorted = nums.sort_by((a, b) => a - b)

// 字符串也可用
let chars = "hello".chars()                // -> List<str>
let upper = "hello".chars().map(c => c.to_upper())  // -> List<str>
```

**与 Rust 的关键差异**：
1. **无 `.iter()`** — Auto 集合方法直接调用，不需要显式创建迭代器
2. **无 `.collect()`** — `map/filter` 直接返回新集合，不需要收集步骤
3. **链式即结果** — 链式调用的每一步都返回具体集合，不是惰性迭代器
4. **闭包语法简洁** — `x => expr` 比 Rust 的 `|x| expr` 更轻量

**不采用列表推导式**：`[expr for x in list if cond]` 是前置式语法，与 Auto 喜欢的后缀式调用链风格冲突。

### 实现步骤

#### Phase 2A：闭包作为 native 函数参数
- 文件：`src/vm/codegen.rs`
- 当 `CALL_NAT` 的参数中包含闭包表达式时，先编译闭包得到 closure_id，再作为参数压栈
- 新增 native 函数调用闭包的能力：native 侧持有 closure_id，通过 VM engine 的 `call_closure(closure_id, args)` 接口回调 Auto 闭包

#### Phase 2B：VM engine 闭包回调 API
- 文件：`src/vm/engine.rs`
- 新增 `pub fn call_closure(&mut self, closure_id: u32, args: Vec<Value>) -> Value` 公共方法
- Native 函数（Rust 实现）通过此 API 回调 Auto 闭包
- 类似 Lua 的 `lua_call` 机制

#### Phase 2C：内置 map/filter/reduce
- 文件：`src/vm/native_registry.rs` 或 `src/vm/ffi/stdlib.rs`
- 注册为 List 类型的原生方法（类似已有的 `list.push`、`list.len`）
- 每个方法的 Rust 实现内部循环调用 `engine.call_closure(closure_id, [element])`
- 注册列表：
  - `list.map(closure)` → 新列表，对每个元素应用闭包
  - `list.filter(closure)` → 新列表，保留闭包返回 true 的元素
  - `list.reduce(init, closure)` → 累积值
  - `list.for_each(closure)` → void，对每个元素执行闭包
  - `list.find(closure)` → `?T`，返回第一个满足条件的元素
  - `list.any(closure)` → bool，是否存在满足条件的元素
  - `list.all(closure)` → bool，是否所有元素都满足条件
- 注册 `list.for_each(closure)` → 副作用遍历
- 这些 native 函数在 Rust 侧实现，接收 closure_id，通过 `CALL_CLOSURE` 调用

#### Phase 2D：字符串的 map/filter
- `"hello".chars()` → 返回 `List<str>` 以支持链式调用
- `"hello".chars().map(c => c.to_upper())` → `["H", "E", "L", "L", "O"]`
- 或者后续增加 `str.map_char(c => expr)` 直接返回字符串

#### Phase 2E：Map 类型的 map/filter
- `map_obj.map((k, v) => ...)` — 键值对变换
- `map_obj.filter((k, v) => ...)` — 键值对过滤
- `map_obj.keys()` → `List<str>`
- `map_obj.values()` → `List<T>`

### 影响的 ac-examples
- `04_token_estimate` — `blocks.map(b => b.text.len()).sum()` 替代手写循环
- `06_line_formatter` — `lines.filter(...).map(...)` 替代手写循环
- `13_tool_trait_def` — `tools.map(t => t.execute(input))` 替代 for 循环
- `20_tool_registry`（Plan 200）— 链式过滤和映射

---

## 问题 3：`!T` + `*Err` 指针 — Auto 原创的 Result 方案

### 现状

- **AST**：`Type::Result(Box<Type>)` — 单参数，error 类型硬编码为 String
- **语法**：`!T` 表示 `Result<T>`，映射到 Rust 的 `Result<T, String>`
- **Runtime**：哨兵整数编码 — Ok=正值，Err=-2，None=-1
  - 错误消息在 CREATE_ERR 时被丢弃！
  - 只能包装 i32 兼容值，不能包装字符串、对象
- **TODO 注释**：engine.rs:1651 "Implement proper Result<T> type tracking in VM"

### 设计目标

**不引入 `!<T, E>` 双参数语法**。保持 `!T` 单参数的简洁性，通过 `*Err` 内部指针实现运行时多态错误类型。

核心思路：`!T` 的 Err 变体内部携带一个 `*Err` 指针（指向实现了 `Err` spec 的对象），而非固定为 String。

```auto
// 定义 Err spec — 所有错误类型必须实现
spec Err {
    fn msg() str
}

// 用 enum 定义具体错误类型（利用 Phase 1 的多字段变体）
enum ParseError {
    InvalidChar { ch str, pos uint }
    UnexpectedEnd
}
ext ParseError for Err {
    fn msg() str {
        is self {
            InvalidChar(ch, _) -> f"invalid character: $ch"
            UnexpectedEnd -> "unexpected end of input"
        }
    }
}

enum IoError {
    NotFound { path str }
    PermissionDenied { path str }
}
ext IoError for Err {
    fn msg() str { ... }
}

// 函数签名 — 始终只需 !T，错误类型由 Err spec 约束
fn parse_int(s str) !int
fn read_file(path str) !str

// 构造 Ok/Err
fn parse_int(s str) !int {
    if s == "" {
        return Err(ParseError.UnexpectedEnd)     // Err 接受任何实现 Err spec 的类型
    }
    Ok(42)
}

// 使用 — is 解构还原具体错误类型
let result = parse_int("abc")
is result {
    Ok(n) -> print(n)
    Err(e) -> is e {                             // e 的运行时类型为 *Err
        ParseError.InvalidChar(ch, pos) -> print(f"bad char '$ch' at $pos")
        ParseError.UnexpectedEnd -> print("unexpected end")
        IoError.NotFound(path) -> print(f"not found: $path")
        _ -> print(f"unknown error: ${e.msg()}") // 兜底：用 Err spec 的 msg()
    }
}

// 错误传播 — .? 操作符（已实现）
let val = parse_int(input).?
```

### 与 Rust `Result<T, E>` 的对比

| 维度 | Rust `Result<T, E>` | Auto `!T` + `*Err` |
|---|---|---|
| 类型参数 | 双参数 `<T, E>` | 单参数 `!T`，E 通过 `*Err` spec 约束 |
| 错误类型约束 | 显式 E 参数 | 隐式：任何实现 `Err` spec 的类型 |
| 错误匹配 | 按具体 E 类型 | `is e { ConcreteType... }` 运行时匹配 |
| PC 运行时 | 单态化泛型 | `*Err` vtable 指针，运行时多态 |
| MCU 运行时 | 不适用 | `*Err` 降级为整数 ID，查错误信息表 |
| 用户心智负担 | 需要思考 E 写什么 | 只需 `!T`，错误类型按需定义 |

### r2a 转译策略

Rust 代码 `fn foo() -> Result<T, E>` 转译时：
1. 检查 E 是否实现了 `Display`/`Error` trait → 映射为实现 `Err` spec
2. 函数签名转译为 `fn foo() !T`（不保留 E 参数）
3. `Err(E::Variant(...))` → `Err(E.Variant(...))`（利用 Phase 1 多字段 enum）

这样 Rust 的 `Result<T, E>` 和 Auto 的 `!T` 形成自然映射，不需要 Auto 引入 `!<T, E>` 语法。

### MCU 降级方案

在 MCU 目标上，`*Err` 指针降级为整数 ID（错误码）：
- 编译时为每个实现 `Err` spec 的类型分配唯一 ID
- `CREATE_ERR` 只压入 `(error_id, variant_index)` — 固定内存占用
- 具体错误数据存储在独立的上位机可读的"错误信息数据表"中
- `is e { ParseError.InvalidChar(ch, pos) -> ... }` 降级为基于 ID 的 switch-case

这使得 `!T` 在 MCU 上内存占用恒定（一个错误码 + 变体索引），适合资源受限环境。

### 依赖关系

**Phase 3 强依赖 Phase 1 和 Phase 4**：

1. **依赖 Phase 1（enum 多字段变体）**：
   - 错误类型通常是 enum，且带多个字段（如 `ParseError.InvalidChar { ch str, pos uint }`）
   - 没有 Phase 1，错误类型只能用单字段 enum 或扁平 struct，表达能力不足
   - `is e { ParseError.InvalidChar(ch, pos) -> ... }` 解构依赖多字段支持

2. **依赖 Phase 4（spec vtable 动态分派）**：
   - `*Err` 指针本质是一个 dyn 对象：指向某个实现了 `Err` spec 的具体类型
   - `e.msg()` 调用通过 vtable 分派到具体类型的实现
   - 没有 Phase 4，`*Err` 无法在运行时还原为具体类型进行 `is` 匹配
   - MCU 降级的整数 ID 方案也依赖 Phase 4 的 spec 注册机制

### `*T` 动态指针机制（Phase 4 的子特性）

`*T` 在 Auto 中根据 T 的种类有不同语义：

```
*int         → 普通指针（T 是普通类型 type/enum）
*Err         → 动态指针（T 是 spec，指向任意实现该 spec 的类型）
*Tool        → 动态指针（同上）
```

**语义区分规则**：
- `PtrType.of` 解引用后检查目标类型
- 如果目标为 `Type::Spec(...)` → 编译为 dyn 对象（堆上存储 type_name + 具体值 + vtable 引用）
- 如果目标为其他类型 → 编译为普通指针（当前已有的 `*T` 行为）

**当前状态**：
- AST 中已有 `Type::Ptr(PtrType)` 和 `Type::Spec(Shared<SpecDecl>)`
- 但 codegen 中 `*T` 统一当作普通指针处理（`TYPE_CAST_PTR` opcode）
- **未实现**：当 `*T` 的 T 是 spec 时，编译为 dyn 对象的逻辑

**需要新增的实现**（属于 Phase 4 范畴）：
1. **编译期**：codegen 检查 `PtrType.of` 是否为 `Type::Spec`，如果是则生成 dyn 构造指令
2. **运行时**：dyn 对象存储 `(spec_name, type_name, concrete_value, vtable_ref)`
3. **方法调用**：`dyn_obj.method(args)` → 通过 vtable_ref 查找 method_addr → 跳转调用
4. **is 匹配**：`is dyn_obj { ConcreteType.Variant(fields) -> ... }` → 检查 type_name → 解构字段

**`*Err` 是 `*T` 动态指针的第一个实际应用**：
- `!T` 的 Err 变体内部存储 `*Err` — 即一个 dyn 对象
- 这个 dyn 对象的具体类型可以是任何实现了 `Err` spec 的类型
- `is e { ParseError.InvalidChar(ch, pos) -> ... }` 利用了 dyn 对象的 type_name 元数据进行分派

**未来 `*T` 的其他应用**：
- `*Drawable` — 指向任何实现了 Drawable spec 的图形对象
- `*Serializable` — 指向任何实现了 Serializable spec 的类型
- `List<*Tool>` — 元素类型为 dyn 指针的列表（动态多态集合）

### 实现步骤

#### Phase 3A：定义 Err spec（内置）
- 文件：`src/vm/native_registry.rs` 或新文件 `src/vm/err_spec.rs`
- 将 `Err` spec 注册为内置 spec（类似内置类型 `int`、`str`）
- 定义默认实现：`fn msg() str` 必须由用户类型提供

#### Phase 3B：Runtime Result 重构为堆对象
- **关键改动**：Result 不再用哨兵整数编码，改用堆对象
- 新增 `ResultValue` 运行时类型：
  ```rust
  enum ResultValue {
      Ok(Value),     // 任意类型的成功值
      Err(DynValue), // *Err 动态指针 — 指向实现 Err spec 的具体类型
  }
  ```
- `DynValue` 是 Phase 4 定义的 dyn 对象：存储 `(type_name, concrete_value, vtable_ref)`
- PC 端：完整的 dyn 对象，通过 vtable 调用 `Err.msg()` 等方法
- MCU 端：`DynValue` 退化为 `(error_type_id: u16, variant_index: u8)`，固定内存占用
- 存储在堆上，Result ID 为堆索引
- Ok/Err 内部值可以是 i32、tagged string、堆引用等任何 Value

#### Phase 3C：Opcode 调整
- `CREATE_OK`：包装值为 ResultValue::Ok，分配堆对象，压入堆 ID
- `CREATE_ERR`：
  - 接受一个堆对象（enum variant 实例）作为参数
  - 检查该对象的类型是否实现了 `Err` spec（编译期已验证）
  - 创建 `ResultValue::Err(err_value)`，分配堆对象，压入堆 ID
- `IS_OK`：从堆取出 ResultValue，判断是 Ok 还是 Err
- `UNWRAP_OK`：取出 Ok 内部值压栈
- `UNWRAP_ERR`：取出 Err 内部的 `*Err` 指针压栈（dyn 对象）
- `ERROR_PROPAGATE`：检查是否 Err，如果是则将 `*Err` 指针传播到调用者

#### Phase 3D：Err 的 is 匹配
- `is e { ConcreteType.Variant(fields) -> ... }` 编译为：
  1. 从 `*Err` 指针获取 type_name（利用 Phase 4 的 dyn 对象元数据）
  2. `IS_VARIANT` 检查类型名 + 变体名
  3. `GET_GENERIC_FIELD` 提取字段值
  4. 绑定到模式变量

#### Phase 3E：向后兼容
- 现有 `!T` 代码不需要改动 — `Err("message")` 自动包装为默认错误类型（实现了 `Err` spec 的内置 `StrError`）
- `Ok(val)` / `Err(msg)` 语法不变
- `is result { Ok(x) -> ... Err(e) -> ... }` 不变
- 新增：`Err(e)` 现在可以接受任何实现了 `Err` spec 的值

### 影响的 ac-examples
- `10_api_error_enum` — `ApiError` enum 实现 `Err` spec，`fn foo() !T` 直接返回强类型错误
- `11_tool_result_serde` — `!T` 作为函数返回类型
- `13_tool_trait_def` — `fn execute(input str) !str` + `ToolError` enum 实现 `Err` spec
- `17_context_compaction`（Plan 200）— Result 模式匹配

---

## 问题 4：Spec 完整替代 Trait（动态分派 vtable）

### 现状

- **AST**：`SpecDecl` + `Ext { trait_name }` — 完整的 spec/impl 语法
- **类型检查**：`trait_checker.rs` 验证方法签名一致性 — 已工作
- **Codegen**：`SpecDecl` 不产生任何字节码（仅元数据）
- **Ext**：方法编译为独立的 mangled 函数（`TypeName.methodName`）
- **动态分派**：**完全不存在于 AutoVM**。只有 C transpiler 生成了 vtable
- **`dyn Trait`**：VM 中无运行时表示

### 目标

```auto
// Spec 定义（已有）
spec Tool {
    fn name() str
    fn execute(input str) !str
    fn is_read_only() bool { false }  // 默认实现
}

// 类型实现 spec（已有）
type EchoTool
ext EchoTool for Tool {
    fn name() str { "Echo" }
    fn execute(input str) !str { Ok(f"echo: $input") }
    fn is_read_only() bool { true }
}

type UpperTool
ext UpperTool for Tool {
    fn name() str { "Upper" }
    fn execute(input str) !str { Ok(input.to_upper()) }
}

// 动态分派（需要新增）
fn run_tool(t Tool, input str) !str {
    // t 的实际类型在运行时决定
    t.execute(input)
}

let tools List<Tool> = [EchoTool, UpperTool]
for tool in tools {
    let result = run_tool(tool, "hello")
    print(result)
}
```

### 实现步骤

#### Phase 4A：VTable 运行时结构
- 文件：`src/vm/vtable.rs`（新文件）
- 定义 `VTable` 结构：
  ```rust
  struct VTable {
      spec_name: String,           // "Tool"
      type_name: String,           // "EchoTool"
      methods: HashMap<String, u32>, // method_name -> func_addr (字节码地址)
  }
  ```
- 全局 `VTABLE_REGISTRY: DashMap<String, VTable>` — key 为 `"TypeName:SpecName"`

#### Phase 4B：Spec 声明产生 vtable 模板
- 文件：`src/vm/codegen.rs`
- `Stmt::SpecDecl` 编译时注册 vtable 模板：
  - 记录 spec 名称和方法列表
  - 为每个方法分配一个方法槽位索引

#### Phase 4C：Ext for Spec 注册 vtable 实例
- 当编译 `ext EchoTool for Tool { ... }` 时：
  1. 编译每个方法为独立函数（已有逻辑）
  2. 额外创建 `VTable { spec_name: "Tool", type_name: "EchoTool", methods: { "name" -> addr1, "execute" -> addr2, ... } }`
  3. 注册到全局 `VTABLE_REGISTRY`

#### Phase 4D：dyn 类型运行时表示 + `*T` 动态指针
- 当变量类型为 spec 类型（如 `Tool`）时：
  - 堆对象存储 `(spec_name, type_name, concrete_value, vtable_ref)`
  - 类似 `GenericInstanceData { mono_name: "EchoTool", fields: [...] }` 但附带 vtable 引用
- 新增 opcode `MAKE_DYN`：接受 type_name + value，创建 dyn 对象
- 新增 opcode `DYN_CALL`：接受 method_name，通过 vtable 查找并调用
- **`*T` 语法集成**：
  - codegen 在编译 `Type::Ptr(PtrType)` 时，检查 `PtrType.of` 的目标类型
  - 如果目标为 `Type::Spec(...)` → 生成 dyn 对象相关指令（`MAKE_DYN` / `DYN_CALL`）
  - 如果目标为其他类型 → 保持现有 `TYPE_CAST_PTR` 行为
  - 这样 `*Err`、`*Tool`、`*Drawable` 等都自动获得 dyn 语义

#### Phase 4E：List<dyn Spec> 支持
- `List<Tool>` 中的每个元素是一个 dyn 对象（type_name + value + vtable_ref）
- `tools[0].name()` 编译为：加载元素 → `DYN_CALL("name")`
- `for tool in tools { tool.execute(input) }` — 每次迭代动态查找 vtable

#### Phase 4F：类型推断与静态分派优化
- 当编译器能确定具体类型时（如 `EchoTool.name()`），仍然使用静态分派（直接函数调用）
- 只有类型为 spec 的变量/参数才使用 dyn 分派
- 这避免了不必要的性能开销

### 设计考量

**Q：为什么不直接用 Rust 的 fat pointer（指针 + vtable）模式？**
A：AutoVM 是基于栈的虚拟机，没有裸指针概念。dyn 对象需要作为值存储在堆上。使用 `(type_name, value)` 对可以让 VM 在运行时查找 vtable。

**Q：性能影响？**
A：dyn 分派比静态分派多一次 HashMap 查找。可以后续优化为数组索引查找（编译时为每个 spec 分配固定槽位）。

**Q：与 C transpiler 的 vtable 的一致性？**
A：C transpiler 已有完整的 vtable 生成。AutoVM 的 vtable 设计应该与 C 输出保持逻辑一致，但实现细节可以不同（VM 用 HashMap，C 用函数指针结构体）。

### 影响的 ac-examples
- `13_tool_trait_def` — 直接使用 `spec Tool` + `List<Tool>` 动态分派
- `20_tool_registry`（Plan 200）— `Map<str, Tool>` 按 name 查找并动态执行
- `29_provider_dispatch`（Plan 200）— spec-based provider 多态

---

## 实现优先级与依赖关系

```
Plan 156（已完成）: enum 三形态统一、单字段异构变体、转译器适配
        │
        ▼
Phase 1 (enum 多字段扩展) ─────────────────────────┐
                                                     ├──> Phase 3 (!T + *Err)
Phase 4 (spec vtable + *T 动态指针) ───────────────┘
Phase 2 (闭包 HOF + 链式调用) ─────────────────── 独立，可并行
```

- **Phase 1** 在 Plan 156 基础上扩展多字段，不重复已完成的工作
- **Phase 1** 和 **Phase 4** 是 Phase 3 的前置依赖
- **Phase 2** 完全独立，可与 Phase 1/4 并行推进
- Phase 3 需要同时有 enum 多字段（错误类型定义）和 spec vtable（`*Err` 动态分派）

建议实施顺序：
1. **Phase 1**（enum 多字段）— 在 Plan 156 基础上扩展，3 个示例直接受益
2. **Phase 2**（闭包 HOF + 链式调用）— 可与 Phase 1 并行
3. **Phase 4**（spec vtable + `*T` 动态指针）— Phase 3 的必要前提
4. **Phase 3**（`!T` + `*Err`）— 在 Phase 1 + Phase 4 完成后实施

## 细节问题记录（后续讨论）

1. **enum 向后兼容**：现有的 `Atom.Int(42)` + `a._0` 模式是否保留？
   - 建议：保留。`_0` 作为位置字段名继续有效。
2. **`*Err` 的默认错误类型**：现有代码 `Err("message")` 需要一个内置的 `StrError` 类型自动包装字符串。
   - 建议：内置 `StrError` struct 实现 `Err` spec，`Err("msg")` 等价于 `Err(StrError("msg"))`。
3. **`*Err` 的 is 匹配跨类型**：一个函数可能返回多种错误类型（ParseError、IoError），`is e { ... }` 需要跨类型匹配。
   - 建议：`*Err` 指针携带 type_name 元数据，`is` 匹配时按 type_name 分派到具体类型。不匹配的走 `_` 分支。
4. **MCU 错误信息表格式**：MCU 降级后错误详情如何编码？
   - 建议：Phase 3 初期只做 PC 端实现，MCU 降级作为后续优化单独讨论。
5. **dyn 对象的所有权**：dyn 对象是引用语义还是值语义？
   - 建议：值语义（复制）。与 Auto 的默认语义一致。
6. **闭包作为 native 函数参数的实现**：native 函数（Rust 实现）如何调用 Auto 闭包？
   - 需要 VM 提供 "从 native 调用 Auto closure" 的 API（类似 Lua 的 lua_call）。
7. **spec 的默认方法**：已有 `body: Option<Box<Expr>>`，如何在 vtable 中使用？
   - 如果 ext 未覆盖，vtable 槽位指向 spec 默认实现的字节码地址。
8. **enum exhaustiveness 检查**：`is` 匹配是否要求穷举所有变体？
   - 当前不要求（允许 `_` 通配）。建议保持，加上编译期 warning。
9. **链式调用的惰性 vs 急切**：`list.map().filter()` 是每步都创建新列表（急切），还是延迟到终端操作？
   - 建议：急切求值（每步返回新列表）。简单直接，与 Auto 的直觉一致。
