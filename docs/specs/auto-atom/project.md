# auto-atom

> **Status**: active
> 路径：`crates/auto-atom`  | 技术栈：Rust（auto-val；serde 可选）

静态 Atom 数据结构与解析器：把 Auto 源码解析为静态 Atom 树，供 auto-gen 等工具消费。

## 目标与范围

- 定义 Atom 数据结构（静态、可序列化可选）。
- 提供 Atom parser：从源码文本产出 Atom 树。
- 不做：不做完整编译前端（语义分析在 auto-lang）；仅依赖 auto-val。

## 模块清单

| 模块 | 职责 | 状态 |
|---|---|---|
| atom | Atom 数据结构定义 | active |
| parser | Atom 解析器 | active |
| error | 错误类型（thiserror） | active |
