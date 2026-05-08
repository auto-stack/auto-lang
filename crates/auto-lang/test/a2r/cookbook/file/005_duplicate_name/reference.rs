use std::collections::HashMap;
use std::fs;

fn main() -> std::io::Result<()> {
    let mut seen = HashMap::new();
    for entry in fs::read_dir(".")? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_str().unwrap();
        if seen.contains_key(name_str) {
            println!("Duplicate: {}", name_str);
        } else {
            seen.insert(name_str.to_string(), true);
        }
    }
    Ok(())
}
