use std::fs;

fn main() -> std::io::Result<()> {
    for entry in fs::read_dir(".")? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_str().unwrap().to_lowercase();
        if name_str.ends_with(".txt") {
            println!("Text file: {}", name_str);
        }
    }
    Ok(())
}
