# Atom API 使用指南

**作者**: AutoLang 团队
**更新日期**: 2025-01-11
**状态**: 完整版

## 目录

1. [概述](#概述)
2. [Atom 类型体系](#atom-类型体系)
3. [三层 API 体系](#三层-api-体系)
4. [链式方法 API](#链式方法-api)
5. [Builder API](#builder-api)
6. [宏 DSL](#宏-dsl)
7. [插值功能](#插值功能)
8. [完整示例](#完整示例)
9. [最佳实践](#最佳实践)
10. [常见问题](#常见问题)

---

## 概述

AutoLang 的 `Atom` 类型系统提供了三种不同的 API 风格来构建树状数据结构：

1. **链式方法** - 简单直观，适合基本操作
2. **Builder 模式** - 灵活强大，支持条件构建
3. **宏 DSL** - 最简洁的语法，零开销声明式

本指南将帮助你：
- 理解 Atom 类型体系
- 掌握三种 API 的使用方法
- 学会选择合适的 API
- 了解插值和高级特性

---

## Atom 类型体系

### Atom 枚举

`Atom` 是 AutoLang 的核心类型，可以表示多种数据结构：

```rust
use auto_lang::atom::Atom;
use auto_val::{Value, Node, Array, Obj};

// Atom 的四种变体
pub enum Atom {
    Node(Node),    // 树节点
    Array(Array),  // 数组
    Obj(Obj),      // 对象（键值对集合）
    Value(Value),  // 简单值（Int, Bool, Str 等）
}
```

### Node 类型

`Node` 表示树状结构中的节点：

```rust
use auto_val::Node;

let node = Node::new("config")
    .with_prop("version", "1.0")
    .with_prop("debug", true);

// 结构
Node {
    name: "config",           // 节点名称
    args: Args::new(),        // 位置参数
    props: Obj::new(),        // 属性集合
    kids: Kids::new(),        // 子节点集合
    // ...
}
```

### Array 和 Obj

```rust
use auto_val::{Array, Obj, Value};

// 数组
let arr = Array::from(vec![
    Value::Int(1),
    Value::Int(2),
    Value::Int(3),
]);

// 对象
let obj = Obj::new()
    .set("name", Value::Str("Alice".into()))
    .set("age", Value::Int(30));
```

---

## 三层 API 体系

### 对比表

| 特性 | 链式方法 | Builder | 宏 DSL |
|------|----------|---------|---------|
| **语法简洁度** | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **条件构建** | ⭐⭐ | ⭐⭐⭐⭐⭐ | ❌ |
| **类型安全** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **运行时灵活性** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ❌ |
| **零开销** | ✅ | ✅ | ✅ |
| **学习曲线** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ |

### 选择建议

```
简单场景 → 宏 DSL (最简洁)
动态场景 → 链式方法 (最灵活)
复杂场景 → Builder (最强大)
```

---

## 链式方法 API

### 基本用法

链式方法是最直接的方式，适合简单的构建操作：

```rust
use auto_lang::atom::Atom;
use auto_val::{Node, Value};

// 创建节点
let node = Node::new("config")
    .with_prop("version", "1.0")
    .with_prop("debug", true);

let atom = Atom::Node(node);

// 添加子节点
let node = Node::new("root")
    .with_kid(Node::new("child1"))
    .with_kid(Node::new("child2"));

// 修改属性
let node = Node::new("config")
    .with_prop("host", "localhost")
    .with_prop("port", 8080)
    .update_prop("port", 9090);  // 更新已存在的属性
```

### 常用方法

#### Node 方法

```rust
// 创建
Node::new(name: &str)

// 属性操作
.with_prop(key, value)           // 添加/更新属性
.with_props(props: Obj)          // 批量添加属性
.get_prop_of(key) -> Value       // 获取属性值
.has_prop(key) -> bool           // 检查属性是否存在
.update_prop(key, value)         // 更新属性（不存在则添加）
.remove_prop(key)                // 删除属性
.props_len() -> usize            // 属性数量

// 子节点操作
.with_kid(node)                   // 添加子节点
.with_kids(vec)                   // 批量添加子节点
.add_kid_unified(node)            // 添加任意类型作为子节点
.kids_len() -> usize              // 子节点数量
.get_kid(index) -> Option<&Kid>  // 获取指定索引的子节点

// 参数操作
.with_arg(value)                  // 添加位置参数
.args_len() -> usize              // 参数数量

// 转换
.to_value() -> Value              // 转换为 Value
.to_atom() -> Atom                // 转换为 Atom
```

#### Array 方法

```rust
use auto_val::Array;

// 创建
Array::new()
Array::from(vec)
Array::with_capacity(capacity)

// 元素操作
.push(value)                      // 添加元素
.len() -> usize                   // 长度
.is_empty() -> bool               // 是否为空
.get(index) -> Option<&Value>     // 获取元素

// 转换
.to_value() -> Value
.to_atom() -> Atom
```

#### Obj 方法

```rust
use auto_val::Obj;

// 创建
Obj::new()
Obj::from_pairs([(key, value), ...])

// 属性操作
.set(key, value)                  // 设置属性
.get(key) -> Option<&Value>       // 获取属性
.has(key) -> bool                 // 检查键是否存在
.remove(key) -> Option<Value>     // 删除属性
.len() -> usize                    // 属性数量

// 转换
.to_value() -> Value
.to_atom() -> Atom
```

### 示例：构建配置树

```rust
use auto_lang::atom::Atom;
use auto_val::{Node, Value};

let config = Node::new("config")
    .with_prop("version", "1.0")
    .with_prop("debug", true)
    .with_kid(
        Node::new("database")
            .with_prop("host", "localhost")
            .with_prop("port", 5432)
    )
    .with_kid(
        Node::new("redis")
            .with_prop("host", "127.0.0.1")
            .with_prop("port", 6379)
    );

let atom = Atom::Node(config);
```

---

## Builder API

### 基本用法

Builder 模式提供更灵活的构建方式，支持条件构建：

```rust
use auto_lang::atom::Atom;
use auto_val::Node;

// 使用 Builder
let node = Node::builder("config")
    .prop("version", "1.0")
    .prop("debug", true)
    .build();

let atom = Atom::Node(node);
```

### 条件构建

Builder 的最大优势是支持条件构建：

```rust
use auto_val::Node;

let use_ssl = true;
let port = 8080;

let node = Node::builder("server")
    .prop("host", "localhost")
    .prop("port", port)
    .prop_if(use_ssl, "ssl", true)           // 条件添加属性
    .prop_if(use_ssl, "cert", "/path/to/cert")
    .build();
```

### 常用方法

```rust
// 创建 Builder
Node::builder(name)

// 属性操作
.prop(key, value)                 // 添加属性
.prop_if(condition, key, value)   // 条件添加属性
.props(props)                     // 批量添加
.props_if(condition, props)       // 条件批量添加

// 子节点操作
.kid(node)                        // 添加子节点
.kid_if(condition, node)          // 条件添加子节点
.kids(vec)                        // 批量添加
.kids_if(condition, vec)          // 条件批量添加

// 参数操作
.arg(value)                       // 添加参数
.arg_if(condition, value)         // 条件添加参数

// 构建
.build() -> Node                   // 构建 Node
.build_atom() -> Atom             // 构建 Atom
.build_value() -> Value           // 构建 Value
```

### 示例：动态配置

```rust
use auto_val::{Node, Value};
use auto_lang::atom::Atom;

struct Config {
    debug: bool,
    log_level: Option<String>,
    database: bool,
    redis: bool,
}

impl Config {
    fn to_atom(&self) -> Atom {
        Node::builder("config")
            .prop("debug", self.debug)
            .prop_if(
                self.log_level.is_some(),
                "log_level",
                self.log_level.as_ref().unwrap()
            )
            .kid_if(self.database, Node::builder("database")
                .prop("host", "localhost")
                .prop("port", 5432)
                .build()
            )
            .kid_if(self.redis, Node::builder("redis")
                .prop("host", "127.0.0.1")
                .prop("port", 6379)
                .build()
            )
            .build_atom()
    }
}

let config = Config {
    debug: true,
    log_level: Some("info".to_string()),
    database: true,
    redis: false,
};

let atom = config.to_atom();
```

---

## 宏 DSL

### 概述

宏 DSL 提供最简洁的语法，直接使用 AutoLang 语法：

```rust
use auto_lang::{value, atom, node};

// value! - 返回 Value
let val = value!{
    config {
        version: "1.0",
        debug: true,
    }
};

// atom! - 返回 Atom
let atom = atom!{
    config {
        version: "1.0",
        debug: true,
    }
};

// node! - 返回 Node
let node = node!{
    config {
        version: "1.0",
        debug: true,
    }
};
```

### value! 宏

#### 语法

```rust
use auto_lang::value;

// 节点
let val = value!{
    config {
        version: "1.0",
        debug: true,
    }
};

// 数组
let val = value![1, 2, 3, 4, 5];

// 对象
let val = value!{name: "Alice", age: 30};

// 嵌套
let val = value!{
    config {
        version: "1.0",
        database {
            host: "localhost",
            port: 5432,
        },
    }
};
```

#### 变量插值

使用 `#{var}` 语法引用外部变量：

```rust
use auto_lang::value;

let count: i32 = 10;
let name: &str = "test";
let active: bool = true;

let val = value!{
    name: #{name},
    count: #{count},
    active: #{active},
    version: 2,           // 字面量
    debug: true,          // 布尔字面量
};

// 支持的类型
// - i32, u32, i64, u64 -> Int/Uint
// - f64 -> Double
// - f32 -> Float
// - bool -> Bool
// - &str, String -> Str
```

### atom! 宏

与 `value!` 类似，但返回 `Atom` 类型：

```rust
use auto_lang::atom;

// 节点
let atom = atom!{
    config {
        version: "1.0",
        debug: true,
    }
};

// 数组
let atom = atom![1, 2, 3];

// 对象
let atom = atom!{name: "Alice", age: 30};
```

### node! 宏

返回 `Node` 类型，自动提取第一个子节点：

```rust
use auto_lang::node;

let node = node!{
    config {
        version: "1.0",
        debug: true,
    }
};

// node 是 Node 类型，不是 Atom
assert_eq!(node.name, "config");
```

### 多行语句

宏支持多行语句和变量定义：

```rust
use auto_lang::atom;

let atom = atom!{
    let name = "Bob";
    let age = 25;
    {name: name, age: age}
};

// 结果：Obj {name: "Bob", age: 25}
```

---

## 插值功能

### ToAutoValue Trait

插值功能使用 `ToAutoValue` trait 将 Rust 类型转换为 `Value`：

```rust
use auto_val::{Value, ToAutoValue};

// 基本类型
let x: i32 = 42;
let val = x.to_auto_value();  // Value::Int(42)

let y: f64 = 3.14;
let val = y.to_auto_value();  // Value::Double(3.14)

let z: bool = true;
let val = z.to_auto_value();  // Value::Bool(true)

let s: &str = "hello";
let val = s.to_auto_value();  // Value::Str("hello")
```

### 支持的类型

| Rust 类型 | Value 类型 | 说明 |
|-----------|-----------|------|
| `i32`, `i64` | `Value::Int` | 有符号整数 |
| `u32`, `u64` | `Value::Uint` | 无符号整数 |
| `f64` | `Value::Double` | 双精度浮点 |
| `f32` | `Value::Float` | 单精度浮点 |
| `bool` | `Value::Bool` | 布尔值 |
| `&str`, `String` | `Value::Str` | 字符串 |
| `Value` | 自身 | Identity |

### 插值示例

```rust
use auto_lang::value;

// 简单插值
let count = 10;
let val = value!{count: #{count}};

// 多个插值
let name = "Alice";
let age = 30;
let active = true;
let val = value!{
    name: #{name},
    age: #{age},
    active: #{active},
};

// 混合字面量和插值
let port = 8080;
let val = value!{
    host: "localhost",   // 字面量
    port: #{port},       // 插值
    debug: true,         // 布尔字面量
    version: 2,          // 数字字面量
};
```

---

## 完整示例

### 示例 1: Web 服务器配置

```rust
use auto_lang::value;
use auto_val::Value;

fn build_server_config(
    host: &str,
    port: u32,
    use_ssl: bool,
    worker_count: u32,
) -> Value {
    value!{
        server {
            host: #{host},
            port: #{port},
            ssl: #{use_ssl},
            workers: #{worker_count},
            timeout: 30,
            keep_alive: true,
        },
        logging {
            level: "info",
            format: "json",
        }
    }
}

let config = build_server_config("0.0.0.0", 8080, true, 4);
```

### 示例 2: 应用配置（Builder 模式）

```rust
use auto_val::{Node, Value};
use auto_lang::atom::Atom;

struct AppConfig {
    name: String,
    version: String,
    debug: bool,
    db_enabled: bool,
    cache_enabled: bool,
}

impl AppConfig {
    fn to_atom(&self) -> Atom {
        Node::builder("app")
            .prop("name", &self.name)
            .prop("version", &self.version)
            .prop("debug", self.debug)
            .kid_if(self.db_enabled, Node::builder("database")
                .prop("host", "localhost")
                .prop("port", 5432)
                .build()
            )
            .kid_if(self.cache_enabled, Node::builder("cache")
                .prop("type", "redis")
                .prop("ttl", 3600)
                .build()
            )
            .build_atom()
    }
}

let config = AppConfig {
    name: "MyApp".to_string(),
    version: "1.0.0".to_string(),
    debug: true,
    db_enabled: true,
    cache_enabled: false,
};

let atom = config.to_atom();
```

### 示例 3: 数据库连接配置（宏 DSL）

```rust
use auto_lang::atom;

let connections = atom!{
    databases {
        primary {
            driver: "postgresql",
            host: "db1.example.com",
            port: 5432,
            database: "myapp",
            pool_size: 20,
        },
        replica {
            driver: "postgresql",
            host: "db2.example.com",
            port: 5432,
            database: "myapp",
            pool_size: 10,
            read_only: true,
        },
    },
    cache {
        driver: "redis",
        host: "cache.example.com",
        port: 6379,
        db: 0,
    }
};
```

### 示例 4: 动态生成配置（混合方法）

```rust
use auto_lang::{atom, value};
use auto_val::{Node, Obj};

fn build_config(
    env: &str,
    port: u32,
    debug: bool,
) -> (Node, Obj, Value) {
    // 使用链式方法构建 Node
    let node = Node::new("config")
        .with_prop("env", env)
        .with_prop("port", port);

    // 使用 Builder 构建子节点
    let logging = Node::builder("logging")
        .prop("level", if debug { "debug" } else { "info" })
        .prop_if(debug, "verbose", true)
        .build();

    // 使用宏构建完整配置
    let value = value!{
        config {
            environment: #{env},
            port: #{port},
            debug: #{debug},
            features: ["auth", "api", "websocket"],
        }
    };

    (node, logging.props_clone(), value)
}
```

---

## 最佳实践

### 1. 选择合适的 API

```
静态配置 → 宏 DSL（最简洁）
动态配置 → Builder（支持条件构建）
简单操作 → 链式方法（直观易懂）
```

### 2. 利用类型系统

```rust
// ✅ 好：使用类型标注
let config: Atom = atom!{config {version: "1.0"}};

// ❌ 差：让编译器推断（可能出错）
let config = atom!{config {version: "1.0"}};
```

### 3. 条件构建优先使用 Builder

```rust
// ✅ 好：使用 Builder 的条件方法
let node = Node::builder("config")
    .prop_if(feature_enabled, "feature", true)
    .build();

// ❌ 差：使用 if 表达式
let mut builder = Node::builder("config");
if feature_enabled {
    builder = builder.prop("feature", true);
}
let node = builder.build();
```

### 4. 宏内使用插值

```rust
// ✅ 好：使用插值语法
let val = value!{count: #{count}, name: #{name}};

// ❌ 差：手动拼接字符串（类型不安全）
let val = value!{count: count, name: name};  // 可能失败
```

### 5. 复用配置构建逻辑

```rust
// ✅ 好：封装为方法
impl DatabaseConfig {
    fn to_node(&self) -> Node {
        Node::builder("database")
            .prop("host", &self.host)
            .prop("port", self.port)
            .build()
    }
}

// ❌ 差：重复代码
let db1 = Node::builder("database").prop("host", host).prop("port", port).build();
let db2 = Node::builder("database").prop("host", host).prop("port", port).build();
```

### 6. 错误处理

```rust
use auto_val::Value;

// 检查类型
fn get_config_port(config: &Value) -> Result<u32, String> {
    match config {
        Value::Node(node) => {
            match node.get_prop_of("port") {
                Value::Int(port) => Ok(*port as u32),
                Value::Uint(port) => Ok(*port),
                _ => Err("port is not a number".to_string()),
            }
        }
        _ => Err("config is not a node".to_string()),
    }
}
```

---

## 常见问题

### Q1: 何时使用 Atom vs Value vs Node?

**A:**
- **Atom** - 需要表示多种可能类型时（如解析器输出）
- **Value** - 只需要简单值或特定类型时
- **Node** - 明确知道是树节点时

```rust
// Atom: 灵活的类型
fn parse(input: &str) -> Atom { /* ... */ }

// Value: 具体类型
fn get_port(config: &Value) -> u32 { /* ... */ }

// Node: 明确的节点
fn build_config() -> Node { /* ... */ }
```

### Q2: 如何在宏中使用外部变量？

**A:** 使用 `#{var}` 插值语法：

```rust
let name = "Alice";
let age = 30;

// ✅ 正确：使用插值
let val = value!{name: #{name}, age: #{age}};

// ❌ 错误：直接使用标识符（会被解析为字符串）
let val = value!{name: name, age: age};
```

### Q3: 为什么使用 Builder 而不是链式方法？

**A:** Builder 支持条件构建：

```rust
// Builder: 条件构建
let node = Node::builder("config")
    .prop_if(feature_enabled, "feature", true)
    .build();

// 链式方法: 需要运行时判断
let mut node = Node::new("config");
if feature_enabled {
    node = node.with_prop("feature", true);
}
```

### Q4: 宏 DSL 是否有运行时开销？

**A:** 没有。宏在编译期展开，生成的代码与手动构造完全相同：

```rust
// 宏展开前
let val = value!{name: "Alice", age: 30};

// 宏展开后（等价于）
let val = {
    use auto_lang::atom::AtomReader;
    let mut reader = AtomReader::new();
    reader.parse("name: \"Alice\"; age: 30")
        .unwrap()
        .to_value()
};
```

### Q5: 如何实现自定义类型的 ToAutoValue？

**A:** 为你的类型实现 `ToAutoValue` trait：

```rust
use auto_val::{Value, ToAutoValue, Obj};

struct Point {
    x: i32,
    y: i32,
}

impl ToAutoValue for Point {
    fn to_auto_value(&self) -> Value {
        let mut obj = Obj::new();
        obj.set("x", self.x.to_auto_value());
        obj.set("y", self.y.to_auto_value());
        Value::Obj(obj)
    }
}

// 使用
let p = Point { x: 10, y: 20 };
let val = value!{point: #{p}};
```

### Q6: 如何处理嵌套结构？

**A:** 使用嵌套的宏调用或链式方法：

```rust
// 宏 DSL
let val = value!{
    config {
        database {
            host: "localhost",
            port: 5432,
        },
    }
};

// Builder
let node = Node::builder("config")
    .kid(Node::builder("database")
        .prop("host", "localhost")
        .prop("port", 5432)
        .build()
    )
    .build();
```

### Q7: 如何验证配置结构？

**A:** 使用模式匹配和类型检查：

```rust
use auto_lang::atom::Atom;

fn validate_config(config: &Atom) -> Result<(), String> {
    match config {
        Atom::Node(node) => {
            if !node.has_prop("version") {
                return Err("missing 'version' property".to_string());
            }
            if node.kids_len() == 0 {
                return Err("config must have at least one child".to_string());
            }
            Ok(())
        }
        _ => Err("config must be a node".to_string()),
    }
}
```

---

## 相关文档

- [016-atom-macro-dsl.md](../plans/016-atom-macro-dsl.md) - 宏 DSL 实现计划
- [015-atom-builder-api.md](../plans/015-atom-builder-api.md) - Builder API 设计
- [AutoLang 语言文档](https://auto-lang.dev) - AutoLang 语法参考

---

**版权所有 © 2025 AutoLang 项目**
