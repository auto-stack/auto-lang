# Auto Atom 序列化系统设计

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现类似 Serde 的 Auto ↔ Atom 自动序列化/反序列化机制，清理现有分散的 `to_node()` 实现。

**Architecture:** 编译期为每个 `type` 自动生成 `to_atom()` 和 `from_atom()` 代码，运行时提供 `AtomWriter`（Compact/Pretty）和 `AtomReader` 抽象。

**Tech Stack:** Rust, auto-val (Value/Node/Array/Obj), 编译器代码生成

---

## 1. 背景与目标

### 1.1 现状问题

- `to_node()` 方法分散在 30+ 个 AST 文件中
- 手动实现，容易出错，难以维护
- 输出格式单一，无"紧凑"和"美丽"两种模式
- 反序列化机制不完善

### 1.2 设计目标

1. **自动序列化** - 编译器为 `type` 自动生成序列化代码
2. **自动反序列化** - 编译器为 `type` 自动生成反序列化代码
3. **双格式输出** - 支持 Compact（紧凑）和 Pretty（美丽）两种格式
4. **清理遗留代码** - 删除分散的 `to_node()` 实现

### 1.3 Postponed 功能

- 用户自定义注解（如 `#[id]`, `#[attr]`, `#[content]`）
- ID 自动生成规则（规则 2、3）
- 字段默认值

---

## 2. 核心概念

### 2.1 Atom 文本格式

Atom 是 Auto 语言的子集，Node 语法：

```
NodeName [id] (attr: val, ...) { field: val; child_node; ... }
```

- `NodeName` - 类型名称
- `[id]` - 可选的 Node ID
- `(...)` - 属性区：短类型字段（str/int/bool）
- `{...}` - 内容区：复合类型字段（Array/Node/嵌套type）

**示例：**

```auto
type Person {
    id str
    name str
    age int
    tags []str
}

// 序列化输出：
Person(id: "u1") { tags: ["rust", "auto"] }
```

### 2.2 保留关键字

| 关键字 | 含义 | 使用场景 |
|--------|------|----------|
| `miss` | 缺失 | 字段在 Atom 中不存在 |
| `err` | 错误 | 字段类型不匹配/解析失败 |
| `nil` | 空值 | 用户主动设置为空 |

**示例：**

```auto
Person {
    id: "u1"
    nickname: nil      // 主动设为空
    email: miss        // 字段缺失
    age: err           // 类型错误
}
```

### 2.3 ID 规则

| 规则 | 条件 | 处理 |
|------|------|------|
| 规则 1 | 有 `id` 字段 | 该字段成为 Node ID |
| 规则 2 | 无 `id` 但有 `#[id]` 标注 | postponed |
| 规则 3 | 都没有 | postponed（系统自动生成） |

**第一版只实现规则 1。**

---

## 3. 架构设计

### 3.1 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                    AutoLang 编译器                           │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ Type Checker / AST                                   │    │
│  │  - 解析 type 定义                                     │    │
│  │  - 收集字段类型信息                                   │    │
│  └─────────────────────────────────────────────────────┘    │
│                           ↓                                  │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ AtomSerializer Generator (新增)                      │    │
│  │  - 为每个 type 自动生成 to_atom() 方法               │    │
│  │  - 为每个 type 自动生成 from_atom() 方法             │    │
│  │  - 生成 AtomWriter 调用代码                          │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│                    运行时 (auto-val)                         │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ AtomWriter (新增)                                    │    │
│  │  - CompactWriter: 紧凑格式，无换行无缩进             │    │
│  │  - PrettyWriter: 美丽格式，带换行缩进                │    │
│  │  - 统一的写入接口                                    │    │
│  └─────────────────────────────────────────────────────┘    │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ AtomReader (新增)                                    │    │
│  │  - 解析 Atom 文本为 Value                            │    │
│  │  - 支持从 Atom 反序列化到类型实例                    │    │
│  │  - 支持 miss/err/nil 关键字                          │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 模块划分

| 模块 | 位置 | 职责 |
|------|------|------|
| `AtomWriter` | `crates/auto-val/src/atom/writer.rs` | Atom 文本输出 |
| `CompactWriter` | `crates/auto-val/src/atom/writer.rs` | 紧凑格式实现 |
| `PrettyWriter` | `crates/auto-val/src/atom/writer.rs` | 美丽格式实现 |
| `AtomReader` | `crates/auto-val/src/atom/reader.rs` | Atom 文本解析 |
| `AtomSerializable` | `crates/auto-val/src/atom/traits.rs` | 序列化 trait |
| `AtomDeserializer` | `crates/auto-val/src/atom/traits.rs` | 反序列化 trait |
| 代码生成器 | `crates/auto-lang/src/codegen/atom.rs` | 编译期代码生成 |

---

## 4. 接口设计

### 4.1 AtomWriter Trait

```rust
/// AtomWriter - Atom 文本输出抽象
pub trait AtomWriter {
    /// 写入节点开始: `name` 或 `name(id: "...")`
    fn write_node_head(&mut self, name: &str, id: Option<&str>) -> io::Result<()>;

    /// 开始属性区: `(`
    fn begin_attrs(&mut self) -> io::Result<()>;

    /// 写入属性: `key: value`
    fn write_attr(&mut self, key: &str, value: &Value) -> io::Result<()>;

    /// 结束属性区: `)`
    fn end_attrs(&mut self) -> io::Result<()>;

    /// 开始内容区: `{`
    fn begin_content(&mut self) -> io::Result<()>;

    /// 写入字段: `key: value` 或子节点
    fn write_field(&mut self, key: &str, value: &Value) -> io::Result<()>;

    /// 结束内容区: `}`
    fn end_content(&mut self) -> io::Result<()>;

    /// 写入原始值（int, str, bool, nil, miss, err）
    fn write_value(&mut self, value: &Value) -> io::Result<()>;

    /// 获取输出字符串
    fn into_string(self) -> String;
}
```

### 4.2 AtomSerializable Trait

```rust
/// AtomSerializable - 可序列化为 Atom 的类型
pub trait AtomSerializable {
    /// 获取类型名称
    fn atom_type_name() -> &'static str;

    /// 序列化到 AtomWriter
    fn write_to(&self, writer: &mut dyn AtomWriter) -> io::Result<()>;

    /// 生成紧凑格式字符串
    fn to_atom_compact(&self) -> String {
        let mut w = CompactWriter::new();
        self.write_to(&mut w).unwrap();
        w.into_string()
    }

    /// 生成美丽格式字符串
    fn to_atom_pretty(&self) -> String {
        let mut w = PrettyWriter::new();
        self.write_to(&mut w).unwrap();
        w.into_string()
    }
}
```

### 4.3 AtomDeserializable Trait

```rust
/// AtomDeserializable - 从 Atom 反序列化
pub trait AtomDeserializable: Sized {
    /// 从 AtomReader 读取并构造实例
    fn read_from(reader: &mut AtomReader) -> Result<Self, AtomError>;

    /// 从字符串解析
    fn from_atom(s: &str) -> Result<Self, AtomError> {
        let mut reader = AtomReader::from_str(s);
        Self::read_from(&mut reader)
    }
}
```

---

## 5. 类型序列化规则

### 5.1 基本类型

| Auto 类型 | Atom 输出 | 示例 |
|-----------|----------|------|
| `int` | 裸数字 | `42` |
| `uint` | 数字 | `42` |
| `float` | 小数 | `3.14` |
| `bool` | `true`/`false` | `true` |
| `str` | 双引号字符串 | `"hello"` |
| `char` | 单引号字符 | `'a'` |
| `nil` | `nil` | `nil` |

### 5.2 复合类型

| Auto 类型 | Atom 输出 | 示例 |
|-----------|----------|------|
| `[]T` | 数组语法 | `[1, 2, 3]` |
| `{}` (Object) | 对象语法 | `{ a: 1, b: 2 }` |
| `tag` (Enum) | 标签名（短格式） | `Active` |

### 5.3 字段位置规则

| 字段类型 | 位置 | 原因 |
|----------|------|------|
| `int/uint/float/bool/str/char` | 属性区 `()` | 短小，适合单行 |
| `nil/miss/err` | 属性区 `()` | 单行占位符 |
| `[]T` (Array) | 内容区 `{}` | 可能很长 |
| `{}` (Object) | 内容区 `{}` | 多行结构 |
| 自定义 type | 内容区 `{}` | 嵌套结构 |
| `tag` (Enum) | 属性区 `()` | 短名称 |

### 5.4 用户自定义 type

```auto
type Person {
    id str           // → Node ID
    name str         // → 属性区
    age int          // → 属性区
    tags []str       // → 内容区
    address Address  // → 内容区（嵌套 type）
}

// 输出：
Person(id: "u1") {
    tags: ["rust", "auto"]
    address: Address { city: "Beijing" }
}
```

---

## 6. 输出格式对比

### 6.1 CompactWriter

```
Person(id:"u1"){tags:["rust","auto"],address:Address{city:"Beijing"}}
```

特点：
- 无换行、无缩进
- 冒号后无空格
- 逗号后无空格
- 适合网络传输、存储

### 6.2 PrettyWriter

```
Person(id: "u1") {
    tags: [
        "rust"
        "auto"
    ]
    address: Address {
        city: "Beijing"
    }
}
```

特点：
- 换行 + 4空格缩进
- 冒号后有空格
- 数组元素换行
- 适合人类阅读、调试

---

## 7. 编译器代码生成

### 7.1 生成时机

当编译器遇到 `type` 定义时，自动生成：
1. `impl AtomSerializable for TypeName`
2. `impl AtomDeserializable for TypeName`

### 7.2 生成示例

**输入：**

```auto
type Person {
    id str
    name str
    age int
    tags []str
}
```

**生成（伪代码）：**

```rust
impl AtomSerializable for Person {
    fn atom_type_name() -> &'static str { "Person" }

    fn write_to(&self, writer: &mut dyn AtomWriter) -> io::Result<()> {
        // 1. 写节点头，id 字段作为 Node ID
        writer.write_node_head("Person", Some(&self.id))?;

        // 2. 属性区（短类型）
        writer.begin_attrs()?;
        writer.write_attr("name", &Value::Str(&self.name))?;
        writer.write_attr("age", &Value::Int(self.age))?;
        writer.end_attrs()?;

        // 3. 内容区（复合类型）
        writer.begin_content()?;
        writer.write_field("tags", &Value::Array(&self.tags))?;
        writer.end_content()?;

        Ok(())
    }
}

impl AtomDeserializable for Person {
    fn read_from(reader: &mut AtomReader) -> Result<Self, AtomError> {
        let head = reader.read_node_head()?;
        if head.name != "Person" {
            return Err(AtomError::TypeMismatch { expected: "Person", found: head.name });
        }

        let attrs = reader.read_attrs()?;
        let content = reader.read_content()?;

        Ok(Person {
            id: head.id.unwrap_or_else(|| "miss".to_string()),
            name: attrs.get_str("name").unwrap_or_else(|_| Value::Str("miss".into())),
            age: attrs.get_int("age").unwrap_or_else(|_| Value::Str("err".into())),
            tags: content.get_array("tags").unwrap_or_else(|_| Value::Array(Array::new())),
        })
    }
}
```

---

## 8. 清理计划

### 8.1 删除的文件/代码

| 文件 | 处理 |
|------|------|
| `ast/*.rs` 中的 `to_node()` 方法 | 删除 |
| `ast/atom_helpers.rs` | 删除或重构 |

### 8.2 保留的文件/代码

| 文件 | 处理 |
|------|------|
| `atom.rs` (Atom/AtomBuilder) | 保留，适配新接口 |
| `auto-val/node.rs` 的 `Display` | 保留，用于调试 |
| `auto-val/value.rs` 的 `Display` | 保留 |

---

## 9. 测试用例

### 9.1 基本类型序列化

```rust
#[test]
fn test_int_serialization() {
    let v = Value::Int(42);
    assert_eq!(v.to_atom_compact(), "42");
}

#[test]
fn test_str_serialization() {
    let v = Value::Str("hello".into());
    assert_eq!(v.to_atom_compact(), "\"hello\"");
}

#[test]
fn test_bool_serialization() {
    assert_eq!(Value::Bool(true).to_atom_compact(), "true");
    assert_eq!(Value::Bool(false).to_atom_compact(), "false");
}

#[test]
fn test_nil_serialization() {
    assert_eq!(Value::Nil.to_atom_compact(), "nil");
}
```

### 9.2 数组序列化

```rust
#[test]
fn test_array_compact() {
    let arr = Array::from(vec![1, 2, 3]);
    assert_eq!(arr.to_atom_compact(), "[1,2,3]");
}

#[test]
fn test_array_pretty() {
    let arr = Array::from(vec![1, 2, 3]);
    assert_eq!(arr.to_atom_pretty(), "[\n    1\n    2\n    3\n]");
}
```

### 9.3 自定义 type 序列化

```rust
#[test]
fn test_person_serialization() {
    let person = Person {
        id: "u1".to_string(),
        name: "Alice".to_string(),
        age: 30,
        tags: vec!["rust".to_string()],
    };

    // Compact
    assert_eq!(
        person.to_atom_compact(),
        r#"Person(id:"u1"){tags:["rust"]}"#
    );

    // Pretty
    assert_eq!(
        person.to_atom_pretty(),
        r#"Person(id: "u1") {
    tags: [
        "rust"
    ]
}"#
    );
}
```

### 9.4 反序列化

```rust
#[test]
fn test_person_deserialization() {
    let atom = r#"Person(id: "u1") { name: "Alice", age: 30 }"#;
    let person = Person::from_atom(atom).unwrap();

    assert_eq!(person.id, "u1");
    assert_eq!(person.name, "Alice");
    assert_eq!(person.age, 30);
}

#[test]
fn test_missing_field() {
    let atom = r#"Person(id: "u1") { name: "Alice" }"#;
    let person = Person::from_atom(atom).unwrap();

    // age 缺失，应该是 err 或默认值
    assert!(person.age == 0 || person.age == -1); // 根据实现确定
}

#[test]
fn test_error_recovery() {
    let atom = r#"Person(id: "u1") { name: "Alice", age: "wrong" }"#;
    let person = Person::from_atom(atom).unwrap();

    // age 类型错误，应该标记为 err
    // 具体行为根据实现确定
}
```

### 9.5 Enum 序列化

```rust
#[test]
fn test_enum_serialization() {
    let status = Status::Active;
    assert_eq!(status.to_atom_compact(), "Active");

    let status = Status::Suspended;
    assert_eq!(status.to_atom_compact(), "Suspended");
}
```

### 9.6 嵌套 type

```rust
#[test]
fn test_nested_type() {
    let article = Article {
        id: "a1".to_string(),
        title: "Hello".to_string(),
        author: Person {
            id: "u1".to_string(),
            name: "Alice".to_string(),
            age: 30,
            tags: vec![],
        },
    };

    let expected = r#"Article(id: "a1") {
    title: "Hello"
    author: Person(id: "u1") {
        name: "Alice"
        age: 30
    }
}"#;

    assert_eq!(article.to_atom_pretty(), expected);
}
```

---

## 10. 实现计划

### Phase 1: 基础设施（估计 2-3 天）

1. 创建 `crates/auto-val/src/atom/` 模块
2. 实现 `AtomWriter` trait
3. 实现 `CompactWriter`
4. 实现 `PrettyWriter`
5. 为基本类型实现 `AtomSerializable`

### Phase 2: 复合类型（估计 2-3 天）

1. 为 `Array` 实现 `AtomSerializable`
2. 为 `Obj` 实现 `AtomSerializable`
3. 为 `Node` 实现 `AtomSerializable`
4. 实现 `miss`/`err`/`nil` 关键字支持

### Phase 3: AtomReader（估计 2-3 天）

1. 实现 `AtomReader` 词法分析
2. 实现递归下降解析器
3. 实现 `AtomDeserializable` trait
4. 为基本类型实现反序列化

### Phase 4: 编译器集成（估计 3-4 天）

1. 创建代码生成器模块
2. 实现 `type` 定义的代码生成
3. 实现 `tag` 定义的代码生成
4. 集成到编译流程

### Phase 5: 清理与测试（估计 1-2 天）

1. 删除分散的 `to_node()` 方法
2. 删除 `atom_helpers.rs`
3. 完善测试覆盖
4. 文档更新

---

## 11. 风险与缓解

| 风险 | 缓解措施 |
|------|----------|
| 编译器代码生成复杂 | 先为预定义类型实现，再扩展到用户 type |
| 反序列化错误恢复 | 使用 `miss`/`err` 占位符，保证不中断 |
| 性能问题 | CompactWriter 使用 `String` 预分配，避免频繁 realloc |
| 与现有代码兼容 | 保留 `Atom` 和 `AtomBuilder` 作为高层 API |

---

## 12. 后续工作（Postponed）

1. **用户自定义注解** - `#[id]`, `#[attr]`, `#[content]`, `#[skip]` 等
2. **ID 自动生成** - 规则 2（`#[id]` 标注）和规则 3（系统生成）
3. **字段默认值** - 支持在 type 定义中指定默认值
4. **流式写入** - 支持直接写入 `Write` trait 对象
5. **Schema 验证** - 支持 Atom Schema 验证

---

## 13. 详细执行任务

> **注意：** 每一步是一个操作（2-5 分钟），遵循 TDD 原则

---

### Task 1: 创建 atom 模块目录

**Files:**
- Create: `crates/auto-val/src/atom/mod.rs`
- Modify: `crates/auto-val/src/lib.rs`

**Step 1: 创建模块目录和 mod.rs**

```bash
mkdir -p crates/auto-val/src/atom
```

创建 `crates/auto-val/src/atom/mod.rs`:

```rust
//! Atom 序列化/反序列化模块
//!
//! 提供类似 Serde 的 Auto ↔ Atom 转换机制

pub mod writer;
pub mod reader;
pub mod error;
pub mod traits;

pub use writer::{AtomWriter, CompactWriter, PrettyWriter};
pub use reader::AtomReader;
pub use error::AtomError;
pub use traits::{AtomSerializable, AtomDeserializable};
```

**Step 2: 更新 lib.rs 导出 atom 模块**

修改 `crates/auto-val/src/lib.rs`，在末尾添加：

```rust
// Atom serialization module
pub mod atom;
pub use atom::*;
```

**Step 3: 验证编译**

Run: `cargo check -p auto-val`
Expected: 编译失败（模块文件不存在）

**Step 4: 创建空的模块文件**

创建空文件占位：
- `crates/auto-val/src/atom/writer.rs`
- `crates/auto-val/src/atom/reader.rs`
- `crates/auto-val/src/atom/error.rs`
- `crates/auto-val/src/atom/traits.rs`

每个文件写入：
```rust
// TODO: Implement
```

**Step 5: 验证编译**

Run: `cargo check -p auto-val`
Expected: 编译成功

**Step 6: Commit**

```bash
rtk git add crates/auto-val/src/atom/ crates/auto-val/src/lib.rs
rtk git commit -m "feat(atom): create atom module structure"
```

---

### Task 2: 实现 AtomError 错误类型

**Files:**
- Modify: `crates/auto-val/src/atom/error.rs`
- Test: `crates/auto-val/tests/atom/error.rs`

**Step 1: 编写失败测试**

创建 `crates/auto-val/tests/atom/error.rs`:

```rust
use auto_val::atom::AtomError;

#[test]
fn test_atom_error_display() {
    let err = AtomError::UnexpectedToken {
        expected: "identifier".to_string(),
        found: "number".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("identifier"));
    assert!(msg.contains("number"));
}

#[test]
fn test_atom_error_type_mismatch() {
    let err = AtomError::TypeMismatch {
        expected: "int".to_string(),
        found: "string".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("int"));
    assert!(msg.contains("string"));
}

#[test]
fn test_atom_error_missing_field() {
    let err = AtomError::MissingField {
        field: "id".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("id"));
}
```

**Step 2: 运行测试验证失败**

Run: `cargo test -p auto-val --test error`
Expected: 编译失败

**Step 3: 实现 AtomError**

修改 `crates/auto-val/src/atom/error.rs`:

```rust
use thiserror::Error;

/// Atom 序列化/反序列化错误
#[derive(Error, Debug)]
pub enum AtomError {
    #[error("Unexpected token: expected {expected}, found {found}")]
    UnexpectedToken {
        expected: String,
        found: String,
    },

    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch {
        expected: String,
        found: String,
    },

    #[error("Missing field: {field}")]
    MissingField {
        field: String,
    },

    #[error("Parse error: {message}")]
    ParseError {
        message: String,
    },

    #[error("Invalid value: {message}")]
    InvalidValue {
        message: String,
    },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Atom 操作结果类型
pub type AtomResult<T> = Result<T, AtomError>;
```

**Step 4: 运行测试验证通过**

Run: `cargo test -p auto-val --test error`
Expected: 3 tests passed

**Step 5: Commit**

```bash
rtk git add crates/auto-val/src/atom/error.rs crates/auto-val/tests/atom/error.rs
rtk git commit -m "feat(atom): add AtomError enum for error handling"
```

---

### Task 3: 实现 AtomWriter Trait

**Files:**
- Modify: `crates/auto-val/src/atom/writer.rs`
- Test: `crates/auto-val/tests/atom/writer.rs`

**Step 1: 编写失败测试**

创建 `crates/auto-val/tests/atom/writer.rs`:

```rust
use auto_val::atom::{AtomWriter, CompactWriter};
use auto_val::Value;
use std::io;

struct TestWriter {
    output: String,
}

impl AtomWriter for TestWriter {
    fn write_node_head(&mut self, name: &str, id: Option<&str>) -> io::Result<()> {
        match id {
            Some(id) => write!(self.output, "{}({})", name, id),
            None => write!(self.output, "{}", name),
        }
    }

    fn begin_attrs(&mut self) -> io::Result<()> {
        write!(self.output, "(")
    }

    fn write_attr(&mut self, key: &str, value: &Value) -> io::Result<()> {
        write!(self.output, "{}:{},", key, value)
    }

    fn end_attrs(&mut self) -> io::Result<()> {
        write!(self.output, ")")
    }

    fn begin_content(&mut self) -> io::Result<()> {
        write!(self.output, "{{")
    }

    fn write_field(&mut self, key: &str, value: &Value) -> io::Result<()> {
        write!(self.output, "{}:{},", key, value)
    }

    fn end_content(&mut self) -> io::Result<()> {
        write!(self.output, "}")
    }

    fn write_value(&mut self, value: &Value) -> io::Result<()> {
        write!(self.output, "{}", value)
    }

    fn into_string(self) -> String {
        self.output
    }
}

#[test]
fn test_atom_writer_trait_exists() {
    let mut writer = TestWriter { output: String::new() };
    writer.write_node_head("Test", None).unwrap();
    assert!(writer.output.contains("Test"));
}
```

**Step 2: 运行测试验证失败**

Run: `cargo test -p auto-val --test writer`
Expected: 编译失败

**Step 3: 实现 AtomWriter Trait**

修改 `crates/auto-val/src/atom/writer.rs`:

```rust
use crate::Value;
use std::io;

/// AtomWriter - Atom 文本输出抽象
///
/// 提供两种实现：CompactWriter（紧凑格式）和 PrettyWriter（美丽格式）
pub trait AtomWriter {
    /// 写入节点开始: `name` 或 `name(id: "...")`
    fn write_node_head(&mut self, name: &str, id: Option<&str>) -> io::Result<()>;

    /// 开始属性区: `(`
    fn begin_attrs(&mut self) -> io::Result<()>;

    /// 写入属性: `key: value`
    fn write_attr(&mut self, key: &str, value: &Value) -> io::Result<()>;

    /// 结束属性区: `)`
    fn end_attrs(&mut self) -> io::Result<()>;

    /// 开始内容区: `{`
    fn begin_content(&mut self) -> io::Result<()>;

    /// 写入字段: `key: value`
    fn write_field(&mut self, key: &str, value: &Value) -> io::Result<()>;

    /// 结束内容区: `}`
    fn end_content(&mut self) -> io::Result<()>;

    /// 写入原始值（int, str, bool, nil, miss, err）
    fn write_value(&mut self, value: &Value) -> io::Result<()>;

    /// 获取输出字符串
    fn into_string(self) -> String;
}
```

**Step 4: 运行测试验证通过**

Run: `cargo test -p auto-val --test writer`
Expected: 1 test passed

**Step 5: Commit**

```bash
rtk git add crates/auto-val/src/atom/writer.rs crates/auto-val/tests/atom/writer.rs
rtk git commit -m "feat(atom): add AtomWriter trait definition"
```

---

### Task 4: 实现 CompactWriter

**Files:**
- Modify: `crates/auto-val/src/atom/writer.rs`
- Test: `crates/auto-val/tests/atom/writer.rs`

**Step 1: 编写失败测试**

在 `crates/auto-val/tests/atom/writer.rs` 添加：

```rust
use auto_val::atom::CompactWriter;

#[test]
fn test_compact_writer_int() {
    let mut writer = CompactWriter::new();
    writer.write_value(&Value::Int(42)).unwrap();
    assert_eq!(writer.into_string(), "42");
}

#[test]
fn test_compact_writer_str() {
    let mut writer = CompactWriter::new();
    writer.write_value(&Value::Str("hello".into())).unwrap();
    assert_eq!(writer.into_string(), "\"hello\"");
}

#[test]
fn test_compact_writer_bool() {
    let mut writer = CompactWriter::new();
    writer.write_value(&Value::Bool(true)).unwrap();
    assert_eq!(writer.into_string(), "true");

    let mut writer = CompactWriter::new();
    writer.write_value(&Value::Bool(false)).unwrap();
    assert_eq!(writer.into_string(), "false");
}

#[test]
fn test_compact_writer_nil() {
    let mut writer = CompactWriter::new();
    writer.write_value(&Value::Nil).unwrap();
    assert_eq!(writer.into_string(), "nil");
}

#[test]
fn test_compact_writer_node() {
    let mut writer = CompactWriter::new();
    writer.write_node_head("Person", Some("u1")).unwrap();
    writer.begin_attrs().unwrap();
    writer.write_attr("name", &Value::Str("Alice".into())).unwrap();
    writer.write_attr("age", &Value::Int(30)).unwrap();
    writer.end_attrs().unwrap();
    writer.begin_content().unwrap();
    writer.end_content().unwrap();
    assert_eq!(writer.into_string(), "Person(u1)(name:\"Alice\",age:30,){}");
}
```

**Step 2: 运行测试验证失败**

Run: `cargo test -p auto-val --test writer`
Expected: 编译失败（CompactWriter 不存在）

**Step 3: 实现 CompactWriter**

在 `crates/auto-val/src/atom/writer.rs` 添加：

```rust
use std::fmt::Write;

/// CompactWriter - 紧凑格式 Atom 输出
///
/// 特点：
/// - 无换行、无缩进
/// - 冒号后无空格
/// - 适合网络传输、存储
pub struct CompactWriter {
    output: String,
}

impl CompactWriter {
    pub fn new() -> Self {
        Self {
            output: String::new(),
        }
    }
}

impl Default for CompactWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl AtomWriter for CompactWriter {
    fn write_node_head(&mut self, name: &str, id: Option<&str>) -> io::Result<()> {
        match id {
            Some(id) => write!(self.output, "{}({})", name, id),
            None => write!(self.output, "{}", name),
        }.map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    fn begin_attrs(&mut self) -> io::Result<()> {
        write!(self.output, "(").map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    fn write_attr(&mut self, key: &str, value: &Value) -> io::Result<()> {
        write!(self.output, "{}:{},", key, value).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    fn end_attrs(&mut self) -> io::Result<()> {
        write!(self.output, ")").map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    fn begin_content(&mut self) -> io::Result<()> {
        write!(self.output, "{{").map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    fn write_field(&mut self, key: &str, value: &Value) -> io::Result<()> {
        write!(self.output, "{}:{},", key, value).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    fn end_content(&mut self) -> io::Result<()> {
        write!(self.output, "}").map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    fn write_value(&mut self, value: &Value) -> io::Result<()> {
        write!(self.output, "{}", value).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    fn into_string(self) -> String {
        self.output
    }
}
```

**Step 4: 运行测试验证通过**

Run: `cargo test -p auto-val --test writer`
Expected: 6 tests passed

**Step 5: Commit**

```bash
rtk git add crates/auto-val/src/atom/writer.rs crates/auto-val/tests/atom/writer.rs
rtk git commit -m "feat(atom): implement CompactWriter for compact Atom output"
```

---

### Task 5: 实现 PrettyWriter

**Files:**
- Modify: `crates/auto-val/src/atom/writer.rs`
- Test: `crates/auto-val/tests/atom/writer.rs`

**Step 1: 编写失败测试**

在 `crates/auto-val/tests/atom/writer.rs` 添加：

```rust
use auto_val::atom::PrettyWriter;

#[test]
fn test_pretty_writer_int() {
    let mut writer = PrettyWriter::new();
    writer.write_value(&Value::Int(42)).unwrap();
    assert_eq!(writer.into_string(), "42");
}

#[test]
fn test_pretty_writer_node() {
    let mut writer = PrettyWriter::new();
    writer.write_node_head("Person", Some("u1")).unwrap();
    writer.begin_attrs().unwrap();
    writer.write_attr("name", &Value::Str("Alice".into())).unwrap();
    writer.end_attrs().unwrap();
    writer.begin_content().unwrap();
    writer.write_field("tags", &Value::Array(vec![Value::Str("rust".into())].into_iter().collect())).unwrap();
    writer.end_content().unwrap();

    let output = writer.into_string();
    // Pretty format should have newlines and indentation
    assert!(output.contains("Person"));
    assert!(output.contains("\n"));
    assert!(output.contains("    ")); // 4-space indent
}
```

**Step 2: 运行测试验证失败**

Run: `cargo test -p auto-val --test writer`
Expected: 编译失败（PrettyWriter 不存在）

**Step 3: 实现 PrettyWriter**

在 `crates/auto-val/src/atom/writer.rs` 添加：

```rust
/// PrettyWriter - 美丽格式 Atom 输出
///
/// 特点：
/// - 换行 + 4空格缩进
/// - 冒号后有空格
/// - 适合人类阅读、调试
pub struct PrettyWriter {
    output: String,
    indent: usize,
}

impl PrettyWriter {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
        }
    }

    fn write_indent(&mut self) -> io::Result<()> {
        for _ in 0..self.indent {
            write!(self.output, "    ").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        }
        Ok(())
    }

    fn increase_indent(&mut self) {
        self.indent += 1;
    }

    fn decrease_indent(&mut self) {
        self.indent = self.indent.saturating_sub(1);
    }
}

impl Default for PrettyWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl AtomWriter for PrettyWriter {
    fn write_node_head(&mut self, name: &str, id: Option<&str>) -> io::Result<()> {
        match id {
            Some(id) => write!(self.output, "{}({})", name, id),
            None => write!(self.output, "{}", name),
        }.map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    fn begin_attrs(&mut self) -> io::Result<()> {
        write!(self.output, "(").map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    fn write_attr(&mut self, key: &str, value: &Value) -> io::Result<()> {
        write!(self.output, "{}: {}, ", key, value).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    fn end_attrs(&mut self) -> io::Result<()> {
        // Remove trailing ", " if present
        if self.output.ends_with(", ") {
            self.output.truncate(self.output.len() - 2);
        }
        write!(self.output, ")").map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    fn begin_content(&mut self) -> io::Result<()> {
        writeln!(self.output, " {{").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        self.increase_indent();
        Ok(())
    }

    fn write_field(&mut self, key: &str, value: &Value) -> io::Result<()> {
        self.write_indent()?;
        writeln!(self.output, "{}: {}", key, value).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    fn end_content(&mut self) -> io::Result<()> {
        self.decrease_indent();
        self.write_indent()?;
        write!(self.output, "}").map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    fn write_value(&mut self, value: &Value) -> io::Result<()> {
        write!(self.output, "{}", value).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    fn into_string(self) -> String {
        self.output
    }
}
```

**Step 4: 运行测试验证通过**

Run: `cargo test -p auto-val --test writer`
Expected: 8 tests passed

**Step 5: Commit**

```bash
rtk git add crates/auto-val/src/atom/writer.rs crates/auto-val/tests/atom/writer.rs
rtk git commit -m "feat(atom): implement PrettyWriter for human-readable Atom output"
```

---

### Task 6: 实现 AtomSerializable Trait

**Files:**
- Modify: `crates/auto-val/src/atom/traits.rs`
- Test: `crates/auto-val/tests/atom/traits.rs`

**Step 1: 编写失败测试**

创建 `crates/auto-val/tests/atom/traits.rs`:

```rust
use auto_val::atom::{AtomSerializable, CompactWriter, PrettyWriter};
use auto_val::Value;

#[test]
fn test_value_to_atom_compact() {
    let v = Value::Int(42);
    assert_eq!(v.to_atom_compact(), "42");
}

#[test]
fn test_value_to_atom_pretty() {
    let v = Value::Int(42);
    assert_eq!(v.to_atom_pretty(), "42");
}

#[test]
fn test_str_to_atom() {
    let v = Value::Str("hello".into());
    assert_eq!(v.to_atom_compact(), "\"hello\"");
}

#[test]
fn test_bool_to_atom() {
    assert_eq!(Value::Bool(true).to_atom_compact(), "true");
    assert_eq!(Value::Bool(false).to_atom_compact(), "false");
}

#[test]
fn test_nil_to_atom() {
    assert_eq!(Value::Nil.to_atom_compact(), "nil");
}
```

**Step 2: 运行测试验证失败**

Run: `cargo test -p auto-val --test traits`
Expected: 编译失败

**Step 3: 实现 AtomSerializable Trait**

修改 `crates/auto-val/src/atom/traits.rs`:

```rust
use super::{AtomWriter, CompactWriter, PrettyWriter};
use crate::Value;
use std::io;

/// AtomSerializable - 可序列化为 Atom 的类型
pub trait AtomSerializable {
    /// 获取类型名称
    fn atom_type_name() -> &'static str;

    /// 序列化到 AtomWriter
    fn write_to(&self, writer: &mut dyn AtomWriter) -> io::Result<()>;

    /// 生成紧凑格式字符串
    fn to_atom_compact(&self) -> String {
        let mut w = CompactWriter::new();
        self.write_to(&mut w).unwrap();
        w.into_string()
    }

    /// 生成美丽格式字符串
    fn to_atom_pretty(&self) -> String {
        let mut w = PrettyWriter::new();
        self.write_to(&mut w).unwrap();
        w.into_string()
    }
}

/// 为 Value 实现 AtomSerializable
impl AtomSerializable for Value {
    fn atom_type_name() -> &'static str {
        "Value"
    }

    fn write_to(&self, writer: &mut dyn AtomWriter) -> io::Result<()> {
        writer.write_value(self)
    }
}
```

**Step 4: 运行测试验证通过**

Run: `cargo test -p auto-val --test traits`
Expected: 5 tests passed

**Step 5: Commit**

```bash
rtk git add crates/auto-val/src/atom/traits.rs crates/auto-val/tests/atom/traits.rs
rtk git commit -m "feat(atom): add AtomSerializable trait and implement for Value"
```

---

### Task 7: 实现 AtomReader 词法分析

**Files:**
- Modify: `crates/auto-val/src/atom/reader.rs`
- Test: `crates/auto-val/tests/atom/reader.rs`

**Step 1: 编写失败测试**

创建 `crates/auto-val/tests/atom/reader.rs`:

```rust
use auto_val::atom::reader::{AtomLexer, AtomToken};

#[test]
fn test_lexer_number() {
    let mut lexer = AtomLexer::new("42");
    assert_eq!(lexer.next_token(), Some(AtomToken::Number("42".to_string())));
}

#[test]
fn test_lexer_string() {
    let mut lexer = AtomLexer::new("\"hello\"");
    assert_eq!(lexer.next_token(), Some(AtomToken::String("hello".to_string())));
}

#[test]
fn test_lexer_identifier() {
    let mut lexer = AtomLexer::new("Person");
    assert_eq!(lexer.next_token(), Some(AtomToken::Identifier("Person".to_string())));
}

#[test]
fn test_lexer_keywords() {
    let mut lexer = AtomLexer::new("nil true false miss err");
    assert_eq!(lexer.next_token(), Some(AtomToken::KwNil));
    assert_eq!(lexer.next_token(), Some(AtomToken::KwTrue));
    assert_eq!(lexer.next_token(), Some(AtomToken::KwFalse));
    assert_eq!(lexer.next_token(), Some(AtomToken::KwMiss));
    assert_eq!(lexer.next_token(), Some(AtomToken::KwErr));
}

#[test]
fn test_lexer_delimiters() {
    let mut lexer = AtomLexer::new("(){}[]:,");
    assert_eq!(lexer.next_token(), Some(AtomToken::LParen));
    assert_eq!(lexer.next_token(), Some(AtomToken::RParen));
    assert_eq!(lexer.next_token(), Some(AtomToken::LBrace));
    assert_eq!(lexer.next_token(), Some(AtomToken::RBrace));
    assert_eq!(lexer.next_token(), Some(AtomToken::LBracket));
    assert_eq!(lexer.next_token(), Some(AtomToken::RBracket));
    assert_eq!(lexer.next_token(), Some(AtomToken::Colon));
    assert_eq!(lexer.next_token(), Some(AtomToken::Comma));
}
```

**Step 2: 运行测试验证失败**

Run: `cargo test -p auto-val --test reader`
Expected: 编译失败

**Step 3: 实现 AtomLexer**

修改 `crates/auto-val/src/atom/reader.rs`:

```rust
use std::iter::Peekable;
use std::str::Chars;

/// Atom token types
#[derive(Debug, Clone, PartialEq)]
pub enum AtomToken {
    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Colon,
    Comma,

    // Literals
    Identifier(String),
    String(String),
    Number(String),

    // Keywords
    KwNil,
    KwTrue,
    KwFalse,
    KwMiss,
    KwErr,

    // End of input
    Eof,
}

/// Atom 词法分析器
pub struct AtomLexer<'a> {
    input: &'a str,
    chars: Peekable<Chars<'a>>,
    pos: usize,
}

impl<'a> AtomLexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.chars().peekable(),
            pos: 0,
        }
    }

    pub fn next_token(&mut self) -> Option<AtomToken> {
        self.skip_whitespace();

        if self.pos >= self.input.len() {
            return None;
        }

        let ch = self.current_char()?;

        match ch {
            '(' => { self.advance(); Some(AtomToken::LParen) }
            ')' => { self.advance(); Some(AtomToken::RParen) }
            '{' => { self.advance(); Some(AtomToken::LBrace) }
            '}' => { self.advance(); Some(AtomToken::RBrace) }
            '[' => { self.advance(); Some(AtomToken::LBracket) }
            ']' => { self.advance(); Some(AtomToken::RBracket) }
            ':' => { self.advance(); Some(AtomToken::Colon) }
            ',' => { self.advance(); Some(AtomToken::Comma) }

            '"' => self.read_string(),

            '0'..='9' | '-' => self.read_number(),

            'a'..='z' | 'A'..='Z' | '_' => self.read_identifier(),

            _ => None,
        }
    }

    fn current_char(&self) -> Option<char> {
        self.chars.clone().peek().copied()
    }

    fn advance(&mut self) {
        if self.pos < self.input.len() {
            self.chars.next();
            self.pos += 1;
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char() {
            if ch.is_whitespace() {
                self.advance();
            } else if ch == '/' && self.peek_next() == Some('/') {
                // Line comment
                self.advance();
                self.advance();
                while let Some(c) = self.current_char() {
                    if c == '\n' {
                        break;
                    }
                    self.advance();
                }
            } else {
                break;
            }
        }
    }

    fn peek_next(&self) -> Option<char> {
        let mut chars = self.chars.clone();
        chars.next();
        chars.peek().copied()
    }

    fn read_string(&mut self) -> Option<AtomToken> {
        self.advance(); // skip opening quote

        let mut result = String::new();
        while let Some(ch) = self.current_char() {
            if ch == '"' {
                self.advance(); // skip closing quote
                return Some(AtomToken::String(result));
            } else if ch == '\\' {
                self.advance();
                if let Some(escaped) = self.current_char() {
                    match escaped {
                        'n' => result.push('\n'),
                        't' => result.push('\t'),
                        'r' => result.push('\r'),
                        '"' => result.push('"'),
                        '\\' => result.push('\\'),
                        _ => result.push(escaped),
                    }
                    self.advance();
                }
            } else {
                result.push(ch);
                self.advance();
            }
        }
        None // Unclosed string
    }

    fn read_number(&mut self) -> Option<AtomToken> {
        let start = self.pos;

        // Handle negative sign
        if self.current_char() == Some('-') {
            self.advance();
        }

        while let Some(ch) = self.current_char() {
            if ch.is_ascii_digit() || ch == '.' {
                self.advance();
            } else {
                break;
            }
        }

        Some(AtomToken::Number(self.input[start..self.pos].to_string()))
    }

    fn read_identifier(&mut self) -> Option<AtomToken> {
        let start = self.pos;

        while let Some(ch) = self.current_char() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let ident = &self.input[start..self.pos];

        // Check for keywords
        match ident {
            "nil" => Some(AtomToken::KwNil),
            "true" => Some(AtomToken::KwTrue),
            "false" => Some(AtomToken::KwFalse),
            "miss" => Some(AtomToken::KwMiss),
            "err" => Some(AtomToken::KwErr),
            _ => Some(AtomToken::Identifier(ident.to_string())),
        }
    }
}
```

**Step 4: 运行测试验证通过**

Run: `cargo test -p auto-val --test reader`
Expected: 5 tests passed

**Step 5: Commit**

```bash
rtk git add crates/auto-val/src/atom/reader.rs crates/auto-val/tests/atom/reader.rs
rtk git commit -m "feat(atom): implement AtomLexer for tokenization"
```

---

### Task 8: 实现 AtomParser 和 AtomReader

**Files:**
- Modify: `crates/auto-val/src/atom/reader.rs`
- Test: `crates/auto-val/tests/atom/reader.rs`

**Step 1: 编写失败测试**

在 `crates/auto-val/tests/atom/reader.rs` 添加：

```rust
use auto_val::atom::AtomReader;
use auto_val::{Value, Array};

#[test]
fn test_reader_parse_int() {
    let mut reader = AtomReader::new("42");
    let v = reader.parse_value().unwrap();
    assert_eq!(v, Value::Int(42));
}

#[test]
fn test_reader_parse_str() {
    let mut reader = AtomReader::new("\"hello\"");
    let v = reader.parse_value().unwrap();
    assert!(matches!(v, Value::Str(_)));
}

#[test]
fn test_reader_parse_array() {
    let mut reader = AtomReader::new("[1, 2, 3]");
    let v = reader.parse_value().unwrap();
    if let Value::Array(arr) = v {
        assert_eq!(arr.len(), 3);
    } else {
        panic!("Expected array");
    }
}

#[test]
fn test_reader_parse_obj() {
    let mut reader = AtomReader::new("{a: 1, b: 2}");
    let v = reader.parse_value().unwrap();
    if let Value::Obj(obj) = v {
        assert_eq!(obj.len(), 2);
    } else {
        panic!("Expected obj");
    }
}

#[test]
fn test_reader_parse_node() {
    let mut reader = AtomReader::new("Person(id: \"u1\") { name: \"Alice\" }");
    let v = reader.parse_value().unwrap();
    if let Value::Node(node) = v {
        assert_eq!(node.name.as_str(), "Person");
    } else {
        panic!("Expected node");
    }
}
```

**Step 2: 运行测试验证失败**

Run: `cargo test -p auto-val --test reader`
Expected: 编译失败

**Step 3: 实现 AtomReader**

在 `crates/auto-val/src/atom/reader.rs` 添加：

```rust
use crate::{Value, Array, Obj, Node};
use super::AtomError;

/// AtomReader - Atom 文本解析器
pub struct AtomReader<'a> {
    lexer: AtomLexer<'a>,
}

impl<'a> AtomReader<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            lexer: AtomLexer::new(input),
        }
    }

    /// 解析一个 Atom 值
    pub fn parse_value(&mut self) -> Result<Value, AtomError> {
        let token = self.lexer.next_token().ok_or_else(|| AtomError::ParseError {
            message: "Unexpected end of input".to_string(),
        })?;

        self.parse_token(token)
    }

    fn parse_token(&mut self, token: AtomToken) -> Result<Value, AtomError> {
        match token {
            AtomToken::Number(n) => {
                if n.contains('.') {
                    n.parse::<f64>()
                        .map(Value::Float)
                        .map_err(|_| AtomError::ParseError { message: format!("Invalid float: {}", n) })
                } else {
                    n.parse::<i32>()
                        .map(Value::Int)
                        .map_err(|_| AtomError::ParseError { message: format!("Invalid int: {}", n) })
                }
            }

            AtomToken::String(s) => Ok(Value::Str(s.into())),

            AtomToken::KwNil => Ok(Value::Nil),
            AtomToken::KwTrue => Ok(Value::Bool(true)),
            AtomToken::KwFalse => Ok(Value::Bool(false)),
            AtomToken::KwMiss => Ok(Value::Str("miss".into())),
            AtomToken::KwErr => Ok(Value::Str("err".into())),

            AtomToken::LBracket => self.parse_array(),
            AtomToken::LBrace => self.parse_object(),
            AtomToken::Identifier(name) => self.parse_node(&name),

            _ => Err(AtomError::UnexpectedToken {
                expected: "value".to_string(),
                found: format!("{:?}", token),
            }),
        }
    }

    fn parse_array(&mut self) -> Result<Value, AtomError> {
        let mut arr = Vec::new();

        loop {
            let token = self.lexer.next_token();
            match token {
                Some(AtomToken::RBracket) => break,
                Some(AtomToken::Comma) => continue,
                Some(t) => {
                    arr.push(self.parse_token(t)?);
                }
                None => return Err(AtomError::ParseError {
                    message: "Unclosed array".to_string(),
                }),
            }
        }

        Ok(Value::Array(Array::from_vec(arr)))
    }

    fn parse_object(&mut self) -> Result<Value, AtomError> {
        let mut obj = Obj::new();

        loop {
            let token = self.lexer.next_token();
            match token {
                Some(AtomToken::RBrace) => break,
                Some(AtomToken::Comma) => continue,
                Some(AtomToken::Identifier(key)) => {
                    // Expect colon
                    match self.lexer.next_token() {
                        Some(AtomToken::Colon) => {
                            let value = self.parse_value()?;
                            obj.set(key, value);
                        }
                        _ => return Err(AtomError::UnexpectedToken {
                            expected: ":".to_string(),
                            found: "something else".to_string(),
                        }),
                    }
                }
                Some(t) => return Err(AtomError::UnexpectedToken {
                    expected: "identifier or }".to_string(),
                    found: format!("{:?}", t),
                }),
                None => return Err(AtomError::ParseError {
                    message: "Unclosed object".to_string(),
                }),
            }
        }

        Ok(Value::Obj(obj))
    }

    fn parse_node(&mut self, name: &str) -> Result<Value, AtomError> {
        let mut node = Node::new(name);

        // Check for optional id (identifier followed by `(` or `{`)
        if let Some(AtomToken::Identifier(id)) = self.lexer.next_token() {
            // Check if next is `(` - then id is actually node name, not id
            // For simplicity, assume first identifier after node name is id
            node.set_id(&id);
        }

        // Parse attributes
        if let Some(AtomToken::LParen) = self.lexer.next_token() {
            loop {
                match self.lexer.next_token() {
                    Some(AtomToken::RParen) => break,
                    Some(AtomToken::Comma) => continue,
                    Some(AtomToken::Identifier(key)) => {
                        if let Some(AtomToken::Colon) = self.lexer.next_token() {
                            let value = self.parse_value()?;
                            node.set_prop(key, value);
                        }
                    }
                    _ => break,
                }
            }
        }

        // Parse content
        if let Some(AtomToken::LBrace) = self.lexer.next_token() {
            loop {
                match self.lexer.next_token() {
                    Some(AtomToken::RBrace) => break,
                    Some(AtomToken::Comma) => continue,
                    Some(AtomToken::Identifier(key)) => {
                        if let Some(AtomToken::Colon) = self.lexer.next_token() {
                            let value = self.parse_value()?;
                            node.set_prop(key, value);
                        }
                    }
                    Some(t) => {
                        // Could be a child node
                        let value = self.parse_token(t)?;
                        if let Value::Node(child) = value {
                            node.add_kid(child);
                        }
                    }
                    None => break,
                }
            }
        }

        Ok(Value::Node(node))
    }
}
```

**Step 4: 运行测试验证通过**

Run: `cargo test -p auto-val --test reader`
Expected: 10 tests passed

**Step 5: Commit**

```bash
rtk git add crates/auto-val/src/atom/reader.rs crates/auto-val/tests/atom/reader.rs
rtk git commit -m "feat(atom): implement AtomReader for parsing Atom text"
```

---

### Task 9: 实现 AtomDeserializable Trait

**Files:**
- Modify: `crates/auto-val/src/atom/traits.rs`
- Test: `crates/auto-val/tests/atom/traits.rs`

**Step 1: 编写失败测试**

在 `crates/auto-val/tests/atom/traits.rs` 添加：

```rust
use auto_val::atom::{AtomDeserializable, AtomReader};

#[test]
fn test_i32_from_atom() {
    let result = i32::from_atom("42");
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_string_from_atom() {
    let result = String::from_atom("\"hello\"");
    assert_eq!(result.unwrap(), "hello");
}

#[test]
fn test_bool_from_atom() {
    assert_eq!(bool::from_atom("true").unwrap(), true);
    assert_eq!(bool::from_atom("false").unwrap(), false);
}
```

**Step 2: 运行测试验证失败**

Run: `cargo test -p auto-val --test traits`
Expected: 编译失败

**Step 3: 实现 AtomDeserializable Trait**

在 `crates/auto-val/src/atom/traits.rs` 添加：

```rust
use super::{AtomReader, AtomError};
use std::io;

/// AtomDeserializable - 从 Atom 反序列化
pub trait AtomDeserializable: Sized {
    /// 从 AtomReader 读取并构造实例
    fn read_from(reader: &mut AtomReader) -> Result<Self, AtomError>;

    /// 从字符串解析
    fn from_atom(s: &str) -> Result<Self, AtomError> {
        let mut reader = AtomReader::new(s);
        Self::read_from(&mut reader)
    }
}

/// 为 i32 实现 AtomDeserializable
impl AtomDeserializable for i32 {
    fn read_from(reader: &mut AtomReader) -> Result<Self, AtomError> {
        let value = reader.parse_value()?;
        match value {
            Value::Int(n) => Ok(n),
            Value::Str(s) if s.as_str() == "miss" || s.as_str() == "err" => {
                Ok(0) // Default value for missing/error
            }
            _ => Err(AtomError::TypeMismatch {
                expected: "int".to_string(),
                found: format!("{:?}", value),
            }),
        }
    }
}

/// 为 String 实现 AtomDeserializable
impl AtomDeserializable for String {
    fn read_from(reader: &mut AtomReader) -> Result<Self, AtomError> {
        let value = reader.parse_value()?;
        match value {
            Value::Str(s) => Ok(s.to_string()),
            Value::Nil => Ok(String::new()),
            _ => Err(AtomError::TypeMismatch {
                expected: "string".to_string(),
                found: format!("{:?}", value),
            }),
        }
    }
}

/// 为 bool 实现 AtomDeserializable
impl AtomDeserializable for bool {
    fn read_from(reader: &mut AtomReader) -> Result<Self, AtomError> {
        let value = reader.parse_value()?;
        match value {
            Value::Bool(b) => Ok(b),
            _ => Err(AtomError::TypeMismatch {
                expected: "bool".to_string(),
                found: format!("{:?}", value),
            }),
        }
    }
}
```

**Step 4: 运行测试验证通过**

Run: `cargo test -p auto-val --test traits`
Expected: 8 tests passed

**Step 5: Commit**

```bash
rtk git add crates/auto-val/src/atom/traits.rs crates/auto-val/tests/atom/traits.rs
rtk git commit -m "feat(atom): add AtomDeserializable trait for basic types"
```

---

### Task 10: 清理现有的 to_node 机制

**Files:**
- Delete: `crates/auto-lang/src/ast/atom_helpers.rs`
- Modify: `crates/auto-lang/src/ast/mod.rs`
- Multiple: `crates/auto-lang/src/ast/*.rs`

**Step 1: 删除 atom_helpers.rs**

Run: `rm crates/auto-lang/src/ast/atom_helpers.rs`

**Step 2: 更新 ast/mod.rs**

从 `crates/auto-lang/src/ast/mod.rs` 中移除：
```rust
pub mod atom_helpers;
```

**Step 3: 查找所有 to_node 方法**

Run: `grep -r "fn to_node" crates/auto-lang/src/ast/ --include="*.rs"`

**Step 4: 记录需要删除的文件列表**

根据 grep 结果，列出所有包含 `to_node` 的文件。

**Step 5: 逐个删除 to_node 方法**

对每个文件：
1. 删除 `to_node` 方法定义
2. 删除相关 imports
3. 验证编译

**Step 6: 验证编译**

Run: `cargo build -p auto-lang`
Expected: 编译成功

**Step 7: Commit**

```bash
rtk git add -A
rtk git commit -m "refactor(atom): remove legacy to_node serialization methods"
```

---

## 执行计划总结

| Phase | 任务数 | 预计时间 |
|-------|--------|----------|
| Phase 1: 基础设施 | 6 tasks | 1-2 天 |
| Phase 2: 序列化 | 4 tasks | 1 天 |
| Phase 3: 反序列化 | 3 tasks | 1-2 天 |
| Phase 4: 清理 | 1 task | 0.5 天 |

**总计: 14 个核心任务，预计 3-5 天**

**Postponed:**
- Phase 4: 编译器集成（为用户 type 自动生成代码）
- 用户自定义注解支持
