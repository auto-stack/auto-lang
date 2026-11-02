# 旧 AVM 不可变赋值 typed 诊断（PLAN-920，2026-11-03）

> **Status**: implemented
> 范围：旧 AutoVM codegen 的 `let` 重赋值拒绝从白文本 `Msg` 迁到既有 typed
> `NameError::ImmutableAssignment` 事实通道。不改 let/var 语义、不重构 VM、
> 不扩公用错误 enum（auto-diagnostics 既有类型零改动）。

## 生产事实

- 三处拒绝分支全部发同一 typed 事实（`crates/auto-lang-core/src/vm/codegen/`）：
  `expr_ops.rs` 裸赋值表达式位（`Stmt::Expr(Bina(Asn))`，B18 命中位）、
  `expr_ops.rs` 复合赋值位（`+=` 等五算符）、`stmt_emit.rs` Store 语句防御位。
  第三位当前语法生产不可达（仅 `StoreKind::Field` 过 `!is_new_declaration`，
  而 Field 语句位由 TypeDecl 路径编译）——按同族对齐改造并以 AST 级证人锁
  行为，不伪造源级用例。
- 错误身份 = `auto_diagnostics::error::NameError::ImmutableAssignment{name, span}`
  （miette code `auto_name_E0203`，Display "cannot assign to immutable variable"，
  help/label 既有文本）；trans/rust A2R 前端（walk.rs）为同族先例。
- span 精度（D-01 裁定）：旧 AST 无节点级 span（`Name = AutoStr`）。合法上下文
  锚点 = Plan 199 行号 × Plan 057 源文本 → 当前语句所在行的字节范围（行粒度，
  不含换行；fn 体与顶层脚本均有行号，实证 (34,9)/(14,5)）。既无行号也无源文本
  的路径（如直接调 Codegen API）退化为 `(0,1)`，与 trans/rust `fn_span(None)`
  同一约定。禁止猜指向其他变量的 span、禁止以全文解析重建诊断。
- 传播：codegen `?` 链直达 `execute_autovm_with_deadline_and_input`；staged
  （`run_with_capture_and_path_staged`）与非 staged（`run_with_capture_and_path`
  /`run_autovm_capture`）同型——非测试入口特供。

## 父侧判面（严格准入不变）

`test_cases::classify_staged` 按**错误类型**判面：`AutoError::Name` →
Language/`type`/`auto_name_E0203`；裸 `Msg`（含与 B18 原文本完全相同的伪造文本）、
IO、panic → Infrastructure，永不满足语言负例。run-stage typed trap 走独立
`Run` 通道，不与编译期 `Type` 串道。adapter 不得按 case id/expected/消息子串
补 stage/code。

## 验证面

- 单元回归锁：`crates/auto-lang-core/src/vm/immutable_assign_920.rs`（10 例：
  staged/非staged 同型、复合、Store 防御位 AST 证人、合法近邻不误拒、span
  精度、伪造 Msg/异族 typed/run-trap 三反控、负→合法修复→负循环）。
- 语料套件：`ace-m1-immutable-avm-920`（固定组 `case-ace-m1-immutable-avm-920`）
  5 检查——B18 原源、复合、顶层脚本三负例 + NB18/同名内层 var 遮蔽两合法近邻。
- M1 B18 avm 格位据此闭为 `ok:P920`（见 [ace-m1-conformance.md](ace-m1-conformance.md)
  PLAN-920 节与 [ace-m1-gap-audit.md](ace-m1-gap-audit.md) F-INFRA-3 关闭记录）。

## 已知边界

- 旧 VM `var_mutability` 为扁平表（非按作用域键控）：同名遮蔽后可变性的陈旧
  状态是既有 let/var 槽语义（如内层 `var x` 复用外层槽），本通道不改其语义，
  仅保证合法写不被误拒。
- 本修复不改变 M2 替换与 disable 计划（Design43 §20）：旧 VM 有限修复仅服务
  原 M1 证据闭合。
