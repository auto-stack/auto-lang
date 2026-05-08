use std::fs;

fn main() -> std::io::Result<()> {
    for entry in fs::read_dir(".")? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_str().unwrap();
        if name_str.ends_with(".rs") {
            println!("Found Rust file: {}", name_str);
        }
    }
    Ok(())
}
