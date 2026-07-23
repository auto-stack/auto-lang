# 类型表示：Type 枚举、修饰符、字符串、enum

## 范围

`ast/types.rs:Type` 的类型表示层：后缀修饰符体系、字符串三类型、统一 enum、union。
不含推断/统一算法（见 type-inference.md）与存储（见 typestore.md）。

## 原则

- 后缀修饰符：`T[]` 动态数组、`T[N]` 定长数组、`T*` 指针、`T&` 引用、`T?` 可选，
  全部附在基类型之后，与 C 声明顺序一致；多维 `int[3][10]` 左到右、外到内（ADR-01）。
- `Type` 枚举约 35 个变体（ast/types.rs:24），基础类型之外含 `List`/`Map`/`Slice`、
  `Array`/`RuntimeArray`（Plan 052）、`Fn`（Plan 060）、`Option`/`Result`（Plan 120）、
  `Tuple`（Plan 200）、`Handle`（Plan 121 task 句柄）、`Rust`（Plan 190 use.rust）、
  `Linear`（move-only）、`Storage`（Plan 055 存储策略）。`May` 已移除（ast/types.rs:51）。

## 细节

### 字符串三类型

| Auto 关键字 | 内部变体 | Rust 等价 | 语义 |
|---|---|---|---|
| `str` | `StrSlice` | `&str` | 借用切片；变量与函数参数的默认 |
| `Str` | `StrOwned` | `String` | 拥有型堆字符串；容器内必须用它 |
| 字面量 `"x"` | `StrFixed(N)` / `CStrLit` | `"x"` | 非命名类型，不能出现在类型标注里 |

不变量：转换单向 `StrLit → StrSlice → StrOwned`；反向必须显式 `.to(str)`。
**容器元素禁止 `str`**，省略类型参数时容器默认 `Str`（`List.new()` → `List<Str>`）。
a2r 映射按上下文分：`str` 在参数位映 `&str`，在变量/字段/容器/返回位映 `String`。

### 统一 enum 三形态

`ast/enums.rs:EnumKind` 判别（ADR-06）：

1. **Scalar**：纯状态，可选 repr 类型与显式值（默认 u8 自增）。
2. **Homogeneous**：`enum Vertex Point { ... }`，全体变体共享一个载荷类型，
   支持免模式匹配的 O(1) 字段直访。
3. **Heterogeneous**：ADT/和类型，变体可带不同载荷（含元组与结构体载荷）。

内建方法：`.tag()`（判别整数）、`.name()`（变体名）。`tag` 关键字废弃。
`TypeStore` 支持按变体名反查（`find_enum_variant_by_name`，Plan 127）。

### union

raw `union` 保留 C 式内存重叠语义（`ast/union.rs`），用于底层内存重解释；
与统一 enum 在类型检查中的交互仍是 design/02 的 Open Question。

## 显式非目标

- 不做 union types（`A | B`）——design/02 Planned，未实现。
- 字符串字面量不是一等类型，不能用于类型标注。
- 多约束泛型（`T as A + B`）不支持，每参数单约束。

> 来源: docs/design/02-type-system.md（§Type Modifiers / §String / §Unified Enum / §Union）、crates/auto-lang/src/ast/types.rs、ast/enums.rs、types.rs
