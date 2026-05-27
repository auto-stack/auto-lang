# 02 — 字符串操作语义

## 字符串类型

Auto 有两种字符串表示：
- `str` — 字符串切片（类似 Rust `&str`）
- `String` — 堆分配字符串（类似 Rust `String`）

大多数情况下 `str` 和 `String` 可互换使用。a2r 将 `str` 和 `String` 统一映射为 Rust `String`。

## 字符串字面量

### 普通字符串 `"hello"`
- **语义**: 不可变字符串字面量
- **AutoVM**: 存入 StringPool，运行时引用索引
- **a2r**: 转译为 `"hello".to_string()` 或 `"hello"` 取决于上下文

### F-string `f"hello $name"` / `f"result: ${1 + 2}"`
- **语义**: 带插值的格式化字符串
- **AutoVM**: `OpCode::BUILD_FSTR` — 读取 part count + 类型标签（0=i32, 1=string, 2=f64, 3=f32, 4=u64），从栈上弹出值拼接
- **a2r**: 转译为 `format!("hello {}", name)`
- **插值规则**:
  - `$var` — 插入变量值（调用 `.to_string()`）
  - `${expr}` — 插入表达式值
- **转义**: 大括号需转义，引号需转义
- **示例**:
  - `f"hello $name"` → `"hello world"` (name = "world")
  - `f"${a} + ${b}"` → `"3 + 4"` (a=3, b=4)

### 反引号字符串 `` `hello ${name}` ``
- **语义**: 等价于 f-string 的替代语法
- **AutoVM**: 同 BUILD_FSTR
- **a2r**: 同 format!()

### 多行字符串 `"""`
- **语义**: 三引号多行字符串，保留换行和缩进
- **AutoVM**: 作为普通字符串处理
- **a2r**: 转译为 Rust 原始字符串

### C 字符串 `c"null-terminated"`
- **语义**: C 兼容的 null-terminated 字符串
- **AutoVM**: 作为普通字符串存储
- **a2r**: 可能用于 FFI 场景

## 字符串拼接 `+`

### `str + str` → `str`
- **语义**: 创建新字符串，内容为两个操作数拼接
- **AutoVM**: `OpCode::STR_CAT` — 优化路径，或 `OpCode::ADD` 类型检测
- **a2r**: `format!("{}{}", a, b)`（当 `expr_contains_string()` 检测到字符串类型时）
- **示例**: `"hello" + " " + "world"` → `"hello world"`

### `str + int` → 编译错误
- **诊断**: 类型不匹配
- **建议**: `str + str(x)` 或使用 f-string

## 字符串比较

### `==` / `!=`
- **语义**: 内容比较（非引用比较）
- **AutoVM**: `OpCode::EQ` — 内容感知比较（Plan 197 Task 2 实现）
- **a2r**: `a == b`（Rust String 的 PartialEq 实现）
- **示例**: `"hello" == "hello"` → `true`

### `<` / `>` / `<=` / `>=`
- **语义**: 字典序比较（lexicographic）
- **AutoVM**: 使用整数比较 opcode，字符串先转为可比较形式
- **a2r**: `a < b`（Rust String 的 Ord 实现）

## 字符串索引

### `str[index]`
- **语义**: 按 byte 索引访问（非字符索引）
- **AutoVM**: `OpCode::GET_ELEM`（Plan 118 Phase 4）
- **a2r**: 转译为字节切片访问
- **越界**: 运行时错误
- **注意**: 非 ASCII 字符的多字节索引可能产生不完整字符

## 字符串方法

### `.len()` → `int`
- **语义**: 返回字符串 byte 长度
- **AutoVM**: native shim `Str.len()`
- **a2r**: `.len()` (Rust)
- **示例**: `"hello".len()` → `5`; `"你好".len()` → `6` (UTF-8)

### `.to_string()` → `str`
- **语义**: 返回自身（identity 操作，用于类型统一）
- **AutoVM**: `OpCode::TO_STR`
- **a2r**: `.to_string()` (Rust)

### `.trim()` → `str`
- **语义**: 去除首尾空白
- **a2r**: `.trim().to_string()`

### `.replace(old, new)` → `str`
- **语义**: 替换所有匹配子串
- **a2r**: `.replace(old, new)`

### `.to_uppercase()` / `.to_lowercase()` → `str`
- **语义**: 大小写转换
- **a2r**: `.to_uppercase()` / `.to_lowercase()`

### `.contains(sub)` → `bool`
- **语义**: 检查是否包含子串
- **a2r**: `.contains(sub)`

### `.substr(start, len)` → `str`
- **语义**: 截取子字符串
- **AutoVM**: native shim
- **a2r**: 转译为切片操作

### `.char_at(index)` → `str`
- **语义**: 获取指定位置的字符（单字符字符串）

### `.split(delim)` → `List<str>`
- **语义**: 按分隔符分割

## 类型转换

### `str(x)` / `x.to_string()`
- **语义**: 将任意值转为字符串
- **int → str**: `str(42)` → `"42"`
- **f64 → str**: `str(3.14)` → `"3.14"`
- **bool → str**: `str(true)` → `"true"`
- **AutoVM**: `OpCode::TO_STR` (0x7A)，类型特定：`TYPE_TO_STR` (0xEC), `TYPE_F64_TO_STR` (0xF3), `TYPE_I64_TO_STR` (0xF4)
- **a2r**: `.to_string()` (Rust)

### `int(str)` / `str.parse_int()`
- **语义**: 字符串转整数
- **失败**: 返回 0 或 panic（取决于实现）

## 已知语义间隙

1. **byte 索引 vs 字符索引**: 字符串索引按 byte 而非 Unicode 字符，非 ASCII 字符串行为需明确
2. **字符串拼接性能**: AutoVM 使用 STR_CAT 优化路径，a2r 使用 format!()，性能差异不影响语义
3. **format 精度**: f-string 的浮点格式化精度（默认 `{}` vs `{:.*}`）在两个后端可能不同
