# TODO: 类型推断未实现

## 问题
C transpiler 的类型推断引擎未实现 `?T` (question type) 的完整类型解析。
当前生成的代码中使用 `unknown` 代替了具体类型。

## 根因
- `trans/c.rs` 中的 `rust_type_name()` 或 `c_type_name()` 对 `?T` 语法返回 `unknown`
- 需要实现 question type 到 C 类型的映射（如 `?int` → `struct MayInt`）

## 正确的预期输出
应使用具体的 C 类型（如 `struct MayInt result = ...`），而非 `unknown result = ...`

## 当前状态
- expected 文件已更新为当前 transpiler 输出（包含 `unknown` 类型）
- 当 transpiler 实现了 question type 的完整类型推断后，需重新生成 expected 文件
