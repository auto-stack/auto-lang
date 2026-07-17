# 从脚本到发布 —— Auto 是 Rust 的脚本层

这是一份**工作流教程**，不是语言教程。它展示 Auto 如何让你像脚本开发者一样开发（跳过编译，刷新即见结果），又像 Rustacean 一样发布（原生性能 + 内存安全）——**同一份源码**。

> 如果你是 Auto**语言**本身（语法、类型、控制流）的新手，请先看[语言教程](../tour/README)。本教程假设你能读懂基本 Auto。

## 一段话的定位

Auto 是 Rust 的脚本层。你（或 AI）写 Auto；AutoVM 即时执行，无需编译，迭代-刷新循环以秒计。工作完成后，`a2r` 转译器把同一份源码转成简短、地道的 Rust，链上 `a2r-std`，以原生性能和内存安全发布。编译器保证脚本模式的行为与发布的 Rust 行为一致。

## 三段式

每一章都围绕三段式构建：

- **开发（Dev）** —— 写 Auto，用 VM 跑，秒级迭代（无编译）。
- **发布（Ship）** —— `a2r` 把同一份源码转成 Rust；`cargo build` 发布。
- **桥梁（Bridge）** —— 转译器保证 VM 输出 == Rust 输出。这不是口号，而是[由 parity 测试验证](../../parity/docs/parity-dashboard.html)。

## 是证据，不是承诺

Auto"VM 与 Rust 行为一致"的声明，由自动化三向 parity 框架（`parity/`）支撑：AutoVM 对 a2r 转译的 Rust 对原生 Rust，基于真实库。查看实时 [parity 仪表盘](../../parity/docs/parity-dashboard.html)了解当前覆盖（L1 = 三向已验证，L2 = VM 稳定，L3 = 路线图）。每一章都链接相关的 L1 证据。

## 章节

1. [你好，脚本与发布](ch01-hello-script-ship) —— 最小的闭环：一个程序，两种执行模式，输出一致。
2. [闭环中的 AI](ch02-ai-in-the-loop) —— 为什么脚本模式是 AI 驱动开发的正确形态。
3. [类型与所有权](ch03-types-ownership) —— struct、enum，以及 Auto 的 `view`/`mut`/`take` 如何映射到 Rust 的 `&`/`&mut`/move。
4. [错误处理](ch04-errors) —— Auto 的 `!` 函数与 `.?` 传播 → Rust 的 `Result` 与 `?`。
5. [Trait 与泛型](ch05-traits-generics) —— Auto 的 `spec` → Rust 的 `trait` / `impl` / `Box<dyn>`。含诚实边界（a2r 当前支持什么）。
6. [发布：上线](ch06-ship-release) —— `a2r` 命令行、链 `a2r-std`、性能与安全的收益。

## 怎么读

每个可运行代码块是一个 `<ScriptShipView>`：在左侧编辑 Auto，点 **Run in VM** 即时执行，点 **Transpile to Rust** 查看 a2r 产出的精确 Rust，（若显示）点 **Run Both & Compare** 实时观察两个后端输出一致。
