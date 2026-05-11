use walkdir::WalkDir;
use std::time::SystemTime;

fn main() {
    for entry in WalkDir::new("src").into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let meta = std::fs::metadata(entry.path()).unwrap();
            let modified = meta.modified().unwrap();
            println!("{}: {:?}", entry.path().display(), modified);
        }
    }
}
