# 第 6 章 —— 发布：上线

到目前为止每一章都是开发（Dev）这一幕。本章是发布（Ship）这一幕：把你迭代过的 Auto 源码变成你部署的 Rust。

<Listing file="script-to-ship/ch06-ship-release/06_fib.at" view="scriptship" compare="true" caption="一个小程序——转译、读 Rust、对比输出" />

## 两条命令

```bash
# 开发（你在每一章里一直在做的）：
auto main.at

# 发布：
auto trans --path main.at rust      # 产出 main.a2r.rs
# 然后，路径上有 a2r-std：
cargo build --release               # 原生二进制，无 VM
```

上面的 `compare="true"` 代码块加了一个 **Run Both & Compare** 按钮。它运行 AutoVM，然后编译并运行 a2r 转译的 Rust，并检查两者输出一致。那个绿色对勾就是这门语言的全部承诺：你发布的东西行为与你迭代的东西一致。

## a2r-std 是干什么的

Auto 的标准库（`auto.io`、`auto.fs`、`auto.http`...）在 Rust 侧由 `a2r-std` crate 镜像。当 a2r 产出 `use crate::io::print`，它在 cargo-build 时解析为 `a2r_std::io::print`。所以 VM 原生执行的同一个调用，发布的二进制通过一个薄 Rust 包装执行——相同行为，Rust 性能。

## 收益

你保留了脚本的迭代速度，同时获得了 Rust 的交付属性。没有把 Python 移植到 C++ 的第二工程。没有"用 Rust 重写"阶段里行为悄悄漂移。转译器为一致性负责，而 [parity 仪表盘](../../parity/docs/parity-dashboard.html)精确展示这个一致性已在哪些库和模式上验证——今天，七个真实库共 232 个测试用例达到 L1。

## 接下来去哪

- **[语言教程](../tour/README)** —— 语言参考（语法、类型、控制流），如果你还没读过。
- **[Parity 仪表盘](../../parity/docs/parity-dashboard.html)** —— 实时证据：哪些库是 L1 已验证，哪些是路线图。
- **[Script-to-Ship 示例](../../examples/script-to-ship-demos/)** —— 可运行的单文件示例（serde_json、regex、wc），可克隆并发布。

← 返回 [Script-to-Ship 概览](README)
