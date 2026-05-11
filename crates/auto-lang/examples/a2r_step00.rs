use auto_lang::trans::rust::transpile_rust;
use std::fs;

fn main() {
    let child = std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let source = fs::read_to_string("d:/autostack/auto-code-rs/snapshots/step-00-api-minimal/main.at").unwrap();
            let mut result = match transpile_rust("step00", &source) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Transpile error: {}", e);
                    std::process::exit(1);
                }
            };
            let rust_code = String::from_utf8(result.done().unwrap().clone()).unwrap();
            let out_path = "d:/autostack/auto-code-rs/snapshots/step-00-api-minimal/tmp-a2r/src/main.rs";
            fs::write(out_path, &rust_code).unwrap();
            eprintln!("Transpiled {} bytes to {}", rust_code.len(), out_path);
        })
        .expect("Failed to spawn thread");
    child.join().expect("Thread panicked");
}
