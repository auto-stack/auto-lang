use auto_lang::trans::rust::transpile_rust;
use std::fs;

fn main() {
    // Increase stack size for large transpilation (step-02 is ~560 lines)
    let builder = std::thread::Builder::new().stack_size(8 * 1024 * 1024);
    let handler = builder.spawn(run).unwrap();
    handler.join().unwrap()
}

fn run() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_path = manifest_dir
        .parent().unwrap()       // crates/
        .parent().unwrap()       // auto-lang/
        .parent().unwrap()       // autostack/
        .join("auto-code-rs/snapshots/step-02-tool-system/main.at");
    let src_path = src_path.canonicalize().unwrap_or_else(|e| {
        eprintln!("Error: cannot find step-02 main.at at {:?}: {}", src_path, e);
        std::process::exit(1);
    });
    let code = fs::read_to_string(&src_path).unwrap_or_else(|e| {
        eprintln!("Error reading {:?}: {}", src_path, e);
        std::process::exit(1);
    });

    println!("=== Transpiling step-02-tool-system/main.at ===");

    match transpile_rust("step02_tool_system", &code) {
        Ok(sink) => {
            if !sink.header.is_empty() {
                println!("=== header ===");
                println!("{}", String::from_utf8_lossy(&sink.header));
            }
            if !sink.body.is_empty() {
                println!("=== body ===");
                println!("{}", String::from_utf8_lossy(&sink.body));
            }
            println!("=== transpilation OK ===");
        }
        Err(e) => {
            eprintln!("a2r transpilation error: {}", e);
            std::process::exit(1);
        }
    }
}
