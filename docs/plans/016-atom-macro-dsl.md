# Atom 宏 DSL 实现计划

**创建日期**: 2025-01-11
**状态**: 🚧 进行中
**预计代码量**: ~400 LOC (宏定义) + ~200 LOC (测试)

## 概述

为 AutoLang 的 Atom/Node/Array/Obj 类型添加**声明式宏 DSL**，提供类似 `serde_json::json!` 的简洁语法，大幅简化树状结构的构建过程。

## 背景

已完成工作：
- ✅ **阶段 1**: 链式方法扩展（~305 LOC，33 测试）
- ✅ **阶段 2**: Builder 模式（~430 LOC，44 测试）

**阶段 3 目标**：宏 DSL 提供最声明式的语法。

## 设计目标

### 1. 核心宏

- `node!` - Node 构造宏
- `atom!` - Atom 构造宏
- `atoms!` - 自动类型推断的简化宏

### 2. 语法特性

- **简洁性**: 最少的语法噪音
- **可读性**: 接近配置文件的自然格式
- **类型安全**: 编译期类型检查
- **零开销**: 宏展开后与手动构造相同

### 3. 设计权衡

**宏 DSL vs Builder**:

| 特性 | 宏 DSL | Builder |
|------|--------|---------|
| 语法简洁度 | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ |
| 条件构建 | ❌ | ✅ |
| 类型推断 | ✅ | ❌ |
| IDE 支持 | ⭐⭐ | ⭐⭐⭐⭐ |
| 错误信息 | ⭐⭐ | ⭐⭐⭐⭐ |
| 学习曲线 | ⭐⭐ | ⭐⭐⭐⭐ |

**建议**:
- 简单场景：使用宏 DSL
- 复杂场景（条件构建）：使用 Builder

## 宏设计

### 1. node! 宏

#### 语法变体

```rust
// 1. 简单节点
node!("config")

// 2. 带参数
node!("db"("my_db"))

// 3. 带属性
node!("config" {
    version: "1.0",
    debug: true,
})

// 4. 带子节点
node!("config" {
    database("database") {
        host: "localhost",
        port: 5432,
    },
    redis("redis") {
        host: "127.0.0.1",
        port: 6379,
    },
})

// 5. 混合属性和子节点
node!("root" {
    name: "test",
    child1("child1") { value: 1 },
    child2("child2") { value: 2 },
})

// 6. 带参数和属性
node!("db"("my_db") {
    host: "localhost",
    port: 5432,
})
```

#### 宏实现

```rust
#[macro_export]
macro_rules! node {
    // 简单节点: node!("name")
    ($name:expr) => {
        Node::new($name)
    };

    // 带参数: node!("name"("arg"))
    ($name:expr ( $arg:expr )) => {
        Node::new($name).with_arg($arg)
    };

    // 带多个参数: node!("name"("arg1", "arg2"))
    ($name:expr ( $($arg:expr),+ $(,)? )) => {
        {
            let mut node = Node::new($name);
            $(
                node.add_pos_arg_unified($arg);
            )+
            node
        }
    };

    // 带属性: node!("name" { key: value, ... })
    ($name:expr { $($key:ident : $value:expr),* $(,)? }) => {
        Node::new($name)
            $(
                .with_prop(stringify!($key), $value)
            )*
    };

    // 带参数和属性: node!("name"("arg") { key: value, ... })
    ($name:expr ( $arg:expr ) { $($key:ident : $value:expr),* $(,)? }) => {
        Node::new($name)
            .with_arg($arg)
            $(
                .with_prop(stringify!($key), $value)
            )*
    };

    // 带子节点（递归）
    // 需要使用 TT muncher 模式处理递归
}
```

**实现挑战**: 宏递归处理嵌套子节点。

**解决方案**: 使用 TT muncher 模式分阶段解析。

### 2. atom! 宏

#### 语法变体

```rust
// 1. 节点
atom!(node("config"))
atom!(node("config") { version: "1.0" })

// 2. 数组
atom!(array[1, 2, 3, 4, 5])
atom!(array["a", "b", "c"])

// 3. 对象
atom!(obj { name: "Alice", age: 30 })

// 4. 嵌套
atom!(node("config") {
    database("db") { host: "localhost" },
    data: array[1, 2, 3],
    meta: obj { version: "1.0" },
})
```

#### 宏实现

```rust
#[macro_export]
macro_rules! atom {
    // 节点
    (node ( $name:expr )) => {
        Atom::Node(Node::new($name))
    };

    (node ( $name:expr ) { $($tt:tt)* }) => {
        Atom::Node(node!($name { $($tt)* }))
    };

    // 数组
    (array [ $($value:expr),* $(,)? ]) => {
        Atom::Array(Array::from(vec![$($value),*]))
    };

    // 对象
    (obj { $($key:ident : $value:expr),* $(,)? }) => {
        Atom::Obj(Obj::from_pairs([
            $((stringify!($key), $value)),*
        ]))
    };
}
```

### 3. atoms! 简化宏

#### 语法变体

```rust
// 1. 字符串 -> 节点
atoms!("config")
atoms!("config" { version: "1.0" })

// 2. 数组
atoms!([1, 2, 3, 4, 5])

// 3. 对象
atoms!({ name: "Alice", age: 30 })

// 4. 嵌套
atoms!("root" {
    db("database") { host: "localhost" },
    items: [1, 2, 3],
    meta: { version: "1.0" },
})
```

#### 宏实现

```rust
#[macro_export]
macro_rules! atoms {
    // 字符串 -> 节点
    ($name:expr) => {
        Atom::Node(Node::new($name))
    };

    // 节点带属性
    ($name:expr { $($key:ident : $value:expr),* $(,)? }) => {
        Atom::Node(node!($name { $($key : $value),* }))
    };

    // 数组
    ([ $($value:expr),* $(,)? ]) => {
        Atom::Array(Array::from(vec![$($value),*]))
    };

    // 对象
    ({ $($key:ident : $value:expr),* $(,)? }) => {
        Atom::Obj(Obj::from_pairs([
            $((stringify!($key), $value)),*
        ]))
    };
}
```

## 文件结构

```
crates/auto-lang/src/
├── macros.rs              # 宏定义（新文件）
├── lib.rs                 # 导出宏
└── atom/
    └── mod.rs             # 重导出宏

crates/auto-lang/src/
└── macros/
    ├── mod.rs             # 模块声明
    ├── node.rs            # node! 宏实现
    ├── atom.rs            # atom! 宏实现
    └── atoms.rs           # atoms! 宏实现
```

## 实现计划

### ✅ Phase 1: 基础宏（已完成 - 2025年）

**状态**: 完成
**交付日期**: 2025年
**实际代码量**: ~449 LOC 实现 + ~322 LOC 测试

**实现的核心宏**:
1. ✅ 创建 `src/macros.rs` 文件
2. ✅ 实现 `node!` 宏（基础版本）
   - 简单节点: `node!("name")`
   - 带参数: `node!("name", arg="value")`
   - 带属性: `node!("name", key=value)`
   - 带参数和属性: `node!("name", arg="val", key=value)`
3. ✅ 实现 `atom!` 宏（基础版本）
   - 节点: `atom!(node("name"))`
   - 带属性的节点: `atom!(node("name", key=value))`
   - 数组: `atom!(array[1, 2, 3])`
   - 对象: `atom!(obj(key=value))`
4. ✅ 实现 `atoms!` 宏
   - 字符串自动推断: `atoms!("config")`
   - 数组优先匹配: `atoms!([1, 2, 3])`
   - 带属性的节点: `atoms!("name", key=value)`
5. ✅ 在 `lib.rs` 中导出宏

**语法设计变更**:
- 使用 `key=value` 代替 `key: value`（避免宏解析限制）
- 使用 `,` 分隔参数和属性（避免 `{` 后跟 `expr` 的问题）

**测试覆盖**:
- ✅ 简单节点构造 (4 tests)
- ✅ 带参数节点 (4 tests)
- ✅ 带属性节点 (4 tests)
- ✅ 数组构造 (4 tests)
- ✅ 对象构造 (2 tests)
- ✅ 集成测试 (2 tests)
- ✅ 54 个 doc tests

**测试结果**: **70 个测试全部通过，零失败**
**编译警告**: **零**
**文档完整性**: **所有公共 API 已文档化**

### Phase 2: 高级特性（第 2 步）

添加高级语法支持：

**任务**:
1. ⏳ 实现 `node!` 宏的子节点递归
2. ⏳ 支持混合属性和子节点
3. ⏳ 添加参数支持

**测试**:
- 深度嵌套
- 混合语法
- 边缘情况

**预期代码量**: ~150 LOC

### Phase 3: 完善和优化（第 3 步）

**任务**:
1. ⏳ 添加完整文档注释
2. ⏳ 添加示例到 rustdoc
3. ⏳ 性能测试（确保零开销）
4. ⏳ 错误信息改进

**预期代码量**: ~50 LOC

## 测试策略

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_simple() {
        let node = node!("config");
        assert_eq!(node.name, "config");
    }

    #[test]
    fn test_node_with_props() {
        let node = node!("config" {
            version: "1.0",
            debug: true,
        });

        assert_eq!(node.get_prop_of("version"), Value::Str("1.0".into()));
        assert_eq!(node.get_prop_of("debug"), Value::Bool(true));
    }

    #[test]
    fn test_atom_array() {
        let atom = atom!(array[1, 2, 3, 4, 5]);
        assert!(atom.is_array());

        if let Atom::Array(arr) = atom {
            assert_eq!(arr.len(), 5);
            assert_eq!(arr.values[0], Value::Int(1));
        }
    }

    #[test]
    fn test_atom_obj() {
        let atom = atom!(obj { name: "Alice", age: 30 });
        assert!(atom.is_obj());

        if let Atom::Obj(obj) = atom {
            assert_eq!(obj.get_str_of("name"), "Alice");
            assert_eq!(obj.get_int_of("age"), 30);
        }
    }

    #[test]
    fn test_atoms_simple() {
        let atom = atoms!("config");
        assert!(atom.is_node());
    }

    #[test]
    fn test_atoms_array() {
        let atom = atoms!([1, 2, 3]);
        assert!(atom.is_array());
    }

    #[test]
    fn test_atoms_obj() {
        let atom = atoms!({ name: "Alice" });
        assert!(atom.is_obj());
    }

    #[test]
    fn test_nested_structure() {
        let atom = atom!(node("config") {
            database("db") { host: "localhost" },
            data: array[1, 2, 3],
            meta: obj { version: "1.0" },
        });

        assert!(atom.is_node());
        // ... 更多验证
    }
}
```

### 集成测试

创建真实的配置场景测试：

```rust
#[test]
fn test_realistic_config() {
    let atom = atoms!("config" {
        version: "1.0",
        debug: true,
        database("database") {
            host: "localhost",
            port: 5432,
            ssl: true,
        },
        redis("redis") {
            host: "127.0.0.1",
            port: 6379,
        },
    });

    // 验证完整结构
    assert!(atom.is_node());
    if let Atom::Node(node) = atom {
        assert_eq!(node.name, "config");
        assert_eq!(node.kids_len(), 2);
        // ... 完整验证
    }
}
```

## 文档

### Rustdoc 文档

所有宏都需要完整的文档示例：

```rust
/// 创建 Node 的声明式宏
///
/// # 示例
///
/// ```rust
/// use auto_lang::atom::node;
///
/// // 简单节点
/// let node = node!("config");
///
/// // 带属性
/// let node = node!("config" {
///     version: "1.0",
///     debug: true,
/// });
/// ```
#[macro_export]
macro_rules! node {
    // ...
}
```

### 使用指南

创建 `docs/atom-macro-dsl-guide.md`：

```markdown
# Atom 宏 DSL 使用指南

## 快速开始

### node! 宏

### atom! 宏

### atoms! 简化宏

## 对比

## 最佳实践

## 限制
```

## 限制和权衡

### 宏的限制

1. **无运行时条件构建**: 宏在编译期展开，不支持运行时条件
   ```rust
   // ❌ 不支持
   let atom = atom!(node("config") {
       child_if(cfg, "child") { value: 1 }
   });

   // ✅ 使用 Builder 代替
   let atom = Atom::builder()
       .node(Node::builder("config")
           .child_if(cfg, Node::builder("child").build())
           .build()
       )
       .build();
   ```

2. **类型推断限制**: 某些复杂类型可能需要显式标注
   ```rust
   // 可能需要标注
   let atom: Atom = atoms!({ data: vec![1, 2, 3] });
   ```

3. **错误信息**: 宏展开错误可能不如普通函数清晰

### 何时使用宏 vs Builder

**使用宏的场景**:
- ✅ 静态配置结构
- ✅ 测试数据构造
- ✅ 简单的树状结构
- ✅ 可读性优先的场景

**使用 Builder 的场景**:
- ✅ 需要条件构建
- ✅ 复杂的运行时逻辑
- ✅ 动态生成结构
- ✅ 需要更好的错误信息

## 性能考虑

### 零成本抽象

宏展开后与手动构造代码完全相同：

```rust
// 宏展开前
let atom = atoms!("config" { version: "1.0" });

// 宏展开后（等价于）
let atom = Atom::Node(
    Node::new("config")
        .with_prop("version", "1.0")
);
```

### 编译期检查

宏在编译期展开，类型错误在编译时捕获：

```rust
// ❌ 编译期错误
let atom = atom!(node("config") {
    port: "not a number",  // 类型不匹配
});
```

## 成功标准

### 功能完整性

- ✅ 支持所有基本语法变体
- ✅ 支持嵌套结构
- ✅ 编译期类型检查
- ✅ 零运行时开销

### 测试覆盖

- ✅ 所有宏变体都有测试
- ✅ 边缘情况测试
- ✅ 错误情况测试
- ✅ 集成测试

### 文档完整性

- ✅ 所有宏有 rustdoc
- ✅ 使用示例完整
- ✅ 限制和权衡说明清晰
- ✅ 最佳实践指南

### 代码质量

- ✅ 零编译警告
- ✅ 清晰的宏模式匹配
- ✅ 可维护的代码结构
- ✅ 完善的错误处理

## 交付物

1. **代码文件**:
   - `crates/auto-lang/src/macros.rs` (~200 LOC)
   - 测试模块 (~200 LOC)

2. **文档**:
   - Rustdoc 文档（所有宏）
   - 使用指南（`docs/atom-macro-dsl-guide.md`）
   - 更新 `docs/plans/015-atom-builder-api.md`

3. **测试**:
   - 30+ 单元测试
   - 5+ 集成测试
   - 所有测试通过

## 未来增强

### 可能的扩展

1. **更简洁的语法**: 探索更短的语法形式
2. **验证模式**: 添加编译期结构验证
3. **序列化支持**: 从字符串反序列化为 Atom
4. **IDE 集成**: 改善宏的 IDE 支持

### 与其他特性集成

1. **与 Builder 结合**: 宏 + Builder 混合使用
2. **与类型推导集成**: 利用类型推导简化语法
3. **与格式化集成**: 格式化宏输出

## 总结

阶段 3（宏 DSL）将提供最声明式的 Atom 构造语法，与阶段 1（链式方法）和阶段 2（Builder）形成完整的三层 API 体系：

```
阶段 1: 链式方法    -> 简单、直观、兼容性好
阶段 2: Builder     -> 灵活、强大、支持条件构建
阶段 3: 宏 DSL      -> 简洁、声明式、零开销
```

用户可以根据场景选择最合适的 API：
- 简单场景 → 宏 DSL
- 复杂场景 → Builder
- 动态场景 → 链式方法
