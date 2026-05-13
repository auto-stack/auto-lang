use auto_lang::trans::rust::transpile_rust;
use std::fs;

fn main() {
    // Read .at file from first CLI arg or "test.at"
    let path = std::env::args().nth(1).unwrap_or_else(|| "test.at".to_string());
    let code = fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("Error reading {}: {}", path, e);
        std::process::exit(1);
    });
    match transpile_rust("test", &code) {
        Ok(sink) => {
            if !sink.body.is_empty() {
                println!("{}", String::from_utf8_lossy(&sink.body));
            }
        }
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
    }
}
