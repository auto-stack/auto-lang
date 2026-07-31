# Plan 377：统一值表示 — 消除 2-slot，让所有值都是单槽 NanoValue

> **状态**：⏳ 待实施（架构重构计划，源自 plan 378 复审发现的根因）
> **来源**：plan 378（`to_uint()` 栈错位）复审 + 用户关于"Auto 多数值类型 vs JS 单 f64"的架构讨论
> **影响仓库**：`auto-lang`（`crates/auto-val`、`crates/auto-lang/src/vm`）
> **风险**：高 — 触动整个 VM 值表示层；但**通过分阶段 + 每阶段全量回归**可控
> **前置**：plan 221（NaN-boxing 迁移，已完成）、plan 073（AutoVM f64/u64 支持，已完成）

---

## 0. 一句话目标

**让 `push_f64` / `push_i64` / `push_u64` 从"占 2 个栈槽"变成"占 1 个栈槽"，从而使栈、局部变量、全局变量、REPL 结果、函数返回值**全部统一为单槽 `NanoValue`，删除 codegen 里所有围绕 2-slot 的特殊逻辑。

---

## 1. 背景：为什么会有这个计划

### 1.1 plan 378 暴露的系统性问题

plan 378 修 `.to_uint()` 的栈错位时，发现根因不是单点 bug，而是**整个 VM 的值表示有一个架构缺陷**：某些类型（f64/i64/u64）在栈上占 **2 个槽**，其它类型占 1 个槽。这迫使 codegen 必须在编译期**精确计算每个表达式占几个槽**，一旦算错（比如没识别出方法调用返回 i64），栈就对不齐，产生垃圾值或崩溃。

plan 378 修了一连串下游症状（`contains_u64`/`is_u64_expr`/`needs_double_coercion`/`expr_type_hint`/print 路由/比较 opcode/REPL 捕获/global 1-slot 截断……），但每修一个，就暴露下一个——因为它们都是**同一个上游缺陷**（2-slot）的不同表现。复审还发现：global 和 local 的 slot 数不一致（global 存单个 NanoValue 只能 1-slot，栈/局部却用 2-slot），导致顶层 u64 变量的方法赋值错乱。

### 1.2 NaN-boxing 本不该有 2-slot

我们的 VM 采用 NaN-boxing，设计文档（plan 221）明确写着"Mirror JSC/SpiderMonkey NaN-boxing"。NaN-boxing 的**核心价值主张**就是：用单个 64 位 `NanoValue` 表示任何类型的值——这正是 SpiderMonkey（punboxing）、JavaScriptCore 的做法。

证据在代码里：

```rust
// nano_value.rs —— encode_f64 已经是单 NanoValue（正确！）
pub fn encode_f64(f: f64) -> NanoValue { f.to_bits() }   // 单个 u64 完整表示 f64

// virt_memory.rs —— push_f64 却占 2 槽（违背 NaN-box 原则）
pub fn push_f64(&mut self, val: f64) {
    self.raw_nv[self.sp] = val.to_bits();      // slot 1: 真值
    self.raw_nv[self.sp+1] = encode_null();    // slot 2: 无用的 padding！
    self.sp += 2;
}
```

`encode_f64` 证明单个 NanoValue 装得下完整 f64，`push_f64` 却硬塞了一个 null padding 槽。i64/u64 照抄了这个 2-slot 模式。这是 plan 073 时代（`Vec<i32>` → `Vec<NanoValue>` 迁移期）的遗留债——当时图省事把 8 字节拆成两个 4 字节槽，plan 221 引入 NaN-boxing 后**栈操作没跟着改**。

### 1.3 这是"早就解决了的问题"

NaN-boxing 在 JS 引擎里商业化运行超过十年。我们的设计文档也声明对齐它们。理论上只要**严格按 NaN-boxing 实现**（每值单槽），plan 378 修的那一大堆 2-slot 补丁**根本不需要存在**——没有 slot 计数、没有特殊 print、没有三套比较 opcode、global 和 local 天然一致。

### 1.4 Auto vs JS 的关键差异（用户提出，必须正面回答）

**JS 只有一种数值类型 f64**（单个 NaN-box 装得下），而 **Auto 有 i32/i64/i8/u32/u64/u8/f32/f64 八种数值类型**。这个差异**影响 i64/u64 的编码策略，但不影响"消除 2-slot"的总目标**。详见 §3 的分类讨论与 i64 编码方案。

---

## 2. 现状调研（事实清单）

### 2.1 当前 NaN-box 位级布局

```
bit:  63        62----52    51-48      47------------0
      [sign=1]  [0x7FF]     [tag 4位]  [payload 48位可用]
      
NANBOX_BASE = 0xFFF0_0000_0000_0000
is_nanboxed: (v >> 52) == 0xFFF   // 检查高 12 位
tag_of:      (v >> 48) & 0xF      // bit 51-48
PAYLOAD_MASK = 0xFFFF_FFFF        // ⚠️ 当前只用了低 32 位！bit 47-32 空闲！
```

**关键事实：payload 区有 48 位（bit 47-0），但当前 `PAYLOAD_MASK` 只用低 32 位，bit 47-32 这 16 位完全空闲。** 这为 i64 单槽编码提供了位级空间。

### 2.2 当前 tag 分配（4 位，已用 8 个，还剩 8 个）

| tag | 类型 | 编码 |
|-----|------|------|
| 0x0 | f64 | 直接位模式（非 nanboxed，靠 `is_nanboxed=false` 识别） |
| 0x1 | i32 | NANBOX_BASE \| TAG_I32 \| (i32 as u32) |
| 0x2 | string | NANBOX_BASE \| TAG_STRING \| (neg idx) |
| 0x3 | bool | NANBOX_BASE \| TAG_BOOL \| (sentinel) |
| 0x4 | null | NANBOX_BASE \| TAG_NULL \| (sentinel) |
| 0x5 | object | NANBOX_BASE \| TAG_OBJECT \| (id) |
| 0x6 | list | NANBOX_BASE \| TAG_LIST \| (id) |
| 0x7 | f32 | NANBOX_BASE \| TAG_F32 \| (f32 bits) |
| 0x8-0xF | **保留** | 可分配给 i64/u64 等 |

### 2.3 当前 2-slot 操作清单（要消除的目标）

| 位置 | 操作 | 当前行为 |
|------|------|---------|
| `virt_memory.rs:303` | `push_f64` | 占 2 槽（bits + null padding） |
| `virt_memory.rs:317` | `pop_f64` | pop 2 槽 |
| `virt_memory.rs:339` | `push_i64` | 占 2 槽（low + high） |
| `virt_memory.rs:347` | `pop_i64` | pop 2 槽 |
| `virt_memory.rs:355` | `push_u64` | 占 2 槽（low + high） |
| `virt_memory.rs:363` | `pop_u64` | pop 2 槽 |
| `virt_memory.rs:411` | `pop_operand`（typed pop） | 特判 f64 的 2-slot padding |

### 2.4 当前因 slot 数分叉的 opcode 族（消除 2-slot 后可合并/简化）

**算术**（每族 4 个变体，因操作数 slot 数不同）：
```
ADD(0x30) ADD_F(0x36) ADD_D(0x3B) ADD_U64(0x4C)   ← 同样是"加"，4 个 opcode
SUB/MUL/DIV/MOD 同理（共 16+ 个 opcode）
NEG/NEG_F/NEG_D（3 个）
```

**比较**（每族 3 个变体）：
```
EQ(0x50) EQ_D(0x56) EQ_U64(0xBA)   ← 同样是"等于"，3 个 opcode
NE/LT/GT/LE/GE 同理（plan 378 新增了 6 个 _U64）
```

**类型转换**（因 slot 布局不同）：
```
I32_TO_F32, I64_TO_F64, U64_TO_F64, PROMOTE_F64, TYPE_CAST_U64, TYPE_CAST_F64 ...
```

**返回**：
```
RET(1-slot) vs RET_D(2-slot)
```

**print**（plan 378 又加了 PRINT_U64）：
```
NATIVE_PRINT_I32, NATIVE_PRINT_F32, NATIVE_PRINT_F64, NATIVE_PRINT_U64, NATIVE_PRINT_STR
```

### 2.5 当前 codegen 里所有 slot 计数逻辑（消除 2-slot 后可大幅删除）

| 函数/逻辑 | 位置 | 作用 | 消除 2-slot 后 |
|-----------|------|------|----------------|
| `contains_u64` | codegen.rs | 递归判断表达式是否含 u64 | **可删除**（无需算 slot） |
| `is_u64_expr` | codegen.rs | 判断是否纯 u64 | **可删除** |
| `is_u64_operation` | codegen.rs | 判断二元运算是否 u64 | **可删除** |
| `needs_double_coercion` | codegen.rs | 判断是否需 f64 提升 | **简化**（只看类型不看 slot） |
| `needs_float_coercion` | codegen.rs | 判断是否需 f32 提升 | **简化** |
| `expr_type_hint` | codegen.rs | f-string 的 slot 推断 | **可删除**（全 1-slot） |
| `is_two_slot`（store 路径） | codegen.rs:2089 | 局部变量存几个槽 | **可删除**（恒 1） |
| `is_two_slot`（assign 路径） | codegen.rs:5615 | 赋值存几个槽 | **可删除** |
| `ret_is_two_slot` | codegen.rs:1404 | 函数返回 1 还是 2 槽 | **可删除** |
| `add_var` 的 2-slot 预留 | codegen.rs:10188 | u64 变量占 2 槽 | **可删除** |
| `body_is_two_slot` | codegen.rs:8129 | 块结果是否 2 槽 | **可删除** |
| global 赋值 POP high | codegen.rs:5586 | plan 378 的 global hack | **可删除** |
| print reloc（选 I32/F32/F64/U64） | codegen.rs:7508 | 按 slot 选 print | **简化为单一 print** |
| 比较运算 is_u64 分支 | codegen.rs:5997+ | 选 GT vs GT_U64 | **可删除**（统一 GT） |
| 算术运算 is_u64/is_double 分支 | codegen.rs:5940+ | 选 ADD vs ADD_U64/ADD_D | **可简化** |

### 2.6 i64/u64 的真实使用场景（决定编码策略）

stdlib 里 i64/u64 的实际用途：

| 场景 | 典型值上限 | 是否 < 2⁴⁸ |
|------|-----------|-----------|
| `time.now_ms()` 毫秒时间戳 | ~1.7×10¹² | ✅（2⁴⁸≈2.8×10¹⁴，157 倍裕量，够用 8800+ 年） |
| `time.now_sec()` 秒时间戳 | ~1.7×10⁹ | ✅ |
| `file.size()` 文件大小 | < 2⁴⁸（256TB） | ✅（远超现实单文件） |
| `math.abs/min/max` | 小整数 | ✅ |
| `str.to_uint()` 字符串解析 | 一般中等值 | ✅ |
| `conv.string_try_to_i64` | 一般中等值 | ✅ |

**结论：所有现实 i64/u64 场景都落在 48 位以内。** 超过 2⁴⁸ 的值（如密码学大数）极罕见，可用堆装箱兜底（BigInt 模式）。

---

## 3. Auto 多数值类型 vs JS 单 f64：设计分析

这是用户提出的核心架构问题，必须正面回答。

### 3.1 数值类型分三类讨论

**第一类：天然适配单槽（无任何问题）**

i32/u32/i8/u8/byte（≤32 位）、f32（32 位）、f64（64 位，直接位模式）。这些**全部能装进单个 NanoValue**。Auto 比 JS 多出来的 i8/u8/byte/f32，都是 32 位以内，塞进 payload 绰绰有余。**JS 的"只有 f64"和 Auto 的"类型多"在这一类里完全不影响设计**——都单槽。

**第二类：真正的难题——i64/u64（JS 没有对应物）**

这是 Auto 与 JS 的差异**真正命中**之处。i64/u64 需要 64 位，但 NaN-box 高 16 位被 tag 占，payload 只有 48 位。**JS 引擎不面对这个**（JS Number 就是 f64，单 NaN-box 装下）。Auto 有原生 i64/u64，**必须自己决定怎么塞**。这是 JS 经验不能直接照搬的唯一一处。

**第三类：非数值（object/list/string/bool/null）**

已经是单槽 handle 或 tag，不受影响。

### 3.2 i64/u64 编码方案对比

| 方案 | 做法 | 优点 | 缺点 | 评价 |
|------|------|------|------|------|
| **A：48 位 payload** | i64 塞进 48 位 payload，>2⁴⁸ 堆装箱 | 单槽、改动小、覆盖所有现实场景 | 失去 2⁴⁸~2⁶⁴ 范围（需堆兜底） | **推荐** |
| B：借 f64 位模式 | i64 当 f64 存 | 单槽 | >2⁵³ 丢精度，重蹈 JS Number 痛点 | ❌ 不可行 |
| C：TAG_I64 + 64 位 payload | 给 i64 留完整 64 位 | 完整范围 | 破坏 NaN-box 核心（f64 零开销） | ❌ 改动巨大 |
| D：一律堆装箱 | i64 永远存堆 handle | 单槽、与 object 一致 | 算术要解引用，性能损失 | 备选（若 A 不够） |

**决策：方案 A（48 位 payload + 堆装箱兜底）。** 理由：
1. 单槽，符合 NaN-boxing 原则。
2. 48 位覆盖所有现实场景（§2.6 验证）。
3. 改动最小（只需新增 encode/decode + 一个 tag）。
4. 溢出优雅降级（堆装箱），语义不丢。
5. 借鉴 JS BigInt 的"大数装箱"思路，但阈值更宽松（2⁴⁸ 而非 2⁵³）。

### 3.3 结论：Auto 类型多不改变"单槽"总目标

- **f32/f64/i32/i8/u8/byte**：JS 经验直接适用，单槽，零障碍。
- **i64/u64**：JS 无对应，需方案 A，但**结果仍是单槽**。
- **"每个值一个槽"的架构原则与"有几种数值类型"是正交的。** Auto 类型越多，单槽模型收益越大（新增类型只加 encode/decode，slot 逻辑永不变）。

---

## 4. 详细设计

### 4.1 i64/u64 单槽编码（方案 A 位级设计）

新增两个 tag（用保留的 0x8/0x9）：

```
TAG_I64 = 0x8   → NANBOX_BASE | TAG_I64 | (i64 as u64 & PAYLOAD48_MASK)
TAG_U64 = 0x9   → NANBOX_BASE | TAG_U64 | (u64 & PAYLOAD48_MASK)

PAYLOAD48_MASK = 0x0000_FFFF_FFFF_FFFF   // bit 47-0
```

encode/decode：

```rust
const PAYLOAD48_MASK: u64 = 0x0000_FFFF_FFFF_FFFF;
const TAG_I64: u64 = 0x0008_0000_0000_0000;
const TAG_U64: u64 = 0x0009_0000_0000_0000;

// i64: 若值落在 [-2^47, 2^47-1] 则直接编码；否则返回 None（调用方堆装箱）
pub fn try_encode_i64(i: i64) -> Option<NanoValue> {
    // 检查是否落在 48 位有符号范围
    if i >= -(1i64 << 47) && i < (1i64 << 47) {
        Some(NANBOX_BASE | TAG_I64 | (i as u64 & PAYLOAD48_MASK))
    } else {
        None  // 溢出 → 堆装箱
    }
}

pub fn decode_i64(v: NanoValue) -> i64 {
    // 48 位有符号扩展
    let raw = (v & PAYLOAD48_MASK) as i64;
    // sign-extend from bit 47
    if raw & (1 << 47) != 0 { raw | (!(PAYLOAD48_MASK as i64)) } else { raw }
}

// u64: 若值 < 2^48 则直接编码；否则堆装箱
pub fn try_encode_u64(u: u64) -> Option<NanoValue> {
    if u < (1u64 << 48) { Some(NANBOX_BASE | TAG_U64 | u) } else { None }
}

pub fn decode_u64(v: NanoValue) -> u64 { v & PAYLOAD48_MASK }
```

**注意：现有 payload 只用低 32 位，新编码用低 48 位，需把 `PAYLOAD_MASK` 从 `0xFFFF_FFFF` 扩到 `0x0000_FFFF_FFFF_FFFF`，并审计所有用 `PAYLOAD_MASK` 的 decode（i32/string/bool/object/list/f32 都只取低 32 位，扩 mask 不影响它们——它们本来就不该读高 16 位）。**

### 4.2 i64/u64 堆装箱（溢出兜底）

超过 2⁴⁸ 的 i64/u64 值，存为堆对象（类似 object handle），用 TAG_BIGINT（0xA）：

```rust
const TAG_BIGINT: u64 = 0x000A_0000_0000_0000;
pub fn encode_bigint(heap_id: u32) -> NanoValue { NANBOX_BASE | TAG_BIGINT | (heap_id as u64) }
```

堆对象存完整 i64/u64。算术 opcode 在执行时检测：若操作数是 BIGINT，走慢路径（解引用堆对象计算，结果若溢出仍是 BIGINT，否则降级回 I64/U64）。

**触发时机**：仅当 encode 时检测到溢出（`try_encode_i64/u64` 返回 None）。这是极罕见路径（现实场景不触发），但保证语义完整。

### 4.3 栈操作：全部改回单槽

```rust
// virt_memory.rs —— 消除 2-slot
pub fn push_f64(&mut self, val: f64) {
    self.push_nv(auto_val::encode_f64(val));   // 单槽！
}
pub fn pop_f64(&mut self) -> f64 {
    auto_val::decode_f64(self.pop_nv())         // 单槽！
}
pub fn push_i64(&mut self, val: i64) {
    // 48 位直接编码；溢出堆装箱
    match auto_val::try_encode_i64(val) {
        Some(nv) => self.push_nv(nv),
        None => self.push_bigint_i64(val),      // 慢路径
    }
}
pub fn pop_i64(&mut self) -> i64 {
    let nv = self.pop_nv();
    match auto_val::tag_of(nv) {
        t if t == TAG_I64 => auto_val::decode_i64(nv),
        t if t == TAG_BIGINT => self.heap_decode_bigint(nv),
        _ => auto_val::decode_i32(nv) as i64,   // 兼容 i32 操作数
    }
}
// push_u64/pop_u64 同理
```

### 4.4 简化 print：单一 PRINT_UNIFIED

消除 2-slot 后，print 只需 pop 单个 NanoValue，按 tag 解码打印：

```rust
pub fn shim_print_unified(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let nv = task.ram.pop_nv();
    let s = match auto_val::tag_of(nv) {
        t if t == 0 && !auto_val::is_nanboxed(nv) => auto_val::decode_f64(nv).to_string(),  // f64
        0x1 => auto_val::decode_i32(nv).to_string(),      // i32
        0x2 => /* string */,                               // string
        0x3 => auto_val::decode_bool(nv).to_string(),      // bool
        0x4 => "None".to_string(),                         // null
        0x5 => /* object */,                               // object
        0x6 => /* list */,                                 // list
        0x7 => auto_val::decode_f32(nv).to_string(),       // f32
        0x8 => auto_val::decode_i64(nv).to_string(),       // i64 (新)
        0x9 => auto_val::decode_u64(nv).to_string(),       // u64 (新)
        0xA => /* bigint */,                               // bigint (新)
        _ => "?".to_string(),
    };
    vm_print(vm, &s);
    Ok(())
}
```

codegen 的 print reloc（选 I32/F32/F64/U64）可**简化为统一 PRINT_UNIFIED**（或保留多入口但都 pop 单槽）。

### 4.5 简化算术/比较 opcode

**理想终态**：算术/比较 opcode 按"操作数类型"而非"slot 数"分派。但这是更大的重构（类型化 opcode），**本计划只做"消除 2-slot"这一步**，opcode 暂时保留 _F/_D/_U64 变体，但它们都 pop 单槽值。完全合并 opcode 留作后续优化（plan 389+）。

**本计划的 opcode 改动**：
- 所有 `_U64` opcode（ADD_U64 等）：执行体改为 `pop_nv` 单槽 + `decode_i64`，不再 pop 2 槽。
- `_D` opcode：改为 `pop_nv` + `decode_f64` 单槽。
- GT/EQ 等比较：plan 378 新增的 `_U64` 变体保留（单槽版），codeg·n 选 opcode 逻辑可简化（因 slot 都一样）。

### 4.6 统一 global/local/REPL/返回

消除 2-slot 后：
- **global**：`DashMap<String, NanoValue>` 单槽存任何值（含 i64/u64/f64），**plan 378 的 global 截断 hack 可删除**。
- **local**：`add_var` 恒预留 1 槽，删除 2-slot 预留逻辑。
- **REPL `last_result`**：`Option<NanoValue>` 单槽装得下任何值，**plan 378 的 `last_result_64` 字段可删除**。
- **函数返回**：RET 恒 1 槽，删除 RET_D。
- **f-string**：BUILD_FSTR 每部分 pop 1 槽，删除 `expr_type_hint`。

---

## 5. 实施方案（分阶段，每阶段全量回归）

### 阶段 0：准备（不破坏现状）
| 步骤 | 内容 | 验证 |
|------|------|------|
| 0.1 | 在 `auto-val/nano_value.rs` 新增 `TAG_I64/TAG_U64/TAG_BIGINT`、`try_encode_i64/decode_i64/try_encode_u64/decode_u64`、`PAYLOAD48_MASK` | 新增单元测试覆盖：48 位内 round-trip、边界值（±2⁴⁷）、溢出返回 None |
| 0.2 | 审计现有 `PAYLOAD_MASK` 使用点，确认扩到 48 位不影响 i32/string/bool/object/list/f32（它们只读低 32 位） | 既有 nano_value 测试全过 |
| 0.3 | 新增 i64/u64 堆对象类型（BigInt heap object）+ `push_bigint_i64`/`heap_decode_bigint` | 单元测试 |

### 阶段 1：f64 单槽化（风险最低，先验证路径）
| 步骤 | 内容 | 验证 |
|------|------|------|
| 1.1 | `push_f64`/`pop_f64` 改为单槽（`push_nv(encode_f64)` / `decode_f64(pop_nv)`） | `pop_operand` 的 f64 特判同步改 |
| 1.2 | codegen：删除 f64 相关的 2-slot 逻辑（`contains_double`/`is_double_operation` 的 slot 部分、`RET_D`→`RET`、store/assign 的 Double 2-slot 分支） | category 25 + 既有 f64 测试 + 全量回归 0 新增失败 |
| 1.3 | `_D` opcode 执行体改单槽 pop | 同上 |

### 阶段 2：i64/u64 单槽化（核心）
| 步骤 | 内容 | 验证 |
|------|------|------|
| 2.1 | `push_i64`/`pop_i64`/`push_u64`/`pop_u64` 改单槽（48 位 + 堆兜底） | 单元测试 + 大值（5e9、1e10、2⁴⁸ 边界）round-trip |
| 2.2 | codegen：删除 `contains_u64`/`is_u64_expr`/`is_u64_operation`、`expr_type_hint`、store/assign 的 2-slot 分支、`add_var` 2-slot 预留、global POP-high hack、`last_result_64` | category 25（含大值 009）+ 全量回归 |
| 2.3 | `_U64` opcode 执行体改单槽 pop；比较 `_U64` 同步 | 同上 |
| 2.4 | native `shim_str_to_uint_nv`：`push_i64` 现在单槽，验证正确 | to_uint 全场景 |

### 阶段 3：简化 print + 清理
| 步骤 | 内容 | 验证 |
|------|------|------|
| 3.1 | 新增 `shim_print_unified`（单槽，按 tag 解码），codegen print reloc 简化 | print 全类型测试 |
| 3.2 | 删除 plan 378 的 `NATIVE_PRINT_U64`（被 unified 取代）或保留为别名 | 全量回归 |
| 3.3 | 删除 `RET_D`（恒用 RET） | 函数返回测试 |
| 3.4 | 清理 codegen 里所有已废弃的 slot 计数函数 | 编译无 warning，全量回归 |

### 阶段 4：验收 + 文档
| 步骤 | 内容 | 验证 |
|------|------|------|
| 4.1 | 全量回归 `cargo test -p auto-lang`，与 plan 377 前基线对比，0 新增失败 | 失败集 diff 为空 |
| 4.2 | 大值端到端测试：顶层 global u64 大值不再截断（plan 378 的遗留解决） | 5e9 在 global 也正确 |
| 4.3 | 性能基准（plan 073 Phase 9.1 的 benchmark），确认单槽不劣化 | benchmark 无显著回退 |
| 4.4 | 更新 plan 378 文档，标注其 2-slot 补丁已被 plan 377 取代/删除 | 文档一致 |

---

## 6. 迁移：plan 378 补丁的去留

plan 378 加的 2-slot 补丁，在 plan 377 完成后**大部分可删除**（因为 2-slot 不存在了）：

| plan 378 的改动 | plan 377 后 |
|----------------|------------|
| `shim_str_to_uint_nv` 推 2-slot i64 | 改为单槽 `push_i64`（48 位编码） |
| `lookup_dot_method_type`/`dot_method_returns_64` | **可删除**（无需算 slot，但仍可用于类型推断，建议保留） |
| `contains_u64`/`is_u64_expr` 的 Dot 分支 | **可删除**（整个函数可删） |
| `needs_double_coercion` 的 Dot 分支 | **简化**（只看类型） |
| `expr_type_hint` 的 Dot 分支 | **可删除**（整个函数可删） |
| store 路径 2-slot 推断 | **可删除** |
| `NATIVE_PRINT_U64` + print 路由 | **被 PRINT_UNIFIED 取代** |
| u64 比较 opcode（EQ_U64 等） | 保留（单槽版），或并入统一比较 |
| global POP-high hack | **可删除**（global 单槽存完整值） |
| REPL `last_result_64` | **可删除**（单槽装得下） |

---

## 7. 风险与缓解

| 风险 | 概率 | 影响 | 缓解 |
|------|------|------|------|
| 改栈操作影响所有 VM 执行 | 高 | 高 | 分阶段（f64 先于 i64），每阶段全量回归 + 基线 diff |
| 48 位 i64 在某未预见场景溢出 | 低 | 中 | 堆装箱兜底（语义不丢）；大值测试 009 守护边界 |
| `PAYLOAD_MASK` 扩 48 位影响既有 decode | 低 | 中 | 阶段 0.2 审计所有 decode；既有 nano_value 测试覆盖 |
| 堆装箱路径（BigInt）未充分测试 | 中 | 中 | 单独测试 2⁴⁸ 边界值的算术 round-trip |
| opcode 改动破坏 a2c/a2r 转译器 | 中 | 中 | 转译器走 AST 不依赖 VM opcode，风险低；回归覆盖 |
| 性能回退（单槽 decode 开销） | 低 | 低 | 阶段 4.3 benchmark；encode/decode 全 `#[inline(always)]` |

---

## 8. 验收标准

1. ✅ `push_f64`/`push_i64`/`push_u64` 全部单槽（`sp += 1`，非 2）。
2. ✅ codegen 不再有 `contains_u64`/`is_u64_operation`/`expr_type_hint`/`is_two_slot` 等 slot 计数逻辑（或它们退化为常量）。
3. ✅ global/local/REPL/函数返回 全部单槽一致；**顶层 `var x u64 = "5e9".to_uint(); print(x)` = 5000000000**（plan 378 遗留解决）。
4. ✅ i64/u64 48 位内 round-trip 正确；2⁴⁸ 边界值堆装箱正确。
5. ✅ `cargo test -p auto-lang` 全量回归 0 新增失败（与基线 diff 为空）。
6. ✅ benchmark 无显著回退（±5%）。
7. ✅ plan 378 的 2-slot 补丁已按 §6 清理。

---

## 9. 非目标

- ❌ 不做"类型化 opcode 完全合并"（ADD/ADD_F/ADD_D/ADD_U64 全合成一个 ADD）——那是 plan 389+ 的优化，本计划只消除 2-slot（opcode 变体保留但都单槽）。
- ❌ 不改 AST/类型系统（`Type::I64` 等不变，只是运行时表示变单槽）。
- ❌ 不改 a2c/a2r 转译器（它们走 AST，不依赖 VM 栈布局）。
- ❌ 不处理 i128/任意精度整数（u64/i64 已是当前上限）。

---

## 10. 参考资料

### JS 引擎值表示
- [Value representation in JavaScript implementations (wingolog)](https://wingolog.org/archives/2011/05/18/value-representation-in-javascript-implementations) — SpiderMonkey punboxing / JSC nan-boxing 经典综述
- [ExBoxing: Bridging tag boxing and NaN-boxing (Mozilla 工程师)](https://medium.com/@kannanvijayan/exboxing-bridging-the-divide-between-tag-boxing-and-nan-boxing-07e39840e0ca) — NaN-boxing 的优势与权衡
- [JavaScript & the magic of 'NaN boxing'](https://www.diaconou.com/blog/javascript-nan-boxing-its-floating-point-numbers-all-the-way-down/) — 通俗解释
- [How JS Engines Store Values: Tagged Pointer and NaN Boxing](https://witch.work/en/posts/javascript-trip-of-js-value-tagged-pointer-nan-boxing) — 系列文章
- [Float Self-Tagging (arXiv)](https://arxiv.org/html/2411.16544v3) — 学术论文
- [Mozilla Bug 1401624 — Object-biased NaN boxing](https://bugzilla.mozilla.org/show_bug.cgi?id=1401624) — SpiderMonkey nan-box 优化细节

### 本仓库相关计划
- plan 221：NaN-boxing 迁移（已完成）— `docs/plans/old/221-nanboxing-migration.md`
- plan 073：AutoVM f64/u64 支持（已完成）— 2-slot 的历史起源
- plan 378：`to_uint()` 栈错位（已完成）— 暴露 2-slot 缺陷，补丁由本计划清理

### 代码位置
- `crates/auto-val/src/nano_value.rs` — NaN-box 编码（encode/decode/tag）
- `crates/auto-lang/src/vm/virt_memory.rs:303-367` — 2-slot 栈操作（push_f64/i64/u64）
- `crates/auto-lang/src/vm/codegen.rs` — slot 计数逻辑（§2.5 清单）
- `crates/auto-lang/src/vm/opcode.rs` — 因 slot 分叉的 opcode（§2.4 清单）
- `crates/auto-lang/src/vm/engine.rs:294` — `globals: DashMap<String, NanoValue>`（单槽，当前与 2-slot 栈不一致的根源）
