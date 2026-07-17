# 第 1 章 —— 你好，脚本与发布

整个理念浓缩在一个代码块里。一个 Auto 程序，两种运行方式，输出一致。

<Listing file="script-to-ship/ch01-hello-script-ship/01_hello.at" view="scriptship" caption="最小的闭环：一份源码，两种执行模式" />

## 刚才发生了什么

点 **Run in VM**。AutoVM 直接解释源码——没有编译器参与，没有构建步骤。你改了代码，点了运行，看到了输出。这就是**开发（Dev）**：迭代延迟以秒计。

现在点 **Transpile to Rust**。右侧窗格显示 `a2r` 为这份源码产出的精确 Rust。它不是草图或近似——它是 `auto trans --path main.at rust` 的真实输出。注意它读起来就像你会手写的 Rust：`fn greet(name: &str) -> String`、`for i in 1..=10`、`println!`。这就是**发布（Ship）**：交付物就是 Rust。

如果你的代码块有 **Run Both & Compare**，试一下：VM 和转译后的 Rust（编译后运行）打印相同内容。这就是**桥梁（Bridge）**，也是 Auto 区别于"现在用 Python、以后重写成 C"的关键：没有重写，编译器为一致性负责。

## 按钮背后的两条命令

```bash
# 开发 —— 即时解释，无编译
auto main.at

# 发布 —— 转译为 Rust，再 cargo build 发布
auto trans --path main.at rust     # 生成 main.a2r.rs
```

这就是完整的工作流。本教程余下部分围绕每一幕展开，在你实际使用的 Rust 模式上展示它。

## 为什么这很重要

Python 让世界认识到快速迭代的价值。Rust 让世界认识到安全的价值。通常的回答是两者都用——先用 Python 写，再付出重写税把它搬到 C/C++ 或 Rust。Auto 拒绝这个前提：你迭代的东西*就是*你发布的东西，而且这门语言的语义与 Rust 对齐。

下一章：[闭环中的 AI →](ch02-ai-in-the-loop)
