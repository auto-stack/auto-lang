# 第 4 章 —— 错误处理

Auto 的错误模型就是 Rust 的错误模型：`Result`、`?` 操作符、显式传播。拼写略有不同；语义完全一致。

<Listing file="script-to-ship/ch04-errors/04_safe_div.at" view="scriptship" caption="一个 !int 函数（Result）带 .? 传播" />

## 拼写对照

| Auto | Rust |
|------|------|
| `fn f() !int` | `fn f() -> Result<i32, String>` |
| `return Err("...")` | `return Err("...".to_string())` |
| `return a / b`（在 `!int` 函数里） | `return Ok(a / b)` |
| `x.?` | `x?` |
| `is r { Ok(v) -> ..., Err(e) -> ... }` | `match r { Ok(v) => ..., Err(e) => ... }` |

在示例中，`half_of` 调用 `safe_div(...).?`——成功则除以 2，失败则把同一个 `Err` 传播给它的调用者。转译它，看 a2r 产出 `?`。然后 `main` 用 `is` 对结果做模式匹配，完全对应 Rust 的 `match`。

## 为什么是 Result，不是异常

Auto 没有异常。这是刻意的：一个能在任意深度意外失败的脚本，不是你能放心转译成 Rust 并发布的基底。通过把错误逼进类型系统（`!T` 返回类型），Auto 让每个失败点对编译器可见——也对 a2r 可见，它需要这个可见性来产出 Rust `Result` 而非 `panic!`。脚本*因为*失败模式是诚实且类型化的而易于迭代，不是尽管如此。

下一章：[Trait 与泛型 →](ch05-traits-generics)
