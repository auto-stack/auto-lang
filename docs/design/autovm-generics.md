这是一份针对 **AutoVM (Rust实现版)** 的泛型系统开发计划。

本计划的核心策略是：**先保证正确性（泛型擦除），再优化热点（特化存储），最后规划极致性能（双层 JIT）。**

---

# AutoVM 泛型实现与优化开发计划

**目标**：在解释器模式下支持 Auto 语言定义的泛型（如 `class Pair<K, V>`），并确保核心数据结构（数组、列表）拥有接近 Native 的性能。

## 第一阶段：基础设施与泛型擦除 (Foundation & Type Erasure)

**核心理念**：在堆内存中，所有用户定义的泛型对象在物理上都是“均一”的。`Pair<int>` 和 `Pair<string>` 共享相同的 Rust 结构体布局，字段统一使用 `Value` 枚举存储。

### 1.1 数据结构定义 (Rust)

在 `auto-core` 或 `auto-vm` crate 中定义元数据和实例结构。

* **`ClassTemplate` (静态模板)**: 对应源代码中的 `class` 定义。
```rust
struct ClassTemplate {
    name: String,
    generic_params: Vec<String>, // e.g., ["K", "V"]
    fields: Vec<FieldDef>,       // e.g., key: K, val: V
    methods: HashMap<String, BytecodeChunk>,
}

```


* **`ClassType` (具体类型)**: 运行时生成的类型标识（Reified Type），用于 `is_instance` 检查。
```rust
struct ClassType {
    template: Rc<ClassTemplate>,
    type_args: Vec<Type>,        // e.g., [Type::Int, Type::String]
}

```


* **`Instance` (对象实例)**: 统一的内存载体。
```rust
struct Instance {
    class: Rc<ClassType>,        // 指向元数据
    fields: Vec<Value>,          // 【关键】类型擦除，统一存 Value
}

```



### 1.2 VM 指令集扩展

* **`NEW_INSTANCE <class_idx>`**: 修改此指令。当实例化泛型类时，栈上必须提供类型参数（Type Arguments）。
* *逻辑*：VM 读取栈上的类型参数 -> 创建 `ClassType` -> 创建 `Instance` -> 初始化 `fields` 为 `Value::Null` 或默认值。


* **`GET_FIELD / SET_FIELD`**:
* *逻辑*：直接通过索引访问 `Instance.fields`。VM 不做泛型类型检查（假设编译期已通过），直接读写 `Value`。



### 1.3 交付物

* 能够运行 `let p = new Pair<int, string>(1, "a");`。
* 能够运行 `let k = p.key;`。
* 内存中 `p` 的物理结构是 `Vec<Value>`。

---

## 第二阶段：核心容器特化存储 (Container Specialization)

**核心理念**：虽然用户自定义类型可以擦除，但**数组（Array/Vector）**必须快。利用 Rust 的 `enum` 实现对基础类型（Primitive Types）的内存紧凑存储。

### 2.1 特化存储结构 (Specialized Storage)

重构 AutoVM 中的数组实现，不再单纯使用 `Vec<Value>`。

```rust
// auto-vm/src/array.rs

pub enum ArrayStorage {
    // 1. 通用模式：存对象、字符串、混合类型 (慢，堆内存分散)
    Generic(Vec<Value>),
    
    // 2. 特化模式：存原生整数 (快，内存连续，SIMD 友好)
    I64(Vec<i64>),
    
    // 3. 特化模式：存浮点数 (快)
    F64(Vec<f64>),
    
    // 4. 特化模式：存布尔/字节 (极度紧凑)
    Bool(Vec<bool>), // 或 BitSet
    U8(Vec<u8>),     // 对应 ByteArray
}

pub struct AutoArray {
    storage: ArrayStorage,
    // ... length, capacity 等元数据
}

```

### 2.2 实例化工厂 (Factory Logic)

修改数组创建逻辑。当执行 `new Array<int>(100)` 时：

1. VM 检查泛型参数 `T`。
2. 如果 `T == int`，初始化 `ArrayStorage::I64(Vec::with_capacity(100))`。
3. 如果 `T == float`，初始化 `ArrayStorage::F64(...)`。
4. 其他情况，初始化 `ArrayStorage::Generic(...)`。

### 2.3 透明访问 (Transparent Access)

实现 `get/set` 方法的自动装箱/拆箱，对 VM 上层指令屏蔽差异。

```rust
impl AutoArray {
    pub fn get(&self, index: usize) -> Value {
        match &self.storage {
            ArrayStorage::I64(vec) => Value::Int(vec[index]), // Read: i64 -> Value
            ArrayStorage::F64(vec) => Value::Float(vec[index]),
            ArrayStorage::Generic(vec) => vec[index].clone(),
            // ...
        }
    }
    
    pub fn set(&mut self, index: usize, val: Value) {
        match &mut self.storage {
            ArrayStorage::I64(vec) => {
                if let Value::Int(v) = val {
                    vec[index] = v; // Write: Value -> i64
                } else {
                    panic!("Type mismatch: Array<int> cannot store {:?}", val);
                }
            }
            // ...
        }
    }
}

```

### 2.4 交付物

* `Array<int>` 在内存中占用空间减少（从 `sizeof(Value)` 降为 8 字节）。
* 数值计算密集型任务（如矩阵乘法、图像处理）性能提升 5-10 倍。

---

## 第三阶段：FFI 与 标准库桥接 (Standard Library Bridging)

**核心理念**：确保 Auto 语言的标准库（如 `std.map`）底层直接调用 Rust 的 `HashMap`，而不是用 Auto 代码去模拟 Map。

### 3.1 泛型 FFI 映射

当 Auto 代码调用 `new Map<string, int>()` 时，不仅泛型参数要传递给 VM，还要传递给 Rust 的 Native 实现。

* **Rust 端**：需要实现一个包装器，能够处理 `AutoMap` (内部持有 `HashMap<String, Value>`)。
* **注意**：由于 Rust 的 `HashMap` 是强类型的，我们可能无法为每种组合都预编译。
* *策略*：标准库 Map 底层统一使用 `HashMap<u64, Value>` (Key hash code) 或者 `HashMap<Value, Value>`。
* *优化*：对于 `Map<string, ...>` 这种高频场景，可以在 Rust 端特化一个 `HashMap<String, Value>` 的变体。



---

## 第四阶段（终极形态）：双层 JIT (Tiered Compilation / Source-based JIT)

**核心理念**：这是 AutoVM 性能优化的终极目标。类似于 Java HotSpot 或 JS V8，通过**运行时分析**来决定是否将解释执行升级为 Native 代码执行。

### 4.1 架构设计

引入 **Tier 1 (Interpreter)** 和 **Tier 2 (Background Compiler)**。

### 4.2 触发机制 (Profiling & Trigger)

1. **热点探测**：VM 在解释执行时维护计数器（方法调用次数、循环回边次数）。
2. **阈值触发**：当 `Pair<int, float>` 的实例化或方法调用超过 10,000 次时，标记为 "Hot"。

### 4.3 编译流水线 (The Pipeline)

当触发 JIT 后，后台线程启动：

1. **代码生成 (Codegen)**:
* 利用 `a2r` 模块，针对该**具体泛型实例**生成 Rust 源码。
* 生成代码：`struct Pair_i64_f64 { key: i64, val: f64 }` 及其方法。


2. **调用工具链 (Invocation)**:
* 调用系统安装的 `rustc` 或 `cargo`，将这段微小的源码编译为动态库 (`.so` / `.dll`)。
* *优化*：为了速度，可以只编译 Release 模式且关闭部分耗时优化。


3. **动态加载 (dlopen)**:
* AutoVM 加载生成的 `.so`。
* 获取 `new_pair_i64_f64` 和 `get_key_native` 的函数指针。



### 4.4 栈上替换 (OSR - On-Stack Replacement) & 指针重定向

这是最难的一步。

1. **重定向**：修改 VM 的类型元数据 `ClassType(Pair<int, float>)`。将其 `allocator` 函数指针指向 Native 实现。
2. **新对象**：以后执行 `new Pair<int, float>`，直接分配 Native 内存，不再产生 `Vec<Value>`。
3. **旧对象兼容**：
* *策略 A (简单)*：旧对象保持原样（解释执行），新对象用 Native。VM 需要能同时处理两种形态的 `Pair`。
* *策略 B (激进)*：STW (Stop-The-World)，扫描堆，将所有旧的 `Generic Instance` 迁移/转换为 `Native Instance`。



### 4.5 价值

一旦进入 Tier 2，Auto 语言的用户自定义泛型代码将获得 **与 Rust 原生代码完全一致的性能**（零开销抽象、内存紧凑、自动向量化）。这将使 Auto 语言在科学计算、游戏引擎脚本等高性能领域具备极强的竞争力。