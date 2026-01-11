//! Atom/Node/Array/Obj 宏 DSL
//!
//! 提供声明式宏语法用于构造 Atom、Node、Array 和 Obj 值。
//!
//! # 示例
//!
//! ## node! 宏
//!
//! ```rust
//! use auto_lang::node;
//! use auto_val::Node;
//!
//! // 简单节点
//! let node = node!("config");
//!
//! // 带属性
//! let node = node!("config", version="1.0", debug=true);
//!
//! // 带参数
//! let node = node!("db", arg="my_db", host="localhost", port=5432);
//! ```
//!
//! ## atom! 宏
//!
//! ```rust
//! use auto_lang::atom;
//! use auto_lang::Atom;
//!
//! // 节点
//! let atom = atom!(node("config"));
//!
//! // 数组
//! let atom = atom!(array[1, 2, 3, 4, 5]);
//!
//! // 对象
//! let atom = atom!(obj(name="Alice", age="30"));
//! ```
//!
//! ## atoms! 简化宏
//!
//! ```rust
//! use auto_lang::{atoms, node};
//! use auto_lang::Atom;
//!
//! // 字符串 -> 节点
//! let atom = atoms!("config");
//!
//! // 带属性的节点
//! let atom = atoms!("config", version="1.0", debug=true);
//!
//! // 数组
//! let atom = atoms!([1, 2, 3, 4, 5]);
//! ```

// ========== node! 宏 ==========

/// 创建 Node 的声明式宏
///
/// # 语法变体
///
/// ## 1. 简单节点
///
/// ```rust
/// use auto_lang::node;
///
/// let node = node!("config");
/// assert_eq!(node.name, "config");
/// ```
///
/// ## 2. 带参数
///
/// ```rust
/// use auto_lang::node;
///
/// let node = node!("db", arg="my_db");
/// assert_eq!(node.main_arg().to_astr(), "my_db");
/// ```
///
/// ## 3. 带属性
///
/// ```rust
/// use auto_lang::node;
/// use auto_val::Value;
///
/// let node = node!("config", version="1.0", debug=true);
/// assert_eq!(node.get_prop_of("version"), Value::Str("1.0".into()));
/// assert_eq!(node.get_prop_of("debug"), Value::Bool(true));
/// ```
///
/// ## 4. 带参数和属性
///
/// ```rust
/// use auto_lang::node;
/// use auto_val::Value;
///
/// let node = node!("db", arg="my_db", host="localhost", port=5432);
/// assert_eq!(node.main_arg().to_astr(), "my_db");
/// assert_eq!(node.get_prop_of("host"), Value::Str("localhost".into()));
/// ```
#[macro_export]
macro_rules! node {
    // 带参数和属性: node!("name", arg="val", key1=val1, ...) - 必须在最前面
    ($name:expr, arg=$arg:expr, $($key:ident = $value:expr),* $(,)?) => {
        auto_val::Node::new($name)
            .with_arg($arg)
            $(
                .with_prop(stringify!($key), $value)
            )*
    };

    // 带参数: node!("name", arg="value")
    ($name:expr, arg=$arg:expr $(,)?) => {
        auto_val::Node::new($name).with_arg($arg)
    };

    // 带属性: node!("name", key1=val1, key2=val2, ...)
    ($name:expr, $($key:ident = $value:expr),* $(,)?) => {
        auto_val::Node::new($name)
            $(
                .with_prop(stringify!($key), $value)
            )*
    };

    // 基础节点: node!("name")
    ($name:expr $(,)?) => {
        auto_val::Node::new($name)
    };
}

// ========== atom! 宏 ==========

/// 创建 Atom 的声明式宏
///
/// # 语法变体
///
/// ## 1. 节点
///
/// ```rust
/// use auto_lang::atom;
/// use auto_lang::Atom;
///
/// let atom = atom!(node("config"));
/// assert!(atom.is_node());
/// ```
///
/// ## 2. 带属性的节点
///
/// ```rust
/// use auto_lang::atom;
/// use auto_lang::Atom;
/// use auto_lang::node;
///
/// let atom = atom!(node("config", version="1.0", debug=true));
/// ```
///
/// ## 3. 数组
///
/// ```rust
/// use auto_lang::atom;
/// use auto_lang::Atom;
///
/// let atom = atom!(array[1, 2, 3, 4, 5]);
/// ```
///
/// ## 4. 对象
///
/// ```rust
/// use auto_lang::atom;
/// use auto_lang::Atom;
///
/// let atom = atom!(obj(name="Alice", age="30"));
/// ```
#[macro_export]
macro_rules! atom {
    // 节点: atom!(node("name"))
    (node ( $name:expr $(,)?) ) => {
        Atom::Node(auto_val::Node::new($name))
    };

    // 带属性的节点: atom!(node("name", key1=val1, key2=val2, ...))
    (node ( $name:expr, $($key:ident = $value:expr),* $(,)? )) => {
        Atom::Node(node!($name, $($key = $value),*))
    };

    // 数组: atom!(array[1, 2, 3, ...])
    (array [ $($value:expr),* $(,)? ]) => {
        Atom::Array(auto_val::Array::from(vec![$($value),*]))
    };

    // 对象: atom!(obj(key1=val1, key2=val2, ...))
    (obj ( $($key:ident = $value:expr),* $(,)? )) => {
        Atom::Obj(auto_val::Obj::from_pairs([
            $((stringify!($key), $value)),*
        ]))
    };
}

// ========== atoms! 简化宏 ==========

/// 极简 Atom 构造宏 - 自动推断类型
///
/// # 语法变体
///
/// ## 1. 数组
///
/// ```rust
/// use auto_lang::atoms;
/// use auto_lang::Atom;
///
/// let atom = atoms!([1, 2, 3, 4, 5]);
/// ```
///
/// ## 2. 字符串 -> 节点
///
/// ```rust
/// use auto_lang::atoms;
/// use auto_lang::Atom;
///
/// let atom = atoms!("config");
/// assert!(atom.is_node());
/// ```
///
/// ## 3. 带属性的节点
///
/// ```rust
/// use auto_lang::atoms;
/// use auto_lang::Atom;
/// use auto_lang::node;
///
/// let atom = atoms!("config", version="1.0", debug=true);
/// assert!(atom.is_node());
/// ```
///
/// # 类型推断规则
///
/// - 方括号 (`[...]`) → Array (优先匹配)
/// - 字符串字面量 (`"name"`) → Node
/// - 键值对 (`key=val`) → 节点属性
#[macro_export]
macro_rules! atoms {
    // 数组优先匹配: atoms!([1, 2, 3, ...])
    ([ $($value:expr),* $(,)? ]) => {
        Atom::Array(auto_val::Array::from(vec![$($value),*]))
    };

    // 字符串 -> 节点: atoms!("config")
    ($name:expr $(,)?) => {
        Atom::Node(auto_val::Node::new($name))
    };

    // 节点带属性: atoms!("name", key1=val1, key2=val2, ...)
    ($name:expr, $($key:ident = $value:expr),* $(,)?) => {
        Atom::Node(node!($name, $($key = $value),*))
    };
}

// ========== 测试 ==========

#[cfg(test)]
mod tests {
    use crate::atom::Atom;
    use auto_val::{Array, Node, Obj, Value};

    // ========== node! 宏测试 ==========

    #[test]
    fn test_node_simple() {
        let node = node!("config");
        assert_eq!(node.name, "config");
    }

    #[test]
    fn test_node_with_arg() {
        let node = node!("db", arg="my_db");
        assert_eq!(node.name, "db");
        assert_eq!(node.main_arg().to_astr(), "my_db");
    }

    #[test]
    fn test_node_with_props() {
        let node = node!("config", version="1.0", debug=true);

        assert_eq!(node.name, "config");
        assert_eq!(node.get_prop_of("version"), Value::Str("1.0".into()));
        assert_eq!(node.get_prop_of("debug"), Value::Bool(true));
    }

    #[test]
    fn test_node_with_arg_and_props() {
        let node = node!("db", arg="my_db", host="localhost", port=5432);

        assert_eq!(node.name, "db");
        assert_eq!(node.main_arg().to_astr(), "my_db");
        assert_eq!(node.get_prop_of("host"), Value::Str("localhost".into()));
        assert_eq!(node.get_prop_of("port"), Value::Int(5432));
    }

    // ========== atom! 宏测试 ==========

    #[test]
    fn test_atom_node() {
        let atom = atom!(node("config"));
        assert!(atom.is_node());

        if let Atom::Node(node) = atom {
            assert_eq!(node.name, "config");
        }
    }

    #[test]
    fn test_atom_node_with_props() {
        let atom = atom!(node("config", version="1.0", debug=true));

        assert!(atom.is_node());

        if let Atom::Node(node) = atom {
            assert_eq!(node.name, "config");
            assert_eq!(node.get_prop_of("version"), Value::Str("1.0".into()));
            assert_eq!(node.get_prop_of("debug"), Value::Bool(true));
        }
    }

    #[test]
    fn test_atom_array() {
        let atom = atom!(array[1, 2, 3, 4, 5]);
        assert!(atom.is_array());

        if let Atom::Array(arr) = atom {
            assert_eq!(arr.len(), 5);
            assert_eq!(arr.values[0], Value::Int(1));
            assert_eq!(arr.values[4], Value::Int(5));
        }
    }

    #[test]
    fn test_atom_array_empty() {
        // 空数组需要类型标注
        let arr = auto_val::Array::from(vec![0i32; 0]);
        let atom = Atom::Array(arr);

        assert!(atom.is_array());

        if let Atom::Array(arr) = atom {
            assert_eq!(arr.len(), 0);
        }
    }

    #[test]
    fn test_atom_obj() {
        // 对象的所有值需要是相同类型或明确标注
        let atom = atom!(obj(name="Alice", age="30"));
        assert!(atom.is_obj());

        if let Atom::Obj(obj) = atom {
            assert_eq!(obj.get_str_of("name"), "Alice");
            assert_eq!(obj.get_str_of("age"), "30");
        }
    }

    #[test]
    fn test_atom_obj_empty() {
        // 使用类型标注的空对组
        let pairs: [(&str, i32); 0] = [];
        let obj = auto_val::Obj::from_pairs(pairs);
        let atom = Atom::Obj(obj);

        assert!(atom.is_obj());

        if let Atom::Obj(obj) = atom {
            assert_eq!(obj.len(), 0);
        }
    }

    // ========== atoms! 简化宏测试 ==========

    #[test]
    fn test_atoms_string_to_node() {
        let atom = atoms!("config");
        assert!(atom.is_node());

        if let Atom::Node(node) = atom {
            assert_eq!(node.name, "config");
        }
    }

    #[test]
    fn test_atoms_node_with_props() {
        let atom = atoms!("config", version="1.0", debug=true);

        assert!(atom.is_node());

        if let Atom::Node(node) = atom {
            assert_eq!(node.name, "config");
            assert_eq!(node.get_prop_of("version"), Value::Str("1.0".into()));
            assert_eq!(node.get_prop_of("debug"), Value::Bool(true));
        }
    }

    #[test]
    fn test_atoms_array() {
        let atom = atoms!([1, 2, 3, 4, 5]);
        assert!(atom.is_array());

        if let Atom::Array(arr) = atom {
            assert_eq!(arr.len(), 5);
            assert_eq!(arr.values[0], Value::Int(1));
            assert_eq!(arr.values[4], Value::Int(5));
        }
    }

    #[test]
    fn test_atoms_array_empty() {
        // 空数组需要类型标注
        let arr = auto_val::Array::from(vec![0i32; 0]);
        let atom = Atom::Array(arr);

        assert!(atom.is_array());

        if let Atom::Array(arr) = atom {
            assert_eq!(arr.len(), 0);
        }
    }

    // ========== 集成测试 ==========

    #[test]
    fn test_nested_structure() {
        // 简化的嵌套测试（暂不支持子节点嵌套）
        let _config_node = node!("config", version="1.0");
        let _db_node = node!("database", host="localhost", port=5432);
        let _data_arr = atom!(array[1, 2, 3]);

        // 基本节点测试
        let atom = atom!(node("root", version="1.0"));
        assert!(atom.is_node());
    }

    #[test]
    fn test_realistic_config() {
        let atom = atoms!("config", version="1.0", debug=true);

        assert!(atom.is_node());

        if let Atom::Node(node) = atom {
            assert_eq!(node.name, "config");
            assert_eq!(node.get_prop_of("version"), Value::Str("1.0".into()));
            assert_eq!(node.get_prop_of("debug"), Value::Bool(true));
        }
    }
}
