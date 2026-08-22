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

**桥梁（Bridge）** —— 转译器为"行为一致"负责。AutoVM 输出 == 转译 Rust 输出。这不是口号：[由 141 个三向 parity 测试验证](https://github.com/zhaopuming/auto-lang/blob/master/parity/docs/parity-dashboard.html)，覆盖七个核心真实库（另有 consumer-mode 用例，见仪表盘）。

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

**L1 —— 当前已三向验证（141 个测试用例）：**

| 库 | 用例数 | 验证点 |
|---------|-------|-------------------|
| regex | 45/45 | 模式匹配、回溯 |
| cli_app | 32/32 | 纯 std 文本处理（wc 风格） |
| string_utils | 22/22 | 跨模块字符串操作 |
| trait_advanced | 18/18 | spec/trait：默认方法、关联类型、有界泛型 |
| tokio | 13/13 | async/await 串行组合 |
| generators | 6/6 | yield 生成器、惰性序列 |
| http_client_sync | 5/5 | 同步 HTTP（mock server） |

完整矩阵和每库详情见 [parity 仪表盘](https://github.com/zhaopuming/auto-lang/blob/master/parity/docs/parity-dashboard.html)。

**诚实的边界（L3 —— 路线图，尚未验证）：**

此前列出的全部 trait 分歧均已修复并三向验证（关联类型、返回值的默认方法、泛型 spec 实现、有界泛型函数、http_client_sync 测试框架）。仍开放的项目在 [known-divergences](https://github.com/zhaopuming/auto-lang/blob/master/parity/docs/known-divergences.md) 中公开记录，不隐瞒：

- **sha2 / rusqlite / reqwest parity 库** —— 已规划，尚未验证。
- **serde_json / url / base64 三向运行** —— 近期回归于 a2r 字符串参数借用（编译阶段）；交付时曾全部通过，正在修复 —— 见 known-divergences。
- **生成器惰性链**（`~Iter` 上的 range → map → filter）—— Auto 尚无此语法，语言路线图项。
- **tokio spawn/join 与 mpsc channel** —— 已验证子集覆盖串行 future 组合；并行 spawn 待 VM 支持。

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
