use auto_lang::trans::rust::transpile_rust;
use std::fs;
use std::path::PathBuf;

fn main() {
    let builder = std::thread::Builder::new().stack_size(8 * 1024 * 1024);
    let handler = builder.spawn(run).unwrap();
    handler.join().unwrap()
}

fn run() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_path = manifest_dir
        .parent().unwrap()
        .parent().unwrap()
        .parent().unwrap()
        .join("auto-code-rs/snapshots/step-04-cli-entry/main.at");
    let src_path = src_path.canonicalize().unwrap_or_else(|e| {
        eprintln!("Error: cannot find step-04 main.at at {:?}: {}", src_path, e);
        std::process::exit(1);
    });
    let code = fs::read_to_string(&src_path).unwrap_or_else(|e| {
        eprintln!("Error reading {:?}: {}", src_path, e);
        std::process::exit(1);
    });

    let out_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()
        .parent().unwrap()
        .parent().unwrap()
        .join("auto-code-rs/target/step04_test/src/main.rs");

    println!("=== Transpiling step-04-cli-entry/main.at ===");

    match transpile_rust("step04_cli_entry", &code) {
        Ok(sink) => {
            let mut output = String::new();
            if !sink.header.is_empty() {
                output.push_str(&String::from_utf8_lossy(&sink.header));
                output.push('\n');
            }
            if !sink.body.is_empty() {
                output.push_str(&String::from_utf8_lossy(&sink.body));
            }
            fs::write(&out_path, &output).unwrap_or_else(|e| {
                eprintln!("Error writing {:?}: {}", out_path, e);
                std::process::exit(1);
            });
            println!("=== transpilation OK -> {:?} ===", out_path);
        }
        Err(e) => {
            eprintln!("a2r transpilation error: {}", e);
            std::process::exit(1);
        }
    }
}
