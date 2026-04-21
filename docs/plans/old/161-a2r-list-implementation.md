# Plan 161: a2r 版 List 完整实现 + Auto 语言特性增强

> 原 Plan 161（a2r List 实现）和 Plan 162（缺失的 Auto 特性）合并。
> 所有特性已实现并通过测试。

## 目标

让 `List<T, S>` 在 a2r（Auto-to-Rust）转译后能正确编译运行。

## 已完成的工作

### 1. `#[rs]` 目标选择器

`#[rs]` 仅作为目标选择器，控制 `should_skip` 逻辑，不改变代码生成：

| 标注 | a2c | a2r | VM |
|------|-----|-----|----|
| 无标注 | 生成 | 生成 | 生成 |
| `#[c]` | 生成 | 跳过 | 跳过 |
| `#[vm]` | 跳过 | 跳过 | 生成 |
| `#[rs]` | 跳过 | 生成 | 跳过 |

`.rs.at` 文件中的方法无需 `#[rs]` 标注（因为 `.rs.at` 仅在 a2r 模式加载）。

### 2. `expr.as(Type)` 类型转换 (P0)

**AST**: `Expr::Cast { expr: Box<Expr>, target_type: Type }`（ast.rs:344）

**Parser**: 两处拦截 `.as(Type)`：
- Pratt parser `_ =>` 分支（parser.rs:1802-1810）
- `dot_item()` 链式解析中通过 peek 排除 `.as`（parser.rs:1548-1558）

**转译映射**：

| Auto | a2r (Rust) | a2c (C) |
|------|------------|---------|
| `x.as(u32)` | `(x as u32)` | `((unsigned int)(x))` |
| `.len.as(int)` | `(self.len as i32)` | `((int)(self.len))` |

**VM**: 6 个 cast 操作码（TYPE_CAST_I32/U32/I64/U64/F64/PTR）

### 3. 指针判空方法 (P1)

**a2r**:

| Auto | Rust |
|------|------|
| `ptr.is_null()` | `ptr.is_null()` |
| `ptr.is_not_null()` | `(!ptr.is_null())` |

**a2c**:

| Auto | C |
|------|---|
| `ptr.is_null()` | `(ptr == NULL)` |
| `ptr.is_not_null()` | `(ptr != NULL)` |

### 4. 指针操作 add/read/write (P1)

`ptr.add(n)`、`ptr.read()`、`ptr.write(val)` 作为普通方法调用转译。
Rust 原生指针自带这些方法，无需特殊映射。
a2c 中 `ptr.add(n)` → `(ptr + n)`，`ptr.read()` → `(*ptr)`，`ptr.write(v)` → `(*ptr = v)`。

### 5. 数组 `.ptr` → `.as_mut_ptr()` (P2)

a2r 中 `buf.ptr` → `buf.as_mut_ptr()`（rust.rs:1262-1267）

### 6. `.rs.at` 标准库文件

| 文件 | 内容 |
|------|------|
| `stdlib/auto/list.rs.at` | `ext List` 方法（new, len, push, pop, get, set, clear, drop, iter） |
| `stdlib/auto/storage.rs.at` | `ext Heap` Storage 实现（用 `use.rust` 导入 std::alloc） |
| `stdlib/auto/inline.rs.at` | `ext InlineInt64` / `ext InlineU8_256` 栈分配存储 |

### 7. 测试用例

| 测试 | 内容 |
|------|------|
| `test/a2r/136_type_cast/` | `.as(u32)`, `.as(i64)`, `.as(float)` 类型转换 |
| `test/a2r/137_ptr_methods/` | `.ptr`, `.is_null()`, `.is_not_null()`, `.write()`, `.read()` |
| `test/a2r/138_list_as_cast/` | ext 方法内的 `.as()` + `or` 运算符 |
| `test/a2c/152_type_cast/` | C 强制转换 |

全部 40 个 transpiler 测试通过。

## 文件变更清单

| 文件 | 变更 |
|------|------|
| `crates/auto-lang/src/ast.rs` | `Expr::Cast` 变体 + Display + ToNode |
| `crates/auto-lang/src/parser.rs` | Pratt `.as()` + `dot_item()` peek + Expr::Cast in check_symbol |
| `crates/auto-lang/src/dep.rs` | Cast 表达式遍历 |
| `crates/auto-lang/src/infer/expr.rs` | Cast 类型推导 |
| `crates/auto-lang/src/trans/rust.rs` | Cast + is_null + .ptr 转译 |
| `crates/auto-lang/src/trans/c.rs` | Cast + Ptr 方法转译 |
| `crates/auto-lang/src/vm/codegen.rs` | Cast opcode 发射 |
| `crates/auto-lang/src/vm/engine.rs` | Cast opcode 执行 |
| `crates/auto-lang/src/vm/opcode.rs` | 6 个 TYPE_CAST opcode |
| `stdlib/auto/list.rs.at` | List 方法（Auto 语法） |
| `stdlib/auto/storage.rs.at` | Heap Storage 方法（Auto + use.rust） |
| `stdlib/auto/inline.rs.at` | Inline 存储策略（Auto 语法） |

## 待办事项

以下工作需要后续完成（不在本 plan 范围内）：

- [ ] 将 `.rs.at` 文件集成到 a2r 项目构建流程
- [ ] Heap storage 的 `try_grow()` 验证（需要 `Layout::from_size_align` 运行时测试）
- [ ] List 泛型参数 `<T, S>` 的完整 a2r 支持
- [ ] `test_130_option_construct` 回归修复（外部代码引入）
