# 错误类型：Option / Result 与错误码

## 范围

`?T`/`!T` 错误类型三层体系、`May<T>` 的废弃决策、编译器错误码分类与诊断协议。
不含所有权（见 ownership.md）。

## 原则

非正常程序状态分三层（ADR-03，plan-120）：

| 层 | 语法 | 名称 | 心智模型 | 处理 |
|---|---|---|---|---|
| L1 | `?T` | Option | 空盒子（数据缺失） | 温和：预期内缺失 |
| L2 | `!T` | Result | 坏盒子（操作失败） | 严格：必须处理 |
| L3 | `T`（裸） | Panic | 炸弹（不可恢复） | 紧急：逻辑错误 |

AST 侧：`Type::Option(Box<Type>)` / `Type::Result(Box<Type>)`（ast/types.rs:61-62）；
`May` 变体已删除，注释保留"用 stdlib `tag May<T>`"的退路（ast/types.rs:51）。

## 细节

### 传播操作符（设计完成，未进 parser）

助记：`?` 管数据、`!` 管错误、`!!` 管 panic；无参=传播，有参=恢复。

| 对象 | 传播 | 恢复 |
|---|---|---|
| `?T` | `val.?`（返回 None） | `val.?(default)` |
| `!T` | `val.!`（返回 Err） | `val.!(default)` |
| `T` | `expr.!!`（立即 panic） | `expr.!!(default)` |

`#[nopanic]`（未实现）：传染式约束——只能调其他 nopanic 函数；禁除法/assert/裸 `.!!`；
`.!!(default)` 在 Debug 下 fail-fast、Release 下捕获记 FATAL 返回默认值。

### 错误码分类（error.rs，以代码为准）

| 类别 | 代码区段（实际） | 例 |
|---|---|---|
| Lexer | auto_lexer_E0001-E0005 | 非法字符、未终止字符串 |
| Syntax | auto_syntax_E0001-E0007（+E0099 多错聚合） | UnexpectedToken、赋值给 let |
| Type | auto_type_E0101-E0106、**E0201-E0204** | TypeMismatch、FieldMismatch、ConstraintViolation、CannotModifyViewParam |
| Name | auto_name_E0201-E0204 | UndefinedVariable、DuplicateDefinition |
| Runtime | auto_runtime_E0301-E0306 | DivisionByZero、IndexOutOfBounds |

分歧：design/03 的表格称 TypeError 为 E0101-E0105，实际到 E0106 且占用 E0201-E0204
（`auto_type_E020x` 与 `auto_name_E020x` 前缀不同故不冲突，但区段编号重叠）。

诊断协议：miette + thiserror；每个错误带错误码、行列、源码片段 + 标注 span、help 文本；
`AutoError` 手动实现 `Diagnostic` 以正确委托 `source_code()`/`labels()`；
多错经 `AutoError::MultipleErrors`（auto_syntax_E0099）聚合。`AutoResult<T>` 为全编译器返回型。

### 未落地的设计

fallible main（`fn main() !` + 入口劫持，OS 目标 exit(1)、嵌入式目标死循环等看门狗）；
Auto Mode（`auto fn` / `auto {}` / `#auto`，3A 协议经 AST 重写降级为标准代码，无运行时支持）。

## 显式非目标

- 不做异常机制；L3 panic 不可捕获（除 `.!!(default)` 局部救援）。
- `May<T>` 不作为内建类型复活；若需要，走 stdlib 泛型 tag 类型。
- 传播操作符、nopanic、fallible main、Auto Mode 当前均为"仅设计"。

> 来源: docs/design/03-error-handling.md、crates/auto-lang/src/error.rs、ast/types.rs:51-62
