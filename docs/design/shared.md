根据我们刚才深入的架构讨论，我整理了这份 **Auto 语言存储与并发修饰符设计文档**。

这份文档固化了关于 `shared`、`atomic`、`volatile` 和 `pub` 作为**存储实体修饰符**的物理定义，并修正了 `let`、`var` 与 `const` 的语意边界。

---

# 📝 Auto 语言设计文档：存储修饰符与并发原语

## 1. 变量声明体系 (Declaration Taxonomy)

Auto 语言严格区分**存储属性**（数据在哪、怎么存）与**数据类型**（数据长什么样）。

### 1.1 核心声明词
* **`const`**：**编译期常量**。在编译时完成替换，不占用运行时的内存地址（符号表级存在）。
* **`let`**：**运行时不可变绑定**。一旦初始化，其内存内容不可更改。
* **`var`**：**运行时可变变量**。

### 1.2 存储修饰符序列 (Keyword Chain)
所有修饰符均作用于**变量实体**而非类型。标准声明顺序如下：
`[可见性] [存储属性] [原子性] [易变性] [声明词] [名称] [类型] [= 初始值]`

---

## 2. 核心修饰符定义

### 2.1 `pub` (关键字)
* **物理语义**：导出符号。
* **功能**：决定链接器（Linker）是否对外暴露该内存地址。

### 2.2 `shared` (关键字)
* **物理语义**：静态存储分配（Static Storage）。
* **功能**：将数据放置在 `.data`（已初始化）或 `.bss`（未初始化）段，而非栈（Stack）上。其生命周期与程序相同。

### 2.3 `atomic` (关键字)
* **物理语义**：并发访问协议。
* **功能**：
    * **修饰变量**：强制该变量的读写使用硬件级原子指令（如 `LDREX/STREX`）。
    * **语句块**：`atomic { ... }` 形成逻辑事务，编译器根据上下文自动选择 **Spinlock**、**Mutex** 或 **硬件信号量**。

### 2.4 `volatile` (关键字)
* **物理语义**：禁止缓存优化。
* **功能**：强制每次访问都必须从物理地址读写，防止编译器将其优化到寄存器中。常用于硬件寄存器映射。

---

## 3. 异步单例与延迟初始化

利用 `shared` 与异步代码块 `~ {}` 的组合，实现安全、按需、非阻塞的单例模式。

```auto
// 异步单例模式
pub shared var DATABASE Database = ~ {
    val config = File.read_json("db.json").await!
    Database.connect(config).await!
}
```

* **物理逻辑**：编译器注入状态位。首次访问时，若未初始化，则执行异步块；若正在初始化，则挂起当前 Task 等待。

---

## 4. 并发控制模型：Spinlock vs Mutex

在 `atomic { ... }` 块中，Auto 编译器根据执行上下文采取自适应策略：

| 策略 | 物理实现 | 适用场景 |
| :--- | :--- | :--- |
| **自旋锁 (Spinlock)** | 忙等 (Busy-wait)，不释放 CPU | 中断上下文 (`#[interrupt]`)、极短的操作、不可挂起的内核态。 |
| **互斥锁 (Mutex)** | 任务挂起 (Suspend)，释放 CPU | 包含 `await` 或长耗时操作的任务上下文。 |
| **硬件信号量** | 利用 SoC 的硬件锁 (如 S32G3 HSEM) | 跨核（A核与M核之间）共享内存的同步。 |



---

## 5. 综合示例 (嵌入式/系统级场景)

### 5.1 硬件寄存器定义
```auto
// 定义一个全域可见、位于固定物理地址、禁止优化的原子状态寄存器
pub shared atomic volatile var SYS_STATUS u32 @ 0x40001000
```

### 5.2 全局计数器
```auto
// 自动映射为单条原子加法指令，无需显式加锁
shared atomic var GLOBAL_TICKS uint = 0

fn on_timer_interrupt() {
    GLOBAL_TICKS += 1
}
```

### 5.3 复杂状态保护
```auto
shared var SYSTEM_STATE State

fn update_state() {
    // 编译器自动根据上下文选择 Spinlock 或 Mutex 保护该块
    atomic SYSTEM_STATE {
        SYSTEM_STATE.mode = .Active
        SYSTEM_STATE.last_update = time.now()
    }
}
```

---

## 6. 与 C/Rust 映射关系参考

| Auto 语法 | C 语言映射 (C11/Intrinsics) | Rust 等效概念 |
| :--- | :--- | :--- |
| `shared atomic var` | `_Atomic` 或 `__atomic_fetch_add` | `AtomicI32` 等类型包装 |
| `atomic { ... }` | `spin_lock()` / `mutex_lock()` | `MutexGuard` |
| `volatile var` | `volatile` 关键字 | `read_volatile()` 函数 |
| `let` | `const type` (运行时只读) | `let` |
| `const` | `#define` 或 `enum` (编译期) | `const` |

---

**设计结论**：
Auto 语言通过将物理属性（`shared`, `atomic`, `volatile`）从数学类型中剥离，实现了对底层硬件行为的精确控制，同时利用 `~ {}` 和 `atomic {}` 提供了现代化的、对 AI 友好的高级抽象。