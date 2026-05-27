# Auto Language Semantic Specification

Auto 的两个执行后端（AutoVM 和 a2r）必须对同一份代码产生完全一致的运行结果。本目录是两者的**单一真相源 (Single Source of Truth)**。

## 规范索引

| 文件 | 覆盖范围 | 状态 |
|------|---------|------|
| [01-arithmetic.md](01-arithmetic.md) | 整数/浮点算术、溢出、除法、取模 | Draft |
| [02-strings.md](02-strings.md) | 字符串拼接、f-string、比较、方法 | Draft |
| [03-collections.md](03-collections.md) | 数组创建/索引、List/Map 操作 | Draft |
| [04-control-flow.md](04-control-flow.md) | if/for/loop/break/continue/is | Draft |
| [05-functions.md](05-functions.md) | 函数调用、闭包、递归 | Planned |
| [06-types.md](06-types.md) | type(struct)/enum/tag 定义与构造 | Planned |
| [07-pattern-matching.md](07-pattern-matching.md) | is 表达式完整语义 | Planned |
| [08-methods.md](08-methods.md) | ext 方法、dot notation、方法链 | Planned |
| [09-error-handling.md](09-error-handling.md) | Option/Result/panic/? 操作符 | Planned |
| [10-builtins.md](10-builtins.md) | print/len/str/type_of 等内置函数 | Draft |

## 规范格式

每个操作用统一格式定义：

```
## 运算符 `+`

### `int + int` → `int`
- **语义**: 数学加法
- **溢出**: wrapping（与 Rust i32::wrapping_add 一致）
- **AutoVM**: opcode::ADD → wrapping_add
- **a2r**: 直接转译为 `a + b`（Rust release wrapping，debug panic）
- **示例**: `1 + 2` → `3`; `2147483647 + 1` → `-2147483648`
```

## 与测试的关系

- **Layer 2 对偶测试** (`test/a2r/conformance/`) 验证规范与实现一致
- **Layer 4 差分测试** (`test_util::program_generator`) 随机生成程序，验证 AutoVM 不 crash
- 发现不一致时以测试结果为准，更新规范
- 新特性开发流程：先写规范 → 对偶测试 → 实现

## Spec-Driven Development 流程 (Phase 5)

新语言特性必须按以下顺序开发：

```
1. 编写语义规范 (docs/spec/XX-feature.md)
   ↓ 定义操作语义、AutoVM opcode、a2r 映射、边界行为
2. 编写对偶测试 (test/a2r/conformance/NNN_name/)
   ↓ input.at + expected_output.txt
3. 实现 AutoVM 支持
   ↓ 添加 opcode、codegen、native shim
4. 运行对偶测试验证 AutoVM
   ↓ cargo test -p auto-lang -- conformance_NNN
5. 实现 a2r 支持
   ↓ 修改 trans/rust.rs
6. 运行差分测试覆盖长尾
   ↓ cargo test -p auto-lang -- conformance_differential
7. 更新特性覆盖率矩阵
```

### Checklist

- [ ] 语义规范覆盖所有新操作
- [ ] 对偶测试包含正常路径和边界条件
- [ ] AutoVM 和 a2r 输出完全一致
- [ ] 已知语义间隙记录在规范的"已知语义间隙"章节

### 差分测试

```bash
# 运行 50 个随机程序验证稳定性
cargo test -p auto-lang -- conformance_differential_stability

# 验证同一 seed 产生相同输出
cargo test -p auto-lang -- conformance_differential_reproducibility
```

## 相关计划

- [Plan 266](../plans/266-vm-a2r-conformance.md) — VM ↔ a2r 语义一致性
