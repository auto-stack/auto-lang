# Atom API Guide

**Author**: AutoLang Team
**Updated**: 2025-01-11
**Status**: Complete

## Table of Contents

1. [Overview](#overview)
2. [Atom Type System](#atom-type-system)
3. [Three-Layer API Architecture](#three-layer-api-architecture)
4. [Method Chaining API](#method-chaining-api)
5. [Builder API](#builder-api)
6. [Macro DSL](#macro-dsl)
7. [Interpolation Feature](#interpolation-feature)
8. [Complete Examples](#complete-examples)
9. [Best Practices](#best-practices)
10. [FAQ](#faq)

---

## Overview

AutoLang's `Atom` type system provides three different API styles for building tree-like data structures:

1. **Method Chaining** - Simple and intuitive, for basic operations
2. **Builder Pattern** - Flexible and powerful, supports conditional building
3. **Macro DSL** - Most concise syntax, zero-overhead declarative style

This guide will help you:
- Understand the Atom type system
- Master all three API styles
- Choose the right API for your use case
- Learn interpolation and advanced features

---

## Atom Type System

### Atom Enum

`Atom` is the core type of AutoLang, representing various data structures:

```rust
use auto_lang::atom::Atom;
use auto_val::{Value, Node, Array, Obj};

// Atom variants
pub enum Atom {
    Node(Node),    // Tree node
    Array(Array),  // Array
    Obj(Obj),      // Object (key-value pairs)
    Value(Value),  // Simple value (Int, Bool, Str, etc.)
}
```

### Node Type

`Node` represents a node in a tree structure:

```rust
use auto_val::Node;

let node = Node::new("config")
    .with_prop("version", "1.0")
    .with_prop("debug", true);

// Structure
Node {
    name: "config",           // Node name
    args: Args::new(),        // Positional arguments
    props: Obj::new(),        // Properties collection
    kids: Kids::new(),        // Children collection
    // ...
}
```

### Array and Obj

```rust
use auto_val::{Array, Obj, Value};

// Array
let arr = Array::from(vec![
    Value::Int(1),
    Value::Int(2),
    Value::Int(3),
]);

// Object
let obj = Obj::new()
    .set("name", Value::Str("Alice".into()))
    .set("age", Value::Int(30));
```

---

## Three-Layer API Architecture

### Comparison Table

| Feature | Method Chaining | Builder | Macro DSL |
|---------|----------------|---------|-----------|
| **Conciseness** | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Conditional Building** | ⭐⭐ | ⭐⭐⭐⭐⭐ | ❌ |
| **Type Safety** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Runtime Flexibility** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ❌ |
| **Zero Overhead** | ✅ | ✅ | ✅ |
| **Learning Curve** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ |

### Selection Guide

```
Simple use cases   → Macro DSL (most concise)
Dynamic scenarios  → Method chaining (most flexible)
Complex scenarios  → Builder (most powerful)
```

---

## Method Chaining API

### Basic Usage

Method chaining is the most direct approach, suitable for simple build operations:

```rust
use auto_lang::atom::Atom;
use auto_val::{Node, Value};

// Create node
let node = Node::new("config")
    .with_prop("version", "1.0")
    .with_prop("debug", true);

let atom = Atom::Node(node);

// Add children
let node = Node::new("root")
    .with_kid(Node::new("child1"))
    .with_kid(Node::new("child2"));

// Update properties
let node = Node::new("config")
    .with_prop("host", "localhost")
    .with_prop("port", 8080)
    .update_prop("port", 9090);  // Update existing property
```

### Common Methods

#### Node Methods

```rust
// Creation
Node::new(name: &str)

// Property operations
.with_prop(key, value)           // Add/update property
.with_props(props: Obj)          // Batch add properties
.get_prop_of(key) -> Value       // Get property value
.has_prop(key) -> bool           // Check if property exists
.update_prop(key, value)         // Update property (add if not exists)
.remove_prop(key)                // Remove property
.props_len() -> usize            // Property count

// Child operations
.with_kid(node)                   // Add child node
.with_kids(vec)                   // Batch add children
.add_kid_unified(node)            // Add any type as child
.kids_len() -> usize              // Child count
.get_kid(index) -> Option<&Kid>  // Get child at index

// Argument operations
.with_arg(value)                  // Add positional argument
.args_len() -> usize              // Argument count

// Conversion
.to_value() -> Value              // Convert to Value
.to_atom() -> Atom                // Convert to Atom
```

#### Array Methods

```rust
use auto_val::Array;

// Creation
Array::new()
Array::from(vec)
Array::with_capacity(capacity)

// Element operations
.push(value)                      // Add element
.len() -> usize                   // Length
.is_empty() -> bool               // Is empty
.get(index) -> Option<&Value>     // Get element

// Conversion
.to_value() -> Value
.to_atom() -> Atom
```

#### Obj Methods

```rust
use auto_val::Obj;

// Creation
Obj::new()
Obj::from_pairs([(key, value), ...])

// Property operations
.set(key, value)                  // Set property
.get(key) -> Option<&Value>       // Get property
.has(key) -> bool                 // Check if key exists
.remove(key) -> Option<Value>     // Remove property
.len() -> usize                    // Property count

// Conversion
.to_value() -> Value
.to_atom() -> Atom
```

### Example: Building Configuration Tree

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

### Basic Usage

Builder pattern provides more flexible building with conditional support:

```rust
use auto_lang::atom::Atom;
use auto_val::Node;

// Using Builder
let node = Node::builder("config")
    .prop("version", "1.0")
    .prop("debug", true)
    .build();

let atom = Atom::Node(node);
```

### Conditional Building

Builder's biggest advantage is conditional building:

```rust
use auto_val::Node;

let use_ssl = true;
let port = 8080;

let node = Node::builder("server")
    .prop("host", "localhost")
    .prop("port", port)
    .prop_if(use_ssl, "ssl", true)           // Conditionally add property
    .prop_if(use_ssl, "cert", "/path/to/cert")
    .build();
```

### Common Methods

```rust
// Create Builder
Node::builder(name)

// Property operations
.prop(key, value)                 // Add property
.prop_if(condition, key, value)   // Conditionally add property
.props(props)                     // Batch add
.props_if(condition, props)       // Conditional batch add

// Child operations
.kid(node)                        // Add child
.kid_if(condition, node)          // Conditionally add child
.kids(vec)                        // Batch add
.kids_if(condition, vec)          // Conditional batch add

// Argument operations
.arg(value)                       // Add argument
.arg_if(condition, value)         // Conditional add argument

// Build
.build() -> Node                   // Build Node
.build_atom() -> Atom             // Build Atom
.build_value() -> Value           // Build Value
```

### Example: Dynamic Configuration

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

## Macro DSL

### Overview

Macro DSL provides the most concise syntax using AutoLang syntax directly:

```rust
use auto_lang::{value, atom, node};

// value! - Returns Value
let val = value!{
    config {
        version: "1.0",
        debug: true,
    }
};

// atom! - Returns Atom
let atom = atom!{
    config {
        version: "1.0",
        debug: true,
    }
};

// node! - Returns Node
let node = node!{
    config {
        version: "1.0",
        debug: true,
    }
};
```

### value! Macro

#### Syntax

```rust
use auto_lang::value;

// Node
let val = value!{
    config {
        version: "1.0",
        debug: true,
    }
};

// Array
let val = value![1, 2, 3, 4, 5];

// Object
let val = value!{name: "Alice", age: 30};

// Nested
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

#### Variable Interpolation

Use `#{var}` syntax to reference external variables:

```rust
use auto_lang::value;

let count: i32 = 10;
let name: &str = "test";
let active: bool = true;

let val = value!{
    name: #{name},
    count: #{count},
    active: #{active},
    version: 2,           // Literal
    debug: true,          // Boolean literal
};

// Supported types
// - i32, u32, i64, u64 -> Int/Uint
// - f64 -> Double
// - f32 -> Float
// - bool -> Bool
// - &str, String -> Str
```

### atom! Macro

Similar to `value!` but returns `Atom` type:

```rust
use auto_lang::atom;

// Node
let atom = atom!{
    config {
        version: "1.0",
        debug: true,
    }
};

// Array
let atom = atom![1, 2, 3];

// Object
let atom = atom!{name: "Alice", age: 30};
```

### node! Macro

Returns `Node` type, automatically extracts first child:

```rust
use auto_lang::node;

let node = node!{
    config {
        version: "1.0",
        debug: true,
    }
};

// node is Node type, not Atom
assert_eq!(node.name, "config");
```

### Multi-line Statements

Macros support multi-line statements and variable definitions:

```rust
use auto_lang::atom;

let atom = atom!{
    let name = "Bob";
    let age = 25;
    {name: name, age: age}
};

// Result: Obj {name: "Bob", age: 25}
```

---

## Interpolation Feature

### ToAutoValue Trait

Interpolation uses `ToAutoValue` trait to convert Rust types to `Value`:

```rust
use auto_val::{Value, ToAutoValue};

// Basic types
let x: i32 = 42;
let val = x.to_auto_value();  // Value::Int(42)

let y: f64 = 3.14;
let val = y.to_auto_value();  // Value::Double(3.14)

let z: bool = true;
let val = z.to_auto_value();  // Value::Bool(true)

let s: &str = "hello";
let val = s.to_auto_value();  // Value::Str("hello")
```

### Supported Types

| Rust Type | Value Type | Description |
|-----------|-----------|-------------|
| `i32`, `i64` | `Value::Int` | Signed integer |
| `u32`, `u64` | `Value::Uint` | Unsigned integer |
| `f64` | `Value::Double` | Double precision float |
| `f32` | `Value::Float` | Single precision float |
| `bool` | `Value::Bool` | Boolean |
| `&str`, `String` | `Value::Str` | String |
| `Value` | Self | Identity |

### Interpolation Examples

```rust
use auto_lang::value;

// Simple interpolation
let count = 10;
let val = value!{count: #{count}};

// Multiple interpolations
let name = "Alice";
let age = 30;
let active = true;
let val = value!{
    name: #{name},
    age: #{age},
    active: #{active},
};

// Mix literals and interpolation
let port = 8080;
let val = value!{
    host: "localhost",   // Literal
    port: #{port},       // Interpolation
    debug: true,         // Boolean literal
    version: 2,          // Number literal
};
```

---

## Complete Examples

### Example 1: Web Server Configuration

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

### Example 2: Application Configuration (Builder Pattern)

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

### Example 3: Database Connection Configuration (Macro DSL)

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

### Example 4: Dynamic Configuration Generation (Mixed Approach)

```rust
use auto_lang::{atom, value};
use auto_val::{Node, Obj};

fn build_config(
    env: &str,
    port: u32,
    debug: bool,
) -> (Node, Obj, Value) {
    // Use method chaining for Node
    let node = Node::new("config")
        .with_prop("env", env)
        .with_prop("port", port);

    // Use Builder for child node
    let logging = Node::builder("logging")
        .prop("level", if debug { "debug" } else { "info" })
        .prop_if(debug, "verbose", true)
        .build();

    // Use macro for complete config
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

## Best Practices

### 1. Choose the Right API

```
Static configuration → Macro DSL (most concise)
Dynamic configuration → Builder (supports conditionals)
Simple operations → Method chaining (intuitive)
```

### 2. Leverage Type System

```rust
// ✅ Good: Use type annotations
let config: Atom = atom!{config {version: "1.0"}};

// ❌ Bad: Let compiler infer (may cause errors)
let config = atom!{config {version: "1.0"}};
```

### 3. Use Builder for Conditional Building

```rust
// ✅ Good: Use Builder's conditional methods
let node = Node::builder("config")
    .prop_if(feature_enabled, "feature", true)
    .build();

// ❌ Bad: Use if expressions
let mut builder = Node::builder("config");
if feature_enabled {
    builder = builder.prop("feature", true);
}
let node = builder.build();
```

### 4. Use Interpolation in Macros

```rust
// ✅ Good: Use interpolation syntax
let val = value!{count: #{count}, name: #{name}};

// ❌ Bad: Direct identifier use (parses as string)
let val = value!{count: count, name: name};  // May fail
```

### 5. Reuse Configuration Building Logic

```rust
// ✅ Good: Encapsulate as methods
impl DatabaseConfig {
    fn to_node(&self) -> Node {
        Node::builder("database")
            .prop("host", &self.host)
            .prop("port", self.port)
            .build()
    }
}

// ❌ Bad: Duplicate code
let db1 = Node::builder("database").prop("host", host).prop("port", port).build();
let db2 = Node::builder("database").prop("host", host).prop("port", port).build();
```

### 6. Error Handling

```rust
use auto_val::Value;

// Check types
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

## FAQ

### Q1: When to use Atom vs Value vs Node?

**A:**
- **Atom** - When you need to represent multiple possible types (e.g., parser output)
- **Value** - When you only need simple values or specific types
- **Node** - When you know it's explicitly a tree node

```rust
// Atom: Flexible type
fn parse(input: &str) -> Atom { /* ... */ }

// Value: Specific type
fn get_port(config: &Value) -> u32 { /* ... */ }

// Node: Explicit node
fn build_config() -> Node { /* ... */ }
```

### Q2: How to use external variables in macros?

**A:** Use the `#{var}` interpolation syntax:

```rust
let name = "Alice";
let age = 30;

// ✅ Correct: Use interpolation
let val = value!{name: #{name}, age: #{age}};

// ❌ Wrong: Direct identifiers (parsed as strings)
let val = value!{name: name, age: age};
```

### Q3: Why use Builder instead of method chaining?

**A:** Builder supports conditional building:

```rust
// Builder: Conditional building
let node = Node::builder("config")
    .prop_if(feature_enabled, "feature", true)
    .build();

// Method chaining: Requires runtime check
let mut node = Node::new("config");
if feature_enabled {
    node = node.with_prop("feature", true);
}
```

### Q4: Does macro DSL have runtime overhead?

**A:** No. Macros expand at compile time, generating code identical to manual construction:

```rust
// Before macro expansion
let val = value!{name: "Alice", age: 30};

// After expansion (equivalent to)
let val = {
    use auto_lang::atom::AtomReader;
    let mut reader = AtomReader::new();
    reader.parse("name: \"Alice\"; age: 30")
        .unwrap()
        .to_value()
};
```

### Q5: How to implement ToAutoValue for custom types?

**A:** Implement the `ToAutoValue` trait for your type:

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

// Usage
let p = Point { x: 10, y: 20 };
let val = value!{point: #{p}};
```

### Q6: How to handle nested structures?

**A:** Use nested macro calls or method chaining:

```rust
// Macro DSL
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

### Q7: How to validate configuration structure?

**A:** Use pattern matching and type checking:

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

## Related Documentation

- [016-atom-macro-dsl.md](../plans/016-atom-macro-dsl.md) - Macro DSL implementation plan
- [015-atom-builder-api.md](../plans/015-atom-builder-api.md) - Builder API design
- [AutoLang Language Documentation](https://auto-lang.dev) - AutoLang syntax reference

---

**Copyright © 2025 AutoLang Project**
