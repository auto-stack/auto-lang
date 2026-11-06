//! PLAN-988 fixture emitter. Generated Rust is compiled and executed by the
//! scoped consumer gate; this emitter alone is not a passing runtime check.
fn main() {
    let output = std::path::PathBuf::from(std::env::args().nth(1).expect("output directory"));
    std::fs::create_dir_all(&output).unwrap();
    let path = output
        .join("bytes.bin")
        .to_string_lossy()
        .replace('\\', "/");
    let mut source = format!("fn bytes() []int {{ return [0, 127, 128, 255] }}\nfn main() {{ let bs = bytes()\nlet rc = File.write_bytes({}, bs) }}", serde_json::to_string(&path).unwrap());
    if std::env::args().nth(2).as_deref() == Some("empty") {
        source = format!(
            "fn main() {{ let rc = File.write_bytes({}, []) }}",
            serde_json::to_string(&path).unwrap()
        );
    }
    std::fs::write(output.join("input.at"), &source).unwrap();
    let mut sink = auto_lang_core::trans::rust::transpile_rust("byte_consumer", &source).unwrap();
    let generated = String::from_utf8(sink.done().unwrap().clone()).unwrap();
    let qualified = auto_generation_core::shared_generation::qualify_a2r_std(generated);
    std::fs::write(output.join("main.rs"), qualified).unwrap();
}
