这是一个更加完善、细节详尽的 **Auto Flow** 设计文档。

根据你的要求，我将 `Iterator` 重命名为 `Iter`，并补充了标准库模块规划、常见算子列表以及关于流式操作符的决策分析。

---

# Design Document: Auto Flow Architecture v1.1

**版本**: 1.1 (Refined)
**状态**: 待实现 (Ready for Implementation)
**适用领域**: Auto 语言标准库 (`std`)

## 1. 概述 (Overview)

Auto Flow 是一套专为系统级编程（特别是嵌入式环境）设计的函数式编程（FP）接口。它旨在提供类似 Rust/Scala 的高层抽象能力，同时严格遵守 **零开销抽象 (Zero-Cost Abstraction)** 和 **显式资源管理 (Explicit Resource Management)** 的原则。

**核心设计哲学：**

> **"默认懒惰，显式勤奋。" (Lazy by default, Eager by bang.)**

---

## 2. 模块规划与 Prelude (Module Structure)

### 2.1 模块位置

所有的流式计算核心组件应当放置在 **`std.iter`** 模块中。

```
std/
└── iter/
    ├── spec.at       // 定义 Iter 和 Iterable 规范
    ├── adapters/     // 定义 Map, Filter, Zip 等结构体
    └── consumers.at  // 定义 Reduce, Collect, Count 等终结方法

```

### 2.2 Prelude 策略

由于迭代器模式极其常用，以下核心类型**必须**加入 `std.prelude`，以便用户无需 `import std.iter` 即可使用：

* `spec Iter<T>`
* `spec Iterable<T>`
* (可选) 常见的适配器类型通常不需要直接导出，用户通过方法链式调用即可。

---

## 3. 核心概念 (Core Concepts)

### 3.1 迭代器规范 (`spec Iter<T>`)

这是所有流式计算的基础。任何能够“逐个产生元素”的对象都必须实现此规范。

```auto
// std/iter/spec.at

spec Iter<T> {
    // 尝试获取下一个元素
    // 返回值: ?T (Option<T>)，nil 表示迭代结束
    // 设计决策: 使用 ?T 而非 Result<T, Stop> 是为了极致的性能和 C 语言兼容性
    fn next() ?T 
}

```

### 3.2 可迭代对象 (`spec Iterable<T>`)

这是容器与迭代器之间的桥梁。

```auto
// std/iter/spec.at

spec Iterable<T> {
    // 关联类型：具体的迭代器实现
    type IterT impl Iter<T>
    
    // 获取一个借用的迭代器
    // 注意: 这里不消耗 self，只是借用
    fn iter() .IterT
}

```

这是一个非常关键的补全。在之前的文档中，我描述了原理，但确实漏掉了实现这一“语法糖”的具体机制。

在 Auto 语言中，要实现 `list.map(...)` 等价于 `list.iter().map(...)`，我们不需要特殊的编译器黑魔法，而是依赖 **Spec 的默认实现 (Default Implementation in Spec)**，这类似于 Rust 的 Trait Default Methods 或 Java 的 Interface Default Methods。

我将这一机制命名为 **"Iterable 自动转发 (Iterable Auto-Forwarding)"**。

以下是补充后的设计细节，你可以将其插入到设计文档的 **"3.2 可迭代对象"** 章节之后。

---

### 3.3 Iterable 自动转发机制

为了让用户可以直接在容器（如 `List`）上调用算子，我们需要在 `Iterable<T>` 规范中定义一套**“转发方法 (Forwarding Methods)”**。这些方法的作用仅仅是：**自动创建迭代器，并将调用转发给迭代器。**

#### 1. 代码实现 (`std/iter/spec.at`)

我们需要在 `spec Iterable<T>` 中直接写出 `map`, `filter` 等函数的默认实现。

```auto
// std/iter/spec.at

spec Iterable<T> {
    // --- 必须实现的核心方法 ---
    type IterT impl Iter<T>
    fn iter() .IterT
    
    // --- 自动转发方法 (Default Impl) ---
    // 这些方法会自动“混入”到所有实现了 Iterable 的容器中
    
    // 转发 Map
    // 注意：返回值是 MapIter，这意味着链式调用的类型瞬间从“容器”变成了“流”
    fn map<U>(f: fn(T)U) MapIter<.IterT, F, U> {
        // 1. 调用自己的 iter() 获取迭代器
        // 2. 调用迭代器的 map()
        return .iter().map(f)
    }

    // 转发 Filter
    fn filter(p: fn(T)bool) FilterIter<.IterT, F> {
        return .iter().filter(p)
    }

    // 转发 Reduce (终结操作也可以转发)
    fn reduce<B>(init: B, f: fn(B, T)B) B {
        return .iter().reduce(init, f)
    }
    
    // ... 对所有通用算子进行类似的转发 ...
}

```

#### 2. 类型流变 (Type Metamorphosis)

理解这个机制的关键在于**类型的转变**。

```auto
// 假设 list 的类型是 List<i32>

// 1. list 是容器 (Container)
let list = List<i32>.new()

// 2. 调用 .map() 的瞬间
// - list.map(...) 调用的是 Iterable.map (转发方法)
// - 内部执行 list.iter().map(...)
// - 返回类型变成了 MapIter<ListIterator<i32>, ...>
let pipeline = list.map(|x| x * 2) 

// 3. 后续的调用是在 Iterator 上进行的
// - pipeline.filter(...) 调用的是 Iter.filter (真实算子)
let result = pipeline.filter(|x| x > 10)

```

#### 3. 性能影响 (Zero Overhead)

你可能会担心：*“多了一层函数调用，会有开销吗？”*

**答案：完全没有。**

由于 Auto 语言（及 C 后端）的内联优化（Inlining）：

1. `List.map` (转发) 会被内联。
2. `List.iter` (创建迭代器) 会被内联。
3. `Iter.map` (创建包装器) 会被内联。

最终生成的 C 代码，仅仅是在栈上初始化了一个结构体，没有任何函数调用指令（Call Instruction）。

#### 4. 更新后的使用体验

有了这个设计，用户侧的代码就变得非常干净：

```auto
// 之前 (如果没有转发机制)
list.iter().map(f).filter(p).!

// 现在 (有转发机制)
list.map(f).filter(p).!

```

---

## 4. 懒加载算子 (Lazy Operators / Adapters)

这些算子是**惰性**的。调用它们**绝不触发循环**，也**绝不分配内存**。它们仅仅是在栈上创建一个轻量级的包装结构体。

### 4.1 通用算子列表

| 算子 | 签名描述 | 作用 | 实现方式 |
| --- | --- | --- | --- |
| **`map`** | `fn map<U>(f: fn(T)U) MapIter` | 将元素  转换为  | 包装结构体，`next()` 中调用  |
| **`filter`** | `fn filter(p: fn(T)bool) FilterIter` | 仅保留满足谓词  的元素 | 包装结构体，`next()` 中循环直到满足  |
| **`take`** | `fn take(n: u32) TakeIter` | 仅获取前  个元素 | 内部维护计数器 `count` |
| **`skip`** | `fn skip(n: u32) SkipIter` | 跳过前  个元素 | 初始化时预先调用  次 `next()` |
| **`enumerate`** | `fn enumerate() EnumerateIter` | 产生 `(index, item)` 对 | 内部维护 `index` 计数器 |
| **`zip`** | `fn zip<U>(other: Iter<U>) ZipIter` | 将两个流合并为 `(T, U)` | 同时调用两个 `next()`，任一为 nil 则结束 |
| **`flatten`** | `fn flatten() FlattenIter` | 将 `Iter<Iter<T>>` 展平为 `Iter<T>` | 维护两级迭代器状态 |
| **`chain`** | `fn chain(other: Iter<T>) ChainIter` | 连接两个流，先由 A 再由 B | A 耗尽后切换到 B |
| **`inspect`** | `fn inspect(f: fn(T)void) InspectIter` | 偷看元素（通常用于调试/日志） | 调用  但不改变元素，原样返回 |

### 4.2 实现示例：Map

```auto
// std/iter/adapters/map.at

type MapIter<I, F, U> {
    iter I      // 上游迭代器
    f    F      // 转换函数
}

impl Iter<U> for MapIter<I, F, U> {
    fn next() ?U {
        let item = .iter.next().? 
        return (.f)(item)
    }
}

```

---

## 5. 实质化算子 (Materialization Operator: `!`)

这是 Auto 语言的特色设计。

### 5.1 语义与实现

* **符号**: `expression!`
* **作用**: 触发流的执行，并收集结果。
* **底层**: 编译器将其重写为 `.collect::<DefaultStorage>()`。

### 5.2 环境敏感策略

```auto
// 编译器内置逻辑 (伪代码)
if target == MCU {
    // 栈分配 / 静态区
    collect_to(List<T, Fixed<CONFIG_DEFAULT_SIZE>>)
} else {
    // 堆分配
    collect_to(List<T, Heap>)
}

```

---

## 6. 终结算子 (Terminal Operators)

这些算子会触发循环，消耗迭代器，并返回一个非迭代器的值。

### 6.1 常用终结算子列表

| 算子 | 签名描述 | 作用 | 实现方式 |
| --- | --- | --- | --- |
| **`reduce`** (或 `fold`) | `fn reduce<B>(init: B, f: fn(B, T)B) B` | 将流归约为单个值 | `while` 循环累加 |
| **`count`** | `fn count() u32` | 统计元素个数 | `while` 循环计数，忽略元素内容 |
| **`any`** | `fn any(p: fn(T)bool) bool` | 是否存在满足条件的元素 | 遇到 `true` 立即短路返回 |
| **`all`** | `fn all(p: fn(T)bool) bool` | 是否所有元素都满足条件 | 遇到 `false` 立即短路返回 |
| **`find`** | `fn find(p: fn(T)bool) ?T` | 查找第一个匹配项 | 遇到匹配项立即返回，否则 nil |
| **`for_each`** | `fn for_each(f: fn(T)void) void` | 对每个元素执行副作用 | 标准 `while` 循环 |
| **`to`** | `fn to<C>() C` | 显式收集到指定容器 | 手动控制存储策略 (Heap/Fixed/Arena) |

---

## 7. 语法设计决策：点号 (`.`) vs 管道符 (`|>`)

针对问题 3：*Auto 语言是否需要引入 `|>` 操作符？*

### 7.1 现有设计分析

* **Haskell/Elixir/OCaml (`|>`)**: 它们不仅是流式操作，更是通用的“函数应用”操作符。`x |> f` 等价于 `f(x)`。这在函数是一等公民且不依附于对象的语言中非常重要。
* **Rust/Java/C# (`.`)**: 使用方法调用链（Method Chaining）。`x.map(f)`。这依赖于类型系统支持扩展方法（Extension Methods）或 Trait 实现。

### 7.2 Auto 的选择：坚持使用点号 (`.`)

**理由：**

1. **IDE 友好性**: 点号是智能提示（IntelliSense）的天然触发器。输入 `list.` 后，IDE 能明确列出所有可用的适配器。管道符很难做到这一点，因为它后面可以接任意全局函数。
2. **统一性**: Auto 是基于 Struct 和 Method 的系统语言。`Iter` 的操作本质上是改变了迭代器的状态或类型，这符合“方法”的直觉。
3. **避免符号膨胀**: 系统级语言应保持符号集的精简。Auto 已经有了强大的 `!` (实质化) 和 `?` (错误处理)，引入 `|>` 会增加认知负担。
4. **C 语言互操作**: 点号调用更容易映射到 C 的结构体函数调用风格。

**结论**：不引入 `|>`，继续沿用 Rust 风格的 `.` 链式调用，通过 `impl Iterable` 的默认方法来实现流畅的 API 体验。

---

## 8. 任务规划建议 (Implementation Plan)

基于本文档，后续的开发任务可以拆解为：

1. **P0 - 核心规范**: 在 `std/iter/spec.at` 中定义 `Iter<T>` 和 `Iterable<T>`。
2. **P0 - 基础适配器**: 实现 `MapIter`, `FilterIter`。这是验证设计是否跑通的关键。
3. **P1 - 容器集成**: 修改 `List<T>` 和 `[N]T`，让它们实现 `Iterable` 接口。
4. **P1 - 编译器魔法**: 实现 `!` 后缀的编译器前端重写逻辑（AST Rewrite）。
5. **P2 - 扩展适配器**: 实现 `Zip`, `Chain`, `Enumerate` 等进阶算子。
6. **P2 - 终结算子**: 实现 `reduce`, `any`, `all` 等。

---

这个设计文档现在已经足够清晰，可以直接作为标准库开发组的需求规格说明书使用。