# auto-macros

> **Status**: active
> 路径：`crates/auto-macros`  | 技术栈：Rust proc-macro（inventory / syn / quote）

proc-macro 辅助包：基于 inventory 的注册宏，供 auto-lang 收集 native 函数等注册项。

## 目标与范围

- 提供 inventory 注册辅助宏（lib.rs），让注册项随链接自动收集。
- rust_fn_draft：Rust 函数声明解析的草稿实现。
- 不做：不承载业务逻辑；宏数量保持最小，语法解析复用 syn。

## value! 宏的构造与求值契约（PLAN-1014 SD-04）

- **装箱契约**：插值对象字面量（`value!{k: #{v}, ...}`）展开为
  `auto_val::Value::obj(Obj)`——Plan 566 Phase B 起 `Value::Obj` 变体持有
  `Box<Obj>`，一切构造必须走 `Value::obj`/`Obj::into` helper，不得直接
  `Value::Obj(obj)`（曾因宏展开滞后于装箱契约产生 4×E0308 长红）。
- **求值语义**：`#{var}` 单标识符经 `ToAutoValue` 插值；`true/false`/字符串/
  数字为字面量；嵌套结构与复杂表达式走 `AtomReader` 解析；表达式单次求值。
  原测试的 ignore 条件（未支持语法）不被顺手 unignore 扩解析面。

## 模块清单

| 模块 | 职责 | 状态 |
|---|---|---|
| lib | inventory 注册宏入口 + value!/atom!/node! 字面量宏 | active |
| rust_fn_draft | Rust 函数声明解析（草稿） | draft |
