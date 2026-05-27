# 10 — 内置函数语义

## print(value)

### `print(expr)` → `void`
- **语义**: 将值转为字符串并输出到 stdout，末尾加换行
- **AutoVM**: `OpCode::PRINT` (0xF0) — 调用 native print shim
- **a2r**: `println!("{}", expr)`
- **多参数**: `print(a, b, c)` 输出 `a b c\n`（空格分隔，末尾换行）
- **类型**: 任意类型均可打印，自动调用 `.to_string()`
- **示例**:
  - `print("hello")` → stdout: `hello\n`
  - `print(42)` → stdout: `42\n`
  - `print(1, 2, 3)` → stdout: `1 2 3\n`

## len(collection)

### `len(str)` → `int`
- **语义**: 字符串 byte 长度
- **AutoVM**: native shim `Str.len()`
- **a2r**: `s.len()` (Rust)
- **示例**: `len("hello")` → `5`

### `len(list)` → `int`
- **语义**: 列表元素数量
- **a2r**: `list.len()`

### `len(map)` → `int`
- **语义**: 映射键值对数量
- **a2r**: `map.len()`

## str(value) — 类型转换

### `str(int)` → `str`
- **语义**: 整数转字符串
- **AutoVM**: `OpCode::TO_STR`
- **a2r**: `value.to_string()`
- **示例**: `str(42)` → `"42"`

### `str(f64)` → `str`
- **语义**: 浮点转字符串
- **示例**: `str(3.14)` → `"3.14"`

### `str(bool)` → `str`
- **示例**: `str(true)` → `"true"`

## 数值转换

### `int(str)` → `int`
- **语义**: 字符串转整数
- **失败**: 运行时错误或 0
- **a2r**: `str.parse::<i32>().unwrap_or(0)` 或 `str.parse::<i32>().unwrap()`

### `int(float)` → `int`
- **语义**: 截断小数部分
- **a2r**: `value as i32`

## 数学函数

### `.sqrt()` → `f64`
- **语义**: 平方根
- **a2r**: `value.sqrt()`

### `.abs()` → 同类型
- **语义**: 绝对值
- **a2r**: `value.abs()`

### `.max(other)` / `.min(other)` → 同类型
- **语义**: 最大/最小值
- **a2r**: `value.max(other)` / `value.min(other)`

### `.pow(exp)` → 同类型
- **语义**: 幂运算
- **a2r**: `value.pow(exp)`

## 集合函数

### `List.new()` → `List<T>`
- **语义**: 创建空列表
- **a2r**: `Vec::new()`

### `Map.new()` → `Map<K, V>`
- **语义**: 创建空映射
- **a2r**: `HashMap::new()`

### `.push(val)` → `void`
- **语义**: 追加元素（见 [03-collections.md](03-collections.md)）

### `.pop()` → `?T`
- **语义**: 弹出末尾元素

### `.get(index)` → `?T`
- **语义**: 安全索引访问

### `.sort()` → `void`
- **语义**: 原地排序

### `.reverse()` → `void`
- **语义**: 原地反转

### `.join(sep)` → `str` (仅 List<str>)
- **语义**: 用分隔符连接所有元素
- **a2r**: `list.join(sep)`
- **示例**: `["a", "b", "c"].join(", ")` → `"a, b, c"`

## Option 操作

### `Some(val)` → `?T`
- **语义**: 构造 Some 值
- **AutoVM**: `CREATE_SOME` opcode
- **a2r**: `Some(val)`

### `None` → `?T`
- **语义**: 空 Option 值
- **AutoVM**: 特殊 nil 值
- **a2r**: `None`

### `??` 空值合并
```auto
let val = opt ?? default
```
- **语义**: 如果 opt 为 Some 返回其值，否则返回 default
- **AutoVM**: `OpCode::NULL_COALESCE`
- **a2r**: `opt.unwrap_or(default)`

### `?.` 安全访问
```auto
let name = obj?.name
```
- **语义**: 如果 obj 为 None，返回 None 而不是报错
- **a2r**: `obj.and_then(|o| Some(o.name))`

## Result 操作

### `Ok(val)` → `Result<T, E>`
- **a2r**: `Ok(val)`

### `Err(msg)` → `Result<T, E>`
- **a2r**: `Err(msg.into())`

### `.?` 错误传播
```auto
let val = might_fail()?
```
- **语义**: 如果 Result 为 Ok 返回其值，如果是 Err 则从当前函数提前返回 Err
- **AutoVM**: `OpCode::ERROR_PROPAGATE`
- **a2r**: `might_fail()?`
- **约束**: 只能在返回 `Result` 或标记 `!` 的函数中使用

## 类型检查

### `type_of(value)` → `str`
- **语义**: 返回值的类型名称字符串
- **AutoVM**: native shim
- **a2r**: `std::any::type_name_of_val(&value)` 或编译时确定
- **示例**: `type_of(42)` → `"int"`

## clone / copy

### `.clone()` → `T`
- **语义**: 深拷贝
- **a2r**: `value.clone()`

## 已知语义间隙

1. **print 输出格式**: 多参数 print 的分隔符（空格 vs 无分隔）需统一确认
2. **str(int) 格式化**: 负数的字符串表示、大数的格式化需确认
3. **parse 错误处理**: `int("abc")` 是 panic 还是返回默认值，行为需统一
4. **type_of 精确度**: AutoVM 和 a2r 返回的类型名可能不同（"int" vs "i32"）
