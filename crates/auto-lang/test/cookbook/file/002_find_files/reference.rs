use walkdir::WalkDir;

fn main() {
    for entry in WalkDir::new("src") {
        let entry = entry.unwrap();
        println!("Found: {}", entry.path().display());
    }
}
