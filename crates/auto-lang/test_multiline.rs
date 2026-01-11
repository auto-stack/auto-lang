use auto_lang::atom::AtomReader;

#[test]
fn test_multiline_let() {
    let mut reader = AtomReader::new();

    let code = r#"
let name = "Alice"
let age = 30
{name: name, age: age}
"#;

    let result = reader.parse(code);
    println!("Result: {:?}", result);
    assert!(result.is_ok());
}
