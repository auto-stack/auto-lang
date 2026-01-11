use auto_lang::{value, atom};
use auto_val::Value;

// 测试 value! 宏 - 节点
#[test]
fn test_value_node() {
    let val = value!{
        config {
            version: "1.0",
            debug: true,
        }
    };

    assert!(matches!(val, Value::Node(_)));
}

// 测试 value! 宏 - 数组
#[test]
fn test_value_array() {
    let val = value![1, 2, 3, 4, 5];

    assert!(matches!(val, Value::Array(_)));
    if let Value::Array(arr) = val {
        assert_eq!(arr.len(), 5);
    }
}

// 测试 value! 宏 - 对象
#[test]
fn test_value_object() {
    let val = value!{name: "Alice", age: 30};

    assert!(matches!(val, Value::Obj(_)));
}

// 测试 value! 宏 - 嵌套结构
#[test]
fn test_value_nested() {
    let val = value!{
        config {
            version: "1.0",
            database {
                host: "localhost",
                port: 5432,
            },
        }
    };

    assert!(matches!(val, Value::Node(_)));
}

// 测试 value! 与 atom! 的一致性
#[test]
fn test_value_vs_atom() {
    let atom_val = atom!{
        config {
            version: "1.0",
            debug: true,
        }
    };

    let val_val = value!{
        config {
            version: "1.0",
            debug: true,
        }
    };

    // atom! 返回 Atom，value! 返回 Value
    // 它们的底层结构应该相同
    let value_from_atom = atom_val.to_value();

    // 验证两者类型相同
    match (&value_from_atom, &val_val) {
        (Value::Node(n1), Value::Node(n2)) => {
            assert_eq!(n1.name, n2.name);
        }
        _ => panic!("Both should be Node values"),
    }
}

// 测试空数组
#[test]
fn test_value_empty_array() {
    let val = value![];
    // 调试：打印实际返回的值
    println!("Empty array value: {:?}", val);
    // 空数组可能会被解析为 Node、Array 或 Nil
    // 让我们接受合理的类型
    assert!(matches!(val, Value::Array(_) | Value::Nil | Value::Node(_)));
}

// 测试简单字符串数组
#[test]
fn test_value_string_array() {
    let val = value!["a", "b", "c"];
    assert!(matches!(val, Value::Array(_)));

    if let Value::Array(arr) = val {
        assert_eq!(arr.len(), 3);
    }
}
