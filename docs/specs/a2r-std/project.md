# a2r-std

> **Status**: active
> 路径：`crates/a2r-std`  | 技术栈：Rust（serde_json / ureq）

a2r（Auto→Rust）转译产物的运行时标准库：让转译出的 Rust 代码有与 Auto 语义对齐的 std 实现。

## 目标与范围

- 提供转译代码依赖的运行时类型与函数：list、hashmap、string_builder、str、json、http、fs、env、math、time。
- 行为对齐 AutoVM 后端的标准库语义（parity 检查的对照对象之一）。
- 不做：不实现转译器本身（在 auto-lang）；只覆盖转译产物实际用到的 std 子集。

## 模块清单

| 模块 | 职责 | 状态 |
|---|---|---|
| list / hashmap / string_builder | 集合与字符串构建 | active |
| str | 字符串函数 | active |
| json | JSON 读写（serde_json） | active |
| http | HTTP 客户端（ureq） | active |
| fs / env / math / time | 文件系统、环境、数学、时间 | active |
