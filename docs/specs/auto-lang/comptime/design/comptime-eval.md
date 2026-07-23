# 编译期求值语义

## 范围

`CTEE::eval_expr` 的求值路径、内置常量、真值/相等判定、`compile_error()` 内省、值与 AST 的相互转换，
以及 `ComptimeError` 错误码。管线与语法分别见 `ctee-pipeline.md`、`hash-syntax.md`。

## 求值双路径

`eval_expr`（transformer.rs:220）按表达式形态分流：

1. **直评路径**（性能考虑，代码注释自述）：`Int`/`I64`/`Bool`/`Str`/`Nil` 字面量直接转 `Value`；
   裸 `Ident` 先查 builtins 表，命中即返回。
2. **VM 路径**：其余一切表达式 `format!("{}\n", expr)` 转成源码字符串，交内嵌 `VmInterpreter::run` 执行。
   由此 comptime 自动获得全语言能力，但有三条边界：
   - VM 环境**未注入 builtins**（plan-095 草稿中的 `set_global` 循环未进最终实现），
     含 `OS`/`ARCH` 的复合条件不命中 builtins 表；
   - 求值经字符串往返，错误 span 丢失（`ComptimeError` 多为零 span）；
   - 每次求值重 parse，是慢路径。

## 内置常量

`init_builtins` 注入四项（transformer.rs:57）：

| 名称 | 值 |
|---|---|
| `OS` | `std::env::consts::OS`（或 `with_target` 指定） |
| `ARCH` | `std::env::consts::ARCH`（或 `with_target` 指定） |
| `DEBUG` | `cfg!(debug_assertions)`——编译器自身的构建模式 |
| `VERSION` | 硬编码 `"0.1.0"` |

`set_builtin` 允许外部注入自定义常量；`#for` 复用该表传循环变量（迭代结束后移除）。

## 判定函数

- `is_truthy`：`Bool` 取自身；`Int` 非零；`Str` 非空；`Nil` 为假；其余一律为真。
- `values_equal`：仅 `Int`/`Bool`/`Str`/`Nil` 四种同型比较，跨型一律不等——`#is` 的匹配能力受此限制。

## `compile_error()` 内省

`eval_expr` 开头特判：表达式为对 `compile_error` 的调用时，取首个字符串参数（缺省 `"compile error"`）
抛 `ComptimeError::CompileError`（E0401）。仅当调用**直接**处于 comptime 条件/目标位置时触发，
藏在普通代码路径里的 `compile_error` 不归此管。

## 值 ↔ AST 转换的边界

- `value_to_iter`：`Int(n)` 展开为 `0..n`；`Array` 返回空 Vec（注释 "needs proper array iteration"）；
  其余报 `SyntaxError::Generic`。
- `value_to_expr`：覆盖 `Nil/Int/Bool/Str/Float`；数组、对象等一律退化为 `Expr::Nil`——
  设计文档的 CRC 表等"编译期算出数组再回写"用例当前不成立。
- `Expr::I64` 直评时截断为 i32（注释 "Truncate for now"）。

## 错误类型

`ComptimeError`（error.rs:1148，miette 诊断）：

| 码 | 变体 | 触发 |
|---|---|---|
| E0401 | `CompileError` | `compile_error()` 被调用（唯一有实际抛出的变体） |
| E0402 | `NonDeterministic` | 类型已定义，无抛出点（沙箱未实现） |
| E0403 | `ResourceLimit` | 类型已定义，无抛出点（限额未实现） |
| E0404 | `UndefinedConstant` | 类型已定义，help 文案列出 OS/ARCH/DEBUG/VERSION |

plan-095 声称 "E0401-E0406"，实际枚举只到 E0404 区间内的四个变体（plan 叙述与代码的分歧）。

## 显式非目标

- 不模拟目标平台数据宽度：VM 即主机语义，`usize` 等随主机（09-compiler.md Open Questions 挂起）。
- 不保证确定性：无 I/O/随机/时间禁用机制（设计见 `determinism-sandbox.md`）。
- 不做编译期函数调用图分析：`#{ }` 内调用的函数能否求值，完全取决于 VmInterpreter 能否执行。

> 来源: crates/auto-lang/src/comptime/transformer.rs、crates/auto-lang/src/error.rs、docs/plans/old/095-compile-time-execution-engine.md、docs/design/09-compiler.md
