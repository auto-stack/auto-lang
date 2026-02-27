use auto_lang::{atom, node};
use auto_lang::atom::Atom;
use auto_lang::atom::AtomReader;

// 调试测试：查看 AtomReader 期望的语法格式
#[test]
fn test_debug_reader_syntax() {
    let mut reader = AtomReader::new();

    // 测试分号分隔的属性
    let code = "config { version: \"1.0\"; debug: true; }";
    println!("Testing: {}", code);
    let result = reader.parse(code);
    println!("Result: {:?}", result);
    assert!(result.is_ok());

    // 测试数组
    let code2 = "[1, 2, 3]";
    println!("\nTesting array: {}", code2);
    let result2 = reader.parse(code2);
    println!("Result: {:?}", result2);
    println!("Result type: {:?}", std::mem::discriminant(&result2.unwrap()));
}

// 调试测试：打印宏生成的代码
#[test]
fn test_debug_macro_expansion() {
    // 手动模拟宏应该生成的字符串
    // 输入: atom!{ config { version: "1.0", debug: true, } }
    // 期望输出: config { version: "1.0"; debug: true; }

    let mut reader = AtomReader::new();

    // 注意：宏展开后的字符串需要手动测试
    // 由于宏展开发生在编译期，这里手动构造测试字符串
    let manual_generated = "config { version: \"1.0\"; debug: true; }";
    println!("Manual generated: {}", manual_generated);
    let result = reader.parse(manual_generated);
    println!("Result: {:?}", result);
    assert!(result.is_ok());
}

// 测试各种语法格式
#[test]
fn test_debug_various_formats() {
    let mut reader = AtomReader::new();

    // 测试1: 带空格的 config
    let code1 = "config { version: \"1.0\"; debug: true; }";
    println!("\nTest 1: {}", code1);
    let r1 = reader.parse(code1);
    println!("Result: {:?}", r1.is_ok());

    // 测试2: 数组
    let code2 = "[1, 2, 3, 4, 5]";
    println!("\nTest 2: {}", code2);
    let r2 = reader.parse(code2);
    println!("Result: {:?}", r2.is_ok());

    // 测试3: 对象（花括号但无节点名）
    let code3 = "{ name: \"Alice\"; age: 30 }";
    println!("\nTest 3: {}", code3);
    let r3 = reader.parse(code3);
    println!("Result: {:?}", r3.is_ok());

    // 测试4: 嵌套节点
    let code4 = "config { version: \"1.0\"; database { host: \"localhost\"; port: 5432; } }";
    println!("\nTest 4: {}", code4);
    let r4 = reader.parse(code4);
    println!("Result: {:?}", r4.is_ok());
}

// 调试测试：检查宏生成的代码格式
#[test]
fn test_debug_macro_output() {
    // 数组测试
    let arr = atom![1, 2, 3];
    println!("Array atom: {:?}", arr);

    // 对象测试
    let obj = atom!{name: "Alice", age: 30};
    println!("Object atom: {:?}", obj);
}

#[test]
fn test_atom_simple_node() {
    let atom = atom!{
        config {
            version: "1.0",
            debug: true,
        }
    };

    assert!(atom.is_node());
    if let Atom::Node(node) = atom {
        assert_eq!(node.name, "config");
    }
}

#[test]
fn test_node_simple() {
    let node = node!{
        config {
            version: "1.0",
        }
    };

    assert_eq!(node.name, "config");
}

#[test]
fn test_atom_array() {
    let atom = atom![1, 2, 3, 4, 5];
    assert!(atom.is_array());
}

#[test]
fn test_atom_object() {
    let atom = atom!{name: "Alice", age: 30};
    assert!(atom.is_obj());
}

#[test]
fn test_nested_structure() {
    let atom = atom!{
        config {
            version: "1.0",
            database {
                host: "localhost",
                port: 5432,
            },
        }
    };

    assert!(atom.is_node());
}
