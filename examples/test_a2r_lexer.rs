use auto_lang::trans::rust::transpile_rust;
use std::fs;

fn main() {
    let d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let files = vec![
        "auto/lib/pos.at",
        "auto/lib/token.at",
        "auto/lib/error.at",
        "auto/lib/lexer.at",
    ];
    let mut code = String::new();
    for f in &files {
        let path = d.join(f);
        code.push_str(&fs::read_to_string(path).unwrap());
        code.push('\n');
    }
    code.push_str(r#"
fn main() {
    let src = "let x = 42"
    let result = tokenize(src)
    print(result)
}
"#);

    match transpile_rust("lexer_char_test", &code) {
        Ok(sink) => {
            for (name, bytes) in &sink.files {
                let content = String::from_utf8_lossy(bytes);
                println!("=== {} ===", name);
                println!("{}", content);
            }
        }
        Err(e) => {
            eprintln!("a2r error: {}", e);
            std::process::exit(1);
        }
    }
}
