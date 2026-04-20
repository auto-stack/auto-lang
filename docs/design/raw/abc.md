这是一份 **AutoByteCode (ABC) 指令集架构 (ISA) 详细设计文档 v1.0**。

这份文档是连接 **AIE (编译器后端)** 和 **AutoVM (AutoVM/MicroVM)** 的契约。它的设计原则是：**紧凑 (Compact)**、**XIP 友好 (XIP-Friendly)**、**易于生成 (Easy to Codegen)**。

---

# AutoByteCode (ABC) Instruction Set Architecture

**Version:** 1.0 (MVP)
**Word Size:** 32-bit
**Endianness:** Little Endian
**Alignment:** Byte-aligned instructions, 4-byte aligned data preferred.

## 1. 数据模型 (Data Model)

### 1.1 基本单位 (Value)

AutoVM 是 32 位虚拟机。栈上的每个槽位 (Slot) 均为 32 位宽。

* **I32**: 32位有符号整数 (Two's complement)。
* **F32**: 32位 IEEE 754 浮点数。
* **PTR**: 32位指针 (指向 RAM 或 Flash)。
* **BOOL**: 0 表示 false，非 0 表示 true (通常为 1)。

### 1.2 指令编码格式 (Instruction Encoding)

采用 **变长编码**，以节省 Flash 空间。

`[OpCode (1 Byte)] [Operand A (Var)] [Operand B (Var)] ...`

* **OpCode**: 1 字节 (0x00 - 0xFF)。
* **Operand**: 紧跟 OpCode，长度取决于具体指令（如 `u8`, `i16`, `i32`）。
* **Immediate Data**: 所有的多字节操作数均为 **小端序 (Little Endian)**。

---

## 2. 指令集参考 (Instruction Reference)

### 符号说明

* `TOS`: 栈顶元素 (Top Of Stack)。
* `TOS-1`: 次栈顶元素。
* `[args]`: 指令后的立即数参数。
* `Stack`: `(Before) -> (After)` 栈的变化。

### 2.1 栈操作 (Stack Manipulation)

| Hex | Mnemonic | Operands | Stack Effect | Description |
| --- | --- | --- | --- | --- |
| 0x00 | **NOP** | - | `() -> ()` | 空操作，用于对齐或占位。 |
| 0x01 | **POP** | - | `(a) -> ()` | 弹出并丢弃栈顶元素。 |
| 0x02 | **POP_N** | `u8 n` | `(v1..vn) -> ()` | 弹出 N 个元素 (用于退出 Block 时的栈清理)。 |
| 0x03 | **DUP** | - | `(a) -> (a, a)` | 复制栈顶元素。 |
| 0x04 | **SWAP** | - | `(a, b) -> (b, a)` | 交换栈顶两个元素。 |

### 2.2 常量加载 (Constants)

| Hex | Mnemonic | Operands | Stack Effect | Description |
| --- | --- | --- | --- | --- |
| 0x10 | **CONST_I32** | `i32 val` | `() -> (val)` | 加载 4 字节整数压栈。 |
| 0x11 | **CONST_U8** | `u8 val` | `() -> (val)` | 加载 1 字节整数 (扩展为 i32) 压栈。节省空间的优化指令。 |
| 0x12 | **CONST_0** | - | `() -> (0)` | 压入 0。极高频指令优化。 |
| 0x13 | **CONST_1** | - | `() -> (1)` | 压入 1。 |
| 0x14 | **CONST_F32** | `f32 val` | `() -> (val)` | 加载 4 字节浮点数压栈。 |
| 0x1F | **LOAD_STR** | `u32 addr` | `() -> (ptr)` | 加载字符串常量（位于 Flash）的地址。 |

### 2.3 局部变量存取 (Local Variables)

AutoVM 基于 `Base Pointer (BP)` 访问局部变量。
`Address = BP + Index`。

| Hex | Mnemonic | Operands | Stack Effect | Description |
| --- | --- | --- | --- | --- |
| 0x20 | **LOAD_LOCAL** | `u8 idx` | `() -> (val)` | 读取局部变量 `stack[BP + idx]`。 |
| 0x21 | **STORE_LOCAL** | `u8 idx` | `(val) -> ()` | 写入局部变量 `stack[BP + idx] = val`。 |
| 0x22 | **LOAD_LOC_0** | - | `() -> (val)` | 读取第 0 号变量 (优化)。 |
| 0x23 | **LOAD_LOC_1** | - | `() -> (val)` | 读取第 1 号变量 (优化)。 |
| 0x24 | **LOAD_LOC_2** | - | `() -> (val)` | 读取第 2 号变量 (优化)。 |
| 0x25 | **STORE_LOC_0** | - | `(val) -> ()` | 写入第 0 号变量 (优化)。 |
| 0x26 | **STORE_LOC_1** | - | `(val) -> ()` | 写入第 1 号变量 (优化)。 |

### 2.4 算术与逻辑 (Arithmetic & Logic)

所有运算默认针对 `i32`。浮点运算有 `F` 前缀。

| Hex | Mnemonic | Operands | Stack Effect | Description |
| --- | --- | --- | --- | --- |
| 0x30 | **ADD** | - | `(a, b) -> (a+b)` | 整数加法。 |
| 0x31 | **SUB** | - | `(a, b) -> (a-b)` | 整数减法 (`TOS-1` 减 `TOS`)。 |
| 0x32 | **MUL** | - | `(a, b) -> (a*b)` | 整数乘法。 |
| 0x33 | **DIV** | - | `(a, b) -> (a/b)` | 整数除法 (需处理除零)。 |
| 0x34 | **MOD** | - | `(a, b) -> (a%b)` | 取模。 |
| 0x35 | **NEG** | - | `(a) -> (-a)` | 取反。 |
| 0x40 | **AND** | - | `(a, b) -> (a&b)` | 按位与 / 逻辑与 (非0为真)。 |
| 0x41 | **OR** | - | `(a, b) -> (a | b)` |
| 0x42 | **XOR** | - | `(a, b) -> (a^b)` | 按位异或。 |
| 0x43 | **NOT** | - | `(a) -> (!a)` | 逻辑非 (0->1, 非0->0)。 |
| 0x44 | **SHL** | - | `(a, b) -> (a<<b)` | 左移。 |
| 0x45 | **SHR** | - | `(a, b) -> (a>>b)` | 算术右移。 |

### 2.5 比较运算 (Comparison)

结果压入 `1` (True) 或 `0` (False)。

| Hex | Mnemonic | Operands | Stack Effect | Description |
| --- | --- | --- | --- | --- |
| 0x50 | **EQ** | - | `(a, b) -> (bool)` | `a == b` |
| 0x51 | **NE** | - | `(a, b) -> (bool)` | `a != b` |
| 0x52 | **LT** | - | `(a, b) -> (bool)` | `a < b` |
| 0x53 | **GT** | - | `(a, b) -> (bool)` | `a > b` |
| 0x54 | **LE** | - | `(a, b) -> (bool)` | `a <= b` |
| 0x55 | **GE** | - | `(a, b) -> (bool)` | `a >= b` |

### 2.6 控制流 (Control Flow)

所有跳转使用 **16位有符号相对偏移 (Signed 16-bit Relative Offset)**。
范围：当前 IP -32768 到 +32767 字节。

* Offset = Target Address - (Current IP + 3)。

| Hex | Mnemonic | Operands | Stack Effect | Description |
| --- | --- | --- | --- | --- |
| 0x60 | **JMP** | `i16 off` | `() -> ()` | 无条件跳转 `IP += off`。 |
| 0x61 | **JMP_IF_Z** | `i16 off` | `(cond) -> ()` | 如果栈顶为 0 (False) 则跳转，**并弹出栈顶**。 |
| 0x62 | **JMP_IF_NZ** | `i16 off` | `(cond) -> ()` | 如果栈顶非 0 (True) 则跳转，**并弹出栈顶**。 |
| 0x63 | **JMP_L** | `i32 off` | `() -> ()` | 长跳转 (32位偏移)，用于超大函数。 |

### 2.7 函数调用 (Function Call)

| Hex | Mnemonic | Operands | Stack Effect | Description |
| --- | --- | --- | --- | --- |
| 0x70 | **CALL** | `u32 fid` | `(args..) -> (ret)` | 静态调用。`fid` 是函数 ID。VM 自动压入 IP/BP 并更新 BP。 |
| 0x71 | **RET** | `u8 n_args` | `(ret) -> (ret)` | 从函数返回。`n_args` 表示调用者压入了多少参数，RET 负责清理这些参数，只保留返回值。 |
| 0x72 | **CALL_NAT** | `u16 nid` | `(args..) -> (ret)` | 调用 Native (C) 函数。`nid` 是 Native Table 索引。 |

### 2.8 调试与杂项 (Debug & Misc)

| Hex | Mnemonic | Operands | Stack Effect | Description |
| --- | --- | --- | --- | --- |
| 0xF0 | **PRINT** | - | `(val) -> ()` | 打印栈顶整数 (Debug用)。 |
| 0xFF | **HALT** | - | `() -> ()` | 停止虚拟机执行。 |

---

## 3. Auto Binary Format (ABF) 文件结构

为了支持 AIE 增量编译，Micro-Object (Frag) 的二进制格式需要包含元数据。

### 3.1 头部 (Frag Header)

```c
struct FragHeader {
    u32 magic;        // "AUTO"
    u32 version;      // 0x00010000
    u32 code_size;    // 字节码长度
    u32 const_size;   // 常量池长度
    u32 reloc_count;  // 重定位表项数量
};

```

### 3.2 字节码段 (Code Section)

纯粹的 OpCode 序列。

### 3.3 重定位表 (Relocation Table)

用于 AutoLive 链接。告诉 Loader 哪些指令的操作数需要被修正。

```c
struct RelocEntry {
    u32 offset;       // 在 Code 段中的偏移量 (例如某条 CALL 指令的参数位置)
    u32 target_id;    // 目标符号的全局 ID (Sid)
    u8  type;         // RELOC_FUNC_CALL, RELOC_GLOBAL_VAR 等
};

```

---

## 4. 关键设计决策说明

1. **关于 `RET n_args**`:
* 为了简化调用者的工作，我们采用 **Callee Cleanup (被调用者清理堆栈)** 的约定。
* 函数知道自己定义了多少个参数，所以 `RET` 指令直接把 SP 回退 `n_args` 个位置，只保留返回值在栈顶。


2. **关于局部变量索引**:
* `LOAD_LOCAL u8` 只支持 256 个局部变量。对于嵌入式函数，这绰绰有余。如果将来不够，可以增加 `LOAD_LOCAL_W (u16)`。


3. **关于 `JMP` 偏移**:
* 使用相对偏移使得代码是 **位置无关 (PIC)** 的。这意味着这块字节码可以被加载到 RAM 的任意位置执行，完全符合 AutoLive 的需求。


4. **关于 Native Call**:
* `CALL_NAT` 使用 `u16` 索引，支持最多 65535 个系统 API。MicroVM 内部维护一个 `NativeFunction[]` 数组。



---

这是 ABC 指令集的 v1.0 版。您可以基于此文档开始编写 AutoVM 的 `decode` 循环。