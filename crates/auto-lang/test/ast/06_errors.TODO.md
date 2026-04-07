# TODO: test_06_errors "Did you mean" 拼写建议功能

## 状态

拼写建议（"Did you mean '...'?"）功能**未启用**。测试期望已更新为不含建议的当前行为。

## 问题

当变量名拼写错误时（如 `myVaraible` 应为 `myVariable`），错误消息应该提供拼写建议：

```
// 期望（未来）:
help: Variable 'myVaraible' is not defined in this scope. Did you mean 'myVariable'?

// 当前:
help: Variable 'myVaraible' is not defined in this scope
```

## 原因

`error.rs` 中已有 `find_best_match()` 和 Levenshtein 距离算法，`NameError::UndefinedVariable` 结构体也有 `suggested` 字段。但在 evaluator 报错时未填充该字段。

具体位置：
- `error.rs:777` — `UndefinedVariable` 的 `suggested` 字段处理（已实现）
- `error.rs:82` — `find_best_match()` 函数（已实现）
- 调用方（evaluator/parser）未传入 `suggested` 参数

## 修复方向

在 parser 或 evaluator 遇到 `UndefinedVariable` 错误时，收集当前作用域的所有变量名，调用 `find_best_match()` 填充 `suggested` 字段。

## 受影响测试

- `test/ast/06_errors.test.md` — 4 个测试 case 的 "Did you mean" 部分已移除
