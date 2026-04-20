这是一份供 AI Agent 或开发人员执行的详细设计与实现文档。

---

# 设计文档：Auto 语言参数传递优化策略 (ABO-01)

**主题**：语义上统一 View，实现上自动 Copy (Semantic View, Implementation Copy)
**目标**：在保持 Auto 语言“默认不可变借用”的简洁语义的同时，利用 Rust 后端实现小对象的“值传递”优化，以获得最大化的运行时性能。
**适用模块**：`auto-lang/a2r` (Auto to Rust Transpiler)

---

## 1. 概述 (Overview)

在 Auto 语言中，函数参数默认采用 **`view` (只读视图)** 模式。这降低了用户的心智负担，避免了不必要的拷贝。然而，对于底层的小型数据（如 `int`, `bool`, `float`），在机器码层面使用指针（引用）传递反而比直接拷贝值（寄存器传递）更慢。

本策略旨在实现编译器的**自动分流优化**：

1. **用户侧**：看到的所有默认参数都是 `view`（不可修改，不可 Move）。
2. **生成侧**：
* **Large Object (String, Vector)**  生成 Rust `&T` (引用传递)。
* **Small Object (int, bool)**  生成 Rust `T` (值传递/Copy)。



---

## 2. 详细设计 (Detailed Design)

### 2.1 类型分类标准 (Type Classification)

我们需要定义什么是“小对象 (Small Object)”。在 V1 版本中，建议采用保守策略。

**定义 `is_small_copyable(Type)`：**

* **True (使用值传递)**:
* `int`, `float`, `bool`, `char`, `byte` (原生类型)。
* `enum` (仅限不包含数据的 C-style enum)。


* **False (使用引用传递)**:
* `string` (堆分配)。
* `vector`, `map` (堆分配)。
* `struct` (无论大小，V1 暂不优化，统一走引用，避免 ABI 复杂性)。
* `closure` / `func`.



### 2.2 语义映射矩阵 (Mapping Matrix)

假设 Auto 函数定义为 `func foo(x: T)`：

| Auto 类型 `T` | Auto 语义 | Rust 生成签名 | Rust 调用点生成 | 备注 |
| --- | --- | --- | --- | --- |
| `int` | View (ReadOnly) | `fn foo(x: i64)` | `foo(val)` | **优化点**：Rust 中 i64 是 Copy，直接传值 |
| `bool` | View (ReadOnly) | `fn foo(x: bool)` | `foo(val)` | **优化点**：直接传值 |
| `string` | View (ReadOnly) | `fn foo(x: &String)` | `foo(&val)` | 标准借用 |
| `Vector` | View (ReadOnly) | `fn foo(x: &Vec<T>)` | `foo(&val)` | 标准借用 |
| `struct` | View (ReadOnly) | `fn foo(x: &MyStruct)` | `foo(&val)` | 标准借用 |

### 2.3 不变性保证 (Immutability Guarantee)

虽然对于 `int` 我们生成了 `fn foo(x: i64)`，这意味着 Rust 函数内部拥有 `x` 的所有权且可以修改它（如果是 `mut x`），但 Auto **编译器前端 (TypeChecker)** 必须拦截所有对 `x` 的修改尝试。

* **Auto 代码**：
```auto
func add_one(x: int) {
    x = x + 1; // Error: Parameter 'x' is view-only (immutable).
}

```


* **Rust 生成代码**：
```rust
fn add_one(x: i64) {
    // 即使 Rust 允许在这里 let mut x = x;
    // 但 Auto 前端不会生成修改 x 的代码。
}

```



---

## 3. 实现文档 (Implementation Guide)

以下代码逻辑基于 Rust 实现的 `a2r` 模块。

### 3.1 数据结构扩展

在 `src/types.rs` 或类似文件中，扩展类型判断逻辑。

```rust
impl Type {
    /// 判断是否应该进行“值传递优化”
    pub fn is_optimized_by_value(&self) -> bool {
        match self {
            Type::Int | Type::Float | Type::Bool | Type::Char | Type::Byte => true,
            // 可以在未来扩展：如果是 Struct 且标记了 @small 且实现了 Copy
            // Type::Struct(s) if s.is_small_pod() => true, 
            _ => false,
        }
    }
}

```

### 3.2 函数签名生成 (`transpile_func_sig`)

修改函数定义生成的逻辑。

```rust
// 输入：Auto 的参数定义 (name: String, ty: Type, mode: ParamMode)
// 输出：Rust 的参数定义字符串
fn transpile_param(name: &str, ty: &Type, mode: &ParamMode) -> String {
    let rust_ty = transpile_type(ty); // e.g., "i64", "String"

    match mode {
        // 核心优化逻辑：默认模式 (View)
        ParamMode::View => {
            if ty.is_optimized_by_value() {
                // 优化：小对象直接传值
                format!("{}: {}", name, rust_ty) 
            } else {
                // 默认：大对象传引用
                format!("{}: &{}", name, rust_ty)
            }
        },
        
        // 其他模式保持原样
        ParamMode::Mut  => format!("{}: &mut {}", name, rust_ty),
        ParamMode::Take => format!("{}: {}", name, rust_ty),
        ParamMode::Copy => format!("{}: {}", name, rust_ty), // 显式 Copy
    }
}

```

### 3.3 函数调用生成 (`transpile_call`)

修改函数调用处的参数传递逻辑。这需要上下文信息（知道目标函数的参数类型）。

```rust
// 输入：目标函数的参数信息，当前传入的表达式
fn transpile_arg(target_ty: &Type, target_mode: &ParamMode, expr: &Expr) -> String {
    let expr_code = transpile_expr(expr); // e.g., "my_var"

    match target_mode {
        // 核心优化逻辑
        ParamMode::View => {
            if target_ty.is_optimized_by_value() {
                // 优化：小对象直接传值 (Rust 会自动处理 Copy)
                expr_code 
            } else {
                // 默认：大对象传引用
                format!("&{}", expr_code)
            }
        },

        // 显式可变引用
        ParamMode::Mut => format!("&mut {}", expr_code),
        
        // 显式所有权转移 (Move)
        ParamMode::Take => expr_code,
        
        // 显式拷贝
        ParamMode::Copy => {
            if target_ty.is_optimized_by_value() {
                expr_code // i64 赋值就是 copy
            } else {
                format!("{}.clone()", expr_code) // String 需要显式 clone
            }
        }
    }
}

```

---

## 4. 边缘情况处理 (Edge Cases)

### 4.1 取地址操作 (`&x`)

如果用户在 Auto 中对一个被优化为“值传递”的参数取地址：

```auto
func check(a: int) {
    let p = &a; // 用户想要 int 的指针
}

```

* **Rust 行为**：`fn check(a: i64) { let p = &a; }`
* **结果**：完全合法。Rust 编译器检测到对参数取引用时，会将该参数从寄存器 Spill（溢出）到栈上，生成一个临时的栈地址。
* **结论**：无需特殊处理，Rust 后端会兜底。

### 4.2 结构体中的字段

如果 `struct Point { x: int, y: int }` 被作为 `view` 传递：

* 目前 V1 策略是 `fn process(p: &Point)`。
* 访问 `p.x` 时：
* Auto: `let val = p.x`
* Rust: `let val = p.x` (因为 i64 是 Copy，Rust 会自动解引用并拷贝)。


* **结论**：Rust 的自动解引用（Auto-deref）特性保证了使用的流畅性。

---

## 5. 测试计划 (Verification Plan)

AI Agent 在执行完代码修改后，需运行以下 Test Cases：

1. **Case A (Primitive View)**:
* Input: `func add(a: int, b: int) -> int { return a + b }`
* Expected Rust: `fn add(a: i64, b: i64) -> i64 { a + b }`
* Check: 确保没有 `&i64`。


2. **Case B (String View)**:
* Input: `func print(s: string) { ... }`
* Expected Rust: `fn print(s: &String) { ... }`
* Check: 确保有 `&`。


3. **Case C (Call Site)**:
* Input: `add(1, 2); print(str);`
* Expected Rust: `add(1, 2); print(&str);`


4. **Case D (Immutability)**:
* Input: `func bad(a: int) { a = 2 }`
* Action: Auto Frontend (TypeChecker) 必须报错。



---

**指令结束**：请依据此文档更新 `a2r` 模块的参数处理逻辑。