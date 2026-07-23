# 模板求值（eval_template 与数据注入）

## 范围

`AutoInterpreter::eval_template` / `merge_atom` / `with_fstr_note`（interpreter/mod.rs）。
主要消费方：auto-gen（`crates/auto-gen/src/lib.rs`、`template.rs`、`bin/autogen.rs`）、
auto-man（`crates/auto-man/src/asset.rs`）。

## Flip 变换机制（mod.rs:124-151）

`eval_template(prelude, template)` 不引入任何模板专用 VM 能力，而是把模板**源码级
变换**成普通 Auto 代码再走 `eval`：

1. 输出缓冲：生成 `var __out__ = ""`。
2. 逐行处理模板：
   - 以 `{fstr_note} `（默认 `$ `，可用 `with_fstr_note` 改）开头的行 → 剥前缀，
     原样作为 Auto 代码行（控制流、赋值等）；
   - 其余行 → 转义 `\` 与 `"` 后包成 f-string 拼接：
     `__out__ = __out__ + f"<行内容>\n"`（行内 `$var` 插值由 Auto f-string 机制完成）。
3. 末尾追加 `__out__`，使结果提取协议能拿到拼接出的字符串。
4. `prelude`（可选）原样前置，用于在同作用域注入变量定义；若非空且不以换行结尾
   自动补 `\n`。

这与 plan-075 的决策同源：模式差异在编译/变换层吸收，VM 保持 mode-agnostic（ADR-03）。

## 数据注入（merge_atom，mod.rs:198-224）

`merge_atom(&Atom)` 把数据展平为 `VmInterpreter.globals` 侧表条目：

- `Atom::Node` / `Atom::Obj`：每个属性/键（取 `key.name()`）→ 同名全局；
- `Atom::Array`：元素 → `item_0`、`item_1`……编号全局；
- `Atom::Empty`：无操作。

**已知限制**：注入值只进侧表，不进入 VM 执行环境（见 design/vm-backed-interpreter.md
状态管理节）——模板代码读不到这些"全局"。要让数据对模板可见，当前可行路径是
`prelude` 字符串（生成 `let name = ...` 代码）。auto-gen 的实际用法以
`crates/auto-gen/src/lib.rs` 为准。

## 显式非目标

- **不做独立模板引擎**：无 AST 级模板节点、无专用 opcode；一切是源码变换 + 普通求值。
- **不覆盖 VM 侧 TemplateCodegen**：plan-075 在 vm/codegen 实现的 TEMPLATE 模式是
  另一条路径（`CompileMode` 体系），与本文件描述的 Flip 路径并存、互不调用。
- **不保证转义完备**：Flip 只转义 `\` 与 `"`，模板行内若含其他与 f-string 语法冲突的
  字符（如未配对的 `${`）需在调用方处理。

> 来源: crates/auto-lang/src/interpreter/mod.rs、crates/auto-gen/src/lib.rs、docs/plans/old/075-config-template-modes.md
