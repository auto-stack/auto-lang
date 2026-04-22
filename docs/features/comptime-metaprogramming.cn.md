# 编译期元编程

Auto 的编译期执行引擎允许你在编译期间运行代码，实现强大的元编程能力，而无需传统的宏。

## 什么是 Comptime？

`comptime` 关键字标记在编译时而非运行时执行的代码。这使你可以：
- 以编程方式生成代码
- 在程序运行前验证不变式
- 从复杂表达式计算常量
- 基于编译期参数配置行为

## 示例

```auto
comptime fn generate_table(size: int) -> [int] {
    var table: [int; size]
    for i in 0..size {
        table[i] = i * i
    }
    return table
}

const squares = generate_table(16)
```

## 与宏的对比

与 C 宏或 Rust 的 `macro_rules!` 不同，comptime 代码具有以下特点：
- 类型安全
- 可调试
- 使用与运行时代码相同的语言编写
- 能够访问完整的编译器 API
