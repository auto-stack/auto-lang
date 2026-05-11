use walkdir::WalkDir;

fn main() {
    for entry in WalkDir::new(".") {
        let entry = entry.unwrap();
        println!("{}", entry.path().display());
    }
}
