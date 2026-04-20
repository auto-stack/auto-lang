这是一份基于我们讨论内容的正式设计文档。你可以将其存档，作为 Auto 语言 v0.5 或 v1.0 阶段“生产力特性”开发的蓝图。

---

# Auto 语言特性规范：Auto 模式 (Productivity Layer)

**版本**: v1.0 (Draft)
**日期**: 2026-02-04
**状态**: 规划中 (待 System Mode 核心稳定后实施)
**标签**: Compiler Frontend, Syntactic Sugar, Error Handling

---

## 1. 概述 (Overview)

**Auto 模式 (Auto Mode)** 是建立在 Auto 语言系统级核心 (System Core) 之上的一层**生产力抽象层**。

它的核心目的是降低应用层逻辑（如 UI 交互、业务流程、配置解析）的编写门槛，通过编译器的**自动推导**和**隐式代码生成**，消除底层系统编程中频繁出现的指针检查 (`.*`, `.?`) 和错误传播 (`.!`) 样板代码。

* **设计哲学**: "Default to Propagation" (默认传播)。在 Auto 模式下，所有的失败（空指针、错误码、越界）默认向上层调用者冒泡，而非导致程序崩溃 (Panic)。
* **目标用户**: 业务逻辑开发者、前端工程师、全栈开发者。

---

## 2. 核心机制：3A 协议 (The 3A Protocol)

在 `auto` 语义作用域内，编译器将强制开启以下三种隐式行为：

### 2.1 Automatic Error Propagation (自动错误传播)

* **规则**: 当调用一个返回 `!T` (Result) 的函数时，无需显式使用 `.!` 操作符。
* **行为**: 编译器自动插入“失败则返回”的逻辑。如果函数返回 `Err`，当前执行流立即中断，并将该 `Err` 作为当前函数的返回值向上抛出。

### 2.2 Automatic Dereference (自动解引用)

* **规则**: 当访问指针 `*T` 的成员 (`ptr.field`) 时，无需显式使用 `.*` 或 `.?`。
* **行为**: 编译器自动插入空指针检查。如果指针为 `Null`，当前执行流立即中断，并返回一个标准的 `NullPointerException` 错误（包装在 `Result` 中）。

### 2.3 Automatic Type Inference (自动类型推导)

* **规则**: 允许使用 `var` 关键字定义变量，允许省略函数返回类型。
* **行为**: 编译器根据右值推导类型。**所有 `auto fn` 的返回值默认被包装为 `!T` (Result)。**

---

## 3. 语法规范 (Syntax Specification)

`auto` 关键字用于界定“自动模式”的作用域。

### 3.1 函数定义 (`auto fn`)

用于定义一个业务级函数。

```auto
// 显式声明
auto fn load_user(id: int) { ... }

// 隐式返回类型推导：
// 实际编译为 -> !User (Result<User>)

```

### 3.2 代码块 (`auto { ... }`)

在系统级函数内部嵌入一段自动模式逻辑。

```auto
fn driver_init() {
    // 严谨代码...
    let config = auto {
        // 自动模式代码...
        var x = read_json()
        x.val
    };
    // config 的类型是 !T
}

```

### 3.3 闭包/Lambda (`auto (...) =>`)

用于快速编写回调函数。

```auto
btn.onClick(auto (e) => {
    log(e.msg)
});

```

### 3.4 文件级指令 (`#auto`)

将整个源文件标记为自动模式。

```auto
#auto
fn on_start() { ... } // 默认为 auto fn

```

---

## 4. 编译器实现模型 (Compilation Model)

Auto 模式不需要新的运行时支持，它通过 **AST Lowering (抽象语法树降级)** 转换为标准的 System Mode 代码。

### 4.1 函数签名的重写

**源码 (Auto Mode):**

```auto
auto fn get_name(u: *User) {
    return u.name
}

```

**降级后 (System Mode):**

```auto
// 返回类型自动包装为 Result
fn get_name(u: *User) -> !String {
    // 自动插入空指针检查
    // match u { Null => return Err("NullPtr"), Valid(p) => ... }
    
    // 这里使用了假想的系统级宏或 try-block 语义
    return !{ u.!("NullPtr").*.name }
}

```

### 4.2 变量与调用的重写

**源码 (Auto Mode):**

```auto
var res = api_call() // api_call 返回 !Data
var val = res.value  // value 是字段

```

**降级后 (System Mode):**

```auto
// 1. 自动插入 .! (Propagate Error)
let res: Data = match api_call() {
    Ok(v) => v,
    Err(e) => return Err(e), // 遇到错误直接甩给上层
};

// 2. 指针访问转换 (假设 res 是对象，value 是字段)
let val = res.value; 
// 或者如果 res 是指针:
// let val = match res { Null => return Err("NullPtr"), ... };

```

### 4.3 数组越界的重写

**源码 (Auto Mode):**

```auto
var item = list[5]

```

**降级后 (System Mode):**

```auto
// 自动边界检查
let item = if (list.len > 5) {
    list.get_unchecked(5)
} else {
    return Err("IndexOutOfBounds");
}

```

---

## 5. 互操作性 (Interoperability)

Auto 模式与 System 模式必须无缝共存。

### 5.1 System 调用 Auto

由于 `auto fn` 总是返回 `!T` (Result)，System 代码调用它时，必须显式处理结果。

```auto
// System Context
fn main() {
    // 必须处理 auto fn 抛出的错误
    let result = login_logic(); 
    match result {
        Ok(_) => print("Success"),
        Err(e) => log_error(e),
    }
}

```

### 5.2 Auto 调用 System

`auto fn` 调用 System 函数时，如果 System 函数返回 `!T`，会自动解包；如果返回 `T`，则直接使用。

```auto
// Auto Context
auto fn logic() {
    // sys_func 返回 !int
    var x = sys_func() // 自动处理 Err
    // pure_func 返回 int
    var y = pure_func()
    return x + y
}

```

---

## 6. 错误处理策略 (Error Strategy)

在 Auto 模式下，所有隐式错误（Implicit Errors）需要标准化为统一的 `Error` 类型（类似于 JS 的 Error 或 Python 的 Exception）。

建议标准库提供以下标准错误变体：

* `Err::NullPointer` (访问了空指针)
* `Err::IndexOutBounds` (数组/Slice越界)
* `Err::UnwrapFailed` (强制解包失败)
* `Err::Propagated(msg)` (下层函数抛出的命名错误)

---

## 7. 演进路线图 (Roadmap)

1. **Phase 1 (Kernel)**: 完成 Auto 语言核心（System Mode），确保 `Result` (`!T`) 和 `Option` (`?T`) 的底层机制健壮，支持 `!{}` 语法糖。
2. **Phase 2 (Desugar)**: 实现编译器的 AST Lowering Pass，支持 `auto` 关键字，将其翻译为 Phase 1 的语法结构。
3. **Phase 3 (IDE Support)**: 优化 IDE 体验。在 `auto` 模式下，尽管底层是 `!String`，IDE 提示应显示为 `String (Auto)`，减少视觉干扰。

---

## 8. 示例对照

| 场景 | System Mode (现有设计) | Auto Mode (未来设计) |
| --- | --- | --- |
| **定义变量** | `let x: *User = ...;` | `var x = ...` |
| **解引用** | `let n = ptr.*.name;` | `var n = ptr.name` |
| **空指针处理** | `if (!ptr) panic("Null");` | (隐式: 为空自动 Return Err) |
| **函数调用** | `let res = func().!;` | `var res = func()` |
| **返回类型** | `-> !bool` | (隐式推导, 默认为 `!T`) |
| **数组访问** | `arr[i]` (可能 Panic) | `arr[i]` (自动 Return Err) |

---

**批准人**: [架构师姓名]
**执行阶段**: 待核心编译器稳定后启动