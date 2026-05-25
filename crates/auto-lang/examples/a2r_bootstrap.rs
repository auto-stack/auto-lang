use auto_lang::trans_rust_merged;
use std::fs;

fn main() {
    let child = std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let result = match trans_rust_merged("auto/lib/") {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Transpile error: {}", e);
                    std::process::exit(1);
                }
            };
            let rust_code = String::from_utf8(result).unwrap();
            let out_path = "tmp/bootstrap.rs";
            fs::write(out_path, &rust_code).unwrap();
            eprintln!("Transpiled {} bytes to {}", rust_code.len(), out_path);
        })
        .expect("Failed to spawn thread");
    child.join().expect("Thread panicked");
}
