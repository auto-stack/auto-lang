这是更新后的 **Auto 语言异常与状态处理设计架构文档 (RFC-001 v1.1)**。

本次更新重点引入了 **`#[nopanic]` 静态契约**，形成了 **“编译器静态阻断” (Static prevention)** 与 **“运行时动态韧性” (Dynamic resilience)** 双重保险的完整安全模型。

---

# Auto 语言设计规范：三层非常态数据处理体系

**版本**：v1.1 (Draft)
**状态**：Reviewing
**目标**：建立清晰的 Data Absence (缺失)、Operation Failure (失败) 与 Logic Panic (崩溃) 的分层处理机制，并提供针对硬实时/嵌入式场景的静态无崩溃保证。

---

## 1. 核心理念：三级状态金字塔

我们将程序中的非正常状态分为三个严格的层级。每个层级对应不同的类型系统表示、操作符后缀以及处理紧迫度。

| 层级 | 类型表示 | 语义名称 | 心理模型 | 处理策略 |
| --- | --- | --- | --- | --- |
| **L1** | **`?T`** | **Option** | **空盒子** (Missing) | **温和**：预期内的数据缺失 (如 DB 查无此人)。 |
| **L2** | **`!T`** | **Result** | **坏盒子** (Failure) | **严厉**：运行时操作失败 (如 IO 错误)，必须显式处理。 |
| **L3** | **`T`** | **Panic** | **炸弹** (Critical) | **紧急**：逻辑悖论或不可恢复错误 (如 1/0, 断言失败)。 |

---

## 2. 操作符设计：后缀流式体系 (Postfix System)

Auto 采用基于 `.` 的后缀操作符，统一解决嵌套问题。

**助记口诀**：

* **`?`** 管数据。
* **`!`** 管错误。
* **`!!`** 管崩溃。
* **无参** = 向上抛出/中断 (Propagate/Break)。
* **有参** = 向下兜底/恢复 (Recover/Rescue)。

### 2.1 行为矩阵

| 对象 | 类型 | **传播/中断 (无参)** <br>

<br> *语义: "发生则停"* | **兜底/恢复 (有参)** <br>

<br> *语义: "发生则救"* | 备注 |
| --- | --- | --- | --- | --- |
| **数据** | `?T` | **`val.?`** <br>

<br> Return None | **`val.?(def)`** <br>

<br> Use `def` | 类似 `??` |
| **错误** | `!T` | **`val.!`** <br>

<br> Return Err | **`val.!(def)`** <br>

<br> Swallow Error, Use `def` | 错误被降级为值 |
| **恐慌** | `T`* | **`expr.!!`** <br>

<br> **Panic Now** | **`expr.!!(def)`** <br>

<br> **Rescue** (Log & Use `def`) | 这里的 Rescue 仅在 Release 模式生效 |
| **流** | Iter | **`iter.$`** <br>

<br> Materialize | (无) | 终结懒惰计算 |

---

## 3. 数组与集合访问策略 (Dual-Track Access)

### 3.1 通用模式：业务逻辑层

* **语法**：`list[int]`
* **返回类型**：**`?T`** (Option)
* **行为**：越界视为“无数据”。
* **典型用法**：
```auto
// 强制处理越界：如果没有，就用 0
let val = list[i].?(0) 

```



### 3.2 高性能模式：内核/算法层

* **语法**：`list[idx]` (参数必须是 `Idx<T>` 类型)
* **返回类型**：**`T`** (Direct Value)
* **行为**：**Zero-Check, Zero-Panic**。
* **原理**：`Idx<T>` 只能通过 `list.indices()` 或 `list.validate(int)` 生成，持有该令牌即证明索引绝对安全。

---

## 4. 静态安全保障：`#[nopanic]` 契约

这是针对 **MCU 中断 (ISR)**、**OS 内核** 和 **高可靠逻辑** 的第一道防线。我们采用 **“默认宽容，按需严格”** 的策略。

### 4.1 默认行为 (Implicit CanPanic)

普通函数默认允许 Panic。这意味着你可以写 `1/0` 或 `assert(x)` 而无需标注。这降低了脚本化代码的编写成本。

### 4.2 严格模式 (`#[nopanic]`)

通过添加属性标注，将函数声明为 **“安全岛”**。

```auto
#[nopanic]
fn interrupt_handler() {
    // ... code ...
}

```

**编译器检查规则 (Static Analysis)**：

1. **传染性阻断**：在 `#[nopanic]` 函数内，调用任何普通函数（隐式 CanPanic）都会导致编译错误。
* *解法*：被调用的函数也必须标记为 `#[nopanic]`。


2. **禁止潜在崩溃源**：
* 禁止除法 `/`，除非编译器能证明除数非零。
* 禁止 `assert(...)` 或 `panic(...)`。
* 禁止裸写 `val.!!` (无参版本)。


3. **强制当场救援 (Local Rescue)**：
* 允许调用 CanPanic 代码，**但必须当场处理掉 Panic 风险**。



### 4.3 如何在 `#[nopanic]` 中生存？

```auto
#[nopanic]
fn safe_control_loop(sensor_val: int) {
    
    // ❌ 错误：除法可能 panic
    // let speed = 100 / sensor_val 

    // ✅ 正确：显式 Rescue
    // 语义：如果 sensor_val 是 0，除法会 panic，但 .!!(0) 会捕获它。
    // 编译器看到这里有个兜底，认可这段代码是 nopanic 的。
    let speed = (100 / sensor_val).!!(0)

    // ❌ 错误：数组访问 list[int] 返回 ?T，如果裸解包会导致 panic
    // let v = buffer[0].unwrap() 

    // ✅ 正确：使用安全索引
    // 编译器知道 i 是 Idx，访问 buffer[i] 绝不会 panic
    for i in buffer.indices() {
        buffer[i] = 0
    }
}

```

---

## 5. 运行时韧性：Resilient Mode (韧性模式)

这是 **第二道防线**。当代码没有 `#[nopanic]` 标注（普通业务代码），且程序员使用了 `.!!(def)` 时，编译器根据构建 Profile 生成不同代码。

### 5.1 Debug 模式 (`--profile=debug`)

* **原则**：**Fail Fast** (死谏)。
* **行为**：`expr.!!(def)` 会忽略 `def`，直接触发 Panic 并打印堆栈。
* **目的**：强迫程序员修 Bug。

### 5.2 Release 模式 (`--profile=release`)

* **原则**：**Fail Safe** (苟活)。
* **行为**：`expr.!!(def)` 会被编译为包含 `try-catch` (或类似跳转) 的保护代码。
1. 捕捉 Panic。
2. **发射 FATAL 日志** (保留案底)。
3. 返回 `def` 值。
4. 线程/Task 继续运行。


* **目的**：确保非核心逻辑的 Bug 不会导致整个服务/设备重启。

---

## 6. 综合范例：MCU 传感器数据处理

这个例子展示了三层体系与 `#[nopanic]` 的完美结合。

```auto
// 全局配置：定长数组
const N = 10
let raw_data: [int; N] = [0, ...]

// 普通函数：可能包含 panic，用于复杂计算
// 返回 !int (Result)
fn complex_algo(val: int) -> !int {
    if val < 0 { return error("Negative Input") } // L2 错误
    
    // 这里用了裸除法，潜在 L3 Panic
    // 所以这个函数不能标记为 #[nopanic]
    return 1000 / val 
}

// 核心中断：绝对不能挂
#[nopanic]
fn on_sensor_interrupt() {
    // 1. [L1 安全访问]
    // 使用 .indices() 获取安全令牌，零检查访问
    for i in raw_data.indices() {
        
        // 2. [L2 处理] Result -> Default
        // 调用普通函数 complex_algo。
        // 因为它是 CanPanic 的，我们在 nopanic 函数里调用它必须小心。
        // .!(0) 处理了 Result (L2) 错误。
        // .!!(0) 处理了潜在的 Panic (L3) 崩溃 (如除以零)。
        // 只有叠了这两层甲，编译器才允许在 #[nopanic] 里调用它。
        let processed = complex_algo(raw_data[i])
            .!(0)    // Handle Result: Error -> 0
            .!!(0)   // Handle Panic:  Crash -> 0
            
        // 更新数据
        raw_data[i] = processed
    }
    
    // 3. [L1 处理] 尝试性读取
    // 假设我们要读第 11 个数据（越界了）
    // list[int] 返回 ?T，.?(0) 完美兜底，不产生 Panic
    let check = raw_data[N].?(0) 
}

```

---

## 7. 总结

Auto 语言通过这套设计实现了一个不可能三角的平衡：

1. **写起来像 Python**：普通业务代码默认允许 Panic，大量使用 `?` 和 `!` 后缀，流畅简洁。
2. **跑起来像 Erlang**：Release 模式下，Panic 被自动隔离和救回，服务永远在线。
3. **稳得像 Rust**：关键路径加上 `#[nopanic]`，编译器通过静态分析确保没有任何隐式崩溃路径，满足硬实时需求。