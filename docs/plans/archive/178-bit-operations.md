# Plan 178: Bit Operations — 二进制字面量 + 位运算方法 + 位域视图

## Context

根据 `docs/design/bit-operations.md` 设计文档，为 AutoLang 添加位操作能力。
目标是覆盖 C 语言位操作的全部功能，并以方法调用和视图模式提供更安全的 API。

本次实现 Layer 1 + Layer 2，Layer 3（声明式位域 type）留待后续。

## 设计决策

- **方法调用，不添加运算符** — `val.and(mask)` 而非 `val & mask`，解放 `|` 给 pipe
- **VM native method 路径** — 复用现有 `.upper()` / `.len()` 相同架构
- **位域视图编译时展开** — `.bits().read()` 直接编译为掩码+移位，无运行时对象
- **方案 A 先行，方案 B 留后** — 先满足嵌入式驱动直接使用场景

## Phase 1: 二进制字面量 (0b prefix)

**修改文件**: `crates/auto-lang/src/lexer.rs`

在 `number()` 方法中，检测到 `0` 后增加 `b` 分支（与 `0x` 对称）：

```rust
// 在 if self.peek('0') 块中，与 if self.peek('x') 并列：
if self.peek('b') {
    text.push('b');
    self.chars.next();
    is_binary = true;
}
```

二进制模式：只接受 `0`/`1`/`_`，使用 `is_digit(2)` 检查。
Token 类型为 `Int`，值存储为十进制字符串（编译时已转换）。

**验证**:
- `0b1010` → 整数 10
- `0b0000_1111` → 整数 15（支持下划线分隔）
- VM 测试: `let a = 0b1010` 后 `print(a)` 输出 `10`

## Phase 2: 基础位运算方法 (Layer 1)

### 方法列表

| 方法 | 语义 | 参数 |
|------|------|------|
| `.and(mask)` | 按位与 | mask: int |
| `.or(mask)` | 按位或 | mask: int |
| `.xor(mask)` | 按位异或 | mask: int |
| `.not()` | 按位取反 | 无参 |
| `.shl(n)` | 逻辑左移 | n: int |
| `.shr(n)` | 逻辑右移 | n: int |
| `.sar(n)` | 算术右移 | n: int |
| `.rol(n)` | 循环左移 | n: int |
| `.ror(n)` | 循环右移 | n: int |

### 修改文件

**`crates/auto-lang/src/vm/codegen.rs`**:
- 在 `compile_native_method` 或等效位置，为 int/uint 类型注册位运算方法名
- 每个方法名映射到一个 native shim ID

**`crates/auto-lang/src/vm/native.rs`**:
- 注册新 shim 函数，实现具体位运算逻辑
- 新增 native ID 常量（NATIVE_BIT_AND 等）

**`crates/auto-lang/src/vm/opcode.rs`**:
- 检查是否需要新增 OpCode，或可复用现有算术指令
- 位运算可直接在 shim 中用 Rust 实现，无需新 OpCode

### 验证

VM 测试用例（放入 `test/vm/02_bit_ops/`）：
- `0b1100.and(0b1010)` → `8` (0b1000)
- `0b1000.or(0b0011)` → `11` (0b1011)
- `0b1100.xor(0b1010)` → `6` (0b0110)
- `0b00001111.not()` → 对应取反值
- `0b0001.shl(3)` → `8`
- `0b1000.shr(2)` → `2`

## Phase 3: 位扫描方法 (Layer 1)

### 方法列表

| 方法 | 语义 | 返回 |
|------|------|------|
| `.count_ones()` | 1 的个数 | int |
| `.leading_zeros()` | 前导零个数 | int |
| `.trailing_zeros()` | 后缀零个数 | int |
| `.flip()` | 位序镜像翻转 (bit-reverse) | int |

### 修改文件

同 Phase 2，在 codegen + native 中注册。

### 验证

- `0b1010.count_ones()` → `2`
- `0b00100000.leading_zeros()` → 对应值
- `0b00101000.trailing_zeros()` → `3`
- `0b10110001.flip()` → 位序反转结果

## Phase 4: 动态位域视图 (Layer 2)

### 设计

编译时展开方案（零成本抽象）。codegen 识别以下链式调用模式并直接生成掩码指令：

**`.bits(start, len).read()`**:
```
(val >> start) & ((1 << len) - 1)
```

**`.bits(start, len).write(v)`**:
```
(val & ~(((1 << len) - 1) << start)) | ((v & ((1 << len) - 1)) << start)
```

**`.bit(n).test()`**: `(val >> n) & 1 != 0` → bool

**`.bit(n).on()`**: `val | (1 << n)`

**`.bit(n).off()`**: `val & ~(1 << n)`

**`.bit(n).flip()`**: `val ^ (1 << n)`

### 实现策略

codegen 中检测 dot-call 链：
1. 识别 `obj.bits(start, len)` 模式 → 记录为 BitFieldView { obj, start, len }
2. 后续遇到 `.read()` / `.write(v)` → 展开为上述掩码运算
3. 识别 `obj.bit(n)` 模式 → 记录为 BitRef { obj, n }
4. 后续遇到 `.on()` / `.off()` / `.flip()` / `.test()` → 展开为上述运算

### 限制

- 视图不能赋值给变量（不支持 `let v = x.bits(2, 3)`）
- 必须链式调用：`.bits(s, l).read()` 或 `.bit(n).on()`
- 未来需要一等公民视图时再实现方案 B

### 验证

- `0b10110011.bits(2, 3).read()` → `6` (0b110)
- `0b00000000.bit(3).on()` → `8` (0b00001000)
- `0b00001000.bit(3).off()` → `0`
- `0b00001000.bit(3).test()` → `true`
- `0b00000000.bit(3).flip()` → `8`

## 关键文件

| 文件 | 改动 |
|------|------|
| `crates/auto-lang/src/lexer.rs` | Phase 1: 0b 二进制字面量 |
| `crates/auto-lang/src/vm/codegen.rs` | Phase 2-4: 注册位方法 + 视图展开 |
| `crates/auto-lang/src/vm/native.rs` | Phase 2-3: shim 函数实现 |
| `crates/auto-lang/src/vm/opcode.rs` | Phase 2: 可能新增 OpCode |
| `crates/auto-lang/test/vm/02_bit_ops/` | Phase 2-4: 测试用例 |
