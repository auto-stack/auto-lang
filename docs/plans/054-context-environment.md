这是一个非常宏大的架构愿景。你正在构建的是一套 **“全生命周期上下文管理系统（Lifecycle Context Management）”**。

这套机制将 **编译期（Prelude/Imports）**、**配置期（Environment Injection）** 和 **运行期（Runtime Startup）** 完美融合。它确保了 Auto 语言的代码在 PC 上是“脚本级”的易用性，而在 MCU 上是“裸机级”的控制力。

以下是完整的系统设计方案。

---

# Auto 语言核心架构：统一运行时与环境注入系统

## 1. 系统概览 (System Overview)

我们将整个程序的启动与上下文构建过程划分为三个阶段：

1. **注入阶段 (Injection Phase)**：编译器根据 Target 和 Root Config，合成出当前的 **Environment**。
2. **预备阶段 (Prelude Phase)**：编译器隐式导入 `std.prelude`。这个 Prelude 不是静态的，它根据 Environment 动态导出 `List`、`String`、`print` 以及内存分配策略。
3. **启动阶段 (Startup Phase)**：生成真正的入口点（`_start` 或 `main`），执行环境初始化（如堆初始化），最后移交控制权给用户的 `main`。

---

## 2. 详细设计：三位一体机制

### 2.1 注入阶段：构建 `std.env`

这是编译器的第一步工作。编译器分析 CLI 参数和用户根文件，生成一个虚拟模块 `std.env`。

**逻辑流：**

1. 读取 `-target` (e.g., `cortex-m4` 或 `x64-linux`).
2. 解析入口文件 (`root.at`)，查找特殊的 `[config]` 块或常量。
3. 合成 `std.env`。

```auto
// 虚拟生成的 std.env 模块
module std.env {
    // 1. 平台特征
    const TARGET_ARCH = "armv7e-m"
    const TARGET_OS   = "none" // MCU
    
    // 2. 内存策略 (由 Target + Root Config 决定)
    // 如果是 MCU，默认 Fixed；如果是 PC，默认 Heap
    type DefaultListStorage<T> = std.storage.Fixed<T, 64>
    
    // 3. 错误处理策略
    // MCU: 死循环; PC: 打印堆栈并退出
    const PANIC_STRATEGY = "loop" 
}

```

### 2.2 预备阶段：动态 Prelude

Auto 编译器会自动在每个用户文件的头部插入 `import std.prelude`。但这个 Prelude 是智能的，它充当了 **“环境适配器”**。

**std/prelude.at (伪代码)**

```auto
import std.env
import std.io
import std.container

// --- 1. 导出基础类型 ---
pub type int = i32
pub type bool = u1

// --- 2. 导出适配后的容器 ---
// 这里的 List 已经携带了环境注入的 Storage 策略
pub type List<T> = std.container.List<T, std.env.DefaultListStorage<T>>

// 这里的 String 根据环境可能是 HeapString 或 StackString
pub type String = if std.env.TARGET_OS == "none" 
                  then std.container.FixedString<256> 
                  else std.container.HeapString

// --- 3. 导出适配后的 IO ---
// MCU 下 print 可能输出到 UART，PC 下输出到 Stdout
pub const print = std.io.get_printer(std.env.TARGET_OS)

// --- 4. 语法糖开关 ---
// 如果是极简 MCU 环境，可能禁用复杂的 Runtime Type Information (RTTI)
# if std.env.TARGET_OS == "none" {
#    disable_feature(RTTI)
# }

```

**用户视角：**
用户什么都不用配置，直接写：

```auto
fn main() {
    let list List<i32> // 自动获得最佳实现
    list.push(1)
    print("Ready")     // 自动路由到串口或控制台
}

```

### 2.3 启动阶段：Runtime Startup (Bootstrap)

这是生成的 C 代码层面的魔法。Auto 编译器不会直接把用户的 `main` 翻译成 C 的 `main`，而是生成一个 **引导程序 (Bootstrap)**。

#### A. PC 平台引导 (`entry_pc.c`)

PC 环境下，操作系统负责堆的初始化，引导很简单。

```c
// Auto 生成的 C 代码
int main(int argc, char** argv) {
    // 1. 初始化 GC (如果 Auto 未来支持) 或 全局分配器
    std_heap_init();
    
    // 2. 处理命令行参数
    auto_args_init(argc, argv);
    
    // 3. 调用用户的 main
    user_main();
    
    // 4. 清理资源
    return 0;
}

```

#### B. MCU 平台引导 (`entry_mcu.c`)

MCU 环境下，Auto 接管一切。假设用户显式配置了 `Heap` 策略，Startup 代码必须负责初始化一块静态内存作为堆。

```c
// Auto 生成的 C 代码

// 定义全局堆空间 (如果 env 配置了需要 Heap)
#ifdef ENV_USE_HEAP
static uint8_t __heap_space[ENV_HEAP_SIZE];
#endif

// 复位处理函数 (由启动文件调用)
void Auto_Reset_Handler() {
    // 1. 硬件初始化 (时钟、看门狗等，由 ext hooks 提供)
    SystemInit();
    
    // 2. 堆分配器初始化 (将 __heap_space 挂载给 malloc)
    #ifdef ENV_USE_HEAP
    std_allocator_init(__heap_space, ENV_HEAP_SIZE);
    #endif
    
    // 3. 全局变量/静态构造函数初始化
    __libc_init_array();
    
    // 4. 跳转到用户逻辑
    user_main();
    
    // 5. 如果 main 返回了，进入死循环 (Panic 策略)
    while(1);
}

```

---

## 3. 场景演练：从 Config 到 Startup

让我们看看这一整套机制如何处理两个极端场景。

### 场景一：极简 MCU 传感器节点

* **Root Config**: 无（全默认）。
* **Target**: `cortex-m0`。

1. **Injection**: `std.env` 判定为 MCU，设定 `DefaultListStorage = Fixed<64>`, `String = FixedString<64>`。需不需要 Heap？否。
2. **Prelude**: 导出静态的 `List` 和 `String`。`print` 映射为空操作或半主机模式。
3. **Startup**: 生成 `entry_mcu.c`。检测到不需要 Heap，**不生成** `__heap_space` 数组，**不调用** `allocator_init`。生成极致精简的二进制。

### 场景二：MCU 高级应用 (使用 Arena)

* **Root Config**:
```auto
// main.at
[config]
const HeapSize = 4096      // 我想要 4K 的堆
const MainStorage = Heap   // 默认 List 使用堆

```


* **Target**: `cortex-m4`。

1. **Injection**: `std.env` 读取配置。设定 `DefaultListStorage = Heap`。标记 `ENV_USE_HEAP = true`，`ENV_HEAP_SIZE = 4096`。
2. **Prelude**: 导出的 `List` 默认为 `List<T, Heap>`。
3. **Startup**: 生成 `entry_mcu.c`。编译器在 `.bss` 段预留 4096 字节数组。`Auto_Reset_Handler` 启动时，调用 `std_allocator_init` 初始化这块内存。之后，用户代码里的 `list.push` 就能正常工作了。

---

## 4. 实施计划 (Execution Plan)

### 阶段 1：Prelude 架构搭建

* 创建一个 `std/prelude.at` 文件。
* 修改编译器，使其在解析任何文件前，先解析 `std/prelude.at` 并将符号注入当前作用域。
* 实现基本的 `if/else` 编译期逻辑，用于在 Prelude 中根据 Target 切换导出类型。

### 阶段 2：Environment 注入编译器

* 在编译器中增加 `--root <file>` 参数。
* 实现 AST 扫描器，用于提取 Root 文件中的 `[config]` 常量。
* 实现 `std.env` 虚拟模块生成器。

### 阶段 3：Startup 代码生成器 (Codegen)

* **A2C 模块更新**：不再直接生成 `main()`。
* **Template 系统**：创建 `entry_pc.c.tpl` 和 `entry_mcu.c.tpl` 模板。
* **Linker 逻辑**：根据是否使用了 Heap 策略，决定是否在生成的 C 代码中包含简单的 malloc 实现（如 `tiny_alloc`）。

---

## 5. 总结

将 **List 泛型策略** 与 **Prelude** 及 **Startup** 结合后，Auto 语言获得了一种 **“生物般的适应性”**：

* 它根据环境（Env）调整自己的器官（List/String）。
* 它根据基因（Config）调整自己的生长方式（Startup）。
* 它给细胞（用户代码）提供了一个看似不变但实则高度适配的生存环境（Prelude）。

这就实现了你想要的：**一套代码，一种接口，涵盖从 8-bit MCU 到 64-bit Server 的全场景。**