use auto_lang::trans::rust::transpile_rust;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Increase stack size for large transpilation (step-03 is ~530 lines)
    let builder = std::thread::Builder::new().stack_size(8 * 1024 * 1024);
    let handler = builder.spawn(run).unwrap();
    handler.join().unwrap()
}

fn run() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_path = manifest_dir
        .parent().unwrap()       // crates/
        .parent().unwrap()       // auto-lang/
        .parent().unwrap()       // autostack/
        .join("auto-code-rs/snapshots/step-03-agent-runtime/main.at");
    let src_path = src_path.canonicalize().unwrap_or_else(|e| {
        eprintln!("Error: cannot find step-03 main.at at {:?}: {}", src_path, e);
        std::process::exit(1);
    });
    let code = fs::read_to_string(&src_path).unwrap_or_else(|e| {
        eprintln!("Error reading {:?}: {}", src_path, e);
        std::process::exit(1);
    });

    let out_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()       // crates/
        .parent().unwrap()       // auto-lang/
        .parent().unwrap()       // autostack/
        .join("auto-code-rs/target/step03_test/src/main.rs");

    println!("=== Transpiling step-03-agent-runtime/main.at ===");

    match transpile_rust("step03_agent_runtime", &code) {
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
