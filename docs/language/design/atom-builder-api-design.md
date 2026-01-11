# Atom 树状结构构造 API 设计

## 问题分析

### 当前 API 的不足

现有的 Atom/Node/Array/Obj API 在构建深层嵌套树状结构时存在以下问题：

#### 1. 冗长的命令式风格

```rust
// 当前方式 - 冗长且需要多次 mut 绑定
let mut root = Node::new("config");
root.set_prop("version", "1.0");
root.set_prop("debug", true);

let mut database = Node::new("database");
database.set_prop("host", "localhost");
database.set_prop("port", 5432);

let mut redis = Node::new("redis");
redis.set_prop("host", "127.0.0.1");
redis.set_prop("port", 6379);

root.add_kid(database);
root.add_kid(redis);

let atom = Atom::node(root);
```

#### 2. 无链式调用能力

所有修改方法返回 `()`，无法流畅地构建结构：

```rust
// 期望：链式调用
let node = Node::new("config")
    .with_prop("version", "1.0")
    .with_prop("debug", true)
    .with_child(Node::new("db").with_prop("port", 5432));

// 实际：不支持
```

#### 3. 无批量操作

无法一次性添加多个属性或子节点：

```rust
// 当前方式
let mut node = Node::new("config");
node.set_prop("a", 1);
node.set_prop("b", 2);
node.set_prop("c", 3);
node.set_prop("d", 4);
node.set_prop("e", 5);

// 期望方式
let node = Node::new("config")
    .with_props([("a", 1), ("b", 2), ("c", 3), ("d", 4), ("e", 5)]);
```

#### 4. 无 DSL 宏支持

没有声明式的宏语法，类似于 `serde_json` 的 `json!` 宏。

## 设计方案

### 阶段 1: 扩展现有类型 (最小侵入)

在现有类型上添加便利方法，保持向后兼容。

#### 1.1 添加链式方法到 `Node`

```rust
impl Node {
    // === 创建时设置属性 ===

    /// 创建节点并设置单个属性
    pub fn with_prop(mut self, key: impl Into<ValueKey>, value: impl Into<Value>) -> Self {
        self.set_prop(key, value);
        self
    }

    /// 创建节点并设置多个属性
    pub fn with_props(mut self, props: impl IntoIterator<Item = (impl Into<ValueKey>, impl Into<Value>)>) -> Self {
        for (key, value) in props.into_iter() {
            self.set_prop(key, value);
        }
        self
    }

    /// 创建节点并从对象添加属性
    pub fn with_obj(mut self, obj: Obj) -> Self {
        self.merge_obj(obj);
        self
    }

    // === 创建时添加子节点 ===

    /// 创建节点并添加子节点
    pub fn with_child(mut self, node: Node) -> Self {
        self.add_kid(node);
        self
    }

    /// 创建节点并添加多个子节点
    pub fn with_children(mut self, children: impl IntoIterator<Item = Node>) -> Self {
        for child in children {
            self.add_kid(child);
        }
        self
    }

    /// 创建节点并添加索引子节点
    pub fn with_node_kid(mut self, index: i32, node: Node) -> Self {
        self.add_node_kid(index, node);
        self
    }

    // === 创建时设置文本 ===

    /// 创建节点并设置文本内容
    pub fn with_text(mut self, text: impl Into<AutoStr>) -> Self {
        self.text = text.into();
        self
    }

    // === 创建时设置参数 ===

    /// 创建节点并添加位置参数
    pub fn with_arg(mut self, arg: impl Into<Value>) -> Self {
        self.set_main_arg(arg);
        self
    }

    /// 创建节点并添加命名参数
    pub fn with_named_arg(mut self, name: impl Into<ValueKey>, value: impl Into<Value>) -> Self {
        let key = name.into();
        self.add_arg_unified(key.clone(), value);
        self
    }
}
```

**使用示例**：

```rust
// 链式调用构建配置树
let config = Node::new("config")
    .with_prop("version", "1.0")
    .with_prop("debug", true)
    .with_child(
        Node::new("database")
            .with_prop("host", "localhost")
            .with_prop("port", 5432)
    )
    .with_child(
        Node::new("redis")
            .with_prop("host", "127.0.0.1")
            .with_prop("port", 6379)
    );

// 批量设置属性
let node = Node::new("person")
    .with_props([
        ("name", "Alice"),
        ("age", 30),
        ("city", "Boston"),
    ]);

// 批量添加子节点
let root = Node::new("root")
    .with_children([
        Node::new("child1"),
        Node::new("child2"),
        Node::new("child3"),
    ]);
```

#### 1.2 扩展 `Array` 类型

```rust
impl Array {
    /// 创建数组并添加元素（链式）
    pub fn with(mut self, value: impl Into<Value>) -> Self {
        self.push(value);
        self
    }

    /// 创建数组并添加多个元素
    pub fn with_values(mut self, values: impl IntoIterator<Item = impl Into<Value>>) -> Self {
        for value in values {
            self.push(value);
        }
        self
    }

    /// 从元素构建数组（替代 from_vec 的链式版本）
    pub fn from(values: impl IntoIterator<Item = impl Into<Value>>) -> Self {
        let mut arr = Self::new();
        for value in values {
            arr.push(value);
        }
        arr
    }
}
```

**使用示例**：

```rust
// 链式构建数组
let arr = Array::new()
    .with(1)
    .with(2)
    .with(3)
    .with(4)
    .with(5);

// 从迭代器构建
let arr = Array::from(vec![1, 2, 3, 4, 5]);
let arr = Array::from(0..10);
```

#### 1.3 扩展 `Obj` 类型

```rust
impl Obj {
    /// 创建对象并设置键值（链式）
    pub fn with(mut self, key: impl Into<ValueKey>, value: impl Into<Value>) -> Self {
        self.set(key, value);
        self
    }

    /// 创建对象并设置多个键值
    pub fn with_pairs(mut self, pairs: impl IntoIterator<Item = (impl Into<ValueKey>, impl Into<Value>)>) -> Self {
        for (key, value) in pairs {
            self.set(key, value);
        }
        self
    }

    /// 从键值对迭代器构建对象
    pub fn from_pairs(pairs: impl IntoIterator<Item = (impl Into<ValueKey>, impl Into<Value>)>) -> Self {
        let mut obj = Self::new();
        for (key, value) in pairs {
            obj.set(key, value);
        }
        obj
    }
}
```

**使用示例**：

```rust
// 链式构建对象
let obj = Obj::new()
    .with("name", "Alice")
    .with("age", 30)
    .with("city", "Boston");

// 从迭代器构建
let obj = Obj::from_pairs([
    ("name", "Alice"),
    ("age", 30),
    ("city", "Boston"),
]);
```

#### 1.4 扩展 `Atom` 类型

```rust
impl Atom {
    // === Node 便利构造器 ===

    /// 创建带属性的节点 Atom
    pub fn node_with_props(
        name: impl Into<AutoStr>,
        props: impl IntoIterator<Item = (impl Into<ValueKey>, impl Into<Value>)>
    ) -> Self {
        let node = Node::new(name).with_props(props);
        Atom::Node(node)
    }

    /// 创建带子节点的节点 Atom
    pub fn node_with_children(
        name: impl Into<AutoStr>,
        children: impl IntoIterator<Item = Node>
    ) -> Self {
        let node = Node::new(name).with_children(children);
        Atom::Node(node)
    }

    /// 创建完整的节点 Atom（属性 + 子节点）
    pub fn node_full(
        name: impl Into<AutoStr>,
        props: impl IntoIterator<Item = (impl Into<ValueKey>, impl Into<Value>)>,
        children: impl IntoIterator<Item = Node>
    ) -> Self {
        let node = Node::new(name)
            .with_props(props)
            .with_children(children);
        Atom::Node(node)
    }

    // === Array 便利构造器 ===

    /// 从值创建数组 Atom
    pub array_from(values: impl IntoIterator<Item = impl Into<Value>>) -> Self {
        let array = Array::from(values);
        Atom::Array(array)
    }

    // === Obj 便利构造器 ===

    /// 从键值对创建对象 Atom
    pub fn obj_from(pairs: impl IntoIterator<Item = (impl Into<ValueKey>, impl Into<Value>)>) -> Self {
        let obj = Obj::from_pairs(pairs);
        Atom::Obj(obj)
    }
}
```

**使用示例**：

```rust
// 便利构造器
let atom = Atom::node_with_props("config", [
    ("version", "1.0"),
    ("debug", true),
]);

let atom = Atom::node_full("config",
    [("version", "1.0")],
    [Node::new("db"), Node::new("cache")]
);

let atom = Atom::array_from(vec![1, 2, 3, 4, 5]);
let atom = Atom::obj_from([("name", "Alice"), ("age", 30)]);
```

### 阶段 2: Builder 模式 (更强大)

提供专门的 Builder 类型，支持更复杂的构建场景。

#### 2.1 NodeBuilder

```rust
/// Node 构建器 - 支持链式调用和复杂嵌套
pub struct NodeBuilder {
    name: AutoStr,
    node: Node,
}

impl NodeBuilder {
    /// 创建新的构建器
    pub fn new(name: impl Into<AutoStr>) -> Self {
        let name = name.into();
        Self {
            name: name.clone(),
            node: Node::new(name),
        }
    }

    // === 属性设置 ===

    /// 设置属性（链式）
    pub fn prop(mut self, key: impl Into<ValueKey>, value: impl Into<Value>) -> Self {
        self.node.set_prop(key, value);
        self
    }

    /// 批量设置属性
    pub fn props(mut self, props: impl IntoIterator<Item = (impl Into<ValueKey>, impl Into<Value>)>) -> Self {
        for (key, value) in props {
            self.node.set_prop(key, value);
        }
        self
    }

    /// 从对象合并属性
    pub fn merge(mut self, obj: Obj) -> Self {
        self.node.merge_obj(obj);
        self
    }

    // === 子节点添加 ===

    /// 添加子节点
    pub fn child(mut self, node: Node) -> Self {
        self.node.add_kid(node);
        self
    }

    /// 添加由构建器创建的子节点
    pub fn child_builder(mut self, builder: NodeBuilder) -> Self {
        self.node.add_kid(builder.build());
        self
    }

    /// 批量添加子节点
    pub fn children(mut self, children: impl IntoIterator<Item = Node>) -> Self {
        for child in children {
            self.node.add_kid(child);
        }
        self
    }

    /// 添加条件子节点
    pub fn child_if(self, condition: bool, node: Node) -> Self {
        if condition {
            self.child(node)
        } else {
            self
        }
    }

    /// 添加可选子节点
    pub fn child_option(self, node: Option<Node>) -> Self {
        if let Some(node) = node {
            self.child(node)
        } else {
            self
        }
    }

    // === 其他设置 ===

    /// 设置文本内容
    pub fn text(mut self, text: impl Into<AutoStr>) -> Self {
        self.node.text = text.into();
        self
    }

    /// 设置主参数
    pub fn arg(mut self, arg: impl Into<Value>) -> Self {
        self.node.set_main_arg(arg);
        self
    }

    /// 条件性设置
    pub fn prop_if(self, condition: bool, key: impl Into<ValueKey>, value: impl Into<Value>) -> Self {
        if condition {
            self.prop(key, value)
        } else {
            self
        }
    }

    // === 构建和转换 ===

    /// 构建节点
    pub fn build(self) -> Node {
        self.node
    }

    /// 构建 Atom
    pub fn build_atom(self) -> Atom {
        Atom::Node(self.build())
    }
}

// 从 AutoStr 直接创建 Builder
impl From<AutoStr> for NodeBuilder {
    fn from(name: AutoStr) -> Self {
        Self::new(name)
    }
}

impl From<&str> for NodeBuilder {
    fn from(name: &str) -> Self {
        Self::new(name)
    }
}
```

**使用示例**：

```rust
// 使用 Builder 构建复杂配置
let config = NodeBuilder::new("config")
    .prop("version", "1.0")
    .prop("debug", true)
    .child(
        NodeBuilder::new("database")
            .prop("host", "localhost")
            .prop("port", 5432)
            .prop("ssl", true)
    )
    .child(
        NodeBuilder::new("redis")
            .prop("host", "127.0.0.1")
            .prop("port", 6379)
    )
    .child_if(
        feature_enabled,
        NodeBuilder::new("monitoring").prop("active", true)
    )
    .build_atom();

// 条件性构建
let node = NodeBuilder::new("server")
    .prop("host", "localhost")
    .prop_if(ssl_enabled, "ssl", true)
    .prop_if(has_auth, "auth", "Bearer token")
    .build();
```

#### 2.2 AtomBuilder (复合构建器)

```rust
/// Atom 构建器 - 支持 Node/Array/Obj
pub enum AtomBuilder {
    Node(NodeBuilder),
    Array(ArrayBuilder),
    Obj(ObjBuilder),
}

impl AtomBuilder {
    /// 创建节点构建器
    pub fn node(name: impl Into<AutoStr>) -> Self {
        AtomBuilder::Node(NodeBuilder::new(name))
    }

    /// 创建数组构建器
    pub fn array() -> Self {
        AtomBuilder::Array(ArrayBuilder::new())
    }

    /// 创建对象构建器
    pub fn obj() -> Self {
        AtomBuilder::Obj(ObjBuilder::new())
    }

    /// 构建 Atom
    pub fn build(self) -> Atom {
        match self {
            AtomBuilder::Node(builder) => builder.build_atom(),
            AtomBuilder::Array(builder) => builder.build_atom(),
            AtomBuilder::Obj(builder) => builder.build_atom(),
        }
    }
}

// Array 构建器
pub struct ArrayBuilder {
    array: Array,
}

impl ArrayBuilder {
    pub fn new() -> Self {
        Self {
            array: Array::new(),
        }
    }

    pub fn value(mut self, value: impl Into<Value>) -> Self {
        self.array.push(value);
        self
    }

    pub fn values(mut self, values: impl IntoIterator<Item = impl Into<Value>>) -> Self {
        for value in values {
            self.array.push(value);
        }
        self
    }

    pub fn build(self) -> Array {
        self.array
    }

    pub fn build_atom(self) -> Atom {
        Atom::Array(self.build())
    }
}

// Obj 构建器
pub struct ObjBuilder {
    obj: Obj,
}

impl ObjBuilder {
    pub fn new() -> Self {
        Self {
            obj: Obj::new(),
        }
    }

    pub fn pair(mut self, key: impl Into<ValueKey>, value: impl Into<Value>) -> Self {
        self.obj.set(key, value);
        self
    }

    pub fn pairs(mut self, pairs: impl IntoIterator<Item = (impl Into<ValueKey>, impl Into<Value>)>) -> Self {
        for (key, value) in pairs {
            self.obj.set(key, value);
        }
        self
    }

    pub fn build(self) -> Obj {
        self.obj
    }

    pub fn build_atom(self) -> Atom {
        Atom::Obj(self.build())
    }
}
```

### 阶段 3: 宏 DSL (最声明式)

提供类似 `json!` 的声明式宏语法。

#### 3.1 node! 宏

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
///
/// // 带子节点
/// let node = node!("config" {
///     database("database") {
///         host: "localhost",
///         port: 5432,
///     },
///     redis("redis") {
///         host: "127.0.0.1",
///         port: 6379,
///     },
/// });
///
/// // 带参数
/// let node = node!("db"("my_db") {
///     host: "localhost",
/// });
///
/// // 混合属性和子节点
/// let node = node!("root" {
///     name: "test",
///     child1("child1") { value: 1 },
///     child2("child2") { value: 2 },
/// });
/// ```
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

    // 带子节点: node!("name" { child("name") { ... }, ... })
    ($name:expr { $($child:ident ( $child_name:expr ) { $($child_inner:tt)* }),* $(,)? }) => {
        Node::new($name)
            $(
                .with_child(node!($child ( $child_name ) { $($child_inner)* }))
            )*
    };

    // 混合属性和子节点
    ($name:expr {
        $($key:ident : $value:expr),* $(,)?;
        $($child:ident ( $child_name:expr ) { $($child_inner:tt)* }),* $(,)?
    }) => {
        Node::new($name)
            $(
                .with_prop(stringify!($key), $value)
            )*
            $(
                .with_child(node!($child ( $child_name ) { $($child_inner)* }))
            )*
    };
}
```

**使用示例**：

```rust
// 简单节点
let node = node!("config");

// 带属性
let node = node!("config" {
    version: "1.0",
    debug: true,
});

// 带参数
let node = node!("db"("my_db") {
    host: "localhost",
    port: 5432,
});

// 带子节点
let node = node!("config" {
    database("database") {
        host: "localhost",
        port: 5432,
    },
    redis("redis") {
        host: "127.0.0.1",
        port: 6379,
    },
});
```

#### 3.2 atom! 宏

```rust
/// 创建 Atom 的声明式宏
///
/// # 示例
///
/// ```rust
/// use auto_lang::atom;
///
/// // 节点
/// let atom = atom!(node("config"));
///
/// // 数组
/// let atom = atom!(array[1, 2, 3, 4, 5]);
///
/// // 对象
/// let atom = atom!(obj { name: "Alice", age: 30 });
///
/// // 嵌套
/// let atom = atom!(node("config") {
///     database("db") { host: "localhost" },
///     data: array[1, 2, 3],
///     meta: obj { version: "1.0" },
/// });
/// ```
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

#### 3.3 简化版 atoms! 宏

```rust
/// 极简 Atom 构造宏 - 自动推断类型
///
/// # 示例
///
/// ```rust
/// use auto_lang::atoms;
///
/// // 节点
/// let atom = atoms!("config");
///
/// // 带属性的节点
/// let atom = atoms!("config" { version: "1.0", debug: true });
///
/// // 数组
/// let atom = atoms!([1, 2, 3, 4, 5]);
///
/// // 对象
/// let atom = atoms!({ name: "Alice", age: 30 });
///
/// // 嵌套
/// let atom = atoms!("root" {
///     db("database") { host: "localhost" },
///     items: [1, 2, 3],
///     meta: { version: "1.0" },
/// });
/// ```
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

## 对比示例

### 构建复杂配置树

#### 当前方式 (命令式)

```rust
let mut config = Node::new("config");
config.set_prop("version", "1.0");
config.set_prop("debug", true);

let mut database = Node::new("database");
database.set_prop("host", "localhost");
database.set_prop("port", 5432);
database.set_prop("ssl", true);

let mut redis = Node::new("redis");
redis.set_prop("host", "127.0.0.1");
redis.set_prop("port", 6379);

config.add_kid(database);
config.add_kid(redis);

let atom = Atom::node(config);
```

#### 阶段 1: 链式方法

```rust
let atom = Atom::node(
    Node::new("config")
        .with_props([("version", "1.0"), ("debug", true)])
        .with_child(
            Node::new("database")
                .with_props([("host", "localhost"), ("port", 5432), ("ssl", true)])
        )
        .with_child(
            Node::new("redis")
                .with_props([("host", "127.0.0.1"), ("port", 6379)])
        )
);
```

#### 阶段 2: Builder 模式

```rust
let atom = NodeBuilder::new("config")
    .props([("version", "1.0"), ("debug", true)])
    .child(
        NodeBuilder::new("database")
            .props([("host", "localhost"), ("port", 5432), ("ssl", true)])
    )
    .child(
        NodeBuilder::new("redis")
            .props([("host", "127.0.0.1"), ("port", 6379)])
    )
    .build_atom();
```

#### 阶段 3: 宏 DSL

```rust
let atom = atom!(node("config") {
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

// 或使用简化版
let atom = atoms!("config" {
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
```

## 实现优先级

### 高优先级 (立即实现)

1. ✅ **链式方法扩展** (阶段 1)
   - `Node::with_prop()`, `with_props()`, `with_child()`, `with_children()`
   - `Array::with()`, `with_values()`, `from()`
   - `Obj::with()`, `with_pairs()`, `from_pairs()`
   - `Atom::node_with_props()`, `node_full()`, `array_from()`, `obj_from()`

   **理由**:
   - 最小侵入性，仅扩展现有类型
   - 完全向后兼容
   - 实现简单 (~300 LOC)
   - 立即提升 API 易用性

### 中优先级 (短期实现)

2. ⏳ **Builder 模式** (阶段 2)
   - `NodeBuilder` 类型
   - `ArrayBuilder`, `ObjBuilder`, `AtomBuilder`
   - 条件性方法 (`child_if`, `prop_if`)

   **理由**:
   - 提供更强大的构建能力
   - 支持条件性构建
   - 适合复杂嵌套场景

### 低优先级 (长期考虑)

3. 🔮 **宏 DSL** (阶段 3)
   - `node!` 宏
   - `atom!` 宏
   - `atoms!` 简化宏

   **理由**:
   - 最声明式的语法
   - 需要仔细设计以避免宏膨胀
   - 需要处理宏匹配边缘情况

## 测试策略

### 单元测试

每个新方法都需要单元测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_prop() {
        let node = Node::new("test").with_prop("key", "value");
        assert_eq!(node.get_prop_of("key"), Value::Str("value".into()));
    }

    #[test]
    fn test_with_props_multiple() {
        let node = Node::new("test")
            .with_props([("a", 1), ("b", 2), ("c", 3)]);

        assert_eq!(node.get_prop_of("a"), Value::Int(1));
        assert_eq!(node.get_prop_of("b"), Value::Int(2));
        assert_eq!(node.get_prop_of("c"), Value::Int(3));
    }

    #[test]
    fn test_with_children() {
        let node = Node::new("root")
            .with_children([
                Node::new("child1"),
                Node::new("child2"),
                Node::new("child3"),
            ]);

        assert_eq!(node.kids_len(), 3);
        assert!(node.has_nodes("child1"));
        assert!(node.has_nodes("child2"));
        assert!(node.has_nodes("child3"));
    }

    #[test]
    fn test_nested_chain() {
        let node = Node::new("root")
            .with_child(
                Node::new("level1")
                    .with_child(
                        Node::new("level2")
                            .with_prop("deep", true)
                    )
            );

        let level1 = node.get_nodes("level1");
        assert_eq!(level1.len(), 1);

        let level2 = level1[0].get_nodes("level2");
        assert_eq!(level2.len(), 1);
        assert_eq!(level2[0].get_prop_of("deep"), Value::Bool(true));
    }

    #[test]
    fn test_array_from() {
        let arr = Array::from(vec![1, 2, 3, 4, 5]);
        assert_eq!(arr.len(), 5);
        assert_eq!(arr.values[0], Value::Int(1));
        assert_eq!(arr.values[4], Value::Int(5));
    }

    #[test]
    fn test_array_with_chain() {
        let arr = Array::new()
            .with(1)
            .with(2)
            .with(3)
            .with(4)
            .with(5);

        assert_eq!(arr.len(), 5);
    }

    #[test]
    fn test_obj_from_pairs() {
        let obj = Obj::from_pairs([
            ("name", "Alice"),
            ("age", 30),
            ("city", "Boston"),
        ]);

        assert_eq!(obj.get_str_of("name"), "Alice");
        assert_eq!(obj.get_int_of("age"), 30);
        assert_eq!(obj.get_str_of("city"), "Boston");
    }

    #[test]
    fn test_atom_convenience() {
        let atom = Atom::node_with_props("config", [
            ("version", "1.0"),
            ("debug", true),
        ]);

        assert!(atom.is_node());
        if let Atom::Node(node) = atom {
            assert_eq!(node.name, "config");
            assert_eq!(node.get_prop_of("version"), Value::Str("1.0".into()));
            assert_eq!(node.get_prop_of("debug"), Value::Bool(true));
        }
    }
}
```

### 集成测试

创建实际的配置树构建示例：

```rust
#[test]
fn test_build_realistic_config() {
    let atom = Atom::node_full("config",
        [("version", "1.0"), ("debug", true)],
        [
            Node::new("database").with_props([
                ("host", "localhost"),
                ("port", 5432),
                ("ssl", true),
            ]),
            Node::new("redis").with_props([
                ("host", "127.0.0.1"),
                ("port", 6379),
            ]),
        ]
    );

    // 验证结构
    assert!(atom.is_node());
    if let Atom::Node(node) = atom {
        assert_eq!(node.name, "config");
        assert_eq!(node.kids_len(), 2);
        assert!(node.has_nodes("database"));
        assert!(node.has_nodes("redis"));

        let db = &node.get_nodes("database")[0];
        assert_eq!(db.get_prop_of("host"), Value::Str("localhost".into()));
        assert_eq!(db.get_prop_of("port"), Value::Int(5432));
        assert_eq!(db.get_prop_of("ssl"), Value::Bool(true));
    }
}
```

## 文档

### API 文档

所有公共方法需要完整的 rustdoc 文档：

```rust
impl Node {
    /// 创建节点并设置属性，返回 self 以支持链式调用
    ///
    /// # 参数
    ///
    /// * `key` - 属性键
    /// * `value` - 属性值
    ///
    /// # 返回
    ///
    /// 返回 `Self` 以支持链式调用
    ///
    /// # 示例
    ///
    /// ```rust
    /// use auto_val::Node;
    ///
    /// let node = Node::new("config")
    ///     .with_prop("version", "1.0")
    ///     .with_prop("debug", true);
    ///
    /// assert_eq!(node.get_prop_of("version"), Value::Str("1.0".into()));
    /// ```
    pub fn with_prop(mut self, key: impl Into<ValueKey>, value: impl Into<Value>) -> Self {
        self.set_prop(key, value);
        self
    }
}
```

### 使用指南

创建 `docs/atom-builder-guide.md`：

```markdown
# Atom 构建器使用指南

## 快速开始

### 创建简单节点

```rust
use auto_lang::atom::{Atom, Node};

// 方式 1: 传统方式
let mut node = Node::new("config");
node.set_prop("version", "1.0");

// 方式 2: 链式调用 (推荐)
let node = Node::new("config")
    .with_prop("version", "1.0")
    .with_prop("debug", true);
```

### 创建嵌套结构

```rust
let config = Node::new("config")
    .with_props([("version", "1.0"), ("debug", true)])
    .with_child(
        Node::new("database")
            .with_props([
                ("host", "localhost"),
                ("port", 5432),
                ("ssl", true),
            ])
    )
    .with_child(
        Node::new("redis")
            .with_props([
                ("host", "127.0.0.1"),
                ("port", 6379),
            ])
    );
```

## API 参考

...

## 最佳实践

...
```

## 兼容性

### 向后兼容

所有新方法都是**纯添加**，不修改现有 API：

- ✅ 现有代码继续工作
- ✅ 新方法是现有方法的便捷包装
- ✅ 无破坏性更改

### 升级路径

```rust
// 旧代码 (继续工作)
let mut node = Node::new("config");
node.set_prop("version", "1.0");
node.add_kid(Node::new("child"));

// 新代码 (更简洁)
let node = Node::new("config")
    .with_prop("version", "1.0")
    .with_child(Node::new("child"));
```

## 性能考虑

### 零开销抽象

链式方法和 Builder 模式应该有零运行时开销：

- 编译器内联小方法
- 无额外分配
- 与手动调用相同的机器码

### 基准测试

```rust
#[bench]
fn bench_manual_construction(b: &mut Bencher) {
    b.iter(|| {
        let mut node = Node::new("config");
        node.set_prop("a", 1);
        node.set_prop("b", 2);
        node.set_prop("c", 3);
        node
    });
}

#[bench]
fn bench_chain_construction(b: &mut Bencher) {
    b.iter(|| {
        Node::new("config")
            .with_prop("a", 1)
            .with_prop("b", 2)
            .with_prop("c", 3)
    });
}
```

## 总结

### 实现优先级

1. **高优先级** (立即): 链式方法扩展
2. **中优先级** (短期): Builder 模式
3. **低优先级** (长期): 宏 DSL

### 预期效果

- ✅ 减少构建代码 ~70%
- ✅ 提高可读性
- ✅ 完全向后兼容
- ✅ 零性能开销
- ✅ 渐进式采用

### 文件清单

实现此设计需要修改/创建以下文件：

1. `crates/auto-val/src/node.rs` - 添加链式方法 (~100 行)
2. `crates/auto-val/src/array.rs` - 添加链式方法 (~40 行)
3. `crates/auto-val/src/obj.rs` - 添加链式方法 (~40 行)
4. `crates/auto-lang/src/atom.rs` - 添加便利构造器 (~60 行)
5. `crates/auto-lang/src/builder.rs` - Builder 模式 (~300 行, 可选)
6. `crates/auto-lang/src/macros.rs` - 宏 DSL (~200 行, 可选)
7. `docs/atom-builder-guide.md` - 使用指南
8. `crates/auto-lang/src/atom/builder_tests.rs` - 测试 (~400 行)

**总代码量**: ~540 LOC (必需) + ~900 LOC (可选)
