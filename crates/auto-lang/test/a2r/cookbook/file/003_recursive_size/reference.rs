use walkdir::WalkDir;
use std::fs;

fn main() {
    let mut total_size: u64 = 0;
    for entry in WalkDir::new("src") {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            let meta = fs::metadata(entry.path()).unwrap();
            total_size += meta.len();
        }
    }
    println!("Total size: {} bytes", total_size);
}
