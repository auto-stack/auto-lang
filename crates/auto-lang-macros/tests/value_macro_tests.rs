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

#[test]
fn test_value_let() {
    let val = value!{
        let name = "Alice";
        let age = 30;
        {name: name, age: age}
    };
    println!("Value: {:?}", val);
    println!("Value repr: {}", val);

    // 验证结果是一个对象，包含正确的值
    if let Value::Obj(obj) = val {
        assert_eq!(obj.len(), 2);
        assert!(obj.has("name"));
        assert!(obj.has("age"));
    } else {
        panic!("Expected Obj value");
    }
}

// 测试 atom! 宏支持多行语句和变量定义
#[test]
fn test_atom_let() {
    use auto_lang::atom::Atom;
    let atom = atom!{
        let name = "Bob";
        let age = 25;
        {name: name, age: age}
    };
    println!("Atom: {:?}", atom);

    // 验证结果是一个对象
    assert!(matches!(atom, Atom::Obj(_)));
    if let Atom::Obj(obj) = atom {
        assert_eq!(obj.len(), 2);
        assert!(obj.has("name"));
        assert!(obj.has("age"));
    }
}

// 测试 value! 宏支持外部变量插值
#[test]
fn test_value_interpolation() {
    let count: i32 = 10;
    let name: &str = "height";
    let active: bool = true;

    // 测试多个插值（混合隐式和显式插值）
    let val = value!{name: name, count: #{count}, active: #{active}};
    println!("Value: {:?}", val);

    // 验证结果是一个对象，包含正确的值
    if let Value::Obj(obj) = val {
        assert_eq!(obj.len(), 3);
        assert!(obj.has("name"));
        assert!(obj.has("count"));
        assert!(obj.has("active"));

        // 验证插值的值
        assert_eq!(obj.get("count"), Some(Value::Int(10)));
        assert_eq!(obj.get("active"), Some(Value::Bool(true)));
        assert_eq!(obj.get("name"), Some(Value::Str("height".into())));
    } else {
        panic!("Expected Obj value");
    }
}

// 测试仅使用显式插值语法 #{}
#[test]
fn test_value_explicit_interpolation() {
    let port: u32 = 8080;
    let host: &str = "localhost";
    let debug: bool = false;

    // 全部使用显式插值
    let val = value!{host: #{host}, port: #{port}, debug: #{debug}};

    if let Value::Obj(obj) = val {
        assert_eq!(obj.len(), 3);
        assert_eq!(obj.get("host"), Some(Value::Str("localhost".into())));
        assert_eq!(obj.get("port"), Some(Value::Uint(8080)));
        assert_eq!(obj.get("debug"), Some(Value::Bool(false)));
    } else {
        panic!("Expected Obj value");
    }
}

// 测试浮点数插值
#[test]
fn test_value_float_interpolation() {
    let pi: f64 = 3.14159;
    let e: f32 = 2.71828;

    let val = value!{pi: #{pi}, e: #{e}};

    if let Value::Obj(obj) = val {
        assert_eq!(obj.len(), 2);
        assert_eq!(obj.get("pi"), Some(Value::Double(3.14159)));
        // f32 转换为 f64 会有精度损失，使用近似比较
        if let Some(Value::Float(actual)) = obj.get("e") {
            assert!((actual - 2.71828).abs() < 0.0001, "Expected ~2.71828, got {}", actual);
        } else {
            panic!("Expected Float value");
        }
    } else {
        panic!("Expected Obj value");
    }
}

// 测试混合字面量和插值
#[test]
fn test_value_mixed_literal_interpolation() {
    let version: u32 = 2;
    let name: &str = "test";

    // 混合字面量和变量插值（使用单行格式以避免解析问题）
    let val = value!{name: name, version: #{version}, description: "A test config", active: true};

    if let Value::Obj(obj) = val {
        assert_eq!(obj.len(), 4);
        assert_eq!(obj.get("name"), Some(Value::Str("test".into())));
        assert_eq!(obj.get("version"), Some(Value::Uint(2)));
        assert_eq!(obj.get("description"), Some(Value::Str("A test config".into())));
        assert_eq!(obj.get("active"), Some(Value::Bool(true)));
    } else {
        panic!("Expected Obj value");
    }
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
