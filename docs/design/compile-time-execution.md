这是一个关于 **Auto 语言编译期执行 (Comptime Execution)** 的完整设计文档。该设计吸收了 D 语言 `static if` 的语义结构、Zig `comptime` 的能力以及 C 预处理器的直观性，形成了一套 **基于 AST 的显式元编程系统**。

---

# Design Document: Auto Meta (Compile-Time Execution)

**版本**: 1.0
**状态**: 定稿 (Finalized)
**目标**: 提供一套显式、类型安全、基于 AST 结构的元编程机制，用于代码生成、常量计算和条件编译。

## 1. 核心哲学 (Philosophy)

1. **显式优于隐式 (Explicit > Implicit)**: 程序员应能通过符号 `#` 一眼区分哪些代码在编译期运行，哪些代码在运行时运行。
2. **两阶段编译 (Two-Stage Compilation)**:
* **Stage 1 (Meta-Eval)**: 编译器执行所有 `#` 标记的代码，进行逻辑计算和 AST 裁剪/展开。
* **Stage 2 (Codegen)**: 对 Stage 1 生成的纯净 AST 进行常规的类型检查和机器码生成。


3. **语句级提升 (Statement-Level Lifting)**: `#` 修饰符作用于整个语句结构（包括子句），而非单个 Token。

---

## 2. 语法规范 (Syntax Specification)

### 2.1 编译期控制流 (Comptime Control Flow)

使用 `#` 作为前缀，标记该语句在编译期执行。其块（Block）内部的代码默认行为是 **发射 (Emit)** 到运行时 AST。

#### 条件分支 (`#if`)

`#` 作用于整个 `if-elif-else` 结构。后续的分支自动继承编译期属性，无需重复加 `#`。

```auto
#if OS == "windows" {
    // [Emit] 只有在 Windows 下才会生成此代码
    init_win32()
} elif OS == "linux" {
    // [Emit] 只有在 Linux 下才会生成此代码
    init_linux()
} else {
    // [Compile Error] 编译期报错
    compile_error("Unsupported OS")
}

```

#### 模式匹配 (`#is`)

整个模式匹配在编译期完成展开。

```auto
#is ARCH {
    "x64" => { include_asm("x64.s") }
    "arm" => { include_asm("arm.s") }
    else  => { panic("Unknown Arch") }
}

```

#### 循环展开 (`#for`)

编译器会执行循环，并将循环体内的代码重复发射。

```auto
// 假设我们要生成 print(0); print(1); print(2);
#for i in 0..3 {
    // #{i} 是插值，将编译期变量转为运行时字面量
    print(#{i})
}

```

### 2.2 编译期声明与计算 (Comptime Declarations)

不再需要 `#let`。利用 Auto 现有的关键字进行正交化设计。

* **`const`**: 定义编译期常量（以及运行时可见的全局常量）。
* **`type`**: 定义类型别名（类型本身就是编译期概念）。
* **`#{ ... }`**: **求值块 (Evaluation Block)**。

#### 求值块 (`#{ ... }`)

这是一个**编译期表达式**。编译器暂停代码生成，执行块内逻辑，并将最后一行结果作为值返回。块内可以使用 `let/var` 定义临时变量。

```auto
// 定义一个复杂的编译期常量
const FIB_TABLE = #{
    // 这里的代码完全在编译期运行，不会进入二进制
    var t = [0; 100]
    t[0] = 0; t[1] = 1
    for i in 2..100 { t[i] = t[i-1] + t[i-2] }
    t // 返回数组
}

// 定义一个根据环境变化的类型
type MyInt = #{
    if ARCH == "x64" { u64 } else { u32 }
}

```

### 2.3 注入与插值 (Injection)

在运行时代码（或生成的代码）中，使用 `#{expr}` 将编译期表达式的值嵌入为 AST 字面量。

```auto
fn get_version() str {
    // 假设 VERSION 是 const 定义的
    // 效果等同于 return "1.0.0"
    return #{VERSION} 
}

```

---

## 3. 原理与分析 (Analysis)

### 3.1 编译流程图解

1. **Parser**: 解析源码，生成包含 `ComptimeStmt`（带 `#` 的节点）的原始 AST。
2. **Interpreter (Stage 1)**: 遍历 AST。
* 遇到普通节点 -> 保留。
* 遇到 `ComptimeStmt` -> 执行逻辑，根据结果保留或丢弃子节点（AST 裁剪），或复制节点（AST 展开）。
* 遇到 `#{...}` -> 计算结果，替换为 `LiteralNode`。


3. **Semantic Analysis (Stage 2)**: 对处理后的 AST 进行类型检查。
4. **Backend**: 生成目标代码。

### 3.2 作用域规则

* **向下可见**: `#` 块内的代码可以读取外部的 `const` 常量。
* **隔离性**: `#{ ... }` 块内定义的 `let/var` 是局部的，计算结束后销毁，不会污染全局命名空间。

---

## 4. 与其他语言比较 (Comparison)

| 特性 | Auto (`#`) | C Preprocessor (`#`) | Zig (`comptime`) | D (`static if`) |
| --- | --- | --- | --- | --- |
| **基础机制** | AST 结构化操作 | 文本替换 (Token) | 语义分析期部分求值 | AST 结构化操作 |
| **类型安全** | ✅ 强类型 | ❌ 无类型 | ✅ 强类型 | ✅ 强类型 |
| **显式性** | ⭐ 高 (必须加 `#`) | ⭐ 高 (必须加 `#`) | 低 (隐式颜色) | 中 (关键字) |
| **作用域** | 语句级 (Statement) | 无 (文本行) | 表达式级 | 语句级 |
| **易读性** | 清晰区分 Meta/Runtime | 宏地狱 | 需推导变量属性 | 类似 C++ |
| **实现难度** | 中等 (需解释器) | 低 | 极高 (部分求值) | 中等 |

**优势总结**:

* 相比 **C**: Auto 拥有完整的语言能力（循环、变量、类型检查），不仅仅是宏替换。
* 相比 **Zig**: Auto 的 `#` 明确了边界，嵌入式开发者不需要猜测代码是在 Host 跑还是在 Target 跑。
* 相比 **D**: Auto 使用符号 `#` 而非冗长的 `static if`，视觉上更符合系统语言的紧凑感。

---

## 5. 实现规划 (Implementation Plan)

为了实现这套功能，编译器团队需要按以下步骤行动：

### Phase 1: 解析器升级 (Parser Support)

* **任务**: 修改 Parser，识别 `#` 前缀。
* **产出**:
* 新增 AST 节点类型：`ComptimeIfStmt`, `ComptimeForStmt`, `ComptimeBlockExpr` (`#{}`).
* 确保 `#if ... else` 被解析为一个完整的树节点，而不是分离的 Token。



### Phase 2: 编译期解释器 (The Meta-Evaluator)

* **任务**: 实现一个轻量级的 Tree-Walk Interpreter 或 Bytecode VM。
* **能力**:
* 支持基础算术、逻辑运算。
* 支持控制流 (if/loop)。
* 支持数组和结构体的创建。
* **关键**: 必须模拟目标平台（Target Platform）的数据宽度（例如在 x64 PC 上编译 arm32 代码，解释器里的 `usize` 必须表现为 32 位）。



### Phase 3: Stage 1 遍历器 (The Transform Pass)

* **任务**: 实现一个 AST 遍历器（Visitor）。
* **逻辑**:
* 遇到 `ComptimeIf`: 计算条件。若 True，替换为 Then-Block 的内容；若 False，替换为 Else-Block 的内容。
* 遇到 `ComptimeFor`: 循环计算，将 Body 节点深拷贝 N 次并拼接。
* 遇到 `#{ expr }`: 调用解释器计算 `expr`，将结果包装成 `LiteralNode` 替换原节点。



### Phase 4: 标准库与反射 (Stdlib & Reflection)

* **任务**: 提供 `std.meta` 库。
* **内容**:
* `os`, `arch`, `compiler_version` 等内置常量。
* 类型反射 API（如 `type_of(val).fields`），允许在 `#{}` 中遍历结构体字段（用于实现序列化代码生成）。



### Phase 5: 错误报告优化 (Diagnostics)

* **任务**: 区分“编译期执行错误”和“代码生成后的错误”。
* **目标**: 当 `#{}` 内部抛出 panic 时，报错信息应指向源码中的 `#{` 行，并打印编译期堆栈。

---

## 6. 示例汇总 (Examples)

### 示例 1: 跨平台系统抽象

```auto
const OS = std.target.os;

// 这是一个编译期接口定义
type FileHandle = #{
    #if OS == "windows" {
        return u64 // Handle
    } else {
        return i32 // FD
    }
};

fn open_file(path: str) FileHandle {
    #if OS == "windows" {
        return win32.CreateFile(path, ...);
    } else {
        return posix.open(path, ...);
    }
}

```

### 示例 2: 编译期生成 CRC32 表

```auto
const POLY = 0xEDB88320;

const CRC_TABLE = #{
    var t = [0; 256];
    for i in 0..256 {
        var c = i;
        for j in 0..8 {
            if (c & 1) != 0 { c = (c >> 1) ^ POLY; }
            else { c = c >> 1; }
        }
        t[i] = c;
    }
    t // Return to const
};

// 运行时直接使用，零开销
fn get_crc_byte(i: u8) u32 {
    return #{CRC_TABLE}[i];
}

```

这份文档现在可以提交给编译器开发组作为核心需求规范了。