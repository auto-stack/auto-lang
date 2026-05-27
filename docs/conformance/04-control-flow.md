# 04 — 控制流语义

## if / else if / else

### 基本形式
```auto
if condition {
    // branch A
} else if condition2 {
    // branch B
} else {
    // branch C
}
```
- **语义**: 按顺序评估条件，执行第一个为 true 的分支。若无匹配且无 else，不执行任何分支
- **AutoVM**: 条件 → `JMP_IF_Z` (0x61) 跳到下一分支 → 分支体 → `JMP` (0x60) 跳到结尾
- **a2r**: 直接转译为 Rust `if/else if/else`
- **条件类型**: 必须为 `bool`。非 bool 值产生类型错误
- **返回值**: 所有分支必须返回相同类型时，if 可作为表达式使用

### if 表达式
```auto
let x = if a > b { a } else { b }
```
- **语义**: 各分支最后一个表达式作为返回值
- **AutoVM**: 结果留在栈顶
- **a2r**: `let x = if a > b { a } else { b }`

## for 循环

### 范围循环 (exclusive) `for i in start..end`
```auto
for i in 0..10 {
    print(i.to_string())
}
```
- **语义**: i 从 start 到 end-1（左闭右开）
- **AutoVM**: `CREATE_RANGE` (0x75) → 循环变量初始化 → `LT` 比较 → `JMP_IF_Z` 退出 → 体 → 递增 → `JMP` 回到比较
- **a2r**: `for i in 0..10 { ... }` (Rust 范围)
- **示例**: `for i in 0..3` 执行 i=0, 1, 2

### 范围循环 (inclusive) `for i in start..=end`
- **语义**: i 从 start 到 end（左闭右闭）
- **AutoVM**: `CREATE_RANGE_EQ` (0x76) → `LE` 比较
- **a2r**: `for i in 0..=3 { ... }`
- **示例**: `for i in 0..=3` 执行 i=0, 1, 2, 3

### 迭代循环 `for item in collection`
```auto
for item in [1, 2, 3] {
    print(item.to_string())
}
```
- **语义**: 依次遍历集合中的每个元素
- **AutoVM**: 迭代器协议 — 创建迭代器 → `ITER_NEXT` 检查 → 获取元素 → 循环
- **a2r**: `for item in collection { ... }`

### 带索引迭代 `for i, item in collection`
- **语义**: i 为零基索引，item 为当前元素
- **a2r**: `for (i, item) in collection.iter().enumerate() { ... }`

### 条件循环 `for condition { }`
```auto
var i = 0
for i < 10 {
    i += 1
}
```
- **语义**: 等价于 while 循环，条件为 false 时退出
- **AutoVM**: 每次迭代前评估条件 → `JMP_IF_Z` 退出
- **a2r**: `while i < 10 { ... }`

## loop 循环

### 无限循环 `loop { }`
```auto
loop {
    if done {
        break
    }
}
```
- **语义**: 无限循环，必须通过 `break` 退出
- **AutoVM**: `JMP` 回到循环开头
- **a2r**: `loop { ... }`

## break 和 continue

### `break`
- **语义**: 立即退出当前循环
- **AutoVM**: `JMP` 到循环出口地址（存储在 `loop_exits` 栈中，退出时 patch 地址）
- **a2r**: `break`
- **约束**: 只能在循环体内使用

### `continue`
- **语义**: 跳过当前迭代，进入下一次迭代
- **AutoVM**: `JMP` 到循环递增步骤（存储在 `loop_continue_positions` 栈中）
- **a2r**: `continue`
- **约束**: 只能在循环体内使用

## is 模式匹配

### 基本形式
```auto
is expr {
    pattern1 -> body1,
    pattern2 -> body2,
    else -> body_else
}
```
- **语义**: 按顺序将 expr 与每个 pattern 匹配，执行第一个匹配的分支
- **AutoVM**: 将 target 存入临时变量 `_is_target` → 每个分支：加载 target → 匹配检查 → `JMP_IF_Z` 跳到下一分支 → 体 → `JMP` 到结尾
- **a2r**: 转译为 Rust `match`

### 字面量匹配
```auto
is x {
    0 -> print("zero"),
    1 -> print("one"),
    else -> print("other")
}
```
- **语义**: 值相等比较
- **a2r**: `match x { 0 => ..., 1 => ..., _ => ... }`

### 枚举变体匹配
```auto
is atom {
    Atom.Int(i) -> print(f"Int: $i"),
    Atom.Char(c) -> print(f"Char: $c")
}
```
- **语义**: `IS_VARIANT` (0xB9) opcode 检查变体类型，解构绑定字段
- **a2r**: `match atom { Atom::Int(i) => ..., Atom::Char(c) => ... }`

### Option/Result 匹配
```auto
is result {
    Ok(val) -> use(val),
    Err(e) -> handle(e)
}
```
- **AutoVM**: `IS_VARIANT` 区分 Ok/Err
- **a2r**: `match result { Ok(val) => ..., Err(e) => ... }`

### `else` 分支
- **语义**: 兜底分支，所有模式都不匹配时执行
- **必须性**: 如果所有可能的模式都已覆盖，`else` 可省略（编译器验证）

## return

### `return expr`
- **语义**: 从当前函数返回表达式的值
- **AutoVM**: 将值压栈 → `RET` opcode
- **a2r**: `return expr`

### `return` (void)
- **语义**: 从 void 函数返回
- **AutoVM**: `RET` opcode
- **a2r**: `return`

## 已知语义间隙

1. **if 表达式 vs 语句**: AutoVM 当前将 if 解析为语句，不作为表达式求值。a2r 支持 if 表达式。需明确规范
2. **break 带值**: `break expr`（从 loop 表达式中返回值）尚未支持
3. **is 穷尽检查**: 编译器不验证 is 匹配是否穷尽所有可能（无 exhaustiveness check）
4. **for 循环变量作用域**: 循环变量在循环体外是否可见的行为需要确认统一
