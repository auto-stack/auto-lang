# auto-val

> **Status**: active
> 路径：`crates/auto-val`  | 技术栈：Rust（ecow / indexmap）

底层值/AST 节点数据结构（Value/Node/Obj/Pair），整个 workspace 最底层的基础库，不依赖其他 auto crate。

## 目标与范围

- 定义 Auto 运行时表示：Value、AST Node、Obj/Pair 键值结构。
- 提供紧凑字符串与集合类型（AutoStr/StrSlice/OwnedStr/CStr、Array/Linear/Kids）。
- 提供路径（AutoPath）、元信息、emit/to_value 等基础工具。
- 不做：不实现解析/求值；不依赖上层 crate，保持零 auto 内部依赖。

## 模块清单

| 模块 | 职责 | 状态 |
|---|---|---|
| value / nano_value | 值表示（Value 枚举及紧凑变体） | active |
| node | AST 节点结构 | active |
| obj / pair | 对象与键值对结构 | active |
| string / str_slice / owned_str / cstr | 字符串类型族（AutoStr 等） | active |
| array / linear / kids | 集合/子节点容器 | active |
| meta / path / shared / types / to_value / emit | 元信息、路径、共享类型与转换输出工具 | active |
