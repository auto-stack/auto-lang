这是关于 **Auto 语言 (.at)** 中核心类型 **`May<T>` (语法缩写为 `?T`)** 的完整设计与实施文档。该设计旨在通过合并 `Option` 与 `Result` 的语义，在 C 语言翻译层面上实现极致的性能，并在语法层面上提供统一的线性流体验。

---

# Auto 语言设计文档：三态复合类型 `May<T>`

## 1. 设计动机

在传统的系统级语言中，程序员通常需要同时处理“数据缺失”（Option）和“执行失败”（Result）。这导致了嵌套逻辑（如 `Result<Option<T>, E>`）和复杂的错误处理分支。
`May<T>` 的目标是：

* **简化模型**：将存在性与正确性合并为统一的三态模型。
* **线性表达**：配合点号属性操作符 `.?` 实现无嵌套的链式调用。
* **跨平台适配**：一套源码，在 PC 上支持富错误信息，在 MCU 上支持零分配错误码。

---

## 2. 逻辑定义

`May<T>`（即 `?T`）是一个三态枚举，其可能的状态包括：

| 状态 | 内部标识 (Tag) | 语义 | C 语言翻译映射 |
| --- | --- | --- | --- |
| **Value** | `0x01` | 操作成功并返回有效数据 `T` | `struct.data.value` |
| **Empty** | `0x00` | 操作成功，但逻辑上没有数据 (`nil`) | 无负载 |
| **Error** | `0x02` | 操作失败，携带异常/错误信息 `E` | `struct.data.err` |

假如用Auto语言来定义，那么May<T>实际上是一个`tag`类型，即`tagged-unino`：

```auto
tag May<T> {
    nil Nil
    err Err
    val T
}
```

这的`nil`是一个全局唯一的空`struct`，表示不存在。相当于Rust里的None。

Auto编译器提供语法糖，将任何形式的`?T`都转换为`May<T>`.

对任意`?T`对象，提供三个方法来判断其属性：

```auto
ext ?T {
    fn is_some() bool
    fn is_nil() bool
    fn is_err() bool
}
```

`May<T>`需要提供几个全局方法来构建不同属性的值：

```auto
fn some(v T) ?T

fn nil() ?T

fn err(e Err) ?T
```

对任意`?T`对象，可以用`?`来实施解构操作：

```auto

let t = some(5)

let n = t.? // n == 5

let z = nil()

let n = z.? // will trigger early return to the upper level 

let e = err("An error")

let n = e.? // will trigger early return of the error to the upper level
```

这里当`.?`遇到了`nil`或`err`的情况时，表现和Rust的`?`操作符类似：

1. 如果当前函数的返回类型是任意`?V`类型，会触发提前返回，返回类型由于是`nil`或`err`，因此可以直接被当做`?V`类型返回。这是因为即使`V`和`T`不同，但他们的`nil`和`err`是相通的，编译器可以做出适当的类型转换来适配

2. 如果当前函数的返回类型不是`?V`，而是某种普通数据类型，这里应该编译器报错。即：由于`z.?`可能会返回`nil`，与声明的返回类型不符。

另外，与Rust的`match`类似，`?T`类型也应当支持用`is`语句解构：

```auto
let t = some(5)

is t {
    nil => {print("t is nil!")}
    err(e) => {print(`error: $e`)}
    n => {print(n)}
}
```

---

## 3. 内存布局与 C 语言实现

为了保证翻译到 C 语言后的可预测性和性能，编译器为每种 `T` 类型生成特定的 `May` 结构体。

### 3.1 通用布局（以 `?i32` 为例）

```c
// 翻译层生成的 C 代码
typedef struct {
    uint8_t tag;       // 判别式：0=Empty, 1=Value, 2=Error
    union {
        int32_t value; // T 类型数据
        void* err;   // 错误负载（PC 为指针，MCU 为地址映射的错误码）
    } data;
} May_i32;

```

### 3.2 空指针优化 (Niche Optimization)

如果 `T` 是指针或引用类型，编译器将压缩 `tag`。

* `0x0` -> `Empty`
* `0x1` -> `Error` (保留低地址)
* `>0x1` -> `Value` (合法内存地址)
**结果**：对于指针类型，`?T` 的开销为 **0 字节**（与原指针等大）。

---

## 4. 语法操作语义

### 4.1 传播操作符 `.?`

当使用 `obj.?.member` 时，编译器执行以下等价展开（伪代码）：

```rust
// Auto 源码
let result = f().?.name

// 编译器生成的控制流
let _tmp = f()
if _tmp.tag == Error { return _tmp.as_error } // 错误冒泡
if _tmp.tag == Empty { return nil }           // 空值传播
let result = _tmp.value.name                  // 正常解包

```

### 4.2 空值合并操作符 `??`

用于提供回退默认值：

```rust
let age = get_age().? ?? 18

```

---

## 5. 跨平台实施策略：双态错误模型

`May<T>` 如何在不修改业务代码的情况下适配 PC 和 MCU？核心在于 `Error` 负载的解释。

### 5.1 PC 模式 (Rich Error)

* **实现**：`data.err` 指向堆上的 `Error` 对象。
* **内容**：包含错误字符串、调用堆栈、嵌套错误。
* **特性**：便于调试。

### 5.2 MCU 模式 (Lean Error)

* **实现**：`data.err` 是一个数值（如 `404`），通过强制类型转换存储在指针变量中。
* **物理表现**：不产生任何内存分配，仅占用 4/8 字节的寄存器/内存空间。
* **特性**：绝对的确定性和极致的性能。

---

## 6. 性能评估

### 6.1 翻译开销

* **分支预测**：由于 `May<T>` 将两次判断（`is_some` 和 `is_ok`）合并为一次三向分支，在 C 语言层面通常编译为一条 `switch` 或 `if-else` 链，对现代 CPU 的分支预测器极为友好。
* **代码体积**：相比于 Rust 生成的庞大泛型单态化代码，Auto 翻译的 C 代码结构体简单，函数内联（Inline）后几乎与手写 C 错误检查等效。

### 6.2 存储效率

由于消灭了嵌套，对于 `?bool` 等小类型，结合位域技术，Auto 可以在 1 个字节内表达“真、假、空、错”四种状态。

---

## 7. 复杂示例：文件读取流

这个例子展示了 `May<T>` 如何在复杂的级联中保持代码简洁：

```rust
// .at 源码
[pub, static]
fn get_first_line(path str) ?str {
    // 每一个 .? 都可能因为 Error 或 Empty 提前返回
    // 这种线性感是 May<T> 的核心优势
    let line = File.open(path).?.readline().?
    
    return line.view
}

// 翻译后的 C 逻辑逻辑简述
// 1. 调用 File_open，检查 tag。若非 Value 则直接 return。
// 2. 调用 readline，检查 tag。若非 Value 则直接 return。
// 3. 返回 line 的视图。

```

---

### 文档总结

`May<T>` 并非简单的语法糖，它是 Auto 语言处理底层不确定性的**核心基础设施**。它通过统一三态逻辑，使得同一段代码既能在 PC 上作为高级业务逻辑运行，也能在资源极度受限的 MCU 上作为硬实时代码运行。
