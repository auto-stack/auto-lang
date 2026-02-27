// 测试宏展开并打印生成的字符串
use auto_lang::atom::AtomReader;

fn main() {
    // 测试1: 简单节点
    println!("=== Test 1: Simple node ===");
    let code1 = "config { version: \"1.0\"; debug: true; }";
    println!("Trying to parse: {}", code1);
    let mut reader = AtomReader::new();
    match reader.parse(code1) {
        Ok(atom) => println!("Success: {:?}", atom),
        Err(e) => println!("Failed: {}", e),
    }

    // 测试2: 数组
    println!("\n=== Test 2: Array ===");
    let code2 = "[1, 2, 3, 4, 5]";
    println!("Trying to parse: {}", code2);
    let mut reader = AtomReader::new();
    match reader.parse(code2) {
        Ok(atom) => println!("Success: {:?}", atom),
        Err(e) => println!("Failed: {}", e),
    }

    // 测试3: 对象
    println!("\n=== Test 3: Object ===");
    let code3 = "{ name: \"Alice\"; age: 30 }";
    println!("Trying to parse: {}", code3);
    let mut reader = AtomReader::new();
    match reader.parse(code3) {
        Ok(atom) => println!("Success: {:?}", atom),
        Err(e) => println!("Failed: {}", e),
    }

    // 测试4: 嵌套结构
    println!("\n=== Test 4: Nested ===");
    let code4 = "config { version: \"1.0\"; database { host: \"localhost\"; port: 5432; } }";
    println!("Trying to parse: {}", code4);
    let mut reader = AtomReader::new();
    match reader.parse(code4) {
        Ok(atom) => println!("Success: {:?}", atom),
        Err(e) => println!("Failed: {}", e),
    }
}
