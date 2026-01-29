# AutoLang Generic Constraints Design

> **Status**: Draft  
> **Created**: 2025-01-29  
> **Author**: Design Discussion  

## 概述

本文档描述 AutoLang 泛型约束语法的设计方案，重点是使用 `#[with(...)]` 标注来声明带约束的泛型参数。

## 设计原则

### 现有关键字语境

| 关键字 | 当前用途 | 语义 |
|--------|----------|------|
| `as` | `impl Foo as Bar` | 类型**实现**某个 spec（能力获取） |
| `is` | `type Dog is Animal` | 类型**继承**某个类型（is-a 关系） |
| `<T>` | `fn foo<T>(x T)` | 声明泛型参数 |

### 设计目标

1. **与现有语法兼容**: 保留 `fn foo<T>(...)` 简单泛型语法
2. **避免冒号**: AutoLang 不使用 `:` 作为类型标注（冒号用于 `key:value`）
3. **语义一致**: 使用 `as` 表示约束，与 `impl X as Y` 语义一致
4. **清晰分离**: 将泛型声明与函数签名分离

## 语法设计

### 简单泛型（无约束）

保持现有语法不变：

```auto
fn identity<T>(x T) T {
    return x
}

type List<T> {
    data []T
    len  int
}
```

### 带约束泛型

使用 `#[with(...)]` 标注：

```auto
#[with(I as Iter<T>, T, U)]
fn map(iter I, f T=>U) U {
    // ...
}

#[with(T as Clone)]
fn duplicate(x T) T {
    return x.clone()
}
```

### 语法规则

```
with_attr     := "#[with(" with_params ")]"
with_params   := with_param ("," with_param)*
with_param    := IDENT ("as" type_constraint)?
type_constraint := IDENT ("<" type_args ">")?
```

### 多约束

```auto
#[with(A as Clone, B as Debug)]
fn process(a A, b B)

#[with(T as Clone + Serialize)]  // 可选：多约束语法（未来扩展）
fn save(data T)
```

## 对比分析

### 方案对比

| 方案 | 示例 | 优点 | 缺点 |
|------|------|------|------|
| `T: Spec` | `fn map<I: Iter<T>>(...)` | 简洁 | 使用冒号，与 Auto 风格冲突 |
| `where I as Iter` | `fn map(...) where I as Iter` | 明确 | 较啰嗦，位于函数后面 |
| **`#[with(...)]`** | `#[with(I as Iter<T>)]` | 清晰、可扩展 | 需要额外行 |

### 选择 `#[with(...)]` 的理由

1. **与现有标注系统一致**: 已有 `#[c, vm]` 等标注基础设施
2. **函数签名干净**: 泛型约束不干扰参数列表
3. **可扩展性**: 未来可添加更多标注类型
4. **明确的语义**: `with` 表示"带有这些类型参数"

## 完整示例

### 基本用法

```auto
// 简单泛型：保持现有语法
fn identity<T>(x T) T {
    return x
}

// 带约束泛型：使用 #[with(...)]
#[with(I as Iter<T>, T, U)]
fn map(iter I, f T=>U) MapIter<I, T, U> {
    return MapIter { iter: iter, f: f }
}
```

### 类型定义

```auto
#[with(T, S as Storage<T>)]
type List {
    data S
    len  int
    cap  int
}
```

### 与其他标注组合

```auto
#[c, vm]
#[with(T)]
fn push(list *List<T>, elem T) {
    // 平台特定实现
}

#[with(I as Iter<T>, T)]
#[inline]
fn count(iter I) uint {
    // 内联优化的计数函数
}
```

### Spec 方法

```auto
spec Iter<T> {
    fn next() May<T>
}

spec Iterable<T> {
    type IterT impl Iter<T>
    fn iter() .IterT
    
    // 默认实现使用 #[with]
    #[with(U)]
    fn map(f T=>U) MapIter<.IterT, T, U> {
        return .iter().map(f)
    }
}
```

## 语法组合规则

### 兼容性规则

1. **无约束时**: 使用现有 `<T>` 语法
2. **有约束时**: 必须使用 `#[with(T as Spec)]`
3. **混合使用**: `#[with(...)]` 中的声明**覆盖** `<T>` 中的同名参数

```auto
// 等价写法（无约束时）
fn foo<T>(x T)
#[with(T)]
fn foo(x T)

// 带约束时必须用 #[with]
#[with(T as Clone)]
fn bar<T>(x T)  // <T> 可省略，因为 #[with] 已声明

// 推荐写法：有约束时省略 <T>
#[with(T as Clone)]
fn bar(x T)
```

### 错误处理

```auto
// 错误：约束必须使用 #[with]
fn bad<T as Clone>(x T)  // ❌ 语法错误

// 正确：使用 #[with]
#[with(T as Clone)]
fn good(x T)  // ✅
```

## 实现要点

### AST 结构

现有 `TypeParam` 已支持约束：

```rust
// crates/auto-lang/src/ast/types.rs
pub struct TypeParam {
    pub name: Name,
    pub constraint: Option<Box<Type>>,  // 已存在！
}
```

### 解析器修改

1. **识别 `#[with(...)]`**: 在属性解析阶段处理
2. **解析 `T as Spec` 语法**: 识别 `as` 关键字
3. **填充 `constraint` 字段**: 将解析的约束存入 `TypeParam`

### 关键代码路径

```
parser.rs
├── parse_attrs()         // 识别 #[with(...)]
├── parse_with_attr()     // 新增：解析 with 参数
│   └── parse_with_param() // 新增：解析 T as Spec
└── parse_fn()            // 合并 with 参数与 <T> 参数
```

## 与 Plan 051 的关系

Plan 051 (Auto Flow) 中的迭代器方法依赖泛型约束：

```auto
// Plan 051 Phase 4-8 需要的语法
#[with(I as Iter<T>, T, U)]
fn map(iter I, f T=>U) MapIter<I, T, U>

#[with(I as Iter<T>, T, B)]
fn reduce(iter I, init B, f (B, T)=>B) B
```

泛型约束的实现是 Plan 051 完整功能的前置条件。

## 相关计划

- **Plan 051**: Auto Flow（依赖泛型约束）
- **Plan 057**: Generic Specs（泛型 spec 系统）
- **Plan 059**: Generic Type Fields（泛型类型字段）
- **Plan 060**: Closure Syntax（闭包语法）

## 总结

`#[with(...)]` 语法设计具有以下优势：

| 特性 | 说明 |
|------|------|
| **兼容性** | 保留现有 `<T>` 语法 |
| **一致性** | 与现有 `#[...]` 标注风格一致 |
| **可读性** | 函数签名干净，约束独立声明 |
| **语义清晰** | `as` 表示"实现"关系 |
| **可扩展** | 易于添加新的约束语法 |
