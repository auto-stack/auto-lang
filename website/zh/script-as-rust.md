---
title: Auto 是 Rust 的脚本层
layout: page
sidebar: false
---

<script setup>
import ScriptShipView from '../.vitepress/theme/components/ScriptShipView.vue'
const heroAuto = [
  'fn fib(n int) int {',
  '    if n < 2 { return n }',
  '    return fib(n - 1) + fib(n - 2)',
  '}',
  '',
  'fn main() {',
  '    var line = ""',
  '    for i in 0..12 {',
  '        if i > 0 { line = line + ", " }',
  '        line = line + fib(i).to(str)',
  '    }',
  '    print("fib: " + line)',
  '}',
].join('\n')
</script>

# Auto 是 Rust 的脚本层

> **Python 让世界认识到快速迭代的价值；Rust 让世界认识到安全的价值。
> Auto 拒绝二选一。**

你（或 AI）写 Auto。AutoVM 直接解释执行——无需编译，迭代-刷新的循环以秒计，而非分钟。工作完成后，`a2r` 把同一份源码转译成简短、地道的 Rust，链上 `a2r-std`，以原生性能和内存安全发布。编译器保证脚本模式的行为与发布的 Rust 行为一致。

## 直观感受：一个程序，两种执行模式

编辑左侧的 Auto 代码。点 **Run in VM** 即时执行（无编译）。点 **Transpile to Rust** 查看 `a2r` 产出的精确 Rust 代码。点 **Run Both & Compare** 实时观察两个后端输出一致。

<ScriptShipView
  :auto="heroAuto"
  :compare-run="true"
  caption="整个宣传点浓缩在一个代码块：当下是脚本，发布即 Rust，输出完全一致。"
/>

## 三段式

**开发（Dev）** —— 写 Auto，用 VM 跑，秒级迭代。无需为每一轮等待编译。AI 可以快速地犯错很多次，因为犯错的成本很低。

**发布（Ship）** —— `a2r` 把同一份源码转成你本会手写的 Rust：真正的 `trait` / `impl` / `Box<dyn>`、泛型、所有权、`Result` + `?`。链 `a2r-std`，`cargo build --release`，部署。

**桥梁（Bridge）** —— 转译器为"行为一致"负责。AutoVM 输出 == 转译 Rust 输出。这不是口号：[由 232 个三向 parity 测试验证](https://github.com/zhaopuming/auto-lang/blob/master/parity/docs/parity-dashboard.html)，覆盖七个真实库。

## 为什么这胜过"先用 Python，再重写成 Rust"

| | Python + C/C++（或 Rust） | Auto + Rust（a2r） |
|---|---|---|
| **生态** | 两个分裂的生态；FFI 是有断层的桥 | 一个生态——Auto 完整支持 Rust 的编程模式 + std + 三方库，`a2r-std` 是薄镜像 |
| **能力对等** | Python 缺类型/所有权/零成本抽象；C/Rust 缺易用性 | Auto 与 Rust 在"程序是什么意思"上一致（同样的 trait/泛型/所有权/async 语义） |
| **迁移成本** | Python → C/C++ 是完整重写工程，需 AI 大量介入 | Auto → Rust 是机械转译；编译器保证行为 |
| **行为一致性** | 无——Python 和 C 在数值/并发/内存上经常不一致 | 强制保证——parity 框架在出现差异时让构建失败 |
| **AI 辅助** | Python 好生成；C/Rust 重写是另一座山 | Auto 好生成（脚本模式容忍不完美）；Rust 步骤是确定性的 |

核心差异：用 Python+C，重写是*设计*问题（两种语言语义不一致）。用 Auto+Rust，"重写"是*编译器*步骤——而编译器比 AI 重写可靠得多。

## 是证据，不是承诺

Auto"VM 与 Rust 行为一致"的声明，由自动化三向 parity 框架支撑：AutoVM 对 a2r 转译的 Rust 对原生 Rust，基于真实库。这是区分可信工具与营销话术的关键。

**L1 —— 当前已三向验证（232 个测试用例）：**

| 库 | 用例数 | 验证点 |
|---------|-------|-------------------|
| base64 | 33/33 | 字节/字符串循环、错误处理 |
| url | 30/30 | 记录类型、Result、模块边界 |
| serde_json | 56/56 | 递归数据、tag/enum、泛型 |
| regex | 45/45 | 模式匹配、回溯 |
| cli_app | 32/32 | 纯 std 文本处理（wc 风格） |
| trait_advanced | 10/10 | spec/trait 分发（L1 子集） |
| tokio | 13/13 | async spawn/join、channel |

完整矩阵和每库详情见 [parity 仪表盘](https://github.com/zhaopuming/auto-lang/blob/master/parity/docs/parity-dashboard.html)。

**诚实的边界（L3 —— 路线图，尚未验证）：**

这些在 [known-divergences](https://github.com/zhaopuming/auto-lang/blob/master/parity/docs/known-divergences.md) 中公开记录，不隐瞒：

- **spec 中的关联类型** —— Auto 语法尚无此构造（语言缺口）。
- **返回值的默认方法体** —— a2r 包裹 bug（void 默认方法可用）。
- **泛型 spec 实现** —— a2r 丢弃具体类型参数。
- **有界泛型函数**（`fn f<T has Spec>`）—— bound 语法 + VM 分发缺口。
- **reqwest / http_client_sync parity** —— 需要 in-process mock-server 框架。

Auto 不会在未完成的地方假装已完成。L1 列表是已验证的；L3 列表是路线图，教程的每一章都会告诉你某特性处于哪一档。

## 从这里开始

→ **[从脚本到发布 —— 互动教程](/zh/docs/script-to-ship/README)** ——
六章，每个代码块都可在浏览器中运行。

→ **[Parity 仪表盘](https://github.com/zhaopuming/auto-lang/blob/master/parity/docs/parity-dashboard.html)** —— 证据。

→ **[Script-to-Ship 示例](https://github.com/zhaopuming/auto-lang/tree/master/examples/script-to-ship-demos/)** ——
可运行的单文件示例（serde_json、regex、wc），可克隆并发布。

```bash
# 开发 —— 即时解释，无编译
auto main.at

# 发布 —— 转译为 Rust，再 cargo build 发布
auto trans --path main.at rust
```
