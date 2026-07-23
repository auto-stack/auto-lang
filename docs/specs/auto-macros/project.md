# auto-macros

> **Status**: active
> 路径：`crates/auto-macros`  | 技术栈：Rust proc-macro（inventory / syn / quote）

proc-macro 辅助包：基于 inventory 的注册宏，供 auto-lang 收集 native 函数等注册项。

## 目标与范围

- 提供 inventory 注册辅助宏（lib.rs），让注册项随链接自动收集。
- rust_fn_draft：Rust 函数声明解析的草稿实现。
- 不做：不承载业务逻辑；宏数量保持最小，语法解析复用 syn。

## 模块清单

| 模块 | 职责 | 状态 |
|---|---|---|
| lib | inventory 注册宏入口 | active |
| rust_fn_draft | Rust 函数声明解析（草稿） | draft |
