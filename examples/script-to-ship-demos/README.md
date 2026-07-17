# Script-to-Ship Demos

面向用户的"Auto 即 Rust 脚本"演示集合。每个 demo 演示 **Dev（VM 脚本秒级迭代）→ Ship（a2r 转 Rust 发布）** 工作流。

## 用法（每个 demo 通用）

### Dev 模式：VM 脚本直接跑（秒级，跳过编译）

```bash
auto main.at
```

### Ship 模式：a2r 转译成 Rust 发布

```bash
auto trans --path main.at rust   # 生成 main.a2r.rs
```

转译后的 Rust 代码链 `a2r-std`，可 `cargo build` 出原生性能 + 内存安全的发布版。

### 行为一致性

每个 demo 对应的 parity 用例（`parity/libs/<name>/`）已通过三向验证：
AutoVM、a2r 转译 Rust、原生 Rust 输出 100% 一致。详见 [parity 仪表盘](../../parity/docs/parity-dashboard.html)。

## Demos

| Demo | 领域 | 对标 Rust 生态 | parity 库 |
|------|------|---------------|-----------|
| [`serde_json-demo`](serde_json-demo/) | JSON 序列化/解析 | serde_json | `parity/libs/serde_json/` (L1 ✓ 56/56) |
| [`regex-demo`](regex-demo/) | 正则匹配 | regex | `parity/libs/regex/` (L1 ✓ 45/45) |
| [`cli-demo`](cli-demo/) | 文本统计 (wc 风格) | std 纯 Rust 输出 | `parity/libs/cli_app/` (L1 ✓ 32/32) |

## 三段式叙事

每个 demo 体现 Auto 作为 Rust 脚本层的三段价值：

1. **Dev**：AI 生成 Auto 脚本，`auto main.at` 秒级验证，跳过 Rust 编译等待。
2. **Ship**：`a2r` 把同一份代码转译成 Rust 短代码，拿原生性能 + 内存安全。
3. **Bridge**：编译器保证两种模式行为一致（parity 仪表盘为证）。
