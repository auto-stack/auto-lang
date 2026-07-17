# 第 3 章 —— 类型与所有权

这一章是 Auto 证明自己不是"像 Rust"，而是*就是 Rust*的地方。struct、enum、所有权/借用词汇一一对应，a2r 产出你会手写的 Rust。

<Listing file="script-to-ship/ch03-types-ownership/03_point.at" view="scriptship" caption="一个 struct、一个借用的读取者、一个未被触碰的所有者" />

## 什么映射到什么

| Auto | Rust（a2r 输出） |
|------|-------------------|
| `type Point { x int, y int }` | `struct Point { x: i32, y: i32 }` |
| `Point(3, 4)` | `Point { x: 3, y: 4 }` |
| 读字段（`p.x`）而不消耗 | `&p` 语义——借用 |
| `p.view` | `&p`（显式共享借用） |
| `p.mut` | `&mut p`（显式可变借用） |
| `p.take` | move |

在示例中，`norm_sq` 读 `p.x` 和 `p.y` 但从不消耗 `p`，所以调用之后 `a` 仍可用（我们再次调用 `show(a)`）。点 **Transpile to Rust**，你会看到 a2r 产出一个借用形态、语义一致的 Rust 函数。

## 为什么这是承重声明

一门"看起来像 Rust"但悄悄 GC 一切、或每次传递都复制的语言，不是 Rust 的脚本层——它是在发布时分叉的方言。Auto 的所有权关键字存在，恰恰是为了让脚本的内存行为与发布的 Rust 内存行为是*同一个*行为。`parity/libs/` 的 parity 测试正是为此设计的：同一份源码，过 AutoVM 和过 a2r 编译的 Rust，必须产出相同输出——包括那些由所有权决定 move 还是 borrow 的情形。

## 诚实的边界

Auto 的 VM 是 32 位解释器；整数类型是 `i32`/`u32`/`f64` 等，匹配 Rust 的宽度。VM 有已知的边界限制（例如用户定义的 struct 在某些 Result 形态下跨模块边界——见 `parity/docs/known-divergences.md` DIV-URL-VM-1），parity 库在源码层面绕过它们并记录 workaround。这些是 VM 实现限制，不是语言语义分歧：a2r 输出不受影响，行为与 Rust 读取的一致。

下一章：[错误处理 →](ch04-errors)
