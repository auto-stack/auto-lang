# 01 — 算术运算语义

## 类型系统

Auto 整数类型：
- `int` (i32, 32-bit signed, 默认)
- `i64` (64-bit signed)
- `uint` (u32, 32-bit unsigned)
- `u64` (64-bit unsigned)
- `byte` (u8, 8-bit unsigned)

Auto 浮点类型：
- `float` / `f32` (32-bit IEEE 754)
- `f64` / `double` (64-bit IEEE 754)

## 加法 `+`

### `int + int` → `int`
- **语义**: 32-bit 有符号整数加法
- **溢出**: wrapping（与 Rust `i32::wrapping_add` 一致）
- **AutoVM**: `OpCode::ADD` → `wrapping_add`
- **a2r**: 直接转译为 `a + b`
- **一致性注意**: a2r 在 Rust debug 模式下溢出会 panic，release 模式下 wrapping。AutoVM 始终 wrapping。对偶测试需在 release 模式下运行
- **示例**:
  - `1 + 2` → `3`
  - `2147483647 + 1` → `-2147483648` (wrapping)

### `i64 + i64` → `i64`
- **语义**: 64-bit 有符号整数加法
- **溢出**: wrapping
- **AutoVM**: `OpCode::ADD_U64` → `wrapping_add`（注：opcode 名含 U64 但用于 i64/u64）
- **a2r**: 直接转译为 `a + b`

### `f64 + f64` → `f64`
- **语义**: IEEE 754 双精度浮点加法
- **AutoVM**: `OpCode::ADD_D`
- **a2r**: 直接转译为 `a + b`
- **示例**: `1.5 + 2.5` → `4.0`

### `f32 + f32` → `f32`
- **语义**: IEEE 754 单精度浮点加法
- **AutoVM**: `OpCode::ADD_F`
- **a2r**: 直接转译为 `a + b`

### `str + str` → `str`
- **语义**: 字符串拼接，返回新字符串（见 [02-strings.md](02-strings.md)）

### `int + str` → 编译错误
- **诊断**: "cannot add int and str" 或 "type mismatch"
- **建议**: `str(x)` 转换或使用 f-string `f"$x$val"`

## 减法 `-`

### `int - int` → `int`
- **溢出**: wrapping（`wrapping_sub`）
- **AutoVM**: `OpCode::SUB`
- **a2r**: 直接转译为 `a - b`
- **示例**: `0 - 1` → `-1`; `-2147483648 - 1` → `2147483647` (wrapping)

### 浮点减法
- 同加法模式，使用 `SUB_F` / `SUB_D`

## 乘法 `*`

### `int * int` → `int`
- **溢出**: wrapping（`wrapping_mul`）
- **AutoVM**: `OpCode::MUL`
- **a2r**: 直接转译为 `a * b`

### 浮点乘法
- 使用 `MUL_F` / `MUL_D`，IEEE 754 语义

## 除法 `/`

### `int / int` → `int`
- **语义**: 截断除法（向零取整），与 Rust `/` 一致
- **除零**: 运行时错误（division by zero）
- **溢出**: `i32::MIN / -1` 使用 `wrapping_div`，结果仍为 `i32::MIN`
- **AutoVM**: `OpCode::DIV` → `wrapping_div`，除零检查
- **a2r**: 直接转译为 `a / b`（Rust debug 模式下 i32::MIN / -1 会 panic）
- **示例**:
  - `7 / 2` → `3`
  - `-7 / 2` → `-3` (向零截断)
  - `7 / 0` → 运行时错误

### `f64 / f64` → `f64`
- **语义**: IEEE 754 浮点除法
- **除零**: 产生 `+inf` / `-inf` / `NaN`，不 panic
- **AutoVM**: `OpCode::DIV_D`
- **a2r**: 直接转译为 `a / b`

## 取模 `%`

### `int % int` → `int`
- **语义**: 余数运算，结果符号与被除数一致（与 Rust `%` 一致）
- **除零**: 运行时错误
- **AutoVM**: `OpCode::MOD`
- **a2r**: 直接转译为 `a % b`
- **示例**:
  - `7 % 3` → `1`
  - `-7 % 3` → `-1`
  - `7 % -3` → `1`

## 取负 `-x`

### `-int` → `int`
- **溢出**: wrapping（`wrapping_neg`）
- **AutoVM**: `OpCode::NEG`
- **a2r**: 直接转译为 `-x`
- **示例**: `-(-2147483648)` → `-2147483648` (wrapping)

## 比较运算符

### `==` / `!=`

| 类型 | AutoVM | a2r | 返回值 |
|------|--------|-----|--------|
| `int == int` | `OpCode::EQ` | `a == b` | `bool` |
| `f64 == f64` | `OpCode::EQ_D` | `a == b` | `bool` |
| `str == str` | `OpCode::EQ` (内容比较) | `a == b` | `bool` |

- **AutoVM bool 编码**: `i32::MIN` = true, `i32::MIN + 1` = false
- **浮点 NaN**: `NaN == NaN` → `false`（IEEE 754）

### `<` / `>` / `<=` / `>=`

| 操作 | AutoVM | a2r |
|------|--------|-----|
| `int < int` | `OpCode::LT` | `a < b` |
| `int > int` | `OpCode::GT` | `a > b` |
| `int <= int` | `OpCode::LE` | `a <= b` |
| `int >= int` | `OpCode::GE` | `a >= b` |
| `f64 < f64` | `OpCode::LT_D` | `a < b` |

## 类型转换

### `int → f64`
- **AutoVM**: `OpCode::I32_TO_F32`（注：opcode 名含 F32 但用于 f64 转换）
- **a2r**: 自动插入 `as f64`
- **触发**: 混合类型运算 `int + f64` 时自动转换 int 操作数

### `f64 → int`
- **语义**: 截断小数部分
- **AutoVM**: 无专用 opcode，通过内置函数
- **a2r**: `value as i32`

## 复合赋值

### `+=` / `-=` / `*=` / `/=` / `%=`
- **语义**: `x += y` 等价于 `x = x + y`
- **AutoVM**: 分别生成 LOAD + 运算 + STORE 指令序列
- **a2r**: 直接转译为 `x += y`
- **示例**: `var x = 5 \n x += 3` → `x` = `8`

## 自增/自减

### `++` / `--`
- **语义**: `x++` 等价于 `x += 1`，`x--` 等价于 `x -= 1`
- **AutoVM**: 生成 `+= 1` / `-= 1` 的指令序列
- **a2r**: 转译为 `x += 1` / `x -= 1`

## 已知语义间隙

1. **整数溢出行为不一致**: AutoVM 始终 wrapping，a2r 在 Rust debug 模式下 panic。对偶测试须在 release 模式运行
2. **混合类型算术**: `int + f64` 自动提升为 f64 运算，但 `int + i64` 的行为需要明确规范
3. **byte 溢出**: `byte` 类型的算术溢出行为未明确定义
