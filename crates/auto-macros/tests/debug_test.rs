// 测试宏生成的字符串
fn main() {
    // 测试简单的字符串转换
    let test_str = token_stream_to_string_test();

    println!("Generated string: {:?}", test_str);
}

// 模拟 token_stream_to_string 的行为
fn token_stream_to_string_test() -> String {
    // 模拟输入: name: "Alice", age: 30
    // TokenStream 会产生: Ident(name), Punct(:), Literal("Alice"), Punct(,), Ident(age), Punct(:), Literal(30)

    let mut result = String::new();

    // 这些 token 之间没有空白字符
    result.push_str("name");      // Ident
    result.push(':');             // Punct
    result.push_str("\"Alice\""); // Literal
    result.push(',');             // Punct
    result.push_str("age");       // Ident
    result.push(':');             // Punct
    result.push_str("30");        // Literal

    result
}
