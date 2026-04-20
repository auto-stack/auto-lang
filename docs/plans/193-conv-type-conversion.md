# Plan 193: Auto 类型转换系统 (`Conv<From, To>` + `.to()` 方法)

## 状态: DRAFT

## 背景

Auto 目前没有统一的类型转换机制。用户无法将 `i64` 转为 `String`、将 `str` 转为 `int` 等。
Rust 使用 `From<T>`/`Into<T>` trait 对；D 语言使用 `std.conv` + `to!()` 泛型函数。
Auto 需要一套同样优雅、类型安全且零成本的转换方案。

## 设计目标

1. **统一接口**: 所有类型转换通过 `.to(TargetType)` 方法调用
2. **编译期类型安全**: `Conv<From, To>` spec 约束确保只有已声明的转换才能通过编译
3. **零成本抽象**: `Conv` 实现编译时展开为直接调用，无虚分发
4. **owned vs slice 语义**: 明确区分 `.to(String)` (owned) 和 `.to(str)` (临时 slice)
5. **可扩展**: 用户可以为自己的类型实现 `Conv`

## 核心设计

### 1. `Conv<From, To>` Spec

```auto
/// 核心转换 spec。实现此 spec 以支持类型之间的转换。
/// From = 源类型, To = 目标类型
spec Conv<From, To> {
    /// 将 self 从 From 类型转换为 To 类型
    fn convert() To
}
```

### 2. `.to(TargetType)` 语法糖

```auto
// 用户代码
let n i64 = 42
let s = n.to(String)    // 编译器查找 Conv<i64, String>

// 编译器展开为
let s = Conv<i64, String>.convert(n)
```

### 3. 搜索顺序

当编译器遇到 `expr.to(TargetType)` 时:

1. **标准库**: 搜索 `auto.conv` 中注册的 `Conv<typeof(expr), TargetType>` 实现
2. **用户导入**: 搜索 `use` 语句导入的模块中的 `Conv` 实现
3. **当前作用域**: 搜索当前文件中的 `ext` 块定义的 `Conv` 实现
4. **未找到**: 编译错误: `"No Conv<{From}, {To}> implementation found. Consider implementing Conv<{From}, {To}>."`

### 4. `TryConv<From, To>` — 可能失败的转换

```auto
/// 可能失败的转换 spec
spec TryConv<From, To> {
    /// 尝试转换，失败返回 None
    fn try_convert() ?To
}
```

使用 `.try_to()` 调用:

```auto
let n = "42".try_to(int)      // Some(42)
let bad = "abc".try_to(int)   // None
let safe = "abc".try_to(int) ?? 0  // 带默认值
```

### 5. `.to(str)` 的临时变量与逃逸检查 (Phase 2)

**规则**: `.to(str)` 生成的 slice 结果不能逃逸当前作用域。

编译器在遇到 `expr.to(str)` 且 `typeof(expr)` 不是 `String` 时:

1. 生成临时变量: `let __conv_tmp_N = expr.to(String)`
2. 返回 `__conv_tmp_N.to(str)` (零成本 slice)
3. `__conv_tmp_N` 的生命周期与结果变量绑定

**逃逸检查** — 以下场景报错:

```auto
// ❌ return x.to(str)  — 逃逸当前函数
fn foo() str {
    let n i64 = 42
    return n.to(str)    // 编译错误
    // 错误提示: ".to(str) result would outlive its temporary.
    //           Use .to(String) instead."
}

// ❌ obj.field = x.to(str)  — 存入结构体字段
type Foo { data str }
let f = Foo()
f.data = 42.to(str)    // 编译错误

// ✅ print(x.to(str))  — 即用即弃
print(42.to(str))      // 合法

// ✅ let s = x.to(str)  — 局部变量（不逃逸）
let s = 42.to(str)     // 合法

// ❌ 但如果 s 被 return — 在 return 处报错
fn bar() str {
    let s = 42.to(str)  // 这里合法
    return s             // 这里报错
}
```

**错误消息**:

```
error: .to(str) result escapes its scope
  ──[main.at:5:12]
   │
 5 │     return n.to(str)
   │            ^^^^^^^^
   │
   = help: Use .to(String) instead to create an owned value.
```

## 实现计划

### Phase 1: 基础 `.to(OwnedType)` 转换

#### Step 1.1: 创建 `auto.conv` 标准库模块

**文件**: `stdlib/auto/conv.at`

```auto
/// Type conversion module
///
/// Provides Conv<From, To> spec for type-safe conversions.
/// Use .to(TargetType) to convert between types.

/// Core conversion spec
pub spec Conv<From, To> {
    fn convert() To
}

/// Fallible conversion spec
pub spec TryConv<From, To> {
    fn try_convert() ?To
}
```

**文件**: `stdlib/auto/conv.vm.at`

```auto
/// VM-implemented conversion functions

// i64 -> String
#[vm]
fn i64_to_string(n i64) String

// i64 -> f64
#[vm]
fn i64_to_f64(n i64) f64

// f64 -> i64 (truncates)
#[vm]
fn f64_to_i64(n f64) i64

// String -> i64 (panics on invalid)
#[vm]
fn string_to_i64(s String) i64

// String -> i64 (returns None on invalid)
#[vm]
fn string_try_to_i64(s String) ?i64

// u64 -> String (hex)
#[vm]
fn u64_to_hex(n u64) String

// bool -> String
#[vm]
fn bool_to_string(b bool) String
```

#### Step 1.2: 注册 Rust FFI shim

**文件**: `crates/auto-lang/src/vm/ffi/stdlib.rs`

在 `register_stdlib_ffi()` 中添加 conv 相关 shim:

```rust
// Conv shim IDs: 1400-1449
const NATIVE_CONV_I64_TO_STRING: u32 = 1400;
const NATIVE_CONV_I64_TO_F64: u32 = 1401;
const NATIVE_CONV_F64_TO_I64: u32 = 1402;
const NATIVE_CONV_STRING_TO_I64: u32 = 1403;
const NATIVE_CONV_STRING_TRY_TO_I64: u32 = 1404;
const NATIVE_CONV_U64_TO_HEX: u32 = 1405;
const NATIVE_CONV_BOOL_TO_STRING: u32 = 1406;

#[auto_macros::rust_fn("Conv.i64_to_string")]
pub fn shim_i64_to_string(n: i64) -> String {
    n.to_string()
}

#[auto_macros::rust_fn("Conv.i64_to_f64")]
pub fn shim_i64_to_f64(n: i64) -> f64 {
    n as f64
}

// ... 其余 shim 类似
```

**文件**: `crates/auto-lang/src/vm/native_registry.rs`

注册 native ID:

```rust
// Conv functions (1400-1449)
registry.register_with_id("auto.conv.i64_to_string", 1400);
registry.register_with_id("auto.conv.i64_to_f64", 1401);
// ...
```

#### Step 1.3: 注册 Conv 实现 (Ext blocks)

**文件**: `stdlib/auto/conv.at` (追加)

```auto
// Conv<i64, String> implementation
ext i64 has Conv<i64, String> {
    fn convert() String {
        i64_to_string(self)
    }
}

// Conv<i64, f64> implementation
ext i64 has Conv<i64, f64> {
    fn convert() f64 {
        i64_to_f64(self)
    }
}

// Conv<String, i64> implementation
ext String has Conv<String, i64> {
    fn convert() i64 {
        string_to_i64(self)
    }
}

// TryConv<String, i64> implementation
ext String has TryConv<String, i64> {
    fn try_convert() ?i64 {
        string_try_to_i64(self)
    }
}

// Conv<u64, String> implementation (hex)
ext u64 has Conv<u64, String> {
    fn convert() String {
        u64_to_hex(self)
    }
}

// Conv<bool, String> implementation
ext bool has Conv<bool, String> {
    fn convert() String {
        bool_to_string(self)
    }
}
```

#### Step 1.4: 解析器支持 `.to(Type)` 语法

**文件**: `crates/auto-lang/src/parser.rs`

当解析器遇到 `expr.to(Type)` 时:

1. 解析 `expr` 作为 receiver
2. 识别 `.to` 作为特殊方法名
3. 解析 `(Type)` 作为目标类型参数
4. 生成 AST 节点 `Expr::Conv { receiver, target_type }`

AST 定义 (添加到 `crates/auto-lang/src/ast/expr.rs`):

```rust
/// Type conversion expression: expr.to(TargetType)
/// Lowers to Conv<typeof(expr), TargetType>.convert(expr)
pub struct ConvExpr {
    pub receiver: Box<Expr>,
    pub target_type: Type,
    pub span: Span,
}
```

#### Step 1.5: 语义分析与 Codegen

**类型检查** (`crates/auto-lang/src/infer/`):

1. 推导 `receiver` 的类型 `From`
2. 构造约束 `Conv<From, TargetType>`
3. 查找已注册的 `Conv` 实现:
   - 检查 `type_decls` 中 `From` 类型的 `methods` 列表
   - 匹配 `has Conv<From, TargetType>` 的 ext 块
4. 未找到则报编译错误

**Codegen** (`crates/auto-lang/src/vm/codegen.rs`):

将 `expr.to(String)` 编译为:

1. 计算 `expr` 的值（push 到栈）
2. 查找 `Conv<typeof(expr), String>` 对应的 native 函数 ID
3. 发射 `CALL_NAT <id>` 指令

#### Step 1.6: `parse_module_to_type_store` 支持 Ext 块

**文件**: `crates/auto-lang/src/compile.rs` (第 519 行附近)

当前 `parse_module_to_type_store` 只处理 `Stmt::Fn`、`Stmt::TypeDecl`、`Stmt::SpecDecl`。
需要添加 `Stmt::Ext` 处理:

```rust
for stmt in &ast.stmts {
    match stmt {
        Stmt::Fn(fn_decl) => {
            type_store.register_fn_decl(fn_decl);
        }
        Stmt::TypeDecl(type_decl) => {
            type_store.register_type_decl(type_decl);
        }
        Stmt::SpecDecl(spec_decl) => {
            type_store.register_spec_decl(spec_decl);
        }
        // 新增: 处理 Ext 块
        Stmt::Ext(ext) => {
            type_store.register_ext_methods(ext);
        }
        _ => {}
    }
}
```

**文件**: `crates/auto-lang/src/types.rs` — 添加 `register_ext_methods`:

```rust
/// 注册 ext 块中的方法到对应类型的 method 列表
pub fn register_ext_methods(&mut self, ext: &Ext) {
    let type_name = AutoStr::from(ext.target.as_str());

    // 如果类型已存在，追加方法
    if let Some(decl) = self.type_decls.get_mut(&type_name) {
        let decl = Arc::make_mut(decl);
        for method in &ext.methods {
            decl.methods.push(method.clone());
        }
    } else {
        // 类型不存在，创建一个占位 TypeDecl 并注册方法
        let placeholder = TypeDecl {
            name: ext.target.clone(),
            kind: TypeDeclKind::External,  // 外部类型占位
            parent: None,
            has: vec![],
            specs: vec![],
            spec_impls: vec![],
            generic_params: vec![],
            members: vec![],
            methods: ext.methods.clone(),
        };
        self.type_decls.insert(type_name, Rc::new(placeholder));
    }
}
```

同样，`import_items` 需要支持从 ext 方法中按名查找:

```rust
pub fn import_items(&mut self, other: &TypeStore, items: &[String]) {
    for item in items {
        // ... 现有逻辑 ...

        // 新增: 在类型声明的方法中查找
        if !found {
            for (_, decl) in &other.type_decls {
                for method in &decl.methods {
                    if method.name.as_str() == item.as_str() {
                        // 找到了 ext 方法，注册到目标类型
                        self.register_ext_method_for_type(decl.name.as_str(), method.clone());
                        found = true;
                        break;
                    }
                }
                if found { break; }
            }
        }
    }
}
```

### Phase 2: `.to(str)` 临时变量 + 逃逸检查

#### Step 2.1: 识别需要临时变量的场景

当 `.to(str)` 的 receiver 类型不是 `String` 时:

```rust
// 语义分析阶段
fn check_conv_expr(&mut self, expr: &ConvExpr) -> AutoResult<()> {
    let from_type = self.infer_type(&expr.receiver)?;
    let to_type = &expr.target_type;

    if to_type == Type::Str && from_type != Type::String {
        // 标记此表达式需要临时变量
        self.mark_needs_temp(expr.id);
    }
    Ok(())
}
```

#### Step 2.2: 逃逸检查

在函数体级别做数据流分析:

```rust
fn check_no_escape(&self, var_id: VarId, fn_body: &[Stmt]) -> AutoResult<()> {
    for stmt in fn_body {
        match stmt {
            Stmt::Return(expr) if self.references_var(expr, var_id) => {
                return Err(AutoError::Msg(
                    ".to(str) result escapes its scope. Use .to(String) instead.".into()
                ));
            }
            Stmt::Store(store) if store.is_field && self.references_var(&store.value, var_id) => {
                return Err(AutoError::Msg(
                    ".to(str) result assigned to field. Use .to(String) instead.".into()
                ));
            }
            _ => {}
        }
    }
    Ok(())
}
```

#### Step 2.3: Codegen 插入临时变量

```rust
// 编译 expr.to(str) 时:
// 1. 生成 let __conv_tmp_N = expr.to(String)
// 2. 生成 __conv_tmp_N.to(str)  (零成本 slice)
// 3. 标记 __conv_tmp_N 生命周期与目标变量绑定
```

## 预定义转换表

| From | To | 方式 | Phase |
|------|-----|------|:-----:|
| `i64` | `String` | `Conv` (#[vm]) | 1 |
| `i64` | `f64` | `Conv` (#[vm]) | 1 |
| `i64` | `int` | `Conv` (#[vm]) | 1 |
| `f64` | `i64` | `Conv` (#[vm], truncates) | 1 |
| `f64` | `int` | `Conv` (#[vm], truncates) | 1 |
| `String` | `i64` | `Conv` (#[vm], panics) | 1 |
| `String` | `int` | `Conv` (#[vm], panics) | 1 |
| `String` | `?i64` | `TryConv` (#[vm]) | 1 |
| `String` | `?int` | `TryConv` (#[vm]) | 1 |
| `u64` | `String` | `Conv` (#[vm], hex) | 1 |
| `u32` | `String` | `Conv` (#[vm]) | 1 |
| `bool` | `String` | `Conv` (#[vm]) | 1 |
| `String` | `str` | `Conv` (零成本 slice) | 1 |
| `str` | `String` | `Conv` (#[vm], alloc) | 1 |
| `i64` | `str` | 临时变量 + 逃逸检查 | 2 |
| `f64` | `str` | 临时变量 + 逃逸检查 | 2 |
| `u64` | `str` | 临时变量 + 逃逸检查 | 2 |
| `bool` | `str` | 临时变量 + 逃逸检查 | 2 |

## 与现有系统的兼容性

### `.as(Type)` vs `.to(Type)`

- `.as(Type)`: 零成本重解释 (reinterpret cast)，如 `byte.as(int)` = 98
- `.to(Type)`: 可能涉及计算的转换，如 `42.to(String)` = "42"

两者并存，语义不同。

### 现有 `to_string()` / `to_int()` 方法

逐步迁移为 `Conv` 实现的语法糖:
- `n.to_string()` → `n.to(String)`
- `s.to_int()` → `s.to(int)` 或 `s.try_to(int)`

旧方法保留为 deprecated，兼容期后移除。

## 验证方案

### 单元测试

```auto
// test_conv.at
use auto.conv

fn test_basic_conversions() {
    // int -> String
    let s = 42.to(String)
    assert_eq(s, "42")

    // int -> f64
    let f = 42.to(f64)
    assert(f == 42.0)

    // String -> int
    let n = "123".to(int)
    assert_eq(n, 123)

    // hex
    let h = 255u64.to(String)
    assert_eq(h, "000000ff")

    // bool
    assert_eq(true.to(String), "true")
    assert_eq(false.to(String), "false")

    print("conv: all tests passed")
}
```

### 逃逸检查测试 (Phase 2)

```auto
fn bad_escape() str {
    let n i64 = 42
    return n.to(str)    // 编译错误: .to(str) result escapes
}

fn good_local() {
    let s = 42.to(str)  // 合法: 局部使用
    print(s)             // 合法: 即用即弃
    // s 不逃逸，OK
}
```

## 文件清单

| 文件 | 变更类型 | Phase |
|------|---------|:-----:|
| `stdlib/auto/conv.at` | 新建 | 1 |
| `stdlib/auto/conv.vm.at` | 新建 | 1 |
| `crates/auto-lang/src/vm/ffi/stdlib.rs` | 修改 (添加 conv shim) | 1 |
| `crates/auto-lang/src/vm/native_registry.rs` | 修改 (注册 ID 1400-1449) | 1 |
| `crates/auto-lang/src/ast/expr.rs` | 修改 (添加 ConvExpr) | 1 |
| `crates/auto-lang/src/parser.rs` | 修改 (解析 .to(Type)) | 1 |
| `crates/auto-lang/src/infer/expr.rs` | 修改 (类型检查 Conv) | 1 |
| `crates/auto-lang/src/vm/codegen.rs` | 修改 (生成 CALL_NAT) | 1 |
| `crates/auto-lang/src/compile.rs` | 修改 (Stmt::Ext 处理) | 1 |
| `crates/auto-lang/src/types.rs` | 修改 (register_ext_methods, import_items) | 1 |
| `crates/auto-lang/src/vm/codegen.rs` | 修改 (临时变量插入) | 2 |
| `crates/auto-lang/src/infer/escape.rs` | 新建 (逃逸检查) | 2 |

## 工期估计

| Phase | 工作量 | 依赖 |
|-------|--------|------|
| Phase 1 | 2-3 天 | spec 约束求解基本可用 |
| Phase 2 | 1-2 天 | Phase 1 完成 |
| **总计** | **3-5 天** | |
